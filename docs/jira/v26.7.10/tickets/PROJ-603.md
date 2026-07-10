# PROJ-603 — Bundle manifest schema naming every input/output digest

Status: CLOSED

Closed by commit `40f6020` (`EvidenceManifest` written to
`<bench_dir>/results/evidence-manifest.json`, `signatures: []` emitted empty). Verification
evidence recorded in `docs/releases/v26.7.10/RELEASE_CONTROL.md` Sec. 7 (ladder item 9: auditor
replay consumed the manifest from the relocated bundle alone) — this ticket cites that record
rather than re-asserting it.

Define a single JSON manifest schema naming every digest currently scattered across
`digests.json` (`{set_dir_path: powl_digest}` only) and separate `RunReport` fields
(`ocel_graph_digest`, `sparql_result_digest`): `obs_digest`, `query_digests{}`,
`ontology_digests{}`, `rules_digest`, `ocel_graph_digest`, `sparql_result_digest`,
`measurement_class`, and a `signatures: []` placeholder field explicitly commented as
unpopulated pending a signing decision — never a fake signature. No ontology-digest or
rules-digest field exists anywhere in the codebase today; this ticket creates them. Links back
to `docs/releases/v26.7.10/PRD.md` (Claims Reconciliation row 8) and `RELEASE_CONTROL.md` Sec.
5.

Implementation detail: `docs/releases/v26.7.10/IMPLEMENTATION_SPEC.md` (exact edits,
anchors, tests, and acceptance commands for this ticket).
