# PROJ-819: `wasm4pm` and `bcinr-pddl` classical planners systematically disagree on plan step count

**Status**: OPEN -- discovered, not fixed
**Dependencies**: none (independent of PROJ-811/814/815/816/817/818)

## Scope

`tests/differential.rs` cross-checks `wasm4pm`'s own planner against
`bcinr-pddl`'s classical `GroundProblem::find_plan` over the same domains
and problems, expecting agreement on solvability and (where both solve)
step count. This file was compile-blocked most of this session (stale
`Result`-based `find_plan`/`find_temporal_plan` API usage, fixed alongside
`indexed_grounding.rs` and `bribery_case_pddl.rs` in commit `37fe4405`) and
has therefore not actually run in a long time. Now that it compiles, 3 of
8 tests fail for real:

```
pair1_scope_classical_exemplars:
  bcinr parses revenue.pddl (classical :strips): ParseError("trailing token at index 214")

pair1_planners_revenue_stage_chain:
  revenue chain disagreement: "[revenue] STEP-COUNT DISAGREEMENT: wasm4pm=13 bcinr=4"

pair1_planners_generated_corpus_triple_agreement (30-instance generated corpus, 22 solvable):
  18 of 22 solvable instances disagree, ALL in the same direction --
  bcinr always finds FEWER steps than wasm4pm, e.g.:
    gen1:  wasm4pm=54 bcinr=12
    gen4:  wasm4pm=24 bcinr=4
    gen17: wasm4pm=50 bcinr=16
    gen19: wasm4pm=17 bcinr=4
  (full 18-entry list in the test's own panic output)
```

## Why this is suspicious, not just "two planners differ"

Every single disagreement in the generated-corpus run goes the same
direction (bcinr shorter, never longer), across 18 independently generated
problem instances. A parser bug (the first failure, a hard `ParseError`
on `revenue.pddl` at "trailing token index 214") is a separate, narrower
issue -- but the systematic one-directional step-count gap across the
generated corpus matches the exact SHAPE of a bug this same domain family
already found and fixed once this session (see
`crates/multifractal-workflow/fixtures/bribery-case/pddl-domain.ttl` lines
168-215's own disclosed history): STRIPS8 has no `(not (= ?a ?b))`
inequality constraint, so a classical grounder/planner can silently alias
two distinct action parameters to the SAME object when nothing in the
domain forbids it, satisfying multiple conjuncts (or reaching the goal
early) with fewer real actions than a planner that doesn't take that
shortcut. That prior bug was fixed by redesigning the DOMAIN (3 distinct
predicate names instead of one shared one) rather than the planner -- but
this new finding is about the PLANNER (`bcinr-pddl`'s `find_plan` itself),
across problems this session did not author, so the same
"redesign-the-domain" fix does not obviously apply here.

**This is a hypothesis, not a confirmed root cause** -- it has not been
verified against `bcinr-pddl`'s actual BFS/search implementation, and the
alternative explanation (bcinr's planner is CORRECT and wasm4pm's is
needlessly longer/suboptimal, since BFS classical planning does not
guarantee shortest-plan optimality is being compared apples-to-apples
between two different search strategies) has not been ruled out either.
`pair4_objective_score_bit_exact` and `pair2_conformance_powl_vs_petri_
agreement`/`pair3_chain_recompute_vs_independent_100_records` (unrelated
non-planning cross-checks in the same file) all pass, so this is scoped
specifically to the classical-planner step-count comparison.

## Why not fixed here

Determining which planner (if either) is wrong requires reading a specific
disagreeing instance's full ground/plan trace from BOTH planners side by
side (not just the aggregate step counts the test currently reports), and
possibly instrumenting one run to print the actual bound action sequence
-- genuinely separate, deeper work than PROJ-818's single-fixture type-
filtering bug, per the same "Fence" discipline
(`.claude/rules/_core/absolute.md`) that scoped PROJ-817/818 as separate
tickets rather than one grab-bag fix.

## Verification plan (once root-caused)

```
cargo test -p my-conforming-project --test differential
```
All 8 tests must pass, or the 3 failing tests' expectations must be
revised with a disclosed, justified reason (e.g. "bcinr is intentionally
non-optimal and this test's equality expectation was wrong") -- not
silently loosened.

## See Also

- `docs/jira/v26.8.16/PROJ-818-bribery-case-grounding-blowup.md` -- the
  sibling finding from the same "this file finally compiles again" wave,
  fixed; this one remains open
- `crates/multifractal-workflow/fixtures/bribery-case/pddl-domain.ttl`
  lines 168-215 -- the prior, structurally similar (but domain-side, not
  planner-side) STRIPS8-inequality-shortcut bug this session's predecessor
  found and fixed by redesigning predicates
