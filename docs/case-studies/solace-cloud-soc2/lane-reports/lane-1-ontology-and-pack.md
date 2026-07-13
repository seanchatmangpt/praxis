# Lane 1 — Ontology and Pack Structure

Status: DONE (Stage 1 work, re-verified this session).

## Source

`packs/soc2-audit-pack/ontology.ttl`, `packs/soc2-audit-pack/pack.toml`,
`packs/soc2-audit-pack/templates/*.tmpl` (13 templates), rendered to
`crates/cng/tests/fixtures/soc2/*.ttl` and `crates/cng/shapes/soc2-shapes.ttl`.

## What was verified

- **Public-vocabulary-first**: `skos`, `prov`, `dcterms`, `org` only; no private RDF/OWL
  namespace minted for the AICPA TSC taxonomy (confirmed via live web search in Stage 1: no
  public ontology models it; CSA's Cloud Controls Matrix ships JSON/YAML/OSCAL, not RDF/OWL).
- 5 SKOS concept schemes: `SOC2-AUDIT-PHASES` (10), `TSC-CATEGORIES` (5), `CC-CRITERIA` (CC1-9,
  with CC9 correctly split into CC9-1/CC9-2 sub-criteria), `AVAILABILITY-CRITERIA` (A1-1..A1-3),
  `CONFIDENTIALITY-CRITERIA` (C1-1/C1-2).
- Two private namespaces are disclosed, not minted here: the pipeline's own ABI
  (`urn:chatman:engine#pddlDomain`/`#pddlProblem`) and the `powl2:` output vocabulary. Unlike
  `togaf-adm-pack`, this pack carries PDDL only as literal text (no `pddl-strips.ttl` union).

## Concurrent-edit disclosure

At the start of this Stage 2 session, this pack directory had zero uncommitted changes
(`git status --short packs/soc2-audit-pack/` was clean). Partway through Stage 2, a separate
concurrent session began modifying every file in this pack (renaming the case-study company from
Solace Cloud to Arclight) — none of that work is reflected in this lane report, which describes
the pack as committed at `756a258470e19bedc3d12b456d9df7b3030ec76b`. See `CASE_STUDY.md` for the
full disclosure.

## Evidence paths

- `packs/soc2-audit-pack/ontology.ttl`
- `crates/cng/tests/fixtures/soc2/*.ttl` (this bundle's `case-study/pddl/` is a scratchpad copy
  of this exact committed content)
