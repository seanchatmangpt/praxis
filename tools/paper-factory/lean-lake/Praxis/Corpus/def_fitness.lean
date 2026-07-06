import Mathlib.Data.Rat.Defs
import Mathlib.Data.Rat.Floor

/-!
# `def:fitness`

Replay fitness is
`Fitness = 1 - (tokens forced on unenabled transitions) / (tokens the replay attempted) ∈ [0,1]`,
represented as Q16.16 with unit `0x0001_0000 = 65536`; the `token_replay` stage accepts a
record iff its replayed fitness equals that unit.

We model the two replay counters as natural numbers (`forced ≤ attempted`, `attempted > 0`),
compose the fitness value as a `ℚ` via Mathlib's rational-number division (no bespoke axiom
needed -- `ℚ` and its field operations already exist in Mathlib), and separately give the
Q16.16 fixed-point encoding as an integer scaled by the unit `65536`.
-/

namespace Praxis

/-- A replay-fitness measurement: `forced` tokens forced on unenabled transitions out of
`attempted` tokens the replay attempted. -/
structure ReplayCounts where
  forced     : Nat
  attempted  : Nat
  forced_le_attempted : forced ≤ attempted
  attempted_pos : 0 < attempted

/-- Replay fitness as a rational number in `[0, 1]`, computed from `ReplayCounts` using
Mathlib's field structure on `ℚ` (division is genuine rational division, not asserted-in). -/
def ReplayCounts.fitness (c : ReplayCounts) : ℚ :=
  1 - (c.forced : ℚ) / (c.attempted : ℚ)

/-- The fitness value always lies in `[0, 1]`. -/
theorem ReplayCounts.fitness_mem_unit_interval (c : ReplayCounts) :
    0 ≤ c.fitness ∧ c.fitness ≤ 1 := by
  have hattempted : (0:ℚ) < (c.attempted : ℚ) := by exact_mod_cast c.attempted_pos
  have hforced_le : (c.forced : ℚ) ≤ (c.attempted : ℚ) := by exact_mod_cast c.forced_le_attempted
  have hforced_nonneg : (0:ℚ) ≤ (c.forced : ℚ) := by exact_mod_cast Nat.zero_le c.forced
  have hdiv_le_one : (c.forced : ℚ) / (c.attempted : ℚ) ≤ 1 := by
    rw [div_le_one hattempted]; exact hforced_le
  have hdiv_nonneg : (0:ℚ) ≤ (c.forced : ℚ) / (c.attempted : ℚ) :=
    div_nonneg hforced_nonneg (le_of_lt hattempted)
  constructor
  · unfold ReplayCounts.fitness; linarith
  · unfold ReplayCounts.fitness; linarith

/-- The Q16.16 fixed-point unit: `0x0001_0000 = 65536`, representing the rational value `1`. -/
def q16_16Unit : Int := 65536

/-- Encode a rational fitness value as Q16.16 fixed point: multiply by the unit `65536` and
take the floor (via `Int.floor`, already provided by Mathlib's `LinearOrderedField`/`FloorRing`
instance for `ℚ`). -/
def encodeQ16_16 (x : ℚ) : Int :=
  ⌊x * (q16_16Unit : ℚ)⌋

/-- The `token_replay` stage's acceptance predicate: a record is accepted iff its replayed
fitness, once encoded in Q16.16, equals the unit `0x0001_0000 = 65536` (i.e. the underlying
rational fitness is exactly `1`). -/
def tokenReplayAccepts (c : ReplayCounts) : Prop :=
  encodeQ16_16 c.fitness = q16_16Unit

end Praxis
