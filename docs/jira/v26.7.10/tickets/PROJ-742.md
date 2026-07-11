# PROJ-742 — Extend V26_7_10_PRODUCTION_READY's conjunction

Status: ALIVE (combinator, pure-function unit-tested) / ALIVE (real two-bundle invocation:
workday + planning) / ALIVE (real three-bundle invocation: workday + planning + distributed,
closed EOD push) — evidenced this session

Track: D (doctrine — marker/evidence reconciliation, Phase 3 of the closure plan).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised closure plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

## Summary

New additive combinator `full_production_ready(workday_markers, planning_markers,
distributed_markers: Option<...>)` (`crates/cng/src/bench/workday.rs:644-665`). Purely
additive: `evaluate_markers`'s own signature and the `workday()` call site are UNCHANGED (the
interim-16 computation is not modified), as is `engine_collect_remote`'s own
`DISTRIBUTED_MARKER_MAP` evaluation. `workday()` never calls `decompose()` (by design —
separate run types), so `evaluate_markers()`'s own `V26_7_10_PRODUCTION_READY` output stays
scoped to the interim single-operator subset; a release-verification step that has run BOTH a
`workday()` bundle AND a `cng plan decompose` bundle (and optionally a distributed bundle)
calls `full_production_ready` to get the DoD-§16-accurate value.

## Evidence (this session) — initial round

`full_production_ready_refuses_when_a_planning_marker_is_false`
(`workday_test.rs:385-405`) — pure combinator fold logic, both sides hand-fabricated.
`planning_markers_prove_true_on_a_healthy_decompose_run` (`workday_test.rs:332-383`) —
combines a REAL `evaluate_planning_markers()` output (from an actual `decompose()` run) with a
HAND-FABRICATED two-key `workday_markers` map (`{"AUTONOMIC_LOOP_CLOSED": true,
"V26_7_10_PRODUCTION_READY": true}`), NOT the output of a real `evaluate_markers()` call over
a real `workday()` bundle. Both tests part of the green 107-test `cargo test -p cng --features
bench` run this session. **Neither test invokes the combinator with a REAL
`evaluate_markers()` output from an actual `workday()` bundle AND a real
`evaluate_planning_markers()` output together** — that end-to-end two-bundle composition was
UNVERIFIED at this point in the session.

## Evidence (follow-up round) — real dual-bundle invocation, and the remaining honest gap

New file `crates/cng/tests/cng_production_ready.rs` (2 tests) closes the two-bundle gap
in-process, no mocks:

1. Runs a real `cng::bench::workday(...)` (seed 742, 4 ticks, `refusal_per_mille: 0`) into a
   scratch dir — `report.markers` is the REAL 17-entry map `evaluate_markers` computed inside
   `workday()` over the real obs ∪ evidence ∪ dialect-registry union store.
2. Runs a real `cng::bench::decomp::decompose(...)` over
   `crates/cng/examples/pddl-strips-potato.ttl`, bridged via `strips_graph_to_surface` exactly
   as `tests/cng_decomp.rs` does, into a scratch dir — producing a real
   `decomposition-result.ttl`.
3. Loads that file with `build_decomp_marker_store` and evaluates it with
   `evaluate_planning_markers` — the REAL 9-entry planning marker map.
4. Calls `full_production_ready(&workday_markers, &planning_markers, None)` and asserts all 26
   combined keys (16 workday-named + 9 planning-named + the recomputed
   `V26_7_10_PRODUCTION_READY`) are `true`
   (`full_production_ready_holds_on_real_dual_bundle_evidence`).
5. A companion negative test
   (`full_production_ready_goes_false_when_a_real_marker_is_forced_false`) reuses the same real
   dual-bundle maps, forces one marker false on the workday side (`AUTONOMIC_LOOP_CLOSED`) and
   separately on the planning side (`LLM_CALLS_ZERO`), and asserts `V26_7_10_PRODUCTION_READY`
   goes `false` in both cases, plus a control assertion that the unmodified real pair stays
   `true` (rules out a trivially-always-false combinator).

Command: `CARGO_TARGET_DIR=target/agent-742 cargo test -p cng --features bench --test
cng_production_ready -- --nocapture` → 2 passed, 0 failed, ~1.15s (0.73s warm-cache re-run).
Full cold build (first compile of the new test binary) took 2m04s; no compiler errors or new
warnings beyond pre-existing ones unrelated to this change.

**Visibility bump made (smallest possible, no logic changes)**: in
`crates/cng/src/bench/workday.rs`, three functions changed from `pub(super)` to `pub`:
`build_decomp_marker_store` (line 587), `evaluate_planning_markers` (line 614),
`full_production_ready` (line 644). `crates/cng/src/bench/mod.rs`'s existing re-export line was
extended to carry the three new names. No other files touched; `MARKER_MAP`,
`evaluate_markers`, `build_marker_store` were left untouched/private (not needed —
`WorkdayReport.markers` is already fully `pub` and supplies the real workday-side map
directly).

## Evidence (EOD push) — real three-way invocation, gap closed

New, independent, additive-only file `crates/cng/tests/cng_production_ready_three_way.rs`
closes the previously-UNVERIFIED third leg. Investigation finding: no visibility bump was
needed — `EngineCoordinateReport` (`crates/cng/src/bench/engine.rs:787`) already has a `pub
markers: BTreeMap<String, bool>` field, populated internally by `engine_collect_remote` from
`evaluate_marker_map(&marker_store, &marker_queries, &DISTRIBUTED_MARKER_MAP)`
(`engine.rs:1222`); `engine_collect_remote`/`engine_dispatch_remote`/`EngineCoordinateReport`
were already `pub` and re-exported from `cng::bench`.

The new test:
1. Runs a real `workday()` bundle (17-entry marker map).
2. Runs a real `decompose()` bundle over the potato fixture (9-entry planning marker map).
3. Runs a real two-engine coordinate round — `engine_dispatch_remote` then two real `cng
   engine serve` OS processes spawned via `CARGO_BIN_EXE_cng` (a local, independent
   `run_cng`/`serve_to_budget` reimplementation — NOT imported from `cng_multi_engine.rs`,
   keeping this file collision-free from concurrent work on that file) — then
   `engine_collect_remote`, whose `report.markers` IS the real, already-evaluated
   `DISTRIBUTED_MARKER_MAP` output; no new marker-evaluation machinery was built.
4. Calls `full_production_ready(&workday_markers, &planning_markers,
   Some(&distributed_markers))` and asserts `V26_7_10_PRODUCTION_READY == true` on the
   combined 29-key map (16 workday + 9 planning + 3 distributed-only + the recomputed
   conjunction) — `full_production_ready_holds_on_real_triple_bundle_evidence`. A companion
   negative test forces `ENGINE_INSTANCES_PROVEN` false and asserts the conjunction goes
   `false` — `full_production_ready_goes_false_when_a_real_distributed_marker_is_forced_false`.

Command: `CARGO_TARGET_DIR=target/agent-threeway just cng-test-one
cng_production_ready_three_way -- --test-threads=1 --nocapture` → 2 passed, 0 failed, 5.62s.
Also re-ran `no_inline_ttl_guard` (2 passed) and the existing two-way `cng_production_ready`
(2 passed) to confirm no regression.

**Claim, updated**: ALIVE for all three legs. `full_production_ready` has now been invoked
end-to-end against REAL `workday()`, `decompose()`, AND a real two-engine coordinate round's
marker maps together, in `cng_production_ready_three_way.rs`. The combined
`V26_7_10_PRODUCTION_READY` conjunction genuinely holds `true` on healthy real triple-bundle
evidence, and genuinely goes `false` when any one leg's real evidence is forced false (proven
independently for the workday leg, the planning leg, and now the distributed leg). No DoD
claim about the three-way combination needs to stay scoped down any longer.

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` §16 (load-bearing correction, PROJ-743)
- `docs/releases/v26.7.10/DOD_SIGNOFF.md` (PROJ-748)
