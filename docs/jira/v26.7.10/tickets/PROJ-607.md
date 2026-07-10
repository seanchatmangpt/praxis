# PROJ-607 — Doc reconciliation pass (tickets, ARD, BENCHMARK.md, control table)

Status: CLOSED (delivered this session)

## Summary

Reconcile the v26.7.10 document set with reality: flip PROJ-601..605 ticket files from PLANNED
to CLOSED citing commit `40f6020`; de-stub `docs/releases/v26.7.10/ARD.md`; fix the
"601..505" typo to "601..605"; create `crates/cng/BENCHMARK.md` (referenced by
`crates/cng/src/bench/run.rs` and `report.rs`); add the PROJ-606..622 next-increment rows plus
the ChatmanEngine-deferred note to `RELEASE_CONTROL.md` Sec. 8.

## Acceptance criteria

1. `docs/jira/v26.7.10/tickets/PROJ-601.md`..`PROJ-605.md` say Status: CLOSED and cite
   `40f6020` plus the `RELEASE_CONTROL.md` Sec. 7 verification ladder.
2. No "601..505" string remains under `docs/releases/v26.7.10/`.
3. `crates/cng/BENCHMARK.md` exists.
4. `RELEASE_CONTROL.md` Sec. 8 carries one row per PROJ-606..622 ticket and the
   ChatmanEngine-deferred sentence.

## Verification

Delivered this session (commit `40f6020` closed the code side; this pass closed the doc side):
PROJ-601..605 files read Status: CLOSED; `RELEASE_CONTROL.md` Sec. 8 table exists with 17
rows; `crates/cng/BENCHMARK.md` exists on disk.

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md`
- `docs/releases/v26.7.10/RELEASE_CONTROL.md` Sec. 7 and Sec. 8
