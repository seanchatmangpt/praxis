import Praxis.Corpus.def_fitness

/-!
# `prop:fitness`

A record's replay attains `Fitness = 0x0001_0000` iff its lifecycle is a genuine firing
sequence of the fixed POWL token model (i.e. no tokens were ever forced onto unenabled
transitions); otherwise the first forced consumption localizes the nonconformant step.

We formalize the "genuine firing sequence" side as `c.forced = 0`: the replay never had to
force a token onto an unenabled transition. The proposition is then that the `token_replay`
stage's Q16.16 acceptance predicate holds iff `forced = 0`.
-/

namespace Praxis

/-- `token_replay` accepts a record's replay iff no tokens were ever forced onto unenabled
transitions during that replay (`forced = 0`), i.e. iff the replayed lifecycle is a genuine
firing sequence of the fixed POWL token model. -/
theorem tokenReplayAccepts_iff_no_forced (c : ReplayCounts) :
    tokenReplayAccepts c ↔ c.forced = 0 := by
  have hattempted : (0:ℚ) < (c.attempted : ℚ) := by exact_mod_cast c.attempted_pos
  have hmem := c.fitness_mem_unit_interval
  unfold tokenReplayAccepts encodeQ16_16 q16_16Unit
  constructor
  · intro h
    -- from `⌊c.fitness * 65536⌋ = 65536` and `c.fitness ≤ 1` deduce `c.fitness = 1`.
    have hle : c.fitness ≤ 1 := hmem.2
    have hval_ge : (65536 : ℚ) ≤ c.fitness * (65536 : ℚ) := by
      have hcast : ((65536 : ℤ) : ℚ) = (65536 : ℚ) := by norm_num
      have h' : (65536 : ℤ) ≤ ⌊c.fitness * (65536 : ℚ)⌋ := le_of_eq h.symm
      have hle' := (Int.le_floor (α := ℚ) (z := (65536:ℤ)) (a := c.fitness * (65536:ℚ))).mp h'
      rwa [hcast] at hle'
    have hval_ge' : (1 : ℚ) ≤ c.fitness := by nlinarith
    have hfitness_eq_one : c.fitness = 1 := le_antisymm hle hval_ge'
    have : (1 : ℚ) - (c.forced : ℚ) / (c.attempted : ℚ) = 1 := hfitness_eq_one
    have hdiv0 : (c.forced : ℚ) / (c.attempted : ℚ) = 0 := by linarith
    have hforced0 : (c.forced : ℚ) = 0 := by
      by_contra hne
      have hpos : (0:ℚ) < (c.forced : ℚ) := lt_of_le_of_ne (by exact_mod_cast Nat.zero_le c.forced) (Ne.symm hne)
      have : (0:ℚ) < (c.forced : ℚ) / (c.attempted : ℚ) := div_pos hpos hattempted
      linarith
    exact_mod_cast hforced0
  · intro h
    have hfitness_eq_one : c.fitness = 1 := by
      unfold ReplayCounts.fitness
      rw [h]
      simp
    rw [hfitness_eq_one]
    norm_num

end Praxis
