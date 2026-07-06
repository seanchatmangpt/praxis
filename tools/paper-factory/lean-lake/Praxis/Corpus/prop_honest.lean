import Praxis.Corpus.def_fitness

/-!
# `prop:honest`

"Fitness and precision are ratios of bit-populations of disjointly-maintained
bitsets; neither can exceed 1, and Fitness = 1 is attained exactly on
completed replays; `enabled_not_taken` feeds precision, not fitness, so the
two metrics measure orthogonal deviations."

We reuse `Praxis.ReplayCounts`/`ReplayCounts.fitness` from `def:fitness`
directly (invariant 6): no new fitness representation is introduced. For
precision we introduce the dual counter structure `PrecisionCounts`, built
from a disjoint pair of natural-number bit-population counts
(`enabled_not_taken`, `offered`), mirroring exactly how `ReplayCounts` is
built from (`forced`, `attempted`) in `def:fitness` -- same shape, disjoint
fields, no shared state between the two metrics. This *is* the "disjointly
maintained bitsets" and "orthogonal deviations" claim: `ReplayCounts` and
`PrecisionCounts` share no field, so a change to `enabled_not_taken` cannot
alter `fitness`, and a change to `forced`/`attempted` cannot alter
`precision`, by construction (`orthogonal_deviations` below, checked
definitionally).

Both ratio bounds reuse Mathlib's `ℚ` field/order lemmas (`div_le_one`,
`div_nonneg`) exactly as `def:fitness` already does -- no bespoke axiom. The
"Fitness = 1 exactly on completed replays" clause is the genuine proof
obligation: a completed (violation-free) replay is one with `forced = 0`,
and we show `fitness_eq_one_iff` characterizing that exactly.
-/

namespace Praxis

/-- A precision measurement: `enabled_not_taken` tokens that were enabled but
never taken, out of `offered` tokens the replay offered. Disjoint fields from
`ReplayCounts` (`forced`/`attempted`): precision and fitness are maintained
from independent bit-populations. -/
structure PrecisionCounts where
  enabled_not_taken     : Nat
  offered               : Nat
  ent_le_offered        : enabled_not_taken ≤ offered
  offered_pos           : 0 < offered

/-- Precision as a rational number in `[0, 1]`, computed from
`PrecisionCounts` using Mathlib's field structure on `ℚ`, mirroring
`ReplayCounts.fitness`. -/
def PrecisionCounts.precision (c : PrecisionCounts) : ℚ :=
  1 - (c.enabled_not_taken : ℚ) / (c.offered : ℚ)

/-- Precision never exceeds `1` and is never negative: same bit-population
ratio argument as `ReplayCounts.fitness_mem_unit_interval`. -/
theorem PrecisionCounts.precision_mem_unit_interval (c : PrecisionCounts) :
    0 ≤ c.precision ∧ c.precision ≤ 1 := by
  have hoffered : (0:ℚ) < (c.offered : ℚ) := by exact_mod_cast c.offered_pos
  have hent_le : (c.enabled_not_taken : ℚ) ≤ (c.offered : ℚ) := by
    exact_mod_cast c.ent_le_offered
  have hent_nonneg : (0:ℚ) ≤ (c.enabled_not_taken : ℚ) := by
    exact_mod_cast Nat.zero_le c.enabled_not_taken
  have hdiv_le_one : (c.enabled_not_taken : ℚ) / (c.offered : ℚ) ≤ 1 := by
    rw [div_le_one hoffered]; exact hent_le
  have hdiv_nonneg : (0:ℚ) ≤ (c.enabled_not_taken : ℚ) / (c.offered : ℚ) :=
    div_nonneg hent_nonneg (le_of_lt hoffered)
  constructor
  · unfold PrecisionCounts.precision; linarith
  · unfold PrecisionCounts.precision; linarith

/-- Fitness attains its ceiling `1` exactly on completed (violation-free)
replays, i.e. exactly when no token was forced onto an unenabled
transition (`forced = 0`). This is the genuine "attained exactly on
completed replays" proof obligation. -/
theorem ReplayCounts.fitness_eq_one_iff (c : ReplayCounts) :
    c.fitness = 1 ↔ c.forced = 0 := by
  have hattempted : (0:ℚ) < (c.attempted : ℚ) := by exact_mod_cast c.attempted_pos
  have hattempted_ne : (c.attempted : ℚ) ≠ 0 := ne_of_gt hattempted
  constructor
  · intro h
    have : (c.forced : ℚ) / (c.attempted : ℚ) = 0 := by
      unfold ReplayCounts.fitness at h; linarith
    have hforced0 : (c.forced : ℚ) = 0 := by
      have := (div_eq_zero_iff).mp this
      rcases this with h0 | h0
      · exact h0
      · exact absurd h0 hattempted_ne
    exact_mod_cast hforced0
  · intro h
    unfold ReplayCounts.fitness
    rw [h]
    simp

/-- Fitness and precision measure orthogonal deviations: they are functions
of disjoint structures (`ReplayCounts` has no `enabled_not_taken` field,
`PrecisionCounts` has no `forced`/`attempted` field), so `enabled_not_taken`
cannot appear in the definition of `fitness` and `forced`/`attempted` cannot
appear in the definition of `precision` -- true definitionally, not merely
proof-theoretically, since the two definitions each only project their own
structure's fields. -/
theorem orthogonal_deviations (c : ReplayCounts) (p : PrecisionCounts) :
    c.fitness = 1 - (c.forced : ℚ) / (c.attempted : ℚ) ∧
    p.precision = 1 - (p.enabled_not_taken : ℚ) / (p.offered : ℚ) :=
  ⟨rfl, rfl⟩

end Praxis
