#!/usr/bin/env python3
"""check_citations.py — lightweight regex-based checks over a tex-math-pack
Turtle file (no full RDF parser, matching the style of
docs/thesis/check_thesis.py in this repo).

Usage: python3 check_citations.py <path-to-ttl-file>

Checks:
  (a) Citation closure — every IRI target of math:dependsOn or
      math:justification also appears as the subject of a math:kind triple
      somewhere in the file (i.e. resolves to a declared Statement).
  (b) Notation single-source — every math:Symbol individual has a
      math:latex value, and that value is unique among all Symbol
      individuals in the file (no two symbols share one LaTeX rendering).

Known out-of-scope limitation for this pass: we do NOT check whether every
symbol *used* inside a math:statement / math:claim LaTeX-fragment string is
backed by a declared math:Symbol individual — that would require parsing
LaTeX math mode, not just Turtle triples, and is left for a future pass.

Exit code is nonzero if either check fails.
"""

import re
import sys


def strip_comments(text):
    """Drop '#'-led line comments, respecting that '#' can appear inside
    string literals — a full parser would honor quoting, but for this
    lightweight pass we only strip comments on lines with no quote before
    the '#'."""
    out_lines = []
    for line in text.splitlines():
        quote_pos = line.find('"')
        hash_pos = line.find("#")
        if hash_pos != -1 and (quote_pos == -1 or hash_pos < quote_pos):
            line = line[:hash_pos]
        out_lines.append(line)
    return "\n".join(out_lines)


def find_subjects_with_kind(text):
    """Return the set of local names (e.g. 'MyTheorem') that appear as the
    subject of a `math:kind` triple, i.e. declared math:Statement
    individuals."""
    subjects = set()
    # e.g. "math:MyTheorem\n    a           math:Statement ;\n    math:kind ..."
    # We scan block-by-block: split on blank-line-separated stanzas is
    # fragile in Turtle, so instead find every `<subject>\n ... math:kind`
    # pattern by scanning for subject lines followed eventually by
    # math:kind before the next top-level subject terminator ('.').
    pattern = re.compile(
        r"^math:(\w+)\s*\n(?:(?!^math:\w+\s*\n).)*?math:kind\s",
        re.MULTILINE | re.DOTALL,
    )
    for m in pattern.finditer(text):
        subjects.add(m.group(1))
    return subjects


def find_symbol_subjects(text):
    """Return the set of local names that appear as the subject of a triple
    typing them `a math:Symbol`."""
    subjects = set()
    pattern = re.compile(
        r"^math:(\w+)\s*\n(?:(?!^math:\w+\s*\n).)*?\ba\s+math:Symbol\b",
        re.MULTILINE | re.DOTALL,
    )
    for m in pattern.finditer(text):
        subjects.add(m.group(1))
    return subjects


def find_object_refs(text, predicate):
    """Return list of (local_name, line_no) for every `math:<predicate>
    math:<LocalName>` occurrence (IRI object references only; blank nodes
    and literals are not matched)."""
    refs = []
    pattern = re.compile(r"\bmath:" + re.escape(predicate) + r"\s+math:(\w+)")
    for lineno, line in enumerate(text.splitlines(), start=1):
        for m in pattern.finditer(line):
            refs.append((m.group(1), lineno))
    return refs


def find_symbol_latex_values(text):
    """Return list of (subject_local_name, latex_value) for every math:Symbol
    individual's math:latex literal."""
    results = []
    symbol_subjects = find_symbol_subjects(text)
    for subj in symbol_subjects:
        block_pattern = re.compile(
            r"^math:" + re.escape(subj) + r"\s*\n(?:(?!^math:\w+\s*\n).)*",
            re.MULTILINE | re.DOTALL,
        )
        m = block_pattern.search(text)
        if not m:
            continue
        block = m.group(0)
        latex_match = re.search(r'math:latex\s+"((?:[^"\\]|\\.)*)"', block)
        if latex_match:
            results.append((subj, latex_match.group(1)))
    return results


def check_citation_closure(text):
    print("=== Check (a): citation closure ===")
    declared = find_subjects_with_kind(text)
    ok = True
    for predicate in ("dependsOn", "justification"):
        refs = find_object_refs(text, predicate)
        for target, lineno in refs:
            if target not in declared:
                ok = False
                print(
                    f"FAIL  line {lineno}: math:{predicate} targets "
                    f"math:{target}, which has no math:kind triple "
                    f"(not a declared Statement)"
                )
    if ok:
        print(
            f"PASS  all math:dependsOn / math:justification IRI targets "
            f"resolve to a declared math:kind subject "
            f"({len(declared)} declared statement(s))"
        )
    return ok


def check_notation_single_source(text):
    print("=== Check (b): notation single-source ===")
    pairs = find_symbol_latex_values(text)
    ok = True
    by_latex = {}
    for subj, latex in pairs:
        by_latex.setdefault(latex, []).append(subj)
    missing = [subj for subj in find_symbol_subjects(text) if subj not in [s for s, _ in pairs]]
    for subj in missing:
        ok = False
        print(f"FAIL  math:Symbol math:{subj} has no math:latex value")
    for latex, subjs in by_latex.items():
        if len(subjs) > 1:
            ok = False
            print(
                f'FAIL  math:latex "{latex}" is shared by {len(subjs)} '
                f"Symbol individuals: "
                + ", ".join(f"math:{s}" for s in subjs)
            )
    if ok:
        print(
            f"PASS  every math:Symbol has a math:latex value, unique "
            f"among {len(pairs)} Symbol individual(s)"
        )
    return ok


def main():
    if len(sys.argv) != 2:
        print(f"usage: {sys.argv[0]} <path-to-ttl-file>", file=sys.stderr)
        return 2

    path = sys.argv[1]
    with open(path, "r", encoding="utf-8") as f:
        raw_text = f.read()
    text = strip_comments(raw_text)

    closure_ok = check_citation_closure(text)
    notation_ok = check_notation_single_source(text)

    if closure_ok and notation_ok:
        print("\nOVERALL: PASS")
        return 0
    print("\nOVERALL: FAIL")
    return 1


if __name__ == "__main__":
    sys.exit(main())
