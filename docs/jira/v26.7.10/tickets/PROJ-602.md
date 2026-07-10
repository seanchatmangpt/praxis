# PROJ-602 — Add `cng evidence replay` verb for third-party auditors

Status: PLANNED

Add a new CLI verb usable by a party who did not produce the original run: takes an observation
graph, a queries directory, and an expected digest, independently re-derives the OCEL evidence
graph (`G_OCEL = CONSTRUCT_P(G_OBS)`, see `docs/releases/v26.7.10/ARD.md` Sec. 3), compares
against the expected `ocel_graph_digest`/`sparql_result_digest`, and exits 0/nonzero. Distinct
from the existing `cng benchmark verify`, which only re-derives the POWL manufacture digest
(`crates/cng/src/bench.rs` `verify()`, ~2050-2115) and never checks OCEL/SPARQL digests. Links
back to `docs/releases/v26.7.10/PRD.md` (Claims Reconciliation row 4) and `RELEASE_CONTROL.md`
Sec. 5.

Implementation detail: `docs/releases/v26.7.10/IMPLEMENTATION_SPEC.md` (exact edits,
anchors, tests, and acceptance commands for this ticket).
