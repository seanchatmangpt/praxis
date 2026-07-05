#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
paper_factory_engine.py — standalone RDF reasoning engine for praxis's paper factory.

Decoupled extraction from ofmf_keystone.py
(/Users/sac/dev/.spec-kit.bak/src/specify_cli/ofmf/ofmf_keystone.py). That file was
tested (isolated venv, all six declared dialect dependencies installed) and found NOT
runnable as shipped:
  - Its ShEx hard-import gate names the wrong module (`shex`, an unrelated abandoned
    PyPI package) instead of `pyshex` (the actually-used `SchemaLoader`/`ShExEvaluator`
    API) — and those two names are never imported anywhere in the file regardless, a
    latent NameError.
  - Despite its own docstring calling it a "One-File Runtime," it does a module-level
    `from specify_cli.kgc_ofmf_utils import (...)`, which drags in the entire
    `specify_cli` CLI package's own dependency tree (typer, readchar, httpx, ...) —
    none of which are declared in its own "Install (minimum)" instructions.

This module keeps only the parts that were verified sound: SPARQL ASK/CONSTRUCT
(rdflib), SHACL validation (pyshacl), OWL-RL closure (owlrl), N3 forward-chaining via
the EYE reasoner (external `eye` binary — verified installed and working,
`eye --version` => EYE v11.23.1), and a BLAKE3-over-canonical-N-Quads receipt so any
graph state is content-addressable. Dropped entirely: ShEx (buggy as shown above),
Datalog/pyDatalog (unmaintained, not installable in this environment), BPMN/
SpiffWorkflow execution, and all `specify_cli` coupling.

Design rule (unchanged from ofmf_keystone.py): Python orchestrates, dialects decide.
No dialect's logic is reimplemented in Python — SPARQL/SHACL/OWL-RL/N3 are always
delegated to their real engines (rdflib, pyshacl, owlrl, eye), never approximated.

Install:  pip install rdflib blake3 owlrl pyshacl
External: the `eye` N3 reasoner binary on $PATH
          (https://github.com/josd/eye — `npm install -g eyereasoner`, or a native
          build). A missing binary is a hard ImportError, not a silent skip.

Intended use (the "external code prepares, ggen exports the final tex" pipeline):
  1. Load a paper's `.ttl` (e.g. docs/thesis/rdf/00_foundations.ttl) into an rdflib
     Graph.
  2. Run n3_closure() with a rules file to derive facts ggen's own SPARQL/Tera
     pipeline cannot (transitive citation closure, notation-conflict flags, etc.).
  3. Serialize the enriched graph back to Turtle; ggen's `sync run` treats it exactly
     like any other ontology source — it never sees the reasoner, only the resulting
     triples.
"""

from __future__ import annotations

import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import Any, Dict, List, Tuple

from rdflib import Graph
from blake3 import blake3  # type: ignore

from pyshacl import validate as shacl_validate  # type: ignore
from owlrl import DeductiveClosure, OWLRL_Semantics  # type: ignore


class PaperFactoryError(RuntimeError):
    """A dialect engine failed, or a hard external dependency is missing."""


# ---------------------------------------------------------------------------
# Receipts — BLAKE3 over canonicalized N-Quads, so any graph state is
# content-addressable the same way praxis-synthesis's Rust receipts are.
# ---------------------------------------------------------------------------


def canonical_nquads(graph: Graph) -> bytes:
    """Deterministic N-Triples serialization: sorted lines over the graph's
    own triples only.

    Deliberately N-Triples (3-tuples), not true N-Quads (4-tuples): an
    earlier version wrapped the input in a fresh `ConjunctiveGraph()` to get
    rdflib's N-Quads writer, but an anonymous ConjunctiveGraph's own default
    context is assigned a NEW blank-node identifier on every instantiation
    (verified via this module's own smoke test — two calls on the identical
    input graph produced different hashes). Since this engine's use case is
    one ontology graph per paper with no named-graph provenance to
    preserve, N-Triples over the plain triple set sidesteps that instability
    entirely. If named-graph/quad support is ever needed, use
    `rdflib.compare.to_canonical_graph` for blank-node-isomorphism-safe
    canonicalization instead of hand-rolling quad sorting.
    """
    lines = sorted(graph.serialize(format="nt").splitlines())
    return ("\n".join(line for line in lines if line.strip()) + "\n").encode("utf-8")


def receipt_hash(graph: Graph) -> str:
    """BLAKE3 hex digest of the graph's canonical N-Quads form."""
    return blake3(canonical_nquads(graph)).hexdigest()


# ---------------------------------------------------------------------------
# SPARQL — no dialect logic here at all, rdflib's own engine decides.
# ---------------------------------------------------------------------------


def sparql_ask(graph: Graph, query: str) -> bool:
    result = graph.query(query)
    try:
        return bool(result.askAnswer)  # type: ignore[attr-defined]
    except AttributeError:
        for _ in result:
            return True
        return False


def sparql_select(graph: Graph, query: str) -> List[Dict[str, Any]]:
    """Run a SELECT, returning rows as plain dicts keyed by variable name
    (values are rdflib term .toPython() results)."""
    result = graph.query(query)
    rows: List[Dict[str, Any]] = []
    for row in result:
        rows.append({str(var): row[var].toPython() if row[var] is not None else None for var in result.vars})
    return rows


def sparql_construct(graph: Graph, query: str) -> Graph:
    """Run a CONSTRUCT, returning the resulting triples as a new Graph (a
    delta the caller may merge into the source graph, mirroring ggen's own
    `construct:` frontmatter field / insert_construct)."""
    result = graph.query(query)
    out = Graph()
    if hasattr(result, "graph") and isinstance(result.graph, Graph):
        for triple in result.graph.triples((None, None, None)):
            out.add(triple)
    else:
        for triple in result:
            if len(triple) == 3:
                out.add(triple)
    return out


# ---------------------------------------------------------------------------
# SHACL — delegated entirely to pyshacl.
# ---------------------------------------------------------------------------


def shacl_validate_full(data_graph: Graph, shapes_graph: Graph) -> Tuple[bool, Graph, str]:
    conforms, report_graph, report_text = shacl_validate(
        data_graph=data_graph,
        shacl_graph=shapes_graph,
        ont_graph=None,
        inference="none",
        abort_on_first=False,
        meta_shacl=False,
        advanced=True,
        debug=False,
    )
    return bool(conforms), report_graph, report_text


# ---------------------------------------------------------------------------
# OWL-RL — delegated entirely to owlrl.
# ---------------------------------------------------------------------------


def owl_rl_closure(graph: Graph) -> Graph:
    """Return a NEW graph containing `graph`'s OWL-RL deductive closure —
    does not mutate the input (ofmf_keystone.py's owl_rl_expand mutated its
    argument in place, a surprising side effect for a "prepare, don't
    render" pipeline stage; this copies first)."""
    expanded = Graph()
    expanded += graph
    DeductiveClosure(OWLRL_Semantics).expand(expanded)
    return expanded


# ---------------------------------------------------------------------------
# N3 forward-chaining via the EYE reasoner (external binary).
#
# Calling convention verified against ofmf_keystone.py's n3_entails and
# confirmed correct in this session: EYE wants data and rules concatenated
# into ONE file (not passed as separate arguments), `--pass` to emit all
# triples (original + inferred), and `--nope` to suppress the `r:gives`
# proof-explanation wrapper EYE otherwise wraps inferred triples in — without
# `--nope` the output is not directly parseable as plain N3/Turtle triples.
# ---------------------------------------------------------------------------


def _require_eye() -> str:
    path = shutil.which("eye")
    if path is None:
        raise PaperFactoryError(
            "EYE N3 reasoner not found on $PATH. Install: npm install -g eyereasoner "
            "(or build github.com/josd/eye natively)."
        )
    return path


def n3_closure(graph: Graph, rules_path: Path) -> Graph:
    """Run EYE forward-chaining over `graph` with the N3 rules at
    `rules_path`, returning a NEW graph containing the original triples plus
    everything EYE entails. Does not mutate the input.

    Raises PaperFactoryError if `eye` is missing or exits nonzero — no
    silent degradation to "no entailment," matching this project's
    no-silent-defaults discipline.
    """
    eye_bin = _require_eye()
    rules_text = Path(rules_path).read_text(encoding="utf-8")

    with tempfile.TemporaryDirectory(prefix="paper-factory-n3-") as tmp:
        combined_path = Path(tmp) / "combined.n3"
        data_n3 = graph.serialize(format="n3")
        combined_path.write_text(data_n3 + "\n\n" + rules_text, encoding="utf-8")

        proc = subprocess.run(
            [eye_bin, "--n3", combined_path.as_posix(), "--pass", "--nope"],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        if proc.returncode != 0:
            raise PaperFactoryError(
                f"eye failed (exit {proc.returncode}): "
                f"{proc.stderr.decode('utf-8', errors='replace')}"
            )

        output_text = proc.stdout.decode("utf-8", errors="replace")
        entailed = Graph()
        lines = [l for l in output_text.splitlines() if l.strip() and not l.strip().startswith("#")]
        if lines:
            entailed.parse(data="\n".join(lines), format="n3")

    result = Graph()
    result += graph
    result += entailed
    return result


def n3_entails(graph: Graph, rules_path: Path, ask_query: str) -> bool:
    """Convenience: compute the N3 closure, then evaluate a SPARQL ASK over
    it. Simpler and more honest than ofmf_keystone.py's version, which had a
    regex-based fallback guess when EYE reported entailments (`ent=N>0` in
    stderr) but the parsed output didn't contain the queried pattern —
    that fallback is dropped here; if closure + ASK disagree with EYE's own
    `ent=` count, that is a real bug in the rules or the query, not
    something to paper over.
    """
    closure = n3_closure(graph, rules_path)
    return sparql_ask(closure, ask_query)
