# THESIS_GROUNDING — v26.7.13 Formal Thesis vs. Repository Record

Status: DISCLOSED companion to `THESIS.md` (adopted verbatim from
`Multifractal_Workflow_PhD_Thesis_v26.7.13_Formal.md`, 2026-07-13). The thesis's own Sec. 26.1
states its standing ledger is "project-reported rather than independently reproduced" where it
did not rerun the repository. This file records what *was* independently re-checked against the
live repository at adoption time, and every delta found. On any disagreement about current
standing, `RELEASE_CONTROL.md` and the Claims Reconciliation tables win — the thesis is the
formal argument, not the standing authority.

## What was re-checked (2026-07-13, this session)

### 1. Sec. 26.4 dual-crown standing — AGREES on the bottom line; edge inventory refined

The thesis asserts `LocalCrownReal = false`, `ExternalCrownReal = false`,
`ObservationToReplayContiguousPath = false`, attributing this to the F08→F09 residual-goal
extraction gap. Checked against `docs/jira/v26.7.12/CROWN_STATUS.md` (the authoritative,
twice-re-audited edge ledger):

- All three `false` verdicts CONFIRMED — the repo record agrees under its strict
  "every edge must be a full `REAL_EDGE`" reading.
- Refinement 1: the thesis's phrase "genuine missing edge" for F08→F09 is stronger than the
  current record. `MISSING_EDGE_COUNT` is **0** on both paths; F08→F09 is
  `PARTIAL_REAL_EDGE` — real `?`-gated sequencing, but F09 re-derives its continuation goal
  from the caller's inputs rather than consuming F08's produced `Pddl8Tape` (no residual-goal
  extractor exists). Partial data-threading, not missing code.
- Refinement 2: F08→F09 is not the only blocker. LOCAL is additionally blocked by F18→F19
  (`PARTIAL_REAL_EDGE`, control-gated only — reclassified deliberately, with the record
  documenting why threading the broker receipt into `resolve_hook_for_action` would be a
  fabricated dependency); EXTERNAL is additionally blocked by F10→F12 (F10 does not itself
  synthesize the `ExternalCut` node it is wrapped in). The thesis under-counts blockers; the
  direction of its conclusion is unaffected.

### 2. Sec. 26.6 / Abstract dry-run blocker list — ACCURATE but non-exhaustive

The thesis names "unversioned path dependencies, license gaps, a missing root license, and
path leakage" — these map to B1, B2/B3, B5, B6 of the authoritative B1–B7 taxonomy in
`DRY_RUN_PUBLISH_VERDICT.md` / `docs/PUBLISH_ALL_PRAXIS_PLAN.md`. Omitted: **B4**
(`tmp_sparql2` entirely git-ignored, zero packageable files) and **B7** (`praxis-lean`
untracked-but-not-ignored files would ship — re-confirmed still open this session via
`git status --short`). This is the same non-exhaustiveness previously recorded for the
Operation Dogfood PRD's C8 claim; the REFUSED verdict itself is CONFIRMED.

### 3. Sec. 26.2 claim table (C1–C12) — matches the PRD; C3/C7 predate Increment 1

The table is byte-consistent with `OPERATION_DOGFOOD_PRD.md`'s independently verified
standings. One staleness relative to work committed this cycle, covered by the thesis's own
Sec. 26.1 evidence boundary but recorded here: C3 and C7 read PLANNED, yet Operation Dogfood
Increment 1 has since landed (committed: `packs/dogfood-lifecycle-pack/` ontology + SHACL
shapes + templates; PostToolUse hook capturing nine tool types as `dfl:ToolEvent` Turtle with
content-addressed BLAKE3 payloads; session-end validation + receipt recipe; acceptance slice
verified including the malformed-node falsifier). Against C7's own promotion test ("complete
pre/post lifecycle coverage with zero orphans") this is PARTIAL, not ALIVE: capture is
post-event only, there is no pre/post intent pairing, and no zero-orphan verifier runs. C3
similarly gains real tool-event capture but no task/claim binding. Neither claim is promoted
by this note; the delta is disclosed so the thesis's PLANNED rows are read with their date.

### 4. Sec. 26.3 formalization progress — NOT re-verified here

The mfact-rail numbers (Rail A landed, 56 orphans integrated, 33 remaining, graft_child,
F09→F10 edge) are project-reported from a separate repository's record. The thesis discloses
this itself; this grounding pass did not independently rerun mfact. The F09→F10 edge claim is
consistent with `CROWN_STATUS.md` row 4 (`REAL_EDGE`, confirmed).

## What was NOT re-checked

The mathematical content (Parts II, IV, VII–VIII), proofs, and the Lean-sketch appendices were
not machine-checked in this pass; per the thesis's own Declaration of Epistemic Discipline,
THEOREM-class statements carry their proofs in-text and KERNEL_CHECKED status belongs to the
mfact rail, not to this document's adoption.

## References

- `THESIS.md` — the adopted document (verbatim)
- `RELEASE_CONTROL.md` — the standing authority this file defers to
- `OPERATION_DOGFOOD_PRD.md` — the C1–C12 table of record and its Grounding Appendix
- `DRY_RUN_PUBLISH_VERDICT.md` — authoritative dry-run gate breakdown (B1–B7 by reference)
- `docs/jira/v26.7.12/CROWN_STATUS.md` — authoritative crown-witness edge ledger
