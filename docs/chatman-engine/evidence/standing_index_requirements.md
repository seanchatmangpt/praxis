# Standing Index Requirements — post-Gate-F only

Do not run `just standing` until `docs/chatman-engine/chicago_tdd_final_report.md` contains an
auditor-written verdict.

## Required entry shape
- artifact id: Chatman Engine v26.7.9
- scope (must be non-empty, per PRODUCTION_READINESS.md's rung-7/8/9 gating rule): "Chatman
  Engine v26.7.9 S1–S6 core admission, planning, workflow-check, receipt, replay, and evidence
  pipeline, excluding N3 cubic-scaling work and excluding the deferred S3→S4 OrchestratedPlan
  projection."
- exclusions listed explicitly (same as above) — do not let them appear only implicitly.
- evidence chain: links/paths to Gates A–F evidence files under `docs/chatman-engine/evidence/`.
- verdict source: the auditor's `chicago_tdd_final_report.md`, not this session's self-report.

## Ladder rung
Computed only via `cargo_cicd_core::standing::model::compute_ladder_level` (run via `just
standing` → `cargo-cicd standing refresh`), never hand-edited. If the computed rung disagrees
with the expected rung given the verdict, report the discrepancy — do not edit
`standing.json`/`standing.ttl` by hand to force agreement.
