# RELEASE_CONTROL — Chatman Engine v26.7.9

Single control surface for `PRD.md` and `ARD.md` in this directory. Both documents' Status
lines tie to this file. If this file and either document disagree, this file wins.

## 1. Evidentiary floor

The interim evidentiary floor for this release is the Gate F audit:
`docs/chatman-engine/chicago_tdd_final_report.md`, verdict `ADMITTED_DRY_RUN_PUBLISHABLE`,
audited against commit `7d76019` by a non-authoring session. Gates A, B (with C folded in), D,
and E all PASS; mutation score and line coverage (Gate E items 4-5) are explicitly advisory /
UNVERIFIED. The verdict is scoped strictly to the S1-S6 core admission/planning/workflow-check/
receipt/replay/evidence pipeline in `crates/praxis-graphlaw/src/chatman/`.

## 2. Six named exclusions (verbatim, reused identically in PRD.md and ARD.md)

1. N3 cubic-scaling work — untouched by this release.
2. Deferred S3→S4 `OrchestratedPlan`/`TapeBridge` engine-side wiring (`bridge.rs` exists;
   full wiring not complete).
3. PROJ-415 — SHACL `CompiledShape` population (`crates/praxis-graphlaw/src/shacl/model.rs`).
4. PROJ-416 — Pattern-4 canonical renders not wired to BLAKE3 receipt hashing
   (`shacl/equivalence.rs`).
5. PROJ-417 — `Status::HashMismatch` not yet surfaced in the WASM crate's `verify_replay` for a
   full pipeline re-run.
6. Crate-wide non-Chatman clippy debt (pre-existing, outside `src/chatman`).

## 3. Standing-index disclosure

Per `docs/standing/CLAUDE_CODE_POLICY.md` ("the index wins" when index and docs disagree): the
compiled `target/praxis-standing/standing.json` / `docs/standing/REALITY_INDEX.md` show only
`crate:chatman-common` (and `crate:praxis-graphlaw` under its own name) at ladder `0`,
`DISCOVERED`. This is a documented schema gap — no `MilestoneArtifact` kind exists to represent
a scoped Gate F verdict (`docs/standing/STANDING_SCHEMA_MILESTONE_GAP.md`) — not a refutation of
Gate F. Both PRD.md and ARD.md must disclose this ladder-0 reading alongside the Gate F verdict
without letting either silently override the other.

## 4. Claims Reconciliation table — single logical table, two files

The `## Claims Reconciliation` section in `PRD.md` and `ARD.md` is one logical table maintained
in two places. Any status change requires updating both files in the same commit. PROJ ticket
numbers cited there must match tickets under `docs/jira/v26.7.8/tickets/`.

## 5. Open items tracked against ticket status

PROJ-411..414 (CE-ABI envelopes, dependency chain, direct-operation pipeline, receipt
generation) are marked "IN PROGRESS" in their individual ticket files
(`docs/jira/v26.7.8/tickets/PROJ-41{1..4}.md`) despite passing Gate F — this is a known,
disclosed discrepancy (paperwork not yet synced to the Gate F closure), tracked in
`docs/jira/v26.7.8/tickets/PROJ-411-417-reconciliation.md`. PROJ-415/416/417 remain OPEN and are
explicitly excluded from this release's verdict (Sec. 2 above).

## 6. Documents governed by this control surface

- `docs/releases/v26.7.9/PRD.md`
- `docs/releases/v26.7.9/ARD.md`
