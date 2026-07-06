# v26.7.6 — OCEL / Playwright / wasm4pm Completion Control Document

All prior release feedback is treated as UNTRUSTED_CLAIM_PENDING_OCEL_AND_WASM4PM_REPLAY until this pass produces Playwright-triggered OCEL v2 evidence and wasm4pm process validation.

## Evidence reversal doctrine

No claim from any earlier release document, review, or chat transcript is standing.
Each claim starts as `UNTRUSTED_CLAIM_PENDING_OCEL_AND_WASM4PM_REPLAY` and is promoted
only when this pass emits a `claim_promoted_to_standing` event into the OCEL v2 log,
backed by a Playwright-driven interaction, a receipt-chain verification, or a wasm4pm
replay/conformance result. A claim without an OCEL event id is not a claim; it is a
draft. Every promotion cites the command that produced it and the OCEL event id it
produced.

## No-terminal-blockers doctrine

This pass may not finish with a terminal blocker. Every open item resolves through one
of the eight statuses defined in `NO_TERMINAL_BLOCKERS.md` (same directory):
`TEMP_BLOCKED` (transient only, forbidden in final standing), `RESOLVED_BY_REPAIR`,
`RESOLVED_BY_EXISTING_SURFACE`, `RESOLVED_BY_LOCAL_EQUIVALENT`,
`RESOLVED_BY_SCOPE_RECLASSIFICATION`, `RESOLVED_BY_EXTERNAL_OPERATOR_SIDE_EFFECT`,
`OCEL_PROVEN`, `WASM4PM_PROVEN`. Resolution routes are: repair in place, use of an
already-existing surface, a local equivalent (library composition instead of a stubbed
CLI), reclassification out of release scope with cited grounds, or classification as an
external operator side effect (actions only a human operator with credentials can take).

## Client surfaces discovered

| Surface | Stack | Role | Status |
|---|---|---|---|
| autonomic-platform | Vite, port 5173 | Primary, release-critical. NavRail `getByTitle` buttons; `/praxis-artifacts/*` endpoints; `?mode=mock` vs `?mode=praxis` switch. | Target of Playwright evidence |
| optimus | Next.js, port 3000 | Secondary; has an existing Playwright harness. | Reused as-is |
| pcp | Vite | Build broken; repairable. | RESOLVED_BY_REPAIR (planned; see ledger) |
| dashboard.bak | Nuxt | 17+ pages depend on Nuxt UI Pro (paid license). | RESOLVED_BY_SCOPE_RECLASSIFICATION |
| wasm4pm/apps/playground-web | Nuxt | Promoted canonical Nuxt shell, replacing dashboard.bak. | RESOLVED_BY_EXISTING_SURFACE |

## Playwright entrypoints

- autonomic-platform: launch Vite dev server on 5173; Playwright navigates `/`,
  drives NavRail via `getByTitle(...)` locators, exercises `?mode=mock` and
  `?mode=praxis`, and observes `/praxis-artifacts/*` API requests.
- optimus: run the existing Playwright harness at port 3000 unchanged; capture its
  traces and screenshots into this pass's evidence directory.
- wasm4pm/apps/playground-web: launch the Nuxt shell and drive the OCEL playground
  routes as the canonical Nuxt evidence surface.
- Every Playwright action emits `ui_action_triggered`; every observed fetch emits
  `api_request_observed`; each session emits `playwright_browser_launched` and
  `screenshot_captured` / `trace_captured` events.

## OCEL v2 object model

26 object types (wire shape per `wasm4pm_compat::ocel::OCEL`, Shape A; sample at
`/Users/sac/wasm4pm/fixtures/world/ocel-v2.json`):

`release`, `validation_run`, `browser_session`, `client_surface`, `route`,
`ui_action`, `api_request`, `graphlaw_state`, `pddl_plan`, `powl_workflow`,
`bcinr_transition`, `ggen_artifact`, `verifier_gate`, `receipt_chain`,
`benchmark_run`, `benchmark_result`, `wasm4pm_process_model`,
`wasm4pm_validation_run`, `wasm4pm_conformance_result`, `screenshot`,
`trace_file`, `report_artifact`, `arxiv_package`, `crate_package`,
`operator_side_effect`, `repair_action`, `local_substitution`.

## OCEL v2 event model

Event types for this pass:

`validation_run_started`, `utc_clock_captured`, `playwright_browser_launched`,
`client_surface_discovered`, `client_surface_repaired`,
`client_surface_build_checked`, `route_loaded`, `ui_action_triggered`,
`api_request_observed`, `graphlaw_export_requested`, `graphlaw_state_loaded`,
`pddl_plan_requested`, `pddl_plan_loaded`, `powl_workflow_compiled`,
`powl_workflow_executed`, `bcinr_transition_executed`, `ggen_artifact_generated`,
`verifier_gate_invoked`, `verifier_gate_completed`, `receipt_chain_verified`,
`benchmark_run_started`, `benchmark_run_completed`, `benchmark_result_attached`,
`wasm4pm_process_model_generated`, `wasm4pm_process_validation_started`,
`wasm4pm_process_validation_completed`, `wasm4pm_conformance_passed`,
`wasm4pm_replay_passed`, `screenshot_captured`, `trace_captured`,
`report_generated`, `ocel_log_written`, `ocel_log_validated`,
`claim_promoted_to_standing`, `claim_resolved_by_repair`,
`claim_resolved_by_local_equivalent`,
`claim_resolved_by_external_operator_side_effect`, `validation_run_finished`.

## UTC timestamp policy

- OCEL event `time` fields are ISO-8601 UTC with `Z` suffix. Evidence time is wall
  clock — it records when evidence was captured, and it is never an input to any
  hash.
- Hash paths remain wall-clock-free: receipt records are hashed with `ts_ns: 0`
  (`crates/ggen/src/sync.rs:945`), and `crates/praxis-core/src/receipt_validator.rs`
  validates `ts_ns` monotonicity against an injected clock, not the system clock.
  (Note: the pass spec cites `crates/praxis-core/src/plan_run.rs`; that path does not
  exist in this tree — the `ts_ns: 0` hash-path invariant lives in
  `crates/ggen/src/sync.rs` and `crates/ggen/tests/receipt_chain_e2e.rs`.)
- One `utc_clock_captured` event opens each validation run, pinning the evidence
  clock used for the rest of the log.

## Benchmark attachment plan

Run the 3 divan `blue_river_dam` benches; each emits `benchmark_run_started` /
`benchmark_run_completed` and a `benchmark_result_attached` event linking the
`benchmark_result` object to the `validation_run`.

## Receipt attachment plan

- `receipt validate` across its 5 stages, each stage a `verifier_gate_invoked` /
  `verifier_gate_completed` pair.
- `ggen receipt verify` and `ggen receipt history` emitting
  `receipt_chain_verified`.
- `receipt export-ocel` emitting `ocel_log_written`, then validated
  (`ocel_log_validated`) via `wasm4pm_compat::ocel::validate`.

## GraphLaw evidence plan

Run the 5 `graphlaw_e2e` tests plus `ggen law derive`; loads and exports emit
`graphlaw_export_requested` / `graphlaw_state_loaded` against `graphlaw_state`
objects.

## Planner loop evidence plan

Run `plan run` twice under identical inputs; assert `powl_chain_hash` determinism
across the two runs (equal hashes, `ts_ns: 0` in the hash path). Events:
`pddl_plan_requested`, `pddl_plan_loaded`, `powl_workflow_compiled`,
`powl_workflow_executed`, `bcinr_transition_executed`.

## wasm4pm validation plan

The lawful path is library composition, not the CLI: wasm4pm's CLI conformance
command is stubbed. Instead:

- `wasm4pm_compat::ocel::validate` over the written OCEL log
  (`ocel_log_validated`, `wasm4pm_process_validation_started/completed`).
- `powl2_decompose` partial-order language membership check over the executed
  traces (`wasm4pm_conformance_passed`, `wasm4pm_replay_passed`).

## Commands run

PENDING — populated by later phases; every entry cites command, exit code, and
OCEL event ids.

## Repairs made

PENDING — each repair emits `claim_resolved_by_repair` and cites the diff.

## Substitutions made

PENDING — each local equivalent emits `claim_resolved_by_local_equivalent`.

## External operator side effects

PENDING — crates.io publish, arXiv submission, repository visibility changes; each
emits `claim_resolved_by_external_operator_side_effect` against an
`operator_side_effect` object.

## Final standing

PENDING — updated by the closing phase; no item may stand as `TEMP_BLOCKED`
(see `NO_TERMINAL_BLOCKERS.md`).
