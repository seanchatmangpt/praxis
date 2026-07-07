# POWL Execution Model

Real process model table from Lane 3 (fixed by Lane 4), re-confirmed by
Lane 6. Source: `case-study/powl_model.json`, produced by
`cargo run --bin ocel_process_validate -- --model case-study` in
`src/bin/ocel_process_validate.rs`.

## Model summary

`ModelReport { alphabet: 16, children: 16, order_pairs: 114 }`

## Children (event-type alphabet, in POWL leaf order)

```
case_study_started, utc_clock_captured, standing_emitted+ (AtLeastOnce),
shacl_validated, shex_validated, n3_materialized, datalog_closed,
pddl_plan_generated, powl_model_compiled, client_smoked, receipts_verified,
benchmarks_attached, graphlaw_judgment_emitted, ocel_log_written,
wasm4pm_process_validated, case_study_finished
```

## Partial order (asserted, see deviation note below)

```
case_study_started < utc_clock_captured < standing_emitted+ <
  {shacl_validated, shex_validated, n3_materialized < datalog_closed} <
  pddl_plan_generated < powl_model_compiled <
  {client_smoked, receipts_verified, benchmarks_attached} <
  graphlaw_judgment_emitted < ocel_log_written <
  wasm4pm_process_validated < case_study_finished
```

114 order pairs after transitive closure over the 16-element alphabet.

## Lane 4's 3 real fixes to Lane 3's original model

1. Added the missing `utc_clock_captured` leaf (Lane 3's original table
   omitted it entirely, though the driver instructions require it as a
   minimum event type).
2. Changed `standing_emitted` from `Once` to `AtLeastOnce` (the real driver
   legitimately runs 5 standing verbs: `refresh`, `report`, `verify`,
   `claude_context show`, `just standing`).
3. Dropped `final_verdict_rendered` from the model entirely — deferred to
   Lane 6 (this document + `FINAL_VERDICT.md`); the order chain goes
   straight from `wasm4pm_process_validated` to `case_study_finished`. A
   placeholder `final_verdict:autonomic-standing-factory` OBJECT still
   exists in `ocel_case_study.json` for a hypothetical future event of this
   type, but Lane 6 did not add that event or extend this process model —
   out of scope for "evidence manifest, claim promotion, and generated
   reports" (see `OCEL_REPLAY_REPORT.md`).

`release-v26.7.6`'s own `CHILD_SPECS`/`ORDER_LABEL_PAIRS` are untouched
(same array literals, values, and order) — confirmed by
`cargo test --bin ocel_process_validate` staying 8/8 green throughout
Lanes 3, 4, and 6.

## Deviation note (Lane 3's own disclosure, carried forward)

The partial order above is ASSERTED from the ticket's specification, not
MINED from an observed trace — no case-study OCEL log existed when Lane 3
built this model. Lane 4's real capture
(`case-study/ocel_case_study.json`) was checked against this asserted order
by Lane 4's `wasm4pm_process_validated` pass (`is_conforming: true,
fitness: 1.0, violations: []`, see `WASM4PM_VALIDATION_REPORT.md`) — the
real execution order the driver produced does conform to this asserted
model. No mismatch was found; the deviation Lane 3 flagged as a
possibility did not materialize.
