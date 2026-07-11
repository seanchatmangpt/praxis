# PROJ-720 — 16-state dispatch machine everywhere + drift test

Status: ALIVE — evidenced this session (uncommitted; HEAD `40f6020`, Phase 6 commit not run)

Track: E (multi-engine execution).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

Extend the interim 13-state machine to 16 states in place (DoD §12): add `ARAZZO_RENDERED`
(before DISPATCH_READY) and `REMOTE_STARTED` (after ACKNOWLEDGED); rename
`IN_PROGRESS→REMOTE_IN_PROGRESS`, `ADMITTED→RESULT_ADMITTED`; split
`RESULT_RETURNED→RESULT_AVAILABLE→RESULT_RECEIVED`. Touch all three co-located authorities in
`dispatch.rs` + `shapes/dispatch-shapes.ttl` individuals + contract template +
string-matching queries + tests; add a drift test (TTL individuals == `as_str` set).
Rename-ripple grep checklist per plan risk 3. Gate: G9.

## Evidence (this session)

`crates/cng/src/bench/dispatch.rs` (16-state `DispatchState` enum, `:109-189`). Tests:
`sixteen_state_transition_law_is_exact` (`dispatch_test.rs:444`),
`shapes_ttl_state_individuals_match_the_enum` (`dispatch_test.rs:496`, the drift test) — part
of the `67 lib` tests in the green 107-test `cargo test -p cng --features bench` run this
session.
