# OCEL v2 + wasm4pm Evidence Report — v26.7.6 "After Neon"

Human-readable narrative of the Playwright-triggered OCEL 2.0 evidence pass
and its wasm4pm process validation. Companion machine artifacts:
`ocel/playwright-wasm4pm-validation.ocel.json` (the ONE final log),
`ocel/wasm4pm-process-validation.json` (validator report),
`ocel/utc-window.json` (evidence clock window), `ocel/raw/*` (raw captures).
Doctrine and status vocabulary: `OCEL_PLAYWRIGHT_WASM4PM_COMPLETION.md`,
`NO_TERMINAL_BLOCKERS.md`. Claim-by-claim standing:
`CLAIM_PROMOTION_TABLE.md`.

## 1. What was triggered

One evidence run, `run_id ocel-evidence-2026-07-06T19-10-42-924Z`, in three
phases, all timestamps ISO-8601 UTC `Z` (evidence time only — never a hash
input; hash paths stay at `ts_ns: 0`, `crates/ggen/src/sync.rs:945`).

### Phase A — CLI driver pass (events `drv_e1..drv_e20`)

Driver: `clients/autonomic-platform/tests/run-evidence-pass.mjs`. It built
the binary, ran the full loop twice, validated receipts, ran the GraphLaw
gate, and exported the ledger OCEL — each command captured to
`ocel/raw/*.txt` with UTC header, exit code, and sha256 (hashes carried in
the events' `evidence_refs`).

### Phase B — Playwright browser pass (events `pw_e1..pw_e15`)

Spec: `clients/autonomic-platform/tests/playwright/ocel-wasm4pm-validation.spec.ts`,
chromium 149.0.7827.55 against the autonomic-platform Vite dev server at
`http://localhost:5173` (`browser_session:autonomic`). UTC window
(`ocel/utc-window.json`): 2026-07-06T19:10:49.311Z → 19:10:50.604Z.

- Route loaded: `/` with `?mode=praxis` (`pw_e4`).
- UI actions (Playwright `getByTitle(...).click`): NavRail **Operations**
  (`pw_e9`), NavRail **Definition of Done** (`pw_e11`), NavRail **Model
  Deck** (`pw_e13`).
- API observations against the `praxisArtifacts` middleware:
  - `GET /praxis-artifacts/receipt.json` → 200, chain head
    `35bc4ab0c984ed5198e2609ec771f17a24d020d6e6882c2bb82ea6feab04765a`
    (`pw_e5`) — the browser observed the same head the CLI verified.
  - `GET /praxis-artifacts/receipt-log.jsonl` → 200, 8 records, same head
    (`pw_e6`).
  - `GET /praxis-artifacts/plan.json` → 200, plan ref
    `target/plan_run/after_neon_det/plan.json` (`pw_e7`).
- Screenshots: `ocel/raw/screenshot-command.png` (`pw_e8`),
  `screenshot-ops.png` (`pw_e10`), `screenshot-dod.png` (`pw_e12`); optimus
  secondary surface `screenshot-optimus.png` (objects
  `browser_session:optimus`, `client_surface:optimus`, `screenshot:optimus`).
- Trace: `pw_e14`, `trace_path`
  `clients/autonomic-platform/test-results/ocel-wasm4pm-validation-OC-5467e-idation-over-real-artifacts-chromium/trace.zip`
  (Playwright `trace=on`, 1.19 MB on disk).
- `pw_e15 ocel_log_written` against `report:playwright_wasm4pm_validation_ocel`.

### Phase C — Benchmark reruns (events `pw_e36..pw_e44`) and validation

Three divan `blue_river_dam` benches rerun and attached (commit `5235ea0`),
then the whole log validated by `cargo run --bin ocel_process_validate`
(commit `81ab966`).

## 2. Commands run (UTC windows, exit codes, hashes)

| Command | UTC window (2026-07-06) | Exit | Raw capture (sha256) |
|---|---|---|---|
| `cargo build --features ggen --bin my-conforming-project` | 19:10:42.925Z → 19:10:43.282Z | 0 | `ocel/raw/cargo-build.txt` |
| `my-conforming-project plan run --goal examples/v26_7_6_after_neon/goal.ttl --out-dir target/plan_run/ocel_pass --receipts-dir target/plan_run/ocel_pass_receipts` | 19:10:43.285Z → 19:10:44.809Z | 0 | `ocel/raw/full-loop.txt` (`b4106106…d3bf84f4`) |
| same, run 2 (`ocel_pass2`) | 19:10:44.809Z → 19:10:44.862Z | 0 | `ocel/raw/full-loop-2.txt` (`f96173fe…88ab0ca4`) |
| `my-conforming-project receipt validate --dir target/plan_run/ocel_pass_receipts` | 19:10:44.863Z → 19:10:44.891Z | 0 | `ocel/raw/receipt-validate.txt` (`25c259c8…9cb01290`) |
| `ggen receipt verify` | 19:10:44.892Z → 19:10:44.921Z | 0 | `ocel/raw/ggen-receipt-verify.txt` (`702da2b9…8ad682a7`) |
| `ggen receipt history` | 19:10:44.921Z → 19:10:44.950Z | 0 | `ocel/raw/ggen-receipt-history.txt` (`c6683d82…dbf4183492`) |
| `cargo test -p ggen --test graphlaw_e2e` | 19:10:44.950Z → 19:10:45.261Z | 0 | `ocel/raw/graphlaw-e2e.txt` (`32953ad3…ccac3587`) — 5 passed, 0 failed |
| `ggen law derive` | 19:10:45.261Z → 19:10:46.777Z | 0 | `ocel/raw/ggen-law-derive.txt` — 55 derived, `graph_hash 181850a7…0036c1b7` |
| `my-conforming-project receipt export-ocel --out docs/releases/v26.7.6/ocel/ledger-export.ocel.json` | 19:10:46.778Z → 19:10:46.811Z | 0 | `ocel/raw/receipt-export-ocel.txt` — 3 events / 4 objects |
| Playwright spec (autonomic-platform) | 19:10:49.311Z → 19:10:50.604Z | pass | trace.zip + 3 screenshots |
| `cargo bench --bench blue_river_dam -p my-conforming-project` | 19:13:25Z → 19:14:52Z | 0 | `ocel/raw/bench-root.txt` (`34570340…57c28db`) |
| `cargo bench --bench blue_river_dam -p ggen` | 19:14:59Z → 19:15:00Z | 0 | `ocel/raw/bench-ggen.txt` (`258944cc…66d726d`) |
| `cargo bench --bench blue_river_dam -p praxis-graphlaw` | 19:15:08Z → 19:15:09Z | 0 | `ocel/raw/bench-graphlaw.txt` (`e02f221e…7ac0aeda`) |
| `cargo run --bin ocel_process_validate` | validated_at 19:39:17.830Z | 0 | report `ocel/wasm4pm-process-validation.json` |
| `cargo publish --dry-run --allow-dirty -p praxis-graphlaw` (closing phase, fresh) | → 19:44:59Z | 0 | `ocel/raw/cargo-publish-dry-run.txt` (`f562ff28…97474241`) — "Uploading praxis-graphlaw v26.7.5 … aborting upload due to dry run" |

## 3. What the OCEL log contains

`ocel/playwright-wasm4pm-validation.ocel.json` — OCEL 2.0 Shape A
(`wasm4pm_compat::ocel::OCEL` wire format): **50 events, 36 objects**, 30
event types and 13 object types instantiated.

Events by type: `api_request_observed` 3, `bcinr_transition_executed` 4,
`benchmark_result_attached` 3, `benchmark_run_completed` 3,
`benchmark_run_started` 3, `claim_promoted_to_standing` 1,
`ggen_artifact_generated` 2, `graphlaw_export_requested` 1,
`graphlaw_state_loaded` 1, `ocel_log_validated` 1, `ocel_log_written` 1,
`pddl_plan_loaded` 1, `pddl_plan_requested` 1,
`playwright_browser_launched` 1, `powl_workflow_compiled` 1,
`powl_workflow_executed` 2, `receipt_chain_verified` 3, `route_loaded` 1,
`screenshot_captured` 3, `trace_captured` 1, `ui_action_triggered` 3,
`utc_clock_captured` 1, `validation_run_finished` 1,
`validation_run_started` 1, `verifier_gate_completed` 2,
`verifier_gate_invoked` 1, `wasm4pm_conformance_passed` 1,
`wasm4pm_process_model_generated` 1,
`wasm4pm_process_validation_completed` 1,
`wasm4pm_process_validation_started` 1.

Objects by type: `bcinr_transition` 4, `benchmark_result` 8,
`benchmark_run` 3, `browser_session` 2, `client_surface` 2,
`ggen_artifact` 1, `graphlaw_state` 1, `pddl_plan` 1, `powl_workflow` 2,
`receipt_chain` 3, `report_artifact` 4, `screenshot` 4, `verifier_gate` 1.

## 4. Integrity + conformance results (wasm4pm)

By `src/bin/ocel_process_validate.rs` (method: library composition —
`wasm4pm_compat::ocel::validate` + `powl2_decompose` partial-order language
membership; full method in `WASM4PM_PROCESS_VALIDATION.md`):

- Integrity (OCEDO/OCPQ Def. 2): `valid: true`, 0 errors.
- UTC ordering: every `event.time` RFC 3339 with literal `Z`, parsed
  instants non-decreasing; 0 violations.
- POWL conformance: `is_conforming: true`, `fitness: 1.0`, 0 violations
  against the 22-child partial-order release-loop model (211 transitively
  closed order pairs; membership decided structurally, validated
  differentially against `Powl::language_upto` in the bin's tests).
- Object participation: all 5 required types present (`browser_session` 2,
  `client_surface` 2, `receipt_chain` 3, `benchmark_result` 8,
  `screenshot` 4).
- Log digests at validation time (match the on-disk file):
  sha256 `628807e0780a7d6f479f1a3d9dc744a17f5158cac3f7b2963244e10614f89e61`,
  blake3 `4c0d8584b1e0a1b786d1d4dc498aa20841d8f4977b8d82a4ab611a7863afca40`.

## 5. Benchmark medians from THIS pass

Divan medians from the 19:13:25Z–19:15:09Z reruns, attached as
`benchmark_result` objects (`pw_e38`, `pw_e41`, `pw_e44`); within noise of
`BLUE_RIVER_DAM_BENCHMARKS.md` Section 1:

| Benchmark (crate) | Median |
|---|---|
| `standing_transition` (root) | 19.98 ns |
| `action_precondition_mask` (root) | 56.93 ns |
| `pddl_action_filter` (root) | 6.457 µs |
| `bcinr_transition_table` (root) | 0.627 ns |
| `powl_step_tick` (root) | 3.512 ns |
| `receipt_frame_link` (root) | 245.7 ns |
| `ggen_render_report_small` (ggen) | 17.41 µs |
| `graphlaw_materialize_delta` (praxis-graphlaw) | 930 µs |

## 6. Receipt chain heads

- `.ggen-v2` factory chain: head
  `35bc4ab0c984ed5198e2609ec771f17a24d020d6e6882c2bb82ea6feab04765a`,
  8 records, `valid: true` (`ggen receipt verify`/`history`, `drv_e15`/
  `drv_e16`); same head observed by the browser (`pw_e5`/`pw_e6`).
- Plan-run ledger (this pass, `ocel_pass_receipts`): head
  `9f8e1e18f87c48e1ef696338698297b5c7906f67c7c6f0569354fe72835f1d91`
  (`drv_e14`). Five-stage `receipt validate` finding on this ledger:
  schema / chain_recompute / chain_linkage / token_replay **Pass**;
  monotonic stage reports
  `record 1: instruction_id (0) not strictly increasing after 0`
  (full output in `RECEIPT_VERIFY_OCEL.md`). The five-for-five Pass
  standing rests on the after_neon demo ledger
  (`RECEIPTS.md`, FINAL_STATUS.md exit criterion 7); the chain-integrity
  stages (recompute + linkage, the BLAKE3 tamper checks) pass on both.
- Lean lane: 202 per-label records at commit `1ea2385`
  (`tools/paper-factory/lean-lake/mathlib_migration_receipts.jsonl`),
  `lake build` replay 826 jobs exit 0, log sha256 `8f757aec…d69312af`
  (`RECEIPTS.md`).

## 7. Determinism recheck

`plan run` twice under identical inputs (windows above):
`powl_chain_hash blake3:1f97313c12be8f1f4b295970aaff506a79c1533be7a8abffb69c2ec8c677e9bb`
identical across both runs; `graph_hash 29f4cf58…177c9e88` identical;
promotion event `drv_e13 claim_promoted_to_standing` cites both raw
captures by sha256. Pinned independently by
`tests/plan_run_e2e.rs::two_runs_identical_chain_hashes` (green in
`just verify-all`). Hash path is wall-clock-free (`ts_ns: 0`,
`crates/ggen/src/sync.rs:945`).

## 8. Repairs made during the pass

All typed `RESOLVED_BY_REPAIR`, details in
`WASM4PM_PROCESS_VALIDATION.md` ("Repairs made") and
`OCEL_PLAYWRIGHT_WASM4PM_COMPLETION.md` ("Repairs made"):

1. E2O_EMPTY on 5 events (`drv_e12`, `drv_e13`, `pw_e1`, `pw_e2`,
   `pw_e15`) — generators fixed + identical in-place log repair.
2. Missing object-attribute timestamps (207) — recorders now stamp the
   observation instant; log repaired by documented derivation.
3. pcp expo web export — pcp commit `7731fea`, gate
   `npx expo export --platform web` exit 0.

No validator check was weakened.
