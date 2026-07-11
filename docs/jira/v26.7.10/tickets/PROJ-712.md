# PROJ-712 — Potato canonical scenario + negative corpus

Status: ALIVE — evidenced this session (uncommitted; HEAD `1f3f9bc`, Phase 6 commit not run)

Track: P (planning/decomposition).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

Hand-authored potato PDDL fixture with exact-output test; decomposition must be derived (not
hardcoded), helper ∥ main POWL composed, and — at integration — executed across H+M engines
with the global goal closed, all read back from OCEL via SPARQL. Also delivers the DoD §18
negative corpus skeleton at `tests/fixtures/decomp-negative/`. Gate: G8.

## Evidence (this session)

Potato fixture: `potato_graph_bridges_to_a_parsed_surface`,
`potato_decomposition_is_typed_receipted_and_replayable` (`cng_decomp.rs:56,78`, 3/3 passed
this session). Negative corpus at `tests/fixtures/decomp-negative/` (3 fixtures) exercised by
`cng_ipc_corpus.rs`'s named negative tests (`subgoal_not_contributing_refuses_cng_r21`,
`helper_unreachable_refuses_cng_r04`, `main_unreachable_after_helper_refuses_cng_r23`,
`helper_retains_resource_refuses_cng_r24`, `interfering_parallel_actions_refuse_cng_r22`,
`actor_lacks_capability_refuses_cng_r05`, `depth_or_cost_bound_exceeded_refuses_cng_r05`),
part of the 10/10 passed `cng_ipc_corpus` run. Two fixtures were corrected this session as
part of PROJ-733's grounder swap: `actor-lacks-capability.*.pddl` now correctly documents
`CNG_R05`, not `CNG_R04` (its earlier documented code depended on the naive grounder's blind
spot); `bound-exceeded.*.pddl` was rebuilt so all 5 params are reachability-constrained,
restoring the intended bound-exceeded scenario under the new grounder. H+M cross-engine
execution of the potato scenario specifically (vs. the corpus problems more generally) is
covered under PROJ-728/729's `cng_multi_engine` evidence, not re-cited here. The general
decompose-to-dispatch bridge mechanism (a different fixture, since potato itself selects
single-actor) is PROJ-749.

## Evidence (this session, round 2) — §18 corpus item 6 closed

`docs/releases/v26.7.10/DEFINITION_OF_DONE.md` §18 item 6 (`NO_BENEFICIAL_DECOMPOSITION`
forced, not merely accepted as one of three possible outcome branches) was PARTIAL after
round 1. New file `crates/cng/tests/cng_decomp_negative_corpus_completeness.rs`, test
`splits_admissible_but_not_beneficial_forces_no_beneficial_decomposition`, passed twice this
session (`CARGO_TARGET_DIR=target/agent-negcorpus just cng-test-one
cng_decomp_negative_corpus_completeness -- --nocapture`, 2/2 passed both runs, 0.05s). The
fixture is a two-chain kitchen domain where `fetch-drawer(?x)` carries a literal (non-variable)
precondition that forces every lawful plan through one unique total order while the split
stays enumerable; the test asserts the exact outcome
`DecompositionOutcome::NoBeneficialDecomposition { best_rejected_id: "cooked(potato)" }` (not
`matches!`), plus the exact score numbers that make the single-actor candidate win
(`makespan=4, dispatch_cost=6` vs. the split's `makespan=4, dispatch_cost=8`). Status: §18 item
6 is now ALIVE (see `DOD_SIGNOFF.md` §18, reconciled this pass).
