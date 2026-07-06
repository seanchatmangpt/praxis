# v26.7.6 — No Terminal Blockers Ledger

Companion to `OCEL_PLAYWRIGHT_WASM4PM_COMPLETION.md`. This pass may not end with a
terminal blocker. Every open item carries exactly one status from the set below,
and `TEMP_BLOCKED` is forbidden in `FINAL_STATUS.md`.

## The 8 statuses

| Status | Meaning |
|---|---|
| `TEMP_BLOCKED` | Transient state during the pass only. An item may pass through this status but may never finish in it. Any `TEMP_BLOCKED` item in `FINAL_STATUS.md` is itself the bug. |
| `RESOLVED_BY_REPAIR` | The blocker was fixed in place with a cited diff (emits `claim_resolved_by_repair`). |
| `RESOLVED_BY_EXISTING_SURFACE` | An already-existing surface satisfies the requirement; no new code, cited by path. |
| `RESOLVED_BY_LOCAL_EQUIVALENT` | A local substitute (e.g., library composition instead of a stubbed CLI) provides equivalent evidence (emits `claim_resolved_by_local_equivalent`). |
| `RESOLVED_BY_SCOPE_RECLASSIFICATION` | The item is reclassified out of release scope, with grounds cited in this ledger. |
| `RESOLVED_BY_EXTERNAL_OPERATOR_SIDE_EFFECT` | The action requires a human operator with external credentials (publishing, submission, visibility); packaged locally, executed operator-side (emits `claim_resolved_by_external_operator_side_effect`). |
| `OCEL_PROVEN` | The claim is backed by events in the validated OCEL v2 log of this pass. |
| `WASM4PM_PROVEN` | The claim is backed by wasm4pm process validation (`wasm4pm_compat::ocel::validate` and/or `powl2_decompose` membership/replay). |

## Ledger

| Item | Status (planned) | Resolution route |
|---|---|---|
| pcp build failure | `RESOLVED_BY_REPAIR` | Create apps-side `src/lib/truex/avatar/matrix.ts` exporting `PROJECTION_MATRIX` (including the `volunteer_shortage` key); apply import-type fixes for `@wasm4pm/types` in `src/hooks/useOcelEvidence.ts` and `src/components/ProcessEvidenceView.tsx`. |
| dashboard.bak | `RESOLVED_BY_SCOPE_RECLASSIFICATION` | 17+ pages require Nuxt UI Pro, a paid license not held; out of release scope. |
| Canonical Nuxt shell | `RESOLVED_BY_EXISTING_SURFACE` | `wasm4pm/apps/playground-web` promoted as the canonical Nuxt shell in place of dashboard.bak. |
| OCEL SQLite export | `RESOLVED_BY_SCOPE_RECLASSIFICATION` | Zero SQLite support exists in wasm4pm; OCEL JSON is the standing artifact; no release document requires SQLite. |
| crates.io publish | `RESOLVED_BY_EXTERNAL_OPERATOR_SIDE_EFFECT` | `crate_package` object built and verified locally; publish requires operator credentials. |
| arXiv submission | `RESOLVED_BY_EXTERNAL_OPERATOR_SIDE_EFFECT` | `arxiv_package` object built locally; submission requires operator account. |
| Repository visibility change | `RESOLVED_BY_EXTERNAL_OPERATOR_SIDE_EFFECT` | Access-control change; operator-only by policy. |
| wasm4pm CLI conformance (stubbed) | `RESOLVED_BY_LOCAL_EQUIVALENT` | Library composition: `wasm4pm_compat::ocel::validate` + `powl2_decompose` partial-order language membership. |
| autonomic-platform Playwright evidence | `OCEL_PROVEN` (target) | NavRail `getByTitle` interactions, `/praxis-artifacts/*` observations, `?mode=mock`/`?mode=praxis` in the OCEL log. |
| optimus Playwright evidence | `OCEL_PROVEN` (target) | Existing harness run, traces/screenshots attached. |
| Receipt chain (validate 5 stages, verify/history, export-ocel) | `OCEL_PROVEN` (target) | `receipt_chain_verified`, `ocel_log_written`, `ocel_log_validated` events. |
| GraphLaw evidence (5 `graphlaw_e2e` tests + `ggen law derive`) | `OCEL_PROVEN` (target) | `graphlaw_state_loaded` events per test. |
| Planner loop determinism (`plan run` x2, `powl_chain_hash`) | `OCEL_PROVEN` (target) | Equal chain hashes across two runs, `ts_ns: 0` hash path. |
| Benchmarks (3 divan `blue_river_dam`) | `OCEL_PROVEN` (target) | `benchmark_result_attached` events. |
| wasm4pm process validation and replay | `WASM4PM_PROVEN` (target) | `wasm4pm_conformance_passed`, `wasm4pm_replay_passed` events. |

## Rule

No item may remain `TEMP_BLOCKED` in `FINAL_STATUS.md`. The closing phase must
rewrite every `(planned)` and `(target)` above to its achieved status, each with a
command citation and OCEL event ids.
