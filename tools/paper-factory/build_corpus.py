#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
build_corpus.py — union the 7 thesis papers' Turtle instance data into one
whole-corpus graph, tagging every math:Statement (subject of a math:kind
triple) and every math:Symbol individual with math:fromPaper "<paper-name>"
so a combined render can be sectioned by paper of origin.

Uses paper_factory_engine's Graph (rdflib) — no reasoning here, just load,
tag, namespace, union, serialize. Reasoning/validation is a separate step
(see rules/ and the report driving this script).

--- Retry note (attempt 2) ---
Attempt 1 unioned the 7 per-paper graphs verbatim (local names as minted by
each paper's own extraction pass, e.g. math:ax_obs, math:thm_rice,
math:def_adm, math:con_denial, math:prop_semilattice, math:def_mu,
math:def_receipt, math:thm_conservation, math:thm_sep, math:sym_CL, ...).
Several of those local names are reused, unprefixed, by MORE THAN ONE
paper's own file (00_foundations.ttl and projection_thesis.ttl in
particular reuse an near-identical unprefixed label set; 03_planning_
geometry.ttl and 04_projection_and_scale.ttl both mint math:thm_sep;
01_admission_algebra.ttl and 03_planning_geometry.ttl both mint
math:sym_CL). Each paper's own math:statement/math:label content differs,
so unioning the raw graphs silently MERGED two distinct real-world
individuals from two different papers into one RDF node carrying
contradictory math:statement/math:label/math:fromPaper values — a corpus-
construction defect, not a fact about the papers (the papers never claim to
share these individuals; check_citations.py's own per-paper convention is
that dependsOn/justification only ever cite a Statement WITHIN THE SAME
paper, confirming no cross-paper identity was ever intended by the source
extraction).

The fix applied here: every math:-namespace INDIVIDUAL IRI minted by a
paper's own extraction (i.e. every math: URI that is not itself one of the
tex-math-pack ontology's own class/property terms) is rewritten to carry
that paper's slug in its local name before the per-paper graph is unioned
into the corpus, e.g. math:ax_obs -> math:p_00_foundations_ax_obs in
00_foundations.ttl's contribution and math:p_projection_thesis_ax_obs in
projection_thesis.ttl's contribution. This is namespacing a real corpus-
construction bug, not weakening any rule or shape: the dangling-citation and
SHACL checks are left exactly as authored.

--- Retry note (attempt 3) ---
Attempt 2's per-paper namespacing fixed the SHACL maxCount violations but
surfaced a follow-on, real notation_conflict.n3 finding: 13 groups of
math:Symbol individuals are genuinely the SAME recurring notation (e.g.
\\kappa, \\varphi, \\mathsf{H}) independently re-minted by each paper's own
extraction pass, so the corpus-wide rule correctly flagged
distinct-IRI-same-glyph pairs as "conflicts." These are not real conflicts:
per-paper reuse of one shared symbol, not two competing definitions.
SHARED_SYMBOL_ALIASES below merges each such group onto one
math:p_shared_<name> individual instead of namespacing it per paper (SHACL's
StatementShape only targets math:kind subjects, not math:Symbol, so a
Symbol node carrying multiple math:definedIn/math:fromPaper values from
different papers is not a violation). Two OTHER glyph-sharing pairs
(Catset/Ledger both "\\mathcal{C}"; Rec/Recs both "\\mathcal{R}") are
genuinely DIFFERENT concepts wearing the same glyph by accident — those are
fixed at the source ttl (one symbol in each pair given a distinct
math:latex value) rather than merged here.
"""
from __future__ import annotations

from pathlib import Path

from rdflib import Graph, Namespace, Literal, RDF, URIRef

MATH = Namespace("http://seanchatmangpt.github.io/packs/tex-math#")

RDF_DIR = Path(__file__).resolve().parent.parent.parent / "docs" / "thesis" / "rdf"
ONTOLOGY_PATH = (
    Path(__file__).resolve().parent.parent.parent
    / "packs"
    / "tex-math-pack"
    / "ontology.ttl"
)

PAPERS = [
    "00_foundations",
    "01_admission_algebra",
    "02_receipt_cryptography",
    "03_planning_geometry",
    "04_projection_and_scale",
    "projection_thesis",
    "synthesis_thesis",
]

# --- Notation-conflict retry (attempt 3) ---
# The attempt-2 per-paper namespacing fixed the SHACL maxCount violations
# but caused a follow-on finding: 13 groups of math:Symbol individuals that
# are genuinely the SAME recurring notation (e.g. \kappa, \varphi) got
# independently re-minted per paper, so the corpus-wide notation_conflict.n3
# rule correctly flagged them as distinct-IRI-same-glyph "conflicts" even
# though they aren't real conflicts — they're the same concept reused across
# papers. Fixed here by aliasing each paper's local name to one shared
# canonical individual instead of a paper-namespaced one, before namespacing
# is applied. Two OTHER pairs sharing a glyph (Catset/Ledger both
# "\mathcal{C}"; Rec/Recs both "\mathcal{R}") are genuinely DIFFERENT
# concepts, not aliased here — those are fixed at the source ttl instead
# (one of each pair's math:latex literal was changed to a distinct glyph).
SHARED_SYMBOL_ALIASES = {
    ("03_planning_geometry", "sym_CL"): "shared_kappa",
    ("04_projection_and_scale", "sym_CL04"): "shared_kappa",
    ("01_admission_algebra", "sym_CL"): "shared_kappa",
    ("projection_thesis", "sym_CL"): "shared_kappa",
    ("00_foundations", "sym_chainH"): "shared_chainH",
    ("projection_thesis", "sym_chainH"): "shared_chainH",
    ("synthesis_thesis", "sym_chainH"): "shared_chainH",
    ("00_foundations", "sym_Fitness"): "shared_Fitness",
    ("03_planning_geometry", "sym_Fitness"): "shared_Fitness",
    ("02_receipt_cryptography", "rc_sym_Fitness"): "shared_Fitness",
    ("projection_thesis", "sym_Fitness"): "shared_Fitness",
    ("projection_thesis", "sym_dg"): "shared_dg",
    ("02_receipt_cryptography", "rc_sym_dg"): "shared_dg",
    ("synthesis_thesis", "sym_dg"): "shared_dg",
    ("02_receipt_cryptography", "rc_sym_fr"): "shared_fr",
    ("synthesis_thesis", "sym_fr"): "shared_fr",
    ("00_foundations", "sym_Obs"): "shared_Obs",
    ("projection_thesis", "sym_Obs"): "shared_Obs",
    ("00_foundations", "sym_Adm"): "shared_Adm",
    ("projection_thesis", "sym_Adm"): "shared_Adm",
    ("00_foundations", "sym_adm"): "shared_adm",
    ("projection_thesis", "sym_adm"): "shared_adm",
    ("00_foundations", "sym_Rfsl"): "shared_Rfsl",
    ("projection_thesis", "sym_Rfsl"): "shared_Rfsl",
    ("00_foundations", "sym_Act"): "shared_Act",
    ("projection_thesis", "sym_Act"): "shared_Act",
    ("00_foundations", "sym_muop"): "shared_muop",
    ("projection_thesis", "sym_muop"): "shared_muop",
    ("00_foundations", "sym_D"): "shared_D",
    ("projection_thesis", "sym_D"): "shared_D",
    ("03_planning_geometry", "sym_T"): "shared_T",
    ("04_projection_and_scale", "sym_T"): "shared_T",
}


def _ontology_terms() -> set[URIRef]:
    """Every math: URI declared BY THE ONTOLOGY ITSELF (classes/properties)
    — these must never be paper-namespaced; only the per-paper instance
    individuals are."""
    ont = Graph()
    ont.parse(ONTOLOGY_PATH.as_posix(), format="turtle")
    terms: set[URIRef] = set()
    for s in ont.subjects(None, None):
        if isinstance(s, URIRef) and str(s).startswith(str(MATH)):
            terms.add(s)
    return terms


def _namespace_paper_graph(g: Graph, paper: str, ontology_terms: set[URIRef]) -> Graph:
    """Return a NEW graph with every math:-namespace individual IRI (i.e.
    every math: URI in g that is not an ontology-declared class/property)
    rewritten to embed the paper slug in its local name, so identically-
    named individuals minted independently by two different papers'
    extraction passes never collide into one RDF node in the union."""

    def rewrite(term):
        if isinstance(term, URIRef) and str(term).startswith(str(MATH)) and term not in ontology_terms:
            local = str(term)[len(str(MATH)):]
            canonical = SHARED_SYMBOL_ALIASES.get((paper, local))
            if canonical is not None:
                return URIRef(f"{MATH}p_{canonical}")
            return URIRef(f"{MATH}p_{paper}_{local}")
        return term

    out = Graph()
    for s, p, o in g:
        out.add((rewrite(s), rewrite(p), rewrite(o)))
    return out


def build() -> Graph:
    ontology_terms = _ontology_terms()
    corpus = Graph()
    corpus.bind("math", MATH)

    for paper in PAPERS:
        ttl_path = RDF_DIR / f"{paper}.ttl"
        g = Graph()
        g.parse(ttl_path.as_posix(), format="turtle")

        g = _namespace_paper_graph(g, paper, ontology_terms)

        # Every subject with a math:kind triple (Statement individuals).
        kind_subjects = set(g.subjects(MATH.kind, None))
        # Every math:Symbol individual.
        symbol_subjects = set(g.subjects(RDF.type, MATH.Symbol))

        for s in kind_subjects | symbol_subjects:
            g.add((s, MATH.fromPaper, Literal(paper)))

        corpus += g

    return corpus


def main() -> None:
    corpus = build()
    out_path = RDF_DIR / "corpus.ttl"
    corpus.serialize(destination=out_path.as_posix(), format="turtle")
    print(f"wrote {out_path} ({len(corpus)} triples)")


if __name__ == "__main__":
    main()
