# OCEL Replay Report

Real event/object counts from Lane 4's `case-study/ocel_case_study.json`,
re-verified by Lane 6 (sha256 recomputed, event/object counts recounted
directly from the JSON — not copied from prior prose).

## Counts

- **events: 20**
- **objects: 11**
- **declared event types: 16**
- **declared object types: 11**

sha256: `5260a884bd70bb0c598843f9cfa650b67100cc4d057c352ef8adde43ebb8c8cb`

## Event types (16, alphabetical)

```
benchmarks_attached, case_study_finished, case_study_started,
client_smoked, datalog_closed, graphlaw_judgment_emitted,
n3_materialized, ocel_log_written, pddl_plan_generated,
powl_model_compiled, receipts_verified, shacl_validated, shex_validated,
standing_emitted (x5 occurrences — AtLeastOnce), utc_clock_captured,
wasm4pm_process_validated
```

All 16 AT-MINIMUM-required event types are present (per Lane 3/4's driver
spec); `standing_emitted` legitimately occurs 5 times (once per standing
verb the driver runs: `refresh`, `report`, `verify`, `claude_context show`,
`just standing`) — the other 15 types occur exactly once, for 5 + 15 = 20
total events.

## Object types (11, alphabetical)

```
benchmark_result, case_study, client_surface, final_verdict,
graphlaw_judgment, ocel_log, pddl_plan, powl_workflow,
process_validation, receipt_chain, standing_envelope
```

`final_verdict` is a declared PLACEHOLDER object (`standing:
"not_yet_produced"` per Lane 4's report) — Lane 6 did not add a
`final_verdict_rendered` event or resolve this placeholder (see
`POWL_EXECUTION_MODEL.md`'s note on scope: extending Lane 3/4's OCEL log
and process model is outside "evidence manifest, claim promotion, and
generated reports"). `FINAL_VERDICT.md` in this same directory IS the
render of the final verdict, generated directly from
`case-study/final_graphlaw_verdict.json` per deliverable 9 — it is just not
additionally represented as its own OCEL event in this log.

## Lane 3 partial-order deviation: resolved, no mismatch

See `PROCESS_MODEL.md`. Lane 4's real capture conforms to Lane 3's asserted
partial order (`is_conforming: true`, 0 violations) — confirmed independently
by Lane 6 re-running `ocel_process_validate -- ocel_case_study.json --model
case-study` (see `WASM4PM_VALIDATION_REPORT.md`).

## Reused evidence, honestly labeled

The `benchmarks_attached` event (`cs_e16`) carries `reused: "true"` and a
`note` field: "reused from the v26.7.6 OCEL evidence pass; no new benchmark
run was warranted for this case study, numbers not refabricated." This is
the real evidence behind Criterion11's promotion (see
`CLAIM_PROMOTION_TABLE.md`) — a benchmark claim with honestly-disclosed
provenance, not a new number invented for this case study.
