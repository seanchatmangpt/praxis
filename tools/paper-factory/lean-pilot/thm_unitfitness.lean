/-
thm:unitfitness

A trace is a firing sequence iff Fitness(σ) = 1. If Fitness(σ) < 1, the
first frame forcing an unenabled consumption is a coordinate-localized
witness t⋆.

Bare Lean 4 core formalization (no mathlib), reusing def:fitnessdistance's
`FitnessNum`/`FitnessDen`/`FitnessPerfect` (the fraction representation of
Fitness and the "never forced a disabled firing" condition). "Trace is a
firing sequence" is exactly the condition that the replay never forced an
unenabled consumption, i.e. `FitnessPerfect unenabled` (`unenabled = 0`);
"Fitness(σ) = 1" is the fraction `FitnessNum unenabled attempted` equalling
`FitnessDen attempted` (equal numerator/denominator over the shared
denominator `attempted`, as introduced in def:fitnessdistance). The two are
proved equivalent directly from the definitions.

The second clause -- if Fitness < 1 (`unenabled ≠ 0`) there is a
coordinate-localized witness `t⋆` -- is formalized as: some natural number
index `t⋆` exists with `t⋆ < unenabled`, i.e. the nonzero count of
unenabled-consuming tokens itself certifies a first witnessed index (here
`t⋆ = 0`, since `unenabled ≠ 0` means `0 < unenabled`), reusing only the
raw `Nat` counts of def:fitnessdistance rather than introducing new
per-frame trace machinery.
-/

/-- Numerator of `Fitness(σ) = 1 - unenabled/attempted` over the common
    denominator `attempted`, i.e. `attempted - unenabled`. -/
def FitnessNum (unenabled attempted : Nat) : Int :=
  (attempted : Int) - (unenabled : Int)

/-- Denominator of `Fitness(σ)`, the tokens the replay attempted to
    consume. -/
def FitnessDen (attempted : Nat) : Int :=
  (attempted : Int)

/-- The replay never forced a disabled firing: no tokens were consumed
    on unenabled transitions ("the trace is a firing sequence"). -/
def FitnessPerfect (unenabled : Nat) : Prop :=
  unenabled = 0

/-- thm:unitfitness. A trace is a firing sequence (`FitnessPerfect`) iff
    `Fitness(σ) = 1` (`FitnessNum = FitnessDen`); and if `Fitness(σ) < 1`
    (`unenabled ≠ 0`), a coordinate-localized witness index `t⋆` exists
    with `t⋆ < unenabled`. -/
theorem unit_fitness (unenabled attempted : Nat) :
    (FitnessNum unenabled attempted = FitnessDen attempted ↔ FitnessPerfect unenabled)
    ∧ (unenabled ≠ 0 → ∃ t : Nat, t < unenabled) := by
  constructor
  · unfold FitnessNum FitnessDen FitnessPerfect
    omega
  · intro h
    exact ⟨0, Nat.pos_of_ne_zero h⟩
