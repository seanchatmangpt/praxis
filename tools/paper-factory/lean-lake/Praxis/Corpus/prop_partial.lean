import Praxis.Corpus.def_decoder

/-!
# prop:partial

`scnop_ℓ` (`scenario_for_denial_lane`) is a partial map, definite (as `Some`) only on
`L \ {Adml}` -- it returns `none` on `Adml` and `some _` on each of the seven named
single-lane constants -- and it is not realizable as a wildcard-free exhaustive match:
its domain `Deny` (`DenialPolarity`, a newtype over `UInt64`) is an open value space of
cardinality `2^64`, not the closed 8-element enum `L`, so there exist `DenialPolarity`
values outside `L` entirely (e.g. the composite of two lane constants). Exhaustiveness
over the seven single lanes is therefore a runtime test obligation, not something the
Lean/Rust type checker can discharge structurally -- matching the informal claim.

Both conjuncts reduce to decidable equalities on concrete `UInt64` numerals (`DenialPolarity`
derives `DecidableEq`), so the whole proposition is closed by `decide` -- no new Mathlib
machinery needed beyond what `def:decoder`/`def:denialcode` already bring in.
-/

open DenialPolarity RefusalScenario

/-- `scnop_ℓ` is definite (as `Some`) only on `L \ {Adml}`: `none` on the clean word,
`some` of the matching scenario on each of the seven named single-lane constants; and
the domain `Deny` is not exhausted by the eight named elements of `L` -- witnessed by
the composite of two lane constants, which lies outside `L` entirely. -/
theorem prop_partial :
    (scenario_for_denial_lane DenialPolarity.Adml = none ∧
      scenario_for_denial_lane DenialPolarity.lane1 = some RefusalScenario.lane1Denial ∧
      scenario_for_denial_lane DenialPolarity.lane2 = some RefusalScenario.lane2Denial ∧
      scenario_for_denial_lane DenialPolarity.lane3 = some RefusalScenario.lane3Denial ∧
      scenario_for_denial_lane DenialPolarity.lane4 = some RefusalScenario.lane4Denial ∧
      scenario_for_denial_lane DenialPolarity.lane5 = some RefusalScenario.lane5Denial ∧
      scenario_for_denial_lane DenialPolarity.lane6 = some RefusalScenario.lane6Denial ∧
      scenario_for_denial_lane DenialPolarity.lane7 = some RefusalScenario.lane7Denial) ∧
    ∃ d : DenialPolarity,
      d ≠ DenialPolarity.Adml ∧
      d ≠ DenialPolarity.lane1 ∧ d ≠ DenialPolarity.lane2 ∧ d ≠ DenialPolarity.lane3 ∧
      d ≠ DenialPolarity.lane4 ∧ d ≠ DenialPolarity.lane5 ∧ d ≠ DenialPolarity.lane6 ∧
      d ≠ DenialPolarity.lane7 := by
  refine ⟨⟨rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl⟩, ?_⟩
  exact ⟨DenialPolarity.compose DenialPolarity.lane1 DenialPolarity.lane2, by decide⟩
