# PROJ-707 — Interface state s-prime replay + CNG_R23

Status: ALIVE — evidenced this session (uncommitted; HEAD `1f3f9bc`, Phase 6 commit not run)

Track: P (planning/decomposition).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

Derive `s′ = E(s_i, π_h)` by replaying the helper tape with per-step precondition
verification; any violation is a typed `CNG_R23 InterfaceStateMismatch` refusal (with
negative test). CONSTRUCT the surviving atom set into the main problem's init. No
triple-level effect application — replay is over the lifted STRIPS state. Gate: G5.

## Evidence (this session)

`crates/cng/src/bench/decomp/interface.rs`, `CNG_R23 InterfaceStateMismatch`
(`crates/cng/src/powl.rs:199-209,261`). Tests: `replay_verifies_preconditions_and_applies_effects`,
`tampered_tape_refuses_cng_r23_interface_state_mismatch` (`decomp_test.rs:162,182`);
`main_unreachable_after_helper_refuses_cng_r23` (`cng_ipc_corpus.rs:331`) — all green in the
107-test `cargo test -p cng --features bench` run this session.
