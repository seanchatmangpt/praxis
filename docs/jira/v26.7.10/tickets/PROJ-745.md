# PROJ-745 — Rust digest-verify seam: digest(render(graph)) check

Status: ALIVE (function, unit-tested, and now wired into the Arazzo-sourced dispatch call
site) — evidenced this session (uncommitted; HEAD `40f6020`, Phase 6 commit not run)

Track: E (multi-engine execution — arazzo-pack wiring, Phase 4 of the closure plan).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised closure plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

## Summary

New function `verify_arazzo_render_digest(project_root: &Path) -> Result<
ArazzoRenderVerification, CngRefusal>` (`crates/cng/src/bench/arazzo.rs:424-469`), additive,
`#[allow(dead_code)]` since not yet wired to a call site. Recomputes BLAKE3 over the rendered
YAML (`generated/arazzo.yaml`) and compares against the ggen receipt's recorded digest
(`.ggen-v2/receipt.json`, `payload.outputs["generated/arazzo.yaml"]`); reuses
`CngRefusal::AuditMismatch` (`CNG_R11`) for every failure mode (missing/unreadable render,
missing/unreadable/unparseable receipt, no matching digest entry, digest mismatch) — no new
refusal code needed.

## Evidence (this session) — initial round

`arazzo_test.rs:148,179` (`verify_arazzo_render_digest` called and asserted, both success and
mismatch paths), part of the green 107-test `cargo test -p cng --features bench` run this
session. At this point the function was `#[allow(dead_code)]` and not called from any
production dispatch path.

## Evidence (follow-up round) — wired, with a scope correction

**Part A — wiring.** `dispatch.rs`'s generic `MANUFACTURED → ARAZZO_RENDERED → DISPATCH_READY`
transition (`dispatch()`, line ~1264) renders the `DispatchContract` itself via
`contract_template` — it has no relationship to the arazzo-pack's ggen-rendered YAML, so the
digest-verify seam was wired into `arazzo::run_arazzo_projection` instead (the actual
Arazzo-sourced path):

- `crates/cng/src/bench/dispatch.rs:1067` — added `project_root: PathBuf` field to
  `DispatchAdapter`.
- `crates/cng/src/bench/dispatch.rs:1098`/`:1122`/`:1138` — `new()` now threads its `out_dir`
  through to `new_with_dirs()` as `project_root` (only caller of `new_with_dirs`).
- `crates/cng/src/bench/dispatch.rs:1217` — new `pub(super) fn project_root(&self) -> &Path`
  accessor.
- `crates/cng/src/bench/arazzo.rs:337` — `run_arazzo_projection` now calls
  `verify_arazzo_render_digest(adapter.project_root())?` right after `project_steps` succeeds
  and before the per-step dispatch loop, so a missing/mismatched render refuses `CNG_R11
  AuditMismatch` before any step reaches `ArazzoRendered`/`DispatchReady`.
- Removed all four `#[allow(dead_code)]` markers on `ArazzoRenderVerification`,
  `GgenReceiptDocument`, `GgenReceiptPayload`, `verify_arazzo_render_digest` — confirmed
  genuinely dead-code-free by a clean `cargo clippy` pass with no warnings on those items.
- Two new tests in `crates/cng/src/bench/dispatch_test.rs`:
  `arazzo_projection_gate_admits_when_render_digest_matches_receipt` (scratch project_root with
  a matching render+receipt; all 4 steps dispatch and admit) and
  `arazzo_projection_gate_refuses_cng_r11_before_any_step_dispatches` (no render/receipt;
  asserts `CNG_R11 AuditMismatch`, zero steps sent, empty outbox — the gate blocks the whole
  projection, not just one step).

**Part B — real receipt, scope boundary held.** `ggen sync run --help`/`--introspect` (JSON
Schema of CLI capabilities) show only `dry_run` and `watch` parameters — no `--pack`/`--only`/
output-scoping flag exists anywhere in the `ggen` binary. No safe pack-scoped or
output-scoped narrow run option exists, so `ggen sync run` was **not** run against the live
repo `ggen.toml` (which would regenerate outputs for all six registered packs) — a
deliberately-avoided scope boundary, not an oversight.

**Honest side-effect finding**: a throwaway diagnostic (`examples/_diag_745_category_scan.rs`,
deleted after use, never committed) ran `workday()` across 200 seeds/8 ticks each against fresh
scratch dirs with no ggen render present. 59/200 seeds hit the new `CNG_R11 AuditMismatch`
refusal when the seed-derived category cycle landed on `api-orchestration` — proof the wiring
is real and load-bearing, not a no-op. Any `workday()` run (test or real) that selects the
`api-orchestration` category now requires a pre-existing `<out_dir>/generated/arazzo.yaml` +
`<out_dir>/.ggen-v2/receipt.json` to succeed. The committed test suite's fixed seeds (616, 742,
etc. at `ticks: 4`) happen not to land on that category, so nothing in the existing suite
regressed — this is the correct, intended tightening per this ticket's own goal, not a bug, but
worth carrying forward as a deployment note.

**Verification (`CARGO_TARGET_DIR=target/agent-745`)**: `cargo build -p cng --lib --features
bench` clean, no new warnings. `cargo fmt -p cng -- --check` clean. `cargo clippy -p cng --lib
--tests --features bench` zero new warnings attributable to this change. `cargo test -p cng
--lib --features bench -- bench::dispatch:: bench::arazzo:: bench::workday::
bench::workday_verify::` → 35 passed, 0 failed. `cargo test -p cng --test cng_workday_verify
--test cng_production_ready --features bench` → 4 passed, 0 failed.

Files touched: `crates/cng/src/bench/dispatch.rs`, `crates/cng/src/bench/arazzo.rs`,
`crates/cng/src/bench/arazzo_test.rs`, `crates/cng/src/bench/dispatch_test.rs`. No changes to
`workday.rs`, `main.rs`, `ipc/`, or `ggen.toml`.

**Remaining honest boundary**: no OpenAPI/AsyncAPI schema validation, no HTTP/broker binding —
this stays the declared, honest cut per the existing "mechanism ALIVE / HTTP binding
UNVERIFIED" boundary language already in `DEFINITION_OF_DONE.md` §20. `dispatch.rs`'s own
generic `ArazzoRendered` transition (rendering `DispatchContract`) remains correctly unwired to
this gate, since it renders an unrelated artifact.

## Links

- `docs/jira/v26.7.10/tickets/PROJ-726.md`, `PROJ-744.md`
- `crates/cng/src/bench/arazzo.rs`, `crates/cng/src/bench/arazzo_test.rs`,
  `crates/cng/src/bench/dispatch.rs`, `crates/cng/src/bench/dispatch_test.rs`
