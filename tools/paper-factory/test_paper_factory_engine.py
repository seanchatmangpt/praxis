#!/usr/bin/env python3
"""Smoke tests for paper_factory_engine.py — real rdflib, real pyshacl/owlrl,
real `eye` binary (must be on $PATH; skips the N3 tests with a clear message
if it isn't, rather than silently passing).

Run: python3 -m pytest tools/paper-factory/test_paper_factory_engine.py -v
     or just: python3 tools/paper-factory/test_paper_factory_engine.py
"""

import shutil
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

import paper_factory_engine as pfe  # noqa: E402
from rdflib import Graph  # noqa: E402

TTL = """
@prefix ex: <http://example.org/> .
ex:foo a ex:Widget .
"""
RULES = """
@prefix ex: <http://example.org/> .
{ ?x a ex:Widget } => { ?x ex:isReady true } .
"""


def _graph():
    g = Graph()
    g.parse(data=TTL, format="turtle")
    return g


def test_receipt_hash_is_deterministic():
    g = _graph()
    assert pfe.receipt_hash(g) == pfe.receipt_hash(g)


def test_receipt_hash_changes_with_content():
    g1 = _graph()
    g2 = Graph()
    g2.parse(data=TTL + "\nex:bar a ex:Widget .\n", format="turtle")
    assert pfe.receipt_hash(g1) != pfe.receipt_hash(g2)


def test_sparql_ask_and_select():
    g = _graph()
    assert pfe.sparql_ask(g, "ASK { ?x a <http://example.org/Widget> }")
    rows = pfe.sparql_select(g, "SELECT ?x WHERE { ?x a <http://example.org/Widget> }")
    assert rows == [{"x": "http://example.org/foo"}]


def test_sparql_construct():
    g = _graph()
    delta = pfe.sparql_construct(
        g,
        "CONSTRUCT { ?x <http://example.org/seen> true } WHERE { ?x a <http://example.org/Widget> }",
    )
    assert pfe.sparql_ask(delta, "ASK { <http://example.org/foo> <http://example.org/seen> true }")


def test_n3_closure_and_entails_via_real_eye():
    if shutil.which("eye") is None:
        print("SKIP: eye binary not on $PATH (npm install -g eyereasoner)")
        return
    rules_path = Path("/tmp/paper_factory_test_rules.n3")
    rules_path.write_text(RULES)
    g = _graph()

    closure = pfe.n3_closure(g, rules_path)
    assert pfe.sparql_ask(closure, "ASK { <http://example.org/foo> <http://example.org/isReady> true }")
    assert pfe.n3_entails(g, rules_path, "ASK { <http://example.org/foo> <http://example.org/isReady> true }")


def test_n3_closure_hard_fails_when_eye_missing():
    real_which = shutil.which

    def fake_which(name):
        return None if name == "eye" else real_which(name)

    shutil.which = fake_which
    try:
        try:
            pfe.n3_closure(_graph(), Path("/tmp/does-not-need-to-exist.n3"))
            raise AssertionError("expected PaperFactoryError")
        except pfe.PaperFactoryError:
            pass
    finally:
        shutil.which = real_which


if __name__ == "__main__":
    tests = [v for k, v in list(globals().items()) if k.startswith("test_")]
    failures = 0
    for t in tests:
        try:
            t()
            print(f"PASS {t.__name__}")
        except Exception as e:  # noqa: BLE001
            failures += 1
            print(f"FAIL {t.__name__}: {e}")
    print(f"\n{len(tests) - failures}/{len(tests)} passed")
    sys.exit(1 if failures else 0)
