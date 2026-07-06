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

Full table with UTC windows and raw-capture hashes:
`OCEL_V2_WASM4PM_REPORT.md` Sec. 2. Summary (all 2026-07-06, all exit 0):

1. `cargo build --features ggen --bin my-conforming-project` (19:10:42.925Z).
2. `plan run` ×2 (`ocel_pass`, `ocel_pass2`) — events `drv_e2..drv_e13`;
   `powl_chain_hash blake3:1f97313c…c677e9bb` equal across runs.
3. `receipt validate --dir target/plan_run/ocel_pass_receipts` — `drv_e14`.
4. `ggen receipt verify` / `ggen receipt history` — `drv_e15`/`drv_e16`,
   head `35bc4ab0…ab04765a`, 8 records, valid true (at evidence time).
5. `cargo test -p ggen --test graphlaw_e2e` — `drv_e18`, 5 passed 0 failed.
6. `ggen law derive` — `drv_e19`, 55 derived.
7. `receipt export-ocel` — `ocel/ledger-export.ocel.json`, 3 events / 4 objects.
8. Playwright spec `clients/autonomic-platform/tests/playwright/ocel-wasm4pm-validation.spec.ts`
   — events `pw_e1..pw_e15` (19:10:49.311Z–19:10:50.604Z).
9. `cargo bench --bench blue_river_dam` ×3 (root, ggen, praxis-graphlaw) —
   events `pw_e36..pw_e44` (19:13:25Z–19:15:09Z).
10. `cargo run --bin ocel_process_validate` — events `val_e1..val_e6`;
    integrity valid, conformance fitness 1.0 (19:39:17.830Z).
11. Closing phase: `cargo publish --dry-run --allow-dirty -p praxis-graphlaw`
    — exit 0, 19:44:59Z, `ocel/raw/cargo-publish-dry-run.txt`
    (sha256 `f562ff28…97474241`).
12. Closing phase: `just verify-all` — first run exit 101
    (`sync_run_help_gives_each_flag_a_non_blank_description`, ggen
    `cli_boundary`); repaired (see the closing-phase repair under "Repairs
    made"); rerun exit 0 — check + test (153 binaries, 1566 passed, 0
    failed) + clippy + doctor all passed, log sha256
    `5e87e7bb7b0458e633cdf926647ca696d57e5b589baf1f225134ff0a8990bab7`.

## Repairs made

- **pcp expo web export** — `RESOLVED_BY_REPAIR`, 2026-07-06, pcp commit
  `7731fea` (`fix(web-export): restore avatar projection matrix + type-only
  wasm4pm imports`). Diagnosed `PROJECTION_MATRIX` as already exported by
  `/Users/sac/pcp/src/lib/truex/avatar/matrix.ts` (re-export of
  `avatar-projection.ts`); actual repairs: `import type` for the type-only
  `@wasm4pm/types` package in `apps/template-app/src/hooks/useOcelEvidence.ts`
  and `apps/template-app/src/components/ProcessEvidenceView.tsx`; new
  `apps/template-app/metro.config.js` mirroring the root tsconfig aliases
  (`@/*`, `~/*`, `@pcp/framework/*`) with a workspace-`src/` fallback for
  relative `../framework/` imports and `.wasm` added to `assetExts`
  (expo-sqlite web worker); restored missing `global.css` and `assets/`
  (fonts, images, proofs, validation); `lib/supabase.ts` build-time placeholder
  fallback for `EXPO_PUBLIC_SUPABASE_URL`/`ANON_KEY` (empty URL crashed the
  static-render pass with "supabaseUrl is required").
  Gate: `cd /Users/sac/pcp/apps/template-app && npx expo export --platform web`
  → exit code 0, "Exported: dist" (all routes rendered).

- **OCEL log E2O_EMPTY (5 events)** — `RESOLVED_BY_REPAIR`. `drv_e12`,
  `drv_e13`, `pw_e1`, `pw_e2`, `pw_e15` had no qualified object reference;
  generators fixed (`run-evidence-pass.mjs`,
  `ocel-wasm4pm-validation.spec.ts`) and the identical fix applied in place
  to the committed log. Details: `WASM4PM_PROCESS_VALIDATION.md` "Repairs
  made" item 1.

- **Missing object-attribute timestamps (207)** — `RESOLVED_BY_REPAIR`.
  The OCEL 2.0 wire shape requires `time` on every object attribute;
  recorders now stamp the observation instant; log repaired by documented
  derivation. Details: `WASM4PM_PROCESS_VALIDATION.md` "Repairs made"
  item 2. No validator check was weakened.

- **Closing-phase verify-all red** — `RESOLVED_BY_REPAIR`, 2026-07-06.
  `sync_run_help_gives_each_flag_a_non_blank_description` (ggen
  `cli_boundary`) failed: clap wrapped the `--watch` help line at terminal
  width, pushing `filesystem` onto a continuation line the test does not
  scan. Fix at the source of truth: reworded the flag description in
  `schema/praxis.ttl` (`praxis:CmdGgenSyncRun`, "Watch the filesystem and
  re-run the pipeline automatically whenever a watched file changes"),
  regenerated `crates/ggen/src/verbs/sync.rs` via `ggen sync run`
  (generated file, "do not edit by hand"). The regeneration is itself
  receipted: `.ggen-v2` chain extended 8 → 9 records, new head
  `345cb056468281a5eda2d3b5af3d829c5c894071e546807a6ab39f4d40d380cb`,
  `ggen receipt verify` valid true. Gates: `cargo test -p ggen --test
  cli_boundary` → 28 passed 0 failed; `just verify-all` rerun → exit 0.

## Substitutions made

- **wasm4pm CLI conformance → library composition** —
  `RESOLVED_BY_LOCAL_EQUIVALENT`. The CLI's
  `check_conformance_token_replay` is a stub
  (`/Users/sac/wasm4pm/crates/wasm4pm-cli/src/commands/mining.rs:25-32`);
  substituted `wasm4pm_compat::ocel::validate` + `powl2_decompose`
  structural language membership, executed by
  `src/bin/ocel_process_validate.rs` (differential tests against
  `Powl::language_upto` included). Result: integrity valid, conformance
  `fitness: 1.0` (`ocel/wasm4pm-process-validation.json`).
- **dashboard.bak → wasm4pm/apps/playground-web** as canonical Nuxt shell —
  `RESOLVED_BY_EXISTING_SURFACE` (build PASS from the wasm4pm root; see
  `NO_TERMINAL_BLOCKERS.md` and `CLIENT_SURFACES.md`).

## External operator side effects

None of these block the release; each is packaged locally and awaits an
operator with external credentials:

1. **crates.io publish** — local packaging fresh-verified in the closing
   phase: `cargo publish --dry-run --allow-dirty -p praxis-graphlaw` exit 0
   (`ocel/raw/cargo-publish-dry-run.txt`). Operator: `cargo login`,
   optionally bump `praxis-graphlaw` 26.7.5 → 26.7.6, then
   `cargo publish -p praxis-graphlaw`.
2. **arXiv submission** — `arxiv-package/arxiv-submission.tar.gz`
   (sha256 `67e0725f…875ad767`) built locally, latexmk exit 0
   (`arxiv-package/MANIFEST.md`). Operator: make the artifact bundle
   public, upload at https://arxiv.org/submit.
3. **Repository visibility change** — operator-only access-control action.

## Final standing

Closed 2026-07-06. No item stands as `TEMP_BLOCKED`; all 15 ledger rows in
`NO_TERMINAL_BLOCKERS.md` carry achieved terminal statuses with citations.
The evidence log validates (integrity 0 errors) and conforms (fitness 1.0)
under wasm4pm process validation. Claim-by-claim promotions with zero
unevidenced promotions: `CLAIM_PROMOTION_TABLE.md`. Narrative:
`OCEL_V2_WASM4PM_REPORT.md`. Release standing:
**ALIVE_WITH_OCEL_AND_WASM4PM_EVIDENCE**, with the two external operator
side effects (crates.io publish, arXiv submission) typed as
non-blocking — see `FINAL_STATUS.md` "OCEL + wasm4pm Final Standing".
