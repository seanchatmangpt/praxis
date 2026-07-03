#!/usr/bin/env python3
"""The foreign graph verifier — a second implementation for workflow receipts.

A SECOND IMPLEMENTATION, in a different language, using a different BLAKE3
binary (`b3sum`), that re-verifies praxis-synthesis WorkflowReceipt JSON
against the source TTL document. If this script and the Rust crate agree,
the receipt is not self-attested.

Usage:
  foreign_verify_graph.py graph <ttl-file> <receipt.json>

Exit 0 = verified; exit 1 = MISMATCH (printed); exit 2 = usage/IO error.

What is recomputed here: the Turtle-subset parse, the canonical form, the
graph_hash, the chain refold, the plan-payload binding, and the exec-payload
hash. What is NOT re-derived: ir/plan/topology/geometry stage hashes are
refolded as claimed (re-derivation needs the Rust replayer). ttl_hash is
recomputed and printed informationally only — it is never folded into the
chain, so a reformat of the same triples is lawful.
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

    # Stage 2: chain refold. Honest note: ir/plan/topology/geometry/exec
    # hashes are refolded as claimed, not re-derived (re-derivation needs
    # the Rust replayer); the graph_hash folded first is the recomputed one.
    chain = genesis(WORKFLOW_DOMAIN)
    chain = fold(chain, graph_hash.encode())
    for h in claimed:
        chain = fold(chain, h.encode())
    if chain != recorded_chain:
        return mismatch("chain", chain, recorded_chain)

    # Stage 3: plan payload binding (forged-body check from replay_workflow).
    if plan_hash_bound != claimed[1]:
        return mismatch("plan payload", plan_hash_bound, claimed[1])

    # Stage 4: exec payload — recompute the supervised receipt's content
    # address; compact separators + ensure_ascii=False reproduce serde_json.
    exec_hash = b3(
        json.dumps(supervised, separators=(",", ":"), ensure_ascii=False)
        .encode()
    )
    if exec_hash != claimed[4]:
        return mismatch("exec payload", exec_hash, claimed[4])

    print(f"VERIFIED graph: {len(triples)} triples, "
          f"graph {graph_hash[:16]}…, chain {chain[:16]}…")
    return 0


def main() -> int:
    if len(sys.argv) < 4 or sys.argv[1] != "graph":
        print(__doc__)
        return 2
    return verify_graph(sys.argv[2], sys.argv[3])


if __name__ == "__main__":
    sys.exit(main())
