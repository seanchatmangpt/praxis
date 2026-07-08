#!/usr/bin/env python3
"""The foreign graph verifier — a second implementation for workflow receipts.

A SECOND IMPLEMENTATION, in a different language, using a different BLAKE3
binary (`b3sum`), that re-verifies praxis-synthesis WorkflowReceipt JSON
against the source TTL document. If this script and the Rust crate agree,
the receipt is not self-attested.

Usage:
  foreign_verify_graph.py graph <ttl-file> <receipt.json>
  foreign_verify_graph.py firing <base.ttl> <adds.ttl> <removes.ttl> \
      <firing_receipt.json>

Exit 0 = verified; exit 1 = MISMATCH (printed); exit 2 = usage/IO error.

What is recomputed here: the Turtle-subset parse, the canonical form, the
graph_hash, the WorkflowIr extraction and its ir_hash, the chain refold, the
plan-payload binding, and the exec-payload hash. What is NOT re-derived:
plan/topology/geometry stage hashes are refolded as claimed (re-derivation
needs the Rust replayer — the named honest limitation, narrowed by one
stage). ttl_hash is recomputed and printed informationally only — it is
never folded into the chain, so a reformat of the same triples is lawful.
"""
import json
import subprocess
import sys

WORKFLOW_DOMAIN = "praxis:workflow:v1"
RDF_TYPE = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"

MAX_TTL_BYTES = 65_536
MAX_TRIPLES = 4_096
MAX_IRI_LEN = 256
MAX_LIT_LEN = 1_024
MAX_PREFIXES = 32
I64_MIN = -(2 ** 63)
I64_MAX = 2 ** 63 - 1


def b3(data: bytes) -> str:
    """BLAKE3 via the b3sum binary — deliberately not the Rust crate."""
    out = subprocess.run(
        ["b3sum", "--no-names"], input=data, capture_output=True, check=True
    )
    return out.stdout.decode().strip()


def genesis(domain: str) -> str:
    return b3(domain.encode())


def fold(prev_hex: str, payload: bytes) -> str:
    return b3(prev_hex.encode() + payload)


class ParseRefusal(Exception):
    """A grammar or cap violation — the verifier refuses the document."""


def refuse(msg: str) -> "ParseRefusal":
    return ParseRefusal(msg)


# ── lex ─────────────────────────────────────────────────────────────────────

BAREWORD_OK = set(
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789:_-?"
)


def lex(src: str) -> list:
    """Tokenize the bounded Turtle subset. Tokens are (kind, value) tuples:
    kinds: prefix, iri, qname (value = (pn, local)), str, int, a, dot,
    semi, comma."""
    toks = []
    i, n = 0, len(src)
    while i < n:
        c = src[i]
        if c in " \t\r\n":
            i += 1
            continue
        if c == "#":
            while i < n and src[i] != "\n":
                i += 1
            continue
        if c == ".":
            toks.append(("dot", None))
            i += 1
            continue
        if c == ";":
            toks.append(("semi", None))
            i += 1
            continue
        if c == ",":
            toks.append(("comma", None))
            i += 1
            continue
        if c == '"':
            i += 1
            out = []
            size = 0
            while True:
                if i >= n:
                    raise refuse("unterminated string literal")
                ch = src[i]
                if ch == '"':
                    i += 1
                    break
                if ch == "\\":
                    if i + 1 >= n:
                        raise refuse("unterminated string literal")
                    esc = src[i + 1]
                    if esc == '"':
                        out.append('"')
                    elif esc == "\\":
                        out.append("\\")
                    elif esc == "n":
                        out.append("\n")
                    elif esc == "t":
                        out.append("\t")
                    else:
                        raise refuse(f"unsupported escape '\\{esc}'")
                    size += 1
                    i += 2
                elif ch in "\n\r":
                    raise refuse("raw newline in string literal")
                elif ord(ch) < 0x20:
                    raise refuse("raw control character in string literal")
                else:
                    out.append(ch)
                    size += len(ch.encode())
                    i += 1
                if size > MAX_LIT_LEN:
                    raise refuse("cap exceeded: lit_len")
            toks.append(("str", "".join(out)))
            continue
        if c == "<":
            i += 1
            out = []
            while True:
                if i >= n:
                    raise refuse("unterminated IRIREF")
                ch = src[i]
                if ch == ">":
                    i += 1
                    break
                if ch in ' \t\n\r"{}|^`' or ord(ch) < 0x20:
                    raise refuse("illegal character in IRIREF")
                out.append(ch)
                i += 1
                if len("".join(out).encode()) > MAX_IRI_LEN:
                    raise refuse("cap exceeded: iri_len")
            toks.append(("iri", "".join(out)))
            continue
        if c in "[]":
            raise refuse("blank node '[]' is refused")
        if c in "()":
            raise refuse("collection '()' is refused")
        if c == "@":
            i += 1
            j = i
            while j < n and src[j] in BAREWORD_OK:
                j += 1
            word = src[i:j]
            i = j
            if word == "prefix":
                toks.append(("prefix", None))
                continue
            if word == "base":
                raise refuse("'@base' is refused")
            raise refuse(f"unsupported directive or language tag '@{word}'")
        if c == "^":
            raise refuse("'^^' datatype is refused")
        if c == "-" or c.isdigit():
            j = i
            if c == "-":
                j += 1
            digits = 0
            while j < n and src[j].isdigit():
                j += 1
                digits += 1
            if digits == 0:
                raise refuse("'-' without digits")
            if j < n:
                if src[j] == "." and j + 1 < n and src[j + 1].isdigit():
                    raise refuse("decimal literal is refused")
                if src[j] in "eE":
                    raise refuse("double literal is refused")
            v = int(src[i:j])
            if not I64_MIN <= v <= I64_MAX:
                raise refuse(f"integer '{src[i:j]}' does not fit i64")
            toks.append(("int", v))
            i = j
            continue
        if c == "_":
            raise refuse("blank node '_:' is refused")
        j = i
        while j < n and src[j] in BAREWORD_OK:
            j += 1
        word = src[i:j]
        if not word:
            raise refuse(f"unexpected character '{c}'")
        i = j
        if word in ("true", "false"):
            raise refuse("boolean literal is refused")
        if ":" in word:
            colon = word.index(":")
            toks.append(("qname", (word[:colon], word[colon + 1:])))
            continue
        if word == "a":
            toks.append(("a", None))
            continue
        raise refuse(f"bare word '{word}' is not a term")
    return toks


# ── parse ───────────────────────────────────────────────────────────────────


class Parser:
    def __init__(self, toks):
        self.toks = toks
        self.i = 0
        self.prefixes = []  # (name, iri) — last declaration wins
        self.triples = []  # (s, p, ("iri"|"str"|"int", value))

    def peek(self):
        return self.toks[self.i] if self.i < len(self.toks) else None

    def bump(self):
        t = self.peek()
        if t is None:
            raise refuse("unexpected end of input")
        self.i += 1
        return t

    def expand(self, pn, local):
        for name, base in reversed(self.prefixes):
            if name == pn:
                iri = base + local
                if len(iri.encode()) > MAX_IRI_LEN:
                    raise refuse("cap exceeded: iri_len")
                return iri
        raise refuse(f"undeclared prefix '{pn}:'")

    def parse_prefix(self):
        kind, val = self.bump()
        if kind != "qname" or val[1] != "":
            raise refuse("expected 'name:' after @prefix")
        pn = val[0]
        kind, base = self.bump()
        if kind != "iri":
            raise refuse("expected <IRI> in @prefix")
        kind, _ = self.bump()
        if kind != "dot":
            raise refuse("expected '.' after @prefix")
        self.prefixes.append((pn, base))
        if len(self.prefixes) > MAX_PREFIXES:
            raise refuse("cap exceeded: prefixes")

    def parse_term_iri(self, what):
        kind, val = self.bump()
        if kind == "iri":
            if len(val.encode()) > MAX_IRI_LEN:
                raise refuse("cap exceeded: iri_len")
            return val
        if kind == "qname":
            return self.expand(val[0], val[1])
        raise refuse(f"expected IRI or prefixed name as {what}")

    def parse_object(self):
        kind, val = self.bump()
        if kind == "iri":
            if len(val.encode()) > MAX_IRI_LEN:
                raise refuse("cap exceeded: iri_len")
            return ("iri", val)
        if kind == "qname":
            return ("iri", self.expand(val[0], val[1]))
        if kind == "str":
            return ("str", val)
        if kind == "int":
            return ("int", val)
        raise refuse("expected object term")

    def parse_stmt(self):
        subject = self.parse_term_iri("subject")
        while True:
            t = self.peek()
            if t is None:
                raise refuse("unexpected end of input")
            if t[0] == "a":
                self.bump()
                pred = RDF_TYPE
            else:
                pred = self.parse_term_iri("predicate")
            while True:
                obj = self.parse_object()
                self.triples.append((subject, pred, obj))
                if len(self.triples) > MAX_TRIPLES:
                    raise refuse("cap exceeded: triples")
                t = self.peek()
                if t is not None and t[0] == "comma":
                    self.bump()
                    continue
                break
            kind, _ = self.bump()
            if kind == "dot":
                return
            if kind == "semi":
                t = self.peek()
                if t is not None and t[0] == "dot":
                    self.bump()
                    return
                continue
            raise refuse("expected '.', ';' or ',' after object")


def parse_ttl(src: str) -> list:
    if len(src.encode()) > MAX_TTL_BYTES:
        raise refuse("cap exceeded: ttl_bytes")
    p = Parser(lex(src))
    while (tok := p.peek()) is not None:
        if tok[0] == "prefix":
            p.bump()
            p.parse_prefix()
        else:
            p.parse_stmt()
    return p.triples


# ── canon ───────────────────────────────────────────────────────────────────


def escape_str(s: str) -> str:
    return (
        s.replace("\\", "\\\\")
        .replace('"', '\\"')
        .replace("\n", "\\n")
        .replace("\t", "\\t")
    )


def render_object(obj) -> str:
    kind, val = obj
    if kind == "iri":
        return f"<{val}>"
    if kind == "str":
        return f'"{escape_str(val)}"'
    return str(val)  # shortest-form decimal; matches i64 Display


def canonical_form(triples) -> str:
    lines = [f"<{s}> <{p}> {render_object(o)} ." for s, p, o in triples]
    lines.sort(key=lambda l: l.encode())
    deduped = []
    for line in lines:
        if not deduped or deduped[-1] != line:
            deduped.append(line)
    return "\n".join(deduped) + "\n"


# ── ir ── re-derivation of WorkflowIr (mirrors graph.rs extract_ir exactly)

WF_NS = "http://seanchatmangpt.github.io/praxis/workflow#"
WF_CLASSES = ["Workflow", "Capability", "Atom", "Constraint"]
WF_PREDICATES = ["budget", "init", "goal", "name", "params", "cost", "pre",
                 "add", "del", "predicate", "arg0", "arg1", "arg2", "arg3",
                 "arg4", "arg5", "arg6", "arg7", "kind", "a", "b", "k"]
U32_MAX = 2 ** 32 - 1

# Rust `Object` derived Ord: variant order Iri < Str < Int, then value.
# ("int" < "iri" < "str" lexically != Rust) — map kinds to ranks instead.
_OBJ_RANK = {"iri": 0, "str": 1, "int": 2}


class IrRefusal(Exception):
    """A workflow-shape violation — extraction refuses the graph."""


def _ir_refuse(subject: str, detail: str) -> "IrRefusal":
    return IrRefusal(f"<{subject}>: {detail}")


class NodeIndex:
    """Per-subject view of the graph in Rust canonical (sorted) triple order:
    triples sorted by (s, p, object-variant rank, value), deduplicated;
    subjects iterated in sorted order (BTreeMap order)."""

    def __init__(self, triples):
        keyed = sorted(
            set(triples),
            key=lambda t: (t[0], t[1], _OBJ_RANK[t[2][0]], t[2][1]),
        )
        self.by_subject = {}
        for s, p, o in keyed:
            if p.startswith(WF_NS) and p[len(WF_NS):] not in WF_PREDICATES:
                raise _ir_refuse(s, f"unknown wf: predicate '{p}'")
            if p == RDF_TYPE and o[0] == "iri" and o[1].startswith(WF_NS):
                local = o[1][len(WF_NS):]
                if local not in WF_CLASSES:
                    raise _ir_refuse(s, f"unknown wf: class '{local}'")
            self.by_subject.setdefault(s, []).append((p, o))

    def objects(self, subject: str, local: str) -> list:
        pred = WF_NS + local
        return [o for p, o in self.by_subject.get(subject, []) if p == pred]

    def subjects_of_class(self, class_local: str) -> list:
        cls = ("iri", WF_NS + class_local)
        return [s for s, props in self.by_subject.items()
                if any(p == RDF_TYPE and o == cls for p, o in props)]

    def one_str(self, subject: str, local: str) -> str:
        objs = self.objects(subject, local)
        if not objs:
            raise _ir_refuse(subject, f"missing wf:{local}")
        if len(objs) > 1:
            raise _ir_refuse(subject, f"multiple wf:{local}")
        if objs[0][0] != "str":
            raise _ir_refuse(subject, f"wf:{local} must be a string literal")
        return objs[0][1]

    def one_int(self, subject: str, local: str) -> int:
        objs = self.objects(subject, local)
        if not objs:
            raise _ir_refuse(subject, f"missing wf:{local}")
        if len(objs) > 1:
            raise _ir_refuse(subject, f"multiple wf:{local}")
        if objs[0][0] != "int":
            raise _ir_refuse(subject, f"wf:{local} must be an integer literal")
        return objs[0][1]

    def opt_str(self, subject: str, local: str):
        objs = self.objects(subject, local)
        if not objs:
            return None
        if len(objs) > 1:
            raise _ir_refuse(subject, f"multiple wf:{local}")
        if objs[0][0] != "str":
            raise _ir_refuse(subject, f"wf:{local} must be a string literal")
        return objs[0][1]

    def opt_int(self, subject: str, local: str):
        objs = self.objects(subject, local)
        if not objs:
            return None
        if len(objs) > 1:
            raise _ir_refuse(subject, f"multiple wf:{local}")
        if objs[0][0] != "int":
            raise _ir_refuse(subject, f"wf:{local} must be an integer literal")
        return objs[0][1]

    def atom_refs(self, subject: str, local: str) -> list:
        out = []
        for kind, val in self.objects(subject, local):
            if kind != "iri":
                raise _ir_refuse(subject, f"wf:{local} must reference an atom IRI")
            out.append(val)
        return out


def _is_var(arg: str) -> bool:
    return len(arg) == 2 and arg[0] == "?" and "0" <= arg[1] <= "7"


def _extract_atom(idx: NodeIndex, iri: str) -> dict:
    if iri not in idx.subjects_of_class("Atom"):
        raise _ir_refuse(iri, "referenced node is not declared 'a wf:Atom'")
    predicate = idx.one_str(iri, "predicate")
    args = []
    ended = False
    for i in range(8):
        v = idx.opt_str(iri, f"arg{i}")
        if v is not None and not ended:
            args.append(v)
        elif v is not None:
            raise _ir_refuse(
                iri, f"wf:arg{i} present but wf:arg{i - 1} missing (argument gap)")
        else:
            ended = True
    # Dict insertion order reproduces IrAtom's serde field order.
    return {"predicate": predicate, "args": args}


def _extract_constraint(idx: NodeIndex, iri: str, cap_names: set) -> dict:
    kind = idx.one_str(iri, "kind")
    a = idx.opt_str(iri, "a")
    b = idx.opt_str(iri, "b")
    k = idx.opt_int(iri, "k")
    if k is not None and not 0 <= k <= U32_MAX:
        raise _ir_refuse(iri, f"wf:k {k} out of range 0..=u32::MAX")
    if kind in ("before", "after", "excludes", "requires"):
        if a is None:
            raise _ir_refuse(iri, f"kind '{kind}' requires wf:a")
        if b is None:
            raise _ir_refuse(iri, f"kind '{kind}' requires wf:b")
    elif kind in ("not-later", "not-earlier", "at-most"):
        if a is None:
            raise _ir_refuse(iri, f"kind '{kind}' requires wf:a")
        if k is None:
            raise _ir_refuse(iri, f"kind '{kind}' requires wf:k")
        if k > 255:
            raise _ir_refuse(iri, f"kind '{kind}' requires wf:k <= 255, got {k}")
    elif kind == "budget":
        if k is None:
            raise _ir_refuse(iri, f"kind '{kind}' requires wf:k")
    else:
        raise _ir_refuse(iri, f"unknown constraint kind '{kind}'")
    for name in (a, b):
        if name is not None and name not in cap_names:
            raise _ir_refuse(
                iri, f"constraint names undeclared capability '{name}'")
    # IrConstraint serde order; Option::None serializes as JSON null (never
    # omitted — graph.rs has no skip attribute), so the key is always emitted.
    return {"kind": kind, "a": a, "b": b, "k": k}


def _constraint_render(c: dict) -> str:
    # Byte-for-byte the Rust IrConstraint::render sort key.
    return (f"{c['kind']}:{c['a'] or ''}:{c['b'] or ''}:"
            f"{'' if c['k'] is None else c['k']}")


def _atom_key(a: dict):
    # Rust IrAtom derived Ord: predicate, then args (Vec<String> lexicographic;
    # UTF-8 byte order == code-point order, so plain tuple sort matches).
    return (a["predicate"], tuple(a["args"]))


def extract_ir(triples) -> dict:
    """Re-derive the WorkflowIr dict, mirroring graph.rs extract_ir exactly —
    including every refusal condition. Dict insertion order reproduces the
    serde struct field order, so ir_hash(extract_ir(...)) is byte-comparable
    to the Rust ir_hash."""
    idx = NodeIndex(triples)

    workflows = idx.subjects_of_class("Workflow")
    if not workflows:
        raise _ir_refuse("(document)", "no 'a wf:Workflow' node")
    if len(workflows) > 1:
        raise _ir_refuse(
            workflows[0],
            f"{len(workflows)} wf:Workflow nodes; exactly one required")
    wf = workflows[0]

    budget = idx.one_int(wf, "budget")
    if budget > 8:
        raise _ir_refuse(wf, f"wf:budget {budget} exceeds budget 8")
    if budget < 1:
        raise _ir_refuse(wf, f"wf:budget {budget} out of range 1..=8")

    init = []
    for iri in idx.atom_refs(wf, "init"):
        atom = _extract_atom(idx, iri)
        for v in atom["args"]:
            if _is_var(v):
                raise _ir_refuse(
                    iri, f"wf:init atom has variable '{v}'; init must be ground")
        init.append(atom)
    init.sort(key=_atom_key)

    goal_refs = idx.atom_refs(wf, "goal")
    if not goal_refs:
        raise _ir_refuse(wf, "missing wf:goal (one or more required)")
    goal = sorted((_extract_atom(idx, i) for i in goal_refs), key=_atom_key)

    capabilities = []
    for subject in idx.subjects_of_class("Capability"):
        name = idx.one_str(subject, "name")
        params = idx.one_int(subject, "params")
        if not 0 <= params <= 8:
            raise _ir_refuse(subject, f"wf:params {params} out of range 0..=8")
        cost = idx.one_int(subject, "cost")
        if not 0 <= cost <= U32_MAX:
            raise _ir_refuse(
                subject, f"wf:cost {cost} out of range 0..=u32::MAX")
        lists = []
        for local in ("pre", "add", "del"):
            atoms = [_extract_atom(idx, i)
                     for i in idx.atom_refs(subject, local)]
            atoms.sort(key=_atom_key)
            lists.append(atoms)
        # IrCapability serde field order.
        capabilities.append({"name": name, "params": params, "cost": cost,
                             "pre": lists[0], "add": lists[1], "del": lists[2]})
    capabilities.sort(key=lambda c: c["name"])
    for x, y in zip(capabilities, capabilities[1:]):
        if x["name"] == y["name"]:
            raise _ir_refuse(wf, f"duplicate capability name '{x['name']}'")
    cap_names = {c["name"] for c in capabilities}

    constraints = sorted(
        (_extract_constraint(idx, iri, cap_names)
         for iri in idx.subjects_of_class("Constraint")),
        key=_constraint_render,
    )

    # WorkflowIr serde field order.
    return {"budget": budget, "init": init, "goal": goal,
            "capabilities": capabilities, "constraints": constraints}


def ir_hash(ir: dict) -> str:
    """Content address of the IR's canonical JSON rendering; compact
    separators + ensure_ascii=False reproduce serde_json::to_string."""
    return b3(json.dumps(ir, separators=(",", ":"), ensure_ascii=False).encode())


# ── verify ──────────────────────────────────────────────────────────────────


def mismatch(stage: str, recomputed: str, recorded: str) -> int:
    print(f"MISMATCH: {stage} recomputed {recomputed[:16]} "
          f"!= recorded {recorded[:16]}")
    return 1


def require_str(receipt: dict, key: str) -> str:
    v = receipt.get(key)
    if not isinstance(v, str):
        raise KeyError(key)
    return v


def verify_graph(ttl_path: str, receipt_path: str) -> int:
    try:
        raw = open(ttl_path, "rb").read()
    except OSError as e:
        print(f"cannot read TTL file: {e}")
        return 2
    try:
        receipt = json.load(open(receipt_path))
    except (OSError, json.JSONDecodeError) as e:
        print(f"cannot read receipt JSON: {e}")
        return 2
    if not isinstance(receipt, dict):
        print("receipt is not a JSON object")
        return 2
    try:
        recorded_graph = require_str(receipt, "graph_hash")
        recorded_chain = require_str(receipt, "chain")
        claimed = [
            require_str(receipt, k)
            for k in ("ir_hash", "plan_hash", "topology_hash",
                      "geometry_hash", "exec_hash")
        ]
        recorded_ttl = require_str(receipt, "ttl_hash")
        plan = receipt["plan"]
        supervised = receipt["supervised"]
        plan_hash_bound = plan["receipt"]["plan_hash"]
        if not isinstance(plan_hash_bound, str) or not isinstance(
            supervised, dict
        ):
            raise KeyError("plan/supervised shape")
    except (KeyError, TypeError) as e:
        print(f"receipt missing or malformed field: {e}")
        return 2

    try:
        src = raw.decode()
    except UnicodeDecodeError:
        print("TTL file is not valid UTF-8")
        return 2
    try:
        triples = parse_ttl(src)
    except ParseRefusal as e:
        print(f"MISMATCH: parse refused: {e}")
        return 1

    # Informational only — ttl_hash is never folded into the chain; a
    # reformat of the same triples is lawful (mirrors replay_workflow).
    recomputed_ttl = b3(raw)
    note = "matches" if recomputed_ttl == recorded_ttl else "differs (lawful reformat)"
    print(f"ttl_hash recomputed {recomputed_ttl[:16]}… — "
          f"{note} vs recorded {recorded_ttl[:16]}…")

    # Stage 1: graph_hash recomputed from the document.
    graph_hash = b3(canonical_form(triples).encode())
    if graph_hash != recorded_graph:
        return mismatch("graph_hash", graph_hash, recorded_graph)

    # Stage 2: ir_hash re-derived — the WorkflowIr is extracted from the
    # parsed triples by a second implementation of graph.rs extract_ir and
    # hashed; a graph-consistent forgery of the IR stage is named here.
    try:
        ir = extract_ir(triples)
    except IrRefusal as e:
        print(f"MISMATCH: ir extraction refused: {e}")
        return 1
    recomputed_ir = ir_hash(ir)
    if recomputed_ir != claimed[0]:
        return mismatch("ir_hash", recomputed_ir, claimed[0])

    # Stage 3: chain refold. Honest note: plan/topology/geometry/exec hashes
    # are refolded as claimed, not re-derived (re-derivation needs the Rust
    # replayer); the graph_hash and ir_hash folded first are recomputed.
    chain = genesis(WORKFLOW_DOMAIN)
    chain = fold(chain, graph_hash.encode())
    chain = fold(chain, recomputed_ir.encode())
    for h in claimed[1:]:
        chain = fold(chain, h.encode())
    if chain != recorded_chain:
        return mismatch("chain", chain, recorded_chain)

    # Stage 4: plan payload binding (forged-body check from replay_workflow).
    try:
        steps_ordered = [
            {"capability": step["capability"], "binding": step["binding"]}
            for step in plan.get("steps", [])
        ]
        computed_plan_hash = b3(
            json.dumps(steps_ordered, separators=(",", ":"), ensure_ascii=False)
            .encode()
        )
    except (KeyError, TypeError) as e:
        print(f"MISMATCH: failed to reconstruct plan steps: {e}")
        return 1

    if computed_plan_hash != claimed[1]:
        return mismatch("plan_hash_of(steps)", computed_plan_hash, claimed[1])

    if plan_hash_bound != claimed[1]:
        return mismatch("plan payload", plan_hash_bound, claimed[1])

    # Stage 5: exec payload — recompute the supervised receipt's content
    # address; compact separators + ensure_ascii=False reproduce serde_json.
    exec_hash = b3(
        json.dumps(supervised, separators=(",", ":"), ensure_ascii=False)
        .encode()
    )
    if exec_hash != claimed[4]:
        return mismatch("exec payload", exec_hash, claimed[4])

    print(f"VERIFIED graph: {len(triples)} triples, "
          f"graph {graph_hash[:16]}…, ir {recomputed_ir[:16]}…, "
          f"chain {chain[:16]}…")
    print("plan/topology/geometry hashes refolded as claimed (not re-derived)")
    return 0


# ── firing ── foreign verification of the OUTER hook-firing chain
#
# Mirrors crates/praxis-synthesis/src/{firing,delta,quarantine,handlers}.rs.
# Re-derived independently from (base.ttl, adds.ttl, removes.ttl): the
# event_hash (delta canonical form), the post state (apply; removal of an
# absent triple is refused), the admission record hash, and the handler
# binding hash. hook_hash and outcome_hash are REFOLDED FROM THE RECEIPT'S
# EMBEDDED PAYLOADS ("refolded-from-payload"): the verdicts array and the
# outcome object are re-serialized to the exact serde rendering and hashed —
# this binds payload bytes to hash but does not re-derive hook evaluation
# (the named limitation; re-derivation needs the Rust evaluator). Inner v1
# chains are folded as claimed from receipt.inner[].chain (each is itself
# verifiable by the `graph` subcommand — a second named limitation). The
# history_hash (the window-history commitment) is likewise folded as claimed
# from the receipt field: the verifier has no history input, so it binds the
# fold position, not the history bytes (a third named limitation).
#
# Adversarial finding (closed): the embedded `admission`, `bindings`, and
# `agents` objects are ALSO payload-bound to their claimed hash strings
# (`refold_admission`/`refold_bindings`/`refold_agents`), the same way
# hook_hash/outcome_hash always were. Previously this script recomputed
# admission_hash/handler_hash/agent_registry_hash independently from the TTL
# inputs and never so much as read the embedded `admission`/`bindings`/
# `agents` fields — so a receipt whose displayed body had been forged, but
# whose flat hash string still matched the (independently-correct) TTL
# recomputation, was reported VERIFIED with the forged body never examined.

FIRING_DOMAIN = "praxis:hook-firing:v1"
NO_ACTION_SENTINEL = "praxis:no-action"
MAX_DELTA_TRIPLES = 64
DELEGABILITY = ("human-only", "assistive", "automatable", "verifiable")

VERDICT_KEYS = ["hook_iri", "hook_name", "condition_kind", "condition_hash",
                "verdict", "effect", "action_iri"]
HOOK_VERDICTS = ("Fired", "NotFired", "Gated")
EFFECT_KINDS = ("EmitDelta", "GroundAction", "Refuse")


class FiringRefusal(Exception):
    """A firing-stage violation — the verifier refuses at a named stage."""


def _triple_key(t):
    # Rust Triple derived Ord: (s, p, Object) with variant order Iri<Str<Int.
    return (t[0], t[1], _OBJ_RANK[t[2][0]], t[2][1])


def _canon_triples(triples) -> list:
    """Sorted, deduplicated triples in Rust canonical Ord order."""
    return sorted(set(triples), key=_triple_key)


def parse_delta(adds_ttl: str, removes_ttl: str):
    """Mirror delta.rs GraphDelta::parse/from_triples: sort, dedup, cap 64
    per side, refuse a triple asserted and retracted by the same event."""
    additions = _canon_triples(parse_ttl(adds_ttl))
    removals = _canon_triples(parse_ttl(removes_ttl))
    for side, name in ((additions, "delta_additions"), (removals, "delta_removals")):
        if len(side) > MAX_DELTA_TRIPLES:
            raise FiringRefusal(f"cap exceeded: {name}")
    removal_set = set(removals)
    for t in additions:
        if t in removal_set:
            raise FiringRefusal(
                "delta asserts and retracts the same triple: "
                f"<{t[0]}> <{t[1]}> {render_object(t[2])} .")
    return additions, removals


def delta_canonical_form(additions, removals) -> str:
    """Mirror delta.rs canonical_form: labeled canonical N-Triples sections."""
    return (f"additions\n{canonical_form(additions)}"
            f"removals\n{canonical_form(removals)}")


def apply_delta(base, additions, removals) -> list:
    """Mirror delta.rs GraphDelta::apply: removal of a triple not present in
    the base is refused; the post-state is sorted, deduplicated, re-capped."""
    post = set(base)
    for r in removals:
        if r not in post:
            raise FiringRefusal(
                "removal of a triple not present in the base graph: "
                f"<{r[0]}> <{r[1]}> {render_object(r[2])} .")
        post.remove(r)
    post.update(additions)
    if len(post) > MAX_TRIPLES:
        raise FiringRefusal("cap exceeded: triples")
    return sorted(post, key=_triple_key)


def extract_handler_bindings(post) -> list:
    """Mirror handlers.rs extract_bindings: every wf:Capability node with
    wf:handler must carry exactly one IRI handler, one string delegability
    (explicit — no default) and one string wf:name."""
    cap_class = ("iri", WF_NS + "Capability")
    caps = sorted({s for s, p, o in post if p == RDF_TYPE and o == cap_class})
    bindings = []
    for cap in caps:
        props = [(p, o) for s, p, o in post if s == cap]

        def one(local: str) -> list:
            pred = WF_NS + local
            return [o for p, o in props if p == pred]

        handlers = one("handler")
        if not handlers:
            continue
        if len(handlers) > 1:
            raise FiringRefusal(f"<{cap}>: multiple wf:handler")
        if handlers[0][0] != "iri":
            raise FiringRefusal(f"<{cap}>: wf:handler must be an IRI")
        handler = handlers[0][1]
        delegs = one("delegability")
        if not delegs:
            raise FiringRefusal(
                f"<{cap}>: wf:handler without wf:delegability — the grade is "
                "explicit or the binding is refused (no default)")
        if len(delegs) > 1:
            raise FiringRefusal(f"<{cap}>: multiple wf:delegability")
        if delegs[0][0] != "str" or delegs[0][1] not in DELEGABILITY:
            raise FiringRefusal(f"<{cap}>: wf:delegability '{delegs[0][1]}' "
                                "not in human-only|assistive|automatable|verifiable")
        names = one("name")
        if len(names) != 1 or names[0][0] != "str":
            raise FiringRefusal(
                f"<{cap}>: handled capability missing unique wf:name")
        bindings.append((names[0][1], handler, delegs[0][1]))
    bindings.sort(key=lambda b: b[0])
    return bindings


AGENT_NS = "http://seanchatmangpt.github.io/praxis/agent#"
AGENT_CLASSES = ["Agent"]
AGENT_PREDICATES = ["tool", "canSpawn", "layerDepth"]
MAX_TOOLS = 8
MAX_CAN_SPAWN = 8
MAX_AGENTS = 8


def extract_agents(post) -> list:
    """Mirror agent_registry.rs extract_agents exactly: closed-world sweep
    over the agent: namespace, then per-agent tool/canSpawn/layerDepth
    extraction — sorted, deduped, bounded. Returns a list of
    (iri, tools, can_spawn, layer_depth) tuples sorted by iri."""
    for s, p, o in post:
        if p.startswith(AGENT_NS) and p[len(AGENT_NS):] not in AGENT_PREDICATES:
            raise FiringRefusal(f"<{s}>: unknown agent: predicate "
                                 f"'{p[len(AGENT_NS):]}'")
        if p == RDF_TYPE and o[0] == "iri" and o[1].startswith(AGENT_NS):
            local = o[1][len(AGENT_NS):]
            if local not in AGENT_CLASSES:
                raise FiringRefusal(f"<{s}>: unknown agent: class '{local}'")

    agent_class = ("iri", AGENT_NS + "Agent")
    subjects = sorted({s for s, p, o in post if p == RDF_TYPE and o == agent_class})
    if len(subjects) > MAX_AGENTS:
        raise FiringRefusal(
            f"(registry): {len(subjects)} agents declared; max {MAX_AGENTS}")

    agents = []
    for subject in subjects:
        props = [(p, o) for s, p, o in post if s == subject]

        tool_p = AGENT_NS + "tool"
        tools = []
        for p, o in props:
            if p != tool_p:
                continue
            if o[0] != "str":
                raise FiringRefusal(f"<{subject}>: agent:tool must be a "
                                     "string literal")
            tools.append(o[1])
        tools = sorted(set(tools))
        if len(tools) > MAX_TOOLS:
            raise FiringRefusal(
                f"<{subject}>: {len(tools)} agent:tool values; max {MAX_TOOLS}")

        spawn_p = AGENT_NS + "canSpawn"
        can_spawn = []
        for p, o in props:
            if p != spawn_p:
                continue
            if o[0] != "iri":
                raise FiringRefusal(f"<{subject}>: agent:canSpawn must be "
                                     "an IRI")
            can_spawn.append(o[1])
        can_spawn = sorted(set(can_spawn))
        if len(can_spawn) > MAX_CAN_SPAWN:
            raise FiringRefusal(
                f"<{subject}>: {len(can_spawn)} agent:canSpawn values; "
                f"max {MAX_CAN_SPAWN}")

        depth_p = AGENT_NS + "layerDepth"
        depths = [o for p, o in props if p == depth_p]
        if not depths:
            raise FiringRefusal(f"<{subject}>: missing agent:layerDepth")
        if len(depths) > 1:
            raise FiringRefusal(f"<{subject}>: multiple agent:layerDepth")
        if depths[0][0] != "int":
            raise FiringRefusal(
                f"<{subject}>: agent:layerDepth must be an integer literal")
        layer_depth = depths[0][1]
        if not (1 <= layer_depth <= 5):
            raise FiringRefusal(
                f"<{subject}>: agent:layerDepth {layer_depth} out of range 1..=5")

        agents.append((subject, tools, can_spawn, layer_depth))
    agents.sort(key=lambda a: a[0])
    return agents


def agent_canonical_form(agents: list) -> str:
    """Mirror agent_registry.rs agent_canonical_form: sorted
    `iri\\ttools-csv\\tcan_spawn-csv\\tdepth` lines, trailing newline."""
    out = []
    for iri, tools, can_spawn, depth in agents:
        out.append(f"{iri}\t{','.join(tools)}\t{','.join(can_spawn)}\t{depth}\n")
    return "".join(out)


def compact_json(value) -> str:
    """serde_json::to_string equivalent."""
    return json.dumps(value, separators=(",", ":"), ensure_ascii=False)


_DELEGABILITY_RENDER = {
    "HumanOnly": "human-only",
    "Assistive": "assistive",
    "Automatable": "automatable",
    "Verifiable": "verifiable",
}


def refold_admission(raw) -> str:
    """Rebuild receipt["admission"] (an AdmissionRecord) in the exact serde
    field order and re-hash it — mirrors replay_firing's payload-binding
    check `receipt.admission.admission_hash()? != receipt.admission_hash`.
    Without this, a forged `admission` body left sitting behind an untouched
    (and independently-correct, TTL-derived) `admission_hash` string would
    never be caught: nothing else in this verifier ever reads the embedded
    object, only the flat hash string."""
    if not isinstance(raw, dict) or set(raw) != {
        "epoch", "base_graph_hash", "post_graph_hash", "event_hash", "verdict"
    }:
        raise FiringRefusal("receipt.admission has unexpected shape")
    if raw["verdict"] not in ("Admitted", "Refused"):
        raise FiringRefusal(f"unknown admission verdict '{raw['verdict']}'")
    for key in ("base_graph_hash", "post_graph_hash", "event_hash"):
        if not isinstance(raw[key], str):
            raise FiringRefusal(f"admission field '{key}' is not a string")
    if not isinstance(raw["epoch"], int):
        raise FiringRefusal("admission field 'epoch' is not an integer")
    rebuilt = {k: raw[k] for k in
               ("epoch", "base_graph_hash", "post_graph_hash", "event_hash", "verdict")}
    return compact_json(rebuilt)


def refold_bindings(raw) -> str:
    """Rebuild receipt["bindings"] (Vec<HandlerBinding>) into the same
    canonical tab-separated form `handler_hash` is computed over — mirrors
    replay_firing's `handler_hash(&receipt.bindings) != receipt.handler_hash`.
    Without this, a forged `bindings` array behind an untouched
    `handler_hash` (independently re-derived from the TTL, which never
    reads this embedded array) would never be caught."""
    if not isinstance(raw, list):
        raise FiringRefusal("receipt.bindings is not an array")
    lines = []
    for b in raw:
        if not isinstance(b, dict) or set(b) != {"capability", "handler", "delegability"}:
            raise FiringRefusal("binding record has unexpected shape")
        deleg = _DELEGABILITY_RENDER.get(b["delegability"])
        if deleg is None:
            raise FiringRefusal(f"unknown delegability '{b['delegability']}'")
        lines.append((b["capability"], f"{b['capability']}\t{b['handler']}\t{deleg}\n"))
    lines.sort(key=lambda x: x[0])
    return "".join(line for _, line in lines)


def refold_agents(raw) -> str:
    """Rebuild receipt["agents"] (Vec<AgentProfile>) into the same canonical
    tab-separated form `agent_registry_hash` is computed over — mirrors the
    same payload-binding doctrine as `refold_bindings`/`refold_admission`
    for the agent registry."""
    if not isinstance(raw, list):
        raise FiringRefusal("receipt.agents is not an array")
    lines = []
    for a in raw:
        if not isinstance(a, dict) or set(a) != {"iri", "tools", "can_spawn", "layer_depth"}:
            raise FiringRefusal("agent record has unexpected shape")
        if not (isinstance(a["tools"], list) and isinstance(a["can_spawn"], list)
                and isinstance(a["layer_depth"], int)):
            raise FiringRefusal("agent record field has unexpected type")
        lines.append((
            a["iri"],
            f"{a['iri']}\t{','.join(a['tools'])}\t{','.join(a['can_spawn'])}\t{a['layer_depth']}\n",
        ))
    lines.sort(key=lambda x: x[0])
    return "".join(line for _, line in lines)


def refold_verdicts(raw) -> str:
    """Rebuild each HookVerdictRecord in the exact serde field order from the
    receipt's embedded payload (refolded-from-payload: binds bytes to hash,
    does not re-derive hook evaluation). Unknown or missing keys refuse."""
    if not isinstance(raw, list):
        raise FiringRefusal("receipt.verdicts is not an array")
    rebuilt = []
    allowed_keys = set(VERDICT_KEYS)
    allowed_keys_with_diag = allowed_keys | {"diagnostics"}
    for row in raw:
        if not isinstance(row, dict):
            raise FiringRefusal("verdict record has unexpected shape")
        row_keys = set(row)
        if row_keys != allowed_keys and row_keys != allowed_keys_with_diag:
            raise FiringRefusal("verdict record has unexpected shape")
        if row["verdict"] not in HOOK_VERDICTS:
            raise FiringRefusal(f"unknown verdict '{row['verdict']}'")
        if row["effect"] not in EFFECT_KINDS:
            raise FiringRefusal(f"unknown effect '{row['effect']}'")
        for key in VERDICT_KEYS[:4]:
            if not isinstance(row[key], str):
                raise FiringRefusal(f"verdict field '{key}' is not a string")
        if row["action_iri"] is not None and not isinstance(row["action_iri"], str):
            raise FiringRefusal("verdict field 'action_iri' is not a string or null")
        
        rebuilt_row = {k: row[k] for k in VERDICT_KEYS}
        if "diagnostics" in row:
            rebuilt_row["diagnostics"] = row["diagnostics"]
        rebuilt.append(rebuilt_row)
    return compact_json(rebuilt)


def refold_outcome(raw) -> str:
    """Rebuild the FiringOutcome serde rendering: the unit variant is the
    plain string \"Completed\"; the struct variant is
    {\"Refused\":{\"stage\":..,\"reason\":..}}."""
    if raw == "Completed":
        return compact_json("Completed")
    if isinstance(raw, dict) and set(raw) == {"Refused"}:
        body = raw["Refused"]
        if (isinstance(body, dict) and set(body) == {"stage", "reason"}
                and isinstance(body["stage"], str)
                and isinstance(body["reason"], str)):
            return compact_json(
                {"Refused": {"stage": body["stage"], "reason": body["reason"]}})
    raise FiringRefusal("receipt.outcome has unexpected shape")


def verify_firing(base_path: str, adds_path: str, removes_path: str,
                  receipt_path: str) -> int:
    try:
        base_src = open(base_path, encoding="utf-8").read()
        adds_src = open(adds_path, encoding="utf-8").read()
        removes_src = open(removes_path, encoding="utf-8").read()
    except (OSError, UnicodeDecodeError) as e:
        print(f"cannot read TTL input: {e}")
        return 2
    try:
        receipt = json.load(open(receipt_path))
    except (OSError, json.JSONDecodeError) as e:
        print(f"cannot read receipt JSON: {e}")
        return 2
    if not isinstance(receipt, dict):
        print("receipt is not a JSON object")
        return 2
    try:
        recorded = {
            k: require_str(receipt, k)
            for k in ("event_hash", "admission_hash", "handler_hash",
                      "agent_registry_hash", "hook_hash", "history_hash",
                      "outcome_hash", "chain")
        }
        inner = receipt["inner"]
        verdicts_raw = receipt["verdicts"]
        outcome_raw = receipt["outcome"]
        admission_raw = receipt["admission"]
        bindings_raw = receipt["bindings"]
        agents_raw = receipt["agents"]
        if not isinstance(inner, list):
            raise KeyError("inner")
        inner_chains = [entry["chain"] for entry in inner]
        if not all(isinstance(c, str) for c in inner_chains):
            raise KeyError("inner[].chain")
    except (KeyError, TypeError) as e:
        print(f"receipt missing or malformed field: {e}")
        return 2

    failures = 0

    def stage(name: str, computed: str, claimed: str, note: str = "") -> None:
        nonlocal failures
        suffix = f" [{note}]" if note else ""
        if computed == claimed:
            print(f"PASS {name}: {computed[:16]}…{suffix}")
        else:
            print(f"FAIL {name}: recomputed {computed[:16]} "
                  f"!= recorded {claimed[:16]}{suffix}")
            failures += 1

    try:
        base = _canon_triples(parse_ttl(base_src))
        additions, removals = parse_delta(adds_src, removes_src)

        # Stage 1: event_hash re-derived from the delta canonical form.
        event_hash = b3(delta_canonical_form(additions, removals).encode())
        stage("event_hash", event_hash, recorded["event_hash"])

        # Stage 2: post state applied (removal-not-present refuses) and the
        # admission record rebuilt field for field (quarantine.rs serde
        # order). Parse the epoch from the receipt payload.
        post = apply_delta(base, additions, removals)
        try:
            epoch = int(admission_raw["epoch"])
        except (KeyError, ValueError, TypeError):
            epoch = 1
        record = {
            "epoch": epoch,
            "base_graph_hash": b3(canonical_form(base).encode()),
            "post_graph_hash": b3(canonical_form(post).encode()),
            "event_hash": event_hash,
            "verdict": "Admitted",
        }
        admission_hash = b3(compact_json(record).encode())
        stage("admission_hash", admission_hash, recorded["admission_hash"])

        # Stage 2b: payload binding — the embedded `admission` object must
        # itself hash to `admission_hash` (mirrors replay_firing's
        # `receipt.admission.admission_hash()? != receipt.admission_hash`).
        # Without this, a forged `admission` body sitting behind an
        # untouched (TTL-derived-correct) hash string would never be
        # caught: stage 2 above never reads the embedded object at all.
        admission_payload_hash = b3(refold_admission(admission_raw).encode())
        stage("admission payload", admission_payload_hash, recorded["admission_hash"],
              "payload-binding")

        # Stage 3: handler bindings re-extracted from the post state;
        # canonical tab-separated lines, sorted by capability.
        bindings = extract_handler_bindings(post)
        lines = "".join(f"{c}\t{h}\t{d}\n" for c, h, d in bindings)
        handler_hash = b3(lines.encode())
        stage("handler_hash", handler_hash, recorded["handler_hash"])

        # Stage 3b: payload binding for `bindings` — mirrors replay_firing's
        # `handler_hash(&receipt.bindings) != receipt.handler_hash`.
        bindings_payload_hash = b3(refold_bindings(bindings_raw).encode())
        stage("bindings payload", bindings_payload_hash, recorded["handler_hash"],
              "payload-binding")

        # Stage 3.5: agent registry re-extracted from the post state (tool
        # sets, spawn edges, layer depth); canonical tab-separated lines,
        # sorted by IRI. The depth-5 spawn law itself is NOT independently
        # re-judged here (same named limitation as handler existence above:
        # this script binds bytes to the claimed hash, it does not re-run
        # policy decisions) — only the extraction is re-derived.
        agents = extract_agents(post)
        agent_registry_hash = b3(agent_canonical_form(agents).encode())
        stage("agent_registry_hash", agent_registry_hash,
              recorded["agent_registry_hash"])

        # Stage 3.5b: payload binding for `agents` — mirrors replay_firing's
        # `agent_registry_hash(&receipt.agents) != receipt.agent_registry_hash`.
        agents_payload_hash = b3(refold_agents(agents_raw).encode())
        stage("agents payload", agents_payload_hash, recorded["agent_registry_hash"],
              "payload-binding")

        # Stages 4-5: refolded-from-payload — the embedded verdicts array and
        # outcome object are re-serialized to the serde rendering and hashed;
        # hook evaluation itself is NOT re-derived (named limitation).
        hook_hash = b3(refold_verdicts(verdicts_raw).encode())
        stage("hook_hash", hook_hash, recorded["hook_hash"],
              "refolded-from-payload")
        outcome_hash = b3(refold_outcome(outcome_raw).encode())
        stage("outcome_hash", outcome_hash, recorded["outcome_hash"],
              "refolded-from-payload")

        # Stage 6: the outer chain folded from genesis over the recomputed
        # stage hashes and each inner v1 chain (folded as claimed — verify
        # each with the `graph` subcommand; empty = the no-action sentinel).
        chain = genesis(FIRING_DOMAIN)
        for payload in (event_hash, admission_hash, handler_hash,
                        agent_registry_hash, hook_hash,
                        recorded["history_hash"]):
            # history_hash folded as claimed (no history input; named
            # limitation).
            chain = fold(chain, payload.encode())
        if inner_chains:
            for c in inner_chains:
                chain = fold(chain, c.encode())
        else:
            chain = fold(chain, NO_ACTION_SENTINEL.encode())
        chain = fold(chain, outcome_hash.encode())
        stage("chain", chain, recorded["chain"])
    except (ParseRefusal, FiringRefusal) as e:
        print(f"FAIL firing: refused: {e}")
        return 1

    if failures:
        print(f"FIRING MISMATCH: {failures} stage(s) failed")
        return 1
    print(f"VERIFIED firing: {len(post)} post triples, "
          f"{len(inner_chains)} inner chain(s), chain {chain[:16]}…")
    print("hook_hash/outcome_hash refolded from embedded payloads "
          "(hook evaluation not re-derived); inner chains folded as claimed")
    return 0


def main() -> int:
    if len(sys.argv) >= 4 and sys.argv[1] == "graph":
        return verify_graph(sys.argv[2], sys.argv[3])
    if len(sys.argv) >= 6 and sys.argv[1] == "firing":
        return verify_firing(sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5])
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main())
