# PROJ-713 — Anti-hardcoding gate

Status: ALIVE — evidenced this session (uncommitted; HEAD `1f3f9bc`, Phase 6 commit not run)

Track: P (planning/decomposition).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

Permuted identities/initial states/roles must causally change decomposition/plan/refusal
digests; a permuted-seed rerun with unchanged digests fails the gate. Canned-subgoal
detection is a typed refusal (DoD §17). Feeds markers `CANNED_SUBGOALS_ZERO` and the
`LLM_CALLS_ZERO` family evidence (name reconciled to on-disk `_ZERO` suffix, PROJ-743).

## Evidence (this session)

`permuted_goal_identities_change_plans_and_receipts_causally` (`cng_ipc_corpus.rs:194-252`) —
`IpcVariant::SwappedGoalIdentities` guarantees a changed problem text, asserts the plan tape
changes causally and the emitted receipt graph bytes differ. `no_canned_helper_subgoal_across_
incompatible_variants` (`cng_ipc_corpus.rs:254-283`) — candidate ids pairwise disjoint across
incompatible domains. Both part of the 10/10 passed `cargo test -p cng --features bench --test
cng_ipc_corpus` run this session.

## Evidence (this session, round 2) — §18 corpus item 7 closed

`docs/releases/v26.7.10/DEFINITION_OF_DONE.md` §18 item 7 (injected canned subgoal) was
PARTIAL after round 1: the existing `no_canned_helper_subgoal_across_incompatible_variants`
test only proved disjoint candidate-id sets across domains with unrelated vocabularies — a
check a canned rule would pass trivially. New file
`crates/cng/tests/cng_decomp_negative_corpus_completeness.rs`, test
`canned_subgoal_detection_catches_identical_goal_labels_with_different_achiever_structure`,
passed twice this session (same command as PROJ-712's round-2 addendum, 2/2 passed both
runs). Fixture: two domains with IDENTICAL goal-atom labels (`cooked(potato)`,
`placed(fork)`, same predicate names and same objects — pinned by an explicit
`plain_problem.goal == heat_problem.goal` assertion) but a genuinely different achiever chain
for `cooked` (plain: `fetch-pantry, cook`, 2 steps; heat-gated: `fetch-pantry, heat, cook`,
3 steps). Both runs enumerate a `"cooked(potato)"` split receipt under the same id (expected —
ids are label-derived), but the receipt CONTENT differs (`makespan` 2 vs. 3, differing
`dispatch_cost`, and the `decomposition-result.ttl` bytes differ byte-for-byte under the same
base IRI) — proving no cached/canned answer is keyed on the id string. Status: §18 item 7 is
now ALIVE (see `DOD_SIGNOFF.md` §18, reconciled this pass).
