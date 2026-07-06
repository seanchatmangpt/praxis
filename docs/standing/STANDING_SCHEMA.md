# Standing Schema

The standing schema (statuses, readiness ladder, artifact/document JSON
shape, evidence model, scoped-readiness validation rule, OCEL/wasm4pm
relation, and consumer responsibilities) is no longer maintained here.
The schema of record now lives at:

- `/Users/sac/cargo-cicd/docs/reference/standing-schema.md` (human-readable
  schema of record)
- `/Users/sac/cargo-cicd/crates/cargo-cicd-core/src/standing/model.rs`
  (the matching Rust types — `serde`-derived, same field names,
  `SCREAMING_SNAKE_CASE` status tags)

Canonical schema id: `cicd-standing.v1`. `praxis-standing.v1` is accepted
as a legacy alias on read (documents predating the rename, or emitted by
consumers still on the old id, continue to parse).

Read the cargo-cicd doc above before treating any `standing.json` as
truth about praxis release readiness. Do not maintain a second copy of
the generic schema in this file — extend it in cargo-cicd, then link
back here if praxis needs a pointer update.

## praxis-specific addendum

The generic schema above is workspace-agnostic. The following is
specific to how praxis uses it and is not part of the schema of record:

- `docs/releases/v26.7.6/NO_TERMINAL_BLOCKERS.md` defines an 8-status
  resolution vocabulary (`TEMP_BLOCKED`, `RESOLVED_BY_REPAIR`,
  `RESOLVED_BY_EXISTING_SURFACE`, `RESOLVED_BY_LOCAL_EQUIVALENT`,
  `RESOLVED_BY_SCOPE_RECLASSIFICATION`,
  `RESOLVED_BY_EXTERNAL_OPERATOR_SIDE_EFFECT`, `OCEL_PROVEN`,
  `WASM4PM_PROVEN`) for *how a blocker on a single pass was closed*. The
  standing schema supersedes/extends that for artifact-level tracking:
  `OCEL_PROVEN` and `WASM4PM_PROVEN` reappear there as standing statuses
  on individual artifacts (not just ledger rows), and the
  resolution-route statuses fold into `external_operator_side_effects` /
  evidence entries on the artifact rather than a separate ledger
  vocabulary.
- `docs/releases/v26.7.6/CLAIM_PROMOTION_TABLE.md` is the hand-written,
  per-release artifact of claim promotions (claim, prior status,
  evidence, OCEL event ids, final standing). A compiled `standing.json`
  produced under this schema is intended to eventually supersede that
  hand-written table: every row of the promotion table becomes a
  `StandingArtifact` with its `standing` list and `evidence` array
  populated from the same OCEL/wasm4pm event ids and artifact paths
  cited there today.
