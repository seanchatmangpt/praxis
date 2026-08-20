# PROJ-819: `wasm4pm` and `bcinr-pddl` classical planners systematically disagree on plan step count

**Status**: OPEN -- root cause CONFIRMED (not a hypothesis anymore), not
fixed. Fix belongs in the external `wasm4pm-planner` sibling repo, out of
this session's scope.

**Confirmed mechanism** (read `/Users/sac/wasm4pm/crates/wasm4pm-planner/
src/ground.rs:198-287`'s `find_temporal_plan`, the "greedy tick-based
scheduler"): its in-flight guard only checks `pending` (actions
*currently* mid-duration) before starting a ground action --
`pending.iter().any(|(_, idx)| *idx == i)` -- it never checks whether that
action was already started and completed **earlier** in the plan. In an
add-monotone domain (no delete effects -- exactly `pair1_planners_
revenue_stage_chain`'s 4-schema chain, and most of the generated corpus),
every action's precondition remains true forever once first satisfied, so
the scheduler keeps re-starting already-completed actions on every
subsequent tick until the goal happens to be reached, inflating the step
count with pure no-op-effect repeats (state.insert on an already-`true`
atom is a no-op, but each restart still pushes a `PlanStep`). This exactly
explains both the direction (`wasm4pm` always >= `bcinr`, never less) and
the corpus-wide consistency (every generated instance is add-monotone by
construction, per `GenModel`'s own domain generator) of the disagreement.
`bcinr-pddl`'s BFS-based classical/temporal planners don't have this
failure mode because BFS naturally finds a shortest action sequence and
has no notion of "keep re-starting anything still applicable."

This is a genuine `wasm4pm-planner` correctness/minimality bug (it
produces plans with redundant, effect-free repeated actions), not a
`bcinr-pddl` or praxis bug -- fixing it means adding a "skip if this exact
ground action instance already fired and its post-state already holds all
its add-effects" guard to that scheduler, a real design change in a
sibling repo this session does not own or have standing to change
unilaterally.

**Dependencies**: none (independent of PROJ-811/814/815/816/817/818)

## Scope

`tests/differential.rs` cross-checks `wasm4pm`'s own planner against
`bcinr-pddl`'s classical `GroundProblem::find_plan` over the same domains
and problems, expecting agreement on solvability and (where both solve)
step count. This file was compile-blocked most of this session (stale
`Result`-based `find_plan`/`find_temporal_plan` API usage, fixed alongside
`indexed_grounding.rs` and `bribery_case_pddl.rs` in commit `37fe4405`) and
has therefore not actually run in a long time. Now that it compiles, 3 of
8 tests failed for real; one of the three (`pair1_scope_classical_
exemplars`) is FIXED, in commit `1eedcd8e` -- it was reading `ontology/
revenue.pddl`'s raw file directly instead of using
`revenue::revenue_domain_text()` to strip that file's trailing example
`(define (problem ...))` block before parsing, and `domain_from_pddl`
correctly refuses a domain+problem concatenation. The remaining 2 failures
(the actual planner-disagreement finding this ticket is about) are
confirmed root-caused below, not fixed:

```
pair1_scope_classical_exemplars (FIXED, commit 1eedcd8e):
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

## Why not fixed here

The mechanism above is confirmed by reading `find_temporal_plan`'s actual
scheduler code, not inferred from step-count arithmetic alone. But fixing
it means changing `wasm4pm-planner`'s scheduling policy in a sibling repo
this session does not own or have standing to modify unilaterally --
adding a completed-action guard is a real design decision (does the
scheduler skip an already-fired ground action forever once its add-effects
hold, or only until something deletes one of them again?) that belongs
with that repo's own maintainers, not a unilateral cross-repo patch.

A first parser failure in the same test run
(`pair1_scope_classical_exemplars`: `bcinr parses revenue.pddl (classical
:strips): ParseError("trailing token at index 214")`) is a separate,
narrower, still-unconfirmed issue -- not yet traced to a root cause.

## Verification plan (once fixed upstream)

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
