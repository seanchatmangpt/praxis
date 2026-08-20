# PROJ-818: bribery-case PDDL grounding produces 4965 ground actions for a 15-object problem (suspected type-filter bug)

**Status**: DONE -- root-caused and fixed in commit `375fc669`. Confirmed by
dumping the actual manufactured PDDL8 domain text (correctly typed) and
tracing `mfg::solve_ir`, which converts the IR directly via `impl
From<&ActionDecl> for Pddl8ActionSchema` rather than re-parsing that text --
that conversion set `typed_params: Vec::new()` unconditionally, discarding
every parameter's type a second time (it was already read once, correctly,
to build the untyped `params` field) instead of copying it from
`a.params: Vec<(String, String)>`. Fixed: `typed_params: a.params.clone()`.
`tests/bribery_case_pddl.rs`'s 3 tests all pass now (was: 1 of 3 failing on
`PDDL8_MAX_GROUND` bound exceeded, `got=4965` against a hand-computed
type-correct estimate of ~26).
**Dependencies**: none (independent of PROJ-811/814/815/816/817)

## Scope

`tests/bribery_case_pddl.rs::closable_case_grounds_and_solves_to_receipted`
was compile-blocked all session (stale `.domain_text` field access, fixed in
commit `37fe4405`) and has therefore not actually **run** in a long time.
Now that it compiles and runs, it fails for real:

```
thread 'closable_case_grounds_and_solves_to_receipted' panicked at
tests/bribery_case_pddl.rs:97:5:
GroundProblem::find_plan must find a real plan reaching receipted:
Some("PDDL8 bound exceeded: ground actions limit=4096 got=4965")
```

## Why this is suspicious, not just "the domain is big"

The manufactured problem
(`crates/multifractal-workflow/fixtures/bribery-case/{pddl-domain,
pddl-problem-closable}.ttl`) has exactly **15 objects across 7 types**:

| Type | Count | Instances |
|---|---|---|
| `lifecycle-stage` | 5 | raw, admitted, validated, blocked, receipted |
| `obligation` | 3 | assess-policy-violation, verify-contractor-authorization-level, verify-transaction-authenticity |
| `evidence-type` | 3 | etype-authorization-record, etype-card-statement, etype-compliance-policy-citation |
| `law-object` | 1 | case-brb-2026-0417 |
| `validator` | 1 | compliance-officer-shreya-patel |
| `authority` | 1 | general-counsel-marcus-webb |
| `chain-token` | 1 | tok-genesis-brb-2026-0417 |

The domain has 9 action schemas (`supply-evidence`,
`clear-{transaction,authorization,policy}-obligation`,
`close-obligations`, `judge`, `admit`, `receipt`,
`block-for-missing-evidence`), each with 2-3 **typed** parameters (`grep -c
"pddl:param"` = 25 total param declarations across all actions,
`pddl:ofType` breakdown: 21 `law-object`, 10 `obligation`, 3 `chain-token`,
2 `evidence-type`, 2 `authority`, 2 `validator`, 1 `lifecycle-stage`).

With type-correct grounding, the largest single-schema product should be on
the order of `1 (law-object) x 3 (obligation) x 3 (evidence-type)` = 9 for
`supply-evidence`, and every other schema is smaller (2 params, smaller
type domains). Summed across all 9 schemas this should land well under
100 ground actions, not 4965 -- roughly **50x** more than the type-correct
estimate.

## Hypothesis (not yet confirmed)

4965 is much closer to what you'd get if grounding ignored `pddl:ofType`
constraints and took the Cartesian product of each untyped parameter slot
over **all 15 objects regardless of type** (e.g. one 3-parameter action
schema alone would produce `15^3 = 3375`, and a couple of 2-parameter
schemas at `15^2 = 225` each would close most of the remaining gap to
4965). This would point at either:

1. A real correctness regression in the currently-vendored `bcinr-pddl`'s
   classical grounder (`GroundProblem::build`) -- type filtering silently
   not applied, or
2. A parsing issue in how `pddl-domain.ttl`'s `pddl:param`/`pddl:ofType`
   pairs get lowered to `Pddl8ActionSchema` (via `crate::mfg`'s RDF ->
   PDDL8 IR path), such that the emitted PDDL8 domain text loses the type
   annotations before it ever reaches `bcinr-pddl`'s parser, or
3. The manufactured PDDL8 **problem** text's `(:objects ...)` block is
   missing type annotations for some objects (so the grounder correctly
   falls back to "untyped = ranges over everything" for those), which
   would be visible directly in `mfg::build_problem`'s or the test's own
   `eprintln!`-printed problem text (already captured in the test's stdout
   above -- every object line does show a `- <type>` suffix, so this third
   hypothesis looks LESS likely on a first read, but was not ruled out by
   inspecting the actual PDDL8 domain text `bcinr-pddl` receives, only the
   problem text).

None of these three has been confirmed by reading `bcinr-pddl`'s actual
grounder code or the exact PDDL8 domain text `mfg::manufacture` emits for
this fixture -- this ticket records the observation and the arithmetic
that makes it suspicious, not a root cause.

## Why not fixed here

This is a potential correctness bug in a sibling repo's (`bcinr-pddl`)
grounder, or in this repo's own RDF-to-PDDL8 lowering (`src/mfg.rs`'s
`emit_domain`/`emit_problem` path) -- either requires reading the exact
domain text handed to the solver and the grounder's own type-matching
logic before proposing a fix, which was not done in this session (already
substantial scope covered by PROJ-817's fix). Per
`.claude/rules/_core/absolute.md`'s "Fence" discipline, the exact
boundary needs identifying before any change.

## Verification plan (once root-caused)

```
cargo test -p my-conforming-project --test bribery_case_pddl
```
`closable_case_grounds_and_solves_to_receipted` must pass, and the fix
should include an assertion or log line on the real ground-action count
for this fixture (so a future regression back toward the thousands is
caught immediately rather than only when it crosses 4096).

## See Also

- `docs/jira/v26.8.16/PROJ-817-mfg-test-module-stale.md` -- the fix that
  made this test compile and run for the first time in a long while,
  surfacing this finding
- `crates/multifractal-workflow/fixtures/bribery-case/pddl-domain.ttl`
  lines 168-215 -- the domain's own doc comments about a related, already
  -fixed grounding-loophole bug this session's predecessor found and
  disclosed (distinct issue: STRIPS8's lack of inequality constraints,
  not a type-filtering blowup)
