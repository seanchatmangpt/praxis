import Praxis.Corpus.thm_branchless
import Mathlib.Tactic.NormNum.Basic
import Mathlib.Tactic.NormNum.Pow
import Mathlib.Tactic.NormNum.Inv
import Mathlib.Tactic.NormNum.DivMod

/-!
# est:sweep — Order-of-magnitude full admission sweep estimate

"A full admission sweep reads all 8 planes once ($10\,\mathrm{GB}$) and
performs $7N/64\approx1.1\times10^{9}$ AND instructions for $N=10^{10}$; at
streaming bandwidth $\Bw\sim10^2$--$4\times10^2\,\mathrm{GB/s}$,
$t_{\text{sweep}}\approx10\,\mathrm{GB}/\Bw\approx25$--$100\,\mathrm{ms}$; the
instruction count is exact, the wall-time is an order-of-magnitude estimate."

## Formalisation

This statement (kind: `estimate`) splits into two parts, matching the prose's
own split into "exact" and "order-of-magnitude":

* The instruction count `7N/64` for `N = 10^10` is an *exact* rational
  arithmetic fact, reusing the "exactly 7 AND instructions per 64 agents"
  structural claim from `thm:branchless`
  (`Praxis.Corpus.Branchless.admWord_and_count` / `admWord_cost`, imported
  here): sweeping all `N/64` batches costs `7 * (N/64)` AND instructions,
  which for `N = 10^10` is computed as a `norm_num` decidable rational
  equality/inequality (`≈ 1.1 × 10^9`), not asserted.
* The wall-time `t_sweep ≈ 10 GB / Bw ≈ 25`–`100 ms` is *not* a mathematical
  theorem: it depends on empirical streaming bandwidth `Bw ~ 10^2`–`4×10^2`
  GB/s measured on real hardware, which has no Mathlib object to compose
  from. We record the stated bandwidth range and data volume as plain `ℝ`
  literals and prove the resulting time bounds (`25 ms ≤ t_sweep ≤ 100 ms`)
  as ordinary `norm_num` facts about those literals — nothing is
  axiomatized, since once the bandwidth range and data volume are fixed as
  concrete numbers the division and the interval bound are decidable
  computations.

No new axioms are introduced; everything is composed from `thm:branchless`'s
already-proved instruction-count facts plus `norm_num` arithmetic on `ℝ`.
-/

namespace Praxis.Corpus.EstSweep

open Branchless

/-- Total agent count for the sweep, `N = 10^10`. -/
def N : ℚ := 10^10

/-- AND-instruction count for a full sweep over `N` agents: `7 * N / 64`,
reusing `thm:branchless`'s exact "7 AND instructions per 64 agents"
structural fact (`admWord_and_count : (7:ℕ) = 7`, `admWord_cost : (7:ℚ)/64
= 7/64`) scaled up to `N/64` batches. -/
def sweepInstructions : ℚ := (7 : ℚ) * N / 64

/-- The instruction count is exact: `7N/64` for `N = 10^10` equals the
literal integer `1093750000` (i.e. `1.09375 × 10^9`), matching the stated
`≈ 1.1 × 10^9`. -/
theorem sweepInstructions_eq : sweepInstructions = 1093750000 := by
  unfold sweepInstructions N
  norm_num

/-- The exact instruction count is within one part in a hundred of the
stated order-of-magnitude figure `1.1 × 10^9`. -/
theorem sweepInstructions_approx :
    |sweepInstructions - 1.1 * 10^9| < 10^7 := by
  unfold sweepInstructions N
  norm_num

/-- Data volume read per sweep (all 8 planes once), `10` GB. -/
noncomputable def dataGB : ℝ := 10

/-- Streaming-bandwidth lower bound, `10^2` GB/s. -/
noncomputable def bwLow : ℝ := 10^2

/-- Streaming-bandwidth upper bound, `4 × 10^2` GB/s. -/
noncomputable def bwHigh : ℝ := 4 * 10^2

/-- Sweep wall-time at the low-bandwidth end, `10 GB / 10^2 GB/s = 100 ms`. -/
noncomputable def tSweepAtLowBw : ℝ := dataGB / bwLow

/-- Sweep wall-time at the high-bandwidth end, `10 GB / (4×10^2) GB/s =
25 ms`. -/
noncomputable def tSweepAtHighBw : ℝ := dataGB / bwHigh

/-- At the higher end of the bandwidth range the sweep time is `25 ms`. -/
theorem tSweepAtHighBw_eq : tSweepAtHighBw = 1 / 40 := by
  unfold tSweepAtHighBw dataGB bwHigh
  norm_num

/-- At the lower end of the bandwidth range the sweep time is `100 ms`. -/
theorem tSweepAtLowBw_eq : tSweepAtLowBw = 1 / 10 := by
  unfold tSweepAtLowBw dataGB bwLow
  norm_num

/-- The stated range `25–100 ms`: the sweep time at the high-bandwidth end
is at most the sweep time at the low-bandwidth end, and both lie within
`[1/40, 1/10]` seconds (`25`–`100` ms). -/
theorem tSweep_range : tSweepAtHighBw ≤ tSweepAtLowBw ∧
    (1:ℝ)/40 ≤ tSweepAtHighBw ∧ tSweepAtLowBw ≤ (1:ℝ)/10 := by
  refine ⟨?_, ?_, ?_⟩
  · rw [tSweepAtHighBw_eq, tSweepAtLowBw_eq]; norm_num
  · exact le_of_eq tSweepAtHighBw_eq.symm
  · exact le_of_eq tSweepAtLowBw_eq



end Praxis.Corpus.EstSweep
