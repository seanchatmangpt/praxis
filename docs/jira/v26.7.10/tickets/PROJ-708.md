# PROJ-708 — Non-interference proof (CNG_R22) + release closure (CNG_R24)

Status: ALIVE — evidenced this session (uncommitted; HEAD `40f6020`, Phase 6 commit not run)

Track: P (planning/decomposition).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

Machine-check Effects ∩ ProtectedPreconditions = ∅ in both directions (`CNG_R22`) and
resource-release closure at the interface (`CNG_R24 ResourceUnreleased`). Both refusals get
negative tests (DoD §18 corpus items 3 and 4). Gate: G6.

## Evidence (this session)

`crates/cng/src/bench/decomp/interference.rs`, `CNG_R22 InterferenceDetected`
(`powl.rs:185-199,260`), `CNG_R24 ResourceUnreleased` (`powl.rs:209-219,262`). Tests:
`concurrent_clobber_refuses_cng_r22_interference`, `ordered_pair_is_not_interference`,
`unreleased_resource_refuses_cng_r24` (`decomp_test.rs:203,226,249`);
`interfering_parallel_actions_refuse_cng_r22`, `helper_retains_resource_refuses_cng_r24`
(`cng_ipc_corpus.rs:389,359`) — all green in the 107-test `cargo test -p cng --features
bench` run this session.
