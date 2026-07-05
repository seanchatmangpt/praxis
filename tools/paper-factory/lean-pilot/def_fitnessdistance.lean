/-
def:fitnessdistance

Fitness(σ) = 1 - (tokens consumed on unenabled transitions) / (tokens the
replay attempted to consume) ∈ [0,1]; Fitness = 1 iff the replay never
forced a disabled firing, and 1 - Fitness is a normalized ℓ¹ excursion
outside the enabled set.

Bare Lean 4 core formalization (no mathlib). Division on a field is not
available in core, so the ratio 1 - unenabled/attempted is represented
as a fraction (numerator, denominator) pair rather than evaluated: the
numerator of `1 - unenabled/attempted` over common denominator
`attempted` is `attempted - unenabled`. This mirrors prop:conformembership's
style of working directly with the raw Int/Nat counts (as in NonnegParikh,
NonnegMarking) rather than introducing a rational-number type.

`unenabled` : tokens consumed on unenabled (disabled) transitions during
  the replay of trace σ.
`attempted` : total tokens the replay attempted to consume.

`FitnessNum`/`FitnessDen` give the fraction `FitnessNum / FitnessDen`
representing Fitness(σ), i.e. `attempted - unenabled` over `attempted`.
`FitnessPerfect` is the "never forced a disabled firing" condition
`unenabled = 0`, which by construction makes the fraction equal 1.
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
    on unenabled transitions. Under this condition `FitnessNum` equals
    `FitnessDen`, i.e. the represented fraction is `1`. -/
def FitnessPerfect (unenabled : Nat) : Prop :=
  unenabled = 0

/-- The excursion `1 - Fitness(σ)`, as a fraction over the same
    denominator: numerator `unenabled`, denominator `attempted`. This is
    the normalized ℓ¹ excursion outside the enabled set. -/
def ExcursionNum (unenabled : Nat) : Int :=
  (unenabled : Int)

/-- The excursion fraction shares `FitnessDen`'s denominator, and its
    numerator plus `FitnessNum`'s numerator recovers `attempted`,
    witnessing `Fitness(σ) + (1 - Fitness(σ)) = 1` at the level of raw
    numerators over the common denominator `attempted`. -/
def excursion_fitness_sum (unenabled attempted : Nat) :
    ExcursionNum unenabled + FitnessNum unenabled attempted = FitnessDen attempted :=
  by simp only [ExcursionNum, FitnessNum, FitnessDen]; omega
