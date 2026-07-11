# PROJ-739 — 6 SPARQL-derivable planning marker queries + MARKER_MAP wiring

Status: ALIVE — evidenced this session (uncommitted; HEAD `1f3f9bc`, Phase 6 commit not run)

Track: D (doctrine — marker/evidence reconciliation, Phase 3 of the closure plan).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised closure plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

## Summary

Six SPARQL-derivable planning marker queries over `decomposition-result.ttl`:
`queries/markers/marker-decomposition-derived.rq` → `DECOMPOSITION_DERIVED_PROVEN`;
`marker-decomposition-receipted.rq` → `DECOMPOSITION_CANDIDATES_RECEIPTED`;
`marker-decomposition-interface-state.rq` → `INTERFACE_STATE_PROVEN`;
`marker-decomposition-non-interference.rq` → `NON_INTERFERENCE_PROVEN`;
`marker-decomposition-release-closure.rq` → `RESOURCE_RELEASE_CLOSED`;
`marker-decomposition-single-actor-typed.rq` → `SINGLE_ACTOR_TYPED_RESULT`. Wired into a new
`PLANNING_MARKER_MAP` (`crates/cng/src/bench/workday.rs:186-219`).

## Evidence (this session)

`planning_markers_prove_true_on_a_healthy_decompose_run` (`workday_test.rs:332-383`) asserts
all six markers (plus the three PROJ-740 absence markers) true over a real `decompose()` run's
`decomposition-result.ttl`, part of the `67 lib` tests in the green 107-test `cargo test -p
cng --features bench` run this session.

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` §16 (reconciled, PROJ-743)
- `docs/jira/v26.7.10/tickets/PROJ-740.md`, `PROJ-742.md`
