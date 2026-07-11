# PROJ-740 — 3 structural/absence markers (LLM_CALLS_ZERO family)

Status: ALIVE — evidenced this session (uncommitted; HEAD `40f6020`, Phase 6 commit not run)

Track: D (doctrine — marker/evidence reconciliation, Phase 3 of the closure plan).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised closure plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

## Summary

Three structural/absence markers in one query file, `queries/markers/marker-no-llm-
authoring.rq` → `LLM_CALLS_ZERO`, `ENGLISH_SUBGOALS_ZERO`, `CANNED_SUBGOALS_ZERO` (note the
`_ZERO` suffix — NOT the DoD's earlier bare `LLM_CALLS=0` form; that was doc prose, not a
shipped identifier; reconciled at PROJ-743). Uses the isolation-marker pattern (structural
fact + negative-obs evidence, same honest framing as `marker-engine-isolation.rq`'s
`SHARED_MEMORY_CROSSINGS_ZERO`/`DIRECT_ENGINE_BYPASSES_ZERO`): the PRIMARY proof is
structural (no LLM/inference-API crate in `crates/cng/Cargo.toml`'s dependency tree, no
natural-language generation code path in `crates/cng/src/bench/decomp/`); the query is the
SECONDARY negative-obs half.

## Evidence (this session)

`planning_markers_prove_true_on_a_healthy_decompose_run` (`workday_test.rs:332-383`) asserts
all three markers true, part of the green 107-test `cargo test -p cng --features bench` run
this session. Structural claim verified by reading `crates/cng/Cargo.toml`'s `[dependencies]`/
`[features]` surface this session (bcinr-pddl/pddl-index/oxigraph/blake3/serde/
praxis-graphlaw/wasm4pm-cognition/chicago-tdd-tools class dependencies only — no LLM/
inference-API crate).

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` §16 (reconciled, PROJ-743)
- `crates/cng/queries/markers/marker-no-llm-authoring.rq`
