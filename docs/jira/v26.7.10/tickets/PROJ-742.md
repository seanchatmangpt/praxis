# PROJ-742 — Extend V26_7_10_PRODUCTION_READY's conjunction

Status: ALIVE (combinator, pure-function unit-tested) / ALIVE (real two-bundle invocation:
workday + planning) / UNVERIFIED (real three-bundle invocation, +distributed) — evidenced this
session (uncommitted; HEAD `40f6020`, Phase 6 commit not run)

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

**Three-way (distributed) coverage: still UNVERIFIED, omitted per the honest-minimum cut
line.** Wiring a real third multi-engine bundle requires spawning real OS `cng engine serve`
processes through `cng_multi_engine.rs`'s own harness helpers (`spawn_engine`,
`serialized_run`, etc.), which are private to that test binary and not importable from a
separate `tests/` integration crate. Two-way (workday + planning) is the honest minimum
achieved this round, matching this ticket's own cut line — `full_production_ready`'s real
THREE-bundle invocation remains UNVERIFIED. See DoD §16's "Honest gap" subsection (updated to
match).

**Claim for a doc-closure agent to cite**: ALIVE — `full_production_ready` has now been
invoked end-to-end against a real `workday()` bundle's marker map and a real
`cng::bench::decomp::decompose` bundle's marker map together, in
`crates/cng/tests/cng_production_ready.rs::full_production_ready_holds_on_real_dual_bundle_evidence`,
run this session. The combined `V26_7_10_PRODUCTION_READY` conjunction genuinely holds `true`
on that healthy real dual-bundle pair, and a companion test confirms it genuinely goes `false`
when either side's real evidence is forced false — this is two-way (workday + planning)
coverage; the three-way distributed extension remains UNVERIFIED by this test, so any DoD
claim about the three-way combination should stay scoped accordingly.

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` §16 (load-bearing correction, PROJ-743)
- `docs/releases/v26.7.10/DOD_SIGNOFF.md` (PROJ-748)
