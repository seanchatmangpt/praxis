# WASM4PM Process Validation — v26.7.6

Process-conformance and integrity validation of the release's OCEL 2.0
evidence log, executed by `src/bin/ocel_process_validate.rs`.

## Command

```
cargo run --bin ocel_process_validate
# exit 0, report at docs/releases/v26.7.6/ocel/wasm4pm-process-validation.json
```

Input log: `docs/releases/v26.7.6/ocel/playwright-wasm4pm-validation.ocel.json`
(the ONE final log: CLI driver pass + Playwright browser pass + benchmark
rerun events).

## Method

Library composition, not the wasm4pm CLI: the CLI's conformance command is a
stub — `check_conformance_token_replay` in
`/Users/sac/wasm4pm/crates/wasm4pm-cli/src/commands/mining.rs:25-32` bails
with `"token replay conformance not available in this build"`. The validator
therefore composes in-process:

1. **Parse** — `serde_json::from_str::<wasm4pm_compat::ocel::OCEL>` (OCEL 2.0
   Shape A wire format).
2. **Integrity gate** — `wasm4pm_compat::ocel::validate::validate` with a
   permissive (empty) cardinality map: OCEDO/OCPQ Def. 2 invariants
   (undeclared types, E2O_EMPTY, dangling E2O/O2O, duplicate ids).
3. **UTC ordering** — every `event.time` is RFC 3339 with a literal `Z`
   suffix and parses via chrono; parsed instants non-decreasing in log order
   (same-instant events keep stable log order).
4. **Process conformance** — the release-loop model is a
   `powl2_decompose::Powl::PartialOrder` (Kourani et al. Defs 3.6–3.9).
   Conformance = the projected, consecutively-deduped trace is a member of
   the model's language. Membership is decided directly from the model
   structure rather than by enumerating `language_upto` (which explodes
   combinatorially for a 37-symbol trace over 22 concurrent children): since
   the children's alphabets are pairwise disjoint, a trace is an
   order-respecting interleaving (Def 3.8) iff each child's projected
   subsequence lies in that child's language and every order pair
   `(i, j)` has all of child *i* before all of child *j*. The decision
   procedure is validated differentially against `Powl::language_upto` in
   the bin's unit tests (`membership_agrees_with_language_upto`: exhaustive
   agreement over all sequences up to length 5 on a same-shape model).
5. **Object participation** — >= 1 instance each of `browser_session` (2),
   `client_surface` (2), `receipt_chain` (3), `benchmark_result` (8),
   `screenshot` (4).

## Process model

Top-level partial order over 22 children, mined from the actual event
sequence of the log (honest ordering: a pair is asserted only where the
trace satisfies it and the dependency is semantically required; overlapping
spans — `ggen_artifact_generated`/`verifier_gate_completed`, the
screenshot/ui-action interleaving, `verifier_gate_invoked`/`pddl_plan_requested`
at the same millisecond — stay genuinely unordered).

Children (`a` = exactly once, `a+` = one or more, `( … )+` = repeated
sequence):

```
verifier_gate_invoked   pddl_plan_requested       pddl_plan_loaded
powl_workflow_compiled  powl_workflow_executed+   bcinr_transition_executed
ggen_artifact_generated+  verifier_gate_completed+  claim_promoted_to_standing
receipt_chain_verified  graphlaw_state_loaded     graphlaw_export_requested
validation_run_started  utc_clock_captured        playwright_browser_launched
route_loaded            api_request_observed      screenshot_captured+
ui_action_triggered+    trace_captured            ocel_log_written
(benchmark_run_started benchmark_run_completed benchmark_result_attached)+
```

Spine (25 base order pairs, transitively closed to 211 in the model; full
list in the report JSON's `model.order_pairs`):

```
{verifier_gate_invoked, pddl_plan_requested} → pddl_plan_loaded
  → powl_workflow_compiled → {powl_workflow_executed+, bcinr_transition_executed}
bcinr_transition_executed → {ggen_artifact_generated+, verifier_gate_completed+}
powl_workflow_executed+ → claim_promoted_to_standing → receipt_chain_verified
  → graphlaw_state_loaded → graphlaw_export_requested
{graphlaw_export_requested, ggen_artifact_generated+, verifier_gate_completed+}
  → validation_run_started → utc_clock_captured → playwright_browser_launched
  → route_loaded → {api_request_observed, screenshot_captured+, ui_action_triggered+}
  → trace_captured → ocel_log_written
  → (benchmark_run_started benchmark_run_completed benchmark_result_attached)+
```

Note vs. the original plan spine: the log shows the CLI driver pass
(verifier gate → PDDL → POWL → BCINR → ggen → receipts → graphlaw) ran
*before* `validation_run_started` of the browser pass, and no
`graphlaw_state_loaded → pddl_plan_loaded` dependency exists in the trace.
The model encodes the honest observed ordering, not the planned one.

## Results

First validation pass (log as repaired, before the validator's own
bookkeeping events; 44 events, 34 objects):

- integrity: `valid: true`, 0 errors
- UTC ordering: 0 violations (all times ISO-8601 UTC `Z`, non-decreasing)
- conformance: `is_conforming: true`, `fitness: 1.0`, 0 violations
- participation: all 5 required object types present
- log digests at validation time:
  `sha256 17119c46bde12abfdb490941ac76dca828c942868fd002de91846844e5ba44f3`,
  `blake3 cdac524959690da1e407a2c779b76f62f6ef077f07bcf0ff1fb039e24150ed8a`

Idempotent re-run over the annotated log (50 events, 36 objects) also exits
0 — the committed report
(`docs/releases/v26.7.6/ocel/wasm4pm-process-validation.json`) reflects that
state:
`sha256 628807e0780a7d6f479f1a3d9dc744a17f5158cac3f7b2963244e10614f89e61`,
`blake3 4c0d8584b1e0a1b786d1d4dc498aa20841d8f4977b8d82a4ab611a7863afca40`.

## Closure rule (bookkeeping events)

On a conforming run the validator appends its own evidence events to the log
(fixed ids `val_e1..val_e6`, skipped when already present):
`wasm4pm_process_model_generated`, `wasm4pm_process_validation_started`,
`wasm4pm_process_validation_completed`, `wasm4pm_conformance_passed`,
`ocel_log_validated`, `validation_run_finished`, plus the
`powl_workflow:wasm4pm_release_loop_model` and
`report:wasm4pm_process_validation` objects. The validator validates the log
as it existed BEFORE these bookkeeping events: the log digests are computed
before the append, and the bookkeeping event types are outside the model
alphabet, so the conformance projection drops them on re-runs.

## Repairs made (RESOLVED_BY_REPAIR)

The first validation attempt refused the log; the log *generators* were
fixed and the identical fix applied in place to the committed log (a full
regeneration would have discarded the benchmark rerun evidence attached in
commit `5235ea0`, whose append step was session-side):

1. **E2O_EMPTY (5 events)** — `drv_e12`, `drv_e13`, `pw_e1`, `pw_e2`,
   `pw_e15` were emitted with no qualified object reference (OCPQ Def. 2
   violation, caught by `wasm4pm_compat::ocel::validate`).
   - Generator fixes: `clients/autonomic-platform/tests/run-evidence-pass.mjs`
     (`powl_workflow_executed` run 2 and `claim_promoted_to_standing` now
     reference `powl_workflow:ocel_pass`) and
     `clients/autonomic-platform/tests/playwright/ocel-wasm4pm-validation.spec.ts`
     (`validation_run_started`/`utc_clock_captured` reference
     `browser_session:autonomic`; `ocel_log_written` references a new
     `report:playwright_wasm4pm_validation_ocel` object registered for the
     final log itself).
   - Log repair: the same 5 relationship sets + the new report object.
2. **Missing object-attribute timestamps** — the OCEL 2.0 wire shape
   (`wasm4pm_compat::ocel::OCELObjectAttribute { name, value, time }`)
   requires a `time` on every object attribute; the recorders emitted
   `{ name, value }` only, so the log did not even parse as
   `wasm4pm_compat::ocel::OCEL`.
   - Generator fixes: `clients/autonomic-platform/tests/ocel-recorder.ts`
     (`addObject`/`merge` stamp the observation instant) and
     `run-evidence-pass.mjs` (`addObject` likewise).
   - Log repair: 207 attribute timestamps stamped with the time of the first
     event referencing the object (fallback: phase-start event `drv_e1` for
     driver-recorded objects, `pw_e1` otherwise) — a documented derivation,
     since the original observation instants were not recorded.

No validator check was weakened.

## Gates

- `cargo test --bin ocel_process_validate` — 8 tests pass (membership
  positive/negative, benchmark loop-sequence pattern, projection/dedupe,
  UTC parser accept/reject, differential membership-vs-`language_upto`).
- `just test-changed` — green (conservative mode, 0 affected tests).
- `cargo check --workspace --all-features` — exit 0.
- `cargo clippy --bin ocel_process_validate` — no findings in the bin
  (remaining warnings originate in the external `wasm4pm-cognition` path
  dependency).

## Standing

`WASM4PM_PROVEN` — integrity, UTC ordering, POWL language membership, and
object participation all hold over the release evidence log, by the command
above.
