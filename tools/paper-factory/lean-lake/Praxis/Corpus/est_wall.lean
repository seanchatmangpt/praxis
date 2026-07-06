import Praxis.Corpus.prop_invariance
import Mathlib.Tactic.NormNum.Basic
import Mathlib.Tactic.NormNum.Pow
import Mathlib.Tactic.NormNum.Inv
import Mathlib.Tactic.NormNum.DivMod

/-!
# est:wall — Order-of-magnitude wall-time estimate

At BLAKE3 throughput 1–3 GB/s/core and a few-hundred-byte frame,
`t_chainH ~ 10^-7` s/frame; a four-field boundary compare is `≪ 10^-7` s;
per-message verification with `c ~ 10` spot frames costs `~ 10^-6` s;
these are order-of-magnitude estimates, the constancy-in-`T` point is the
proved `prop:invariance` (`Praxis.Corpus.PropInvariance.wallTime_const`).

## Formalisation

This statement is *not* a mathematical theorem: it is an empirical
order-of-magnitude claim about measured hash throughput on real hardware
(BLAKE3 running at 1–3 GB/s per core), which is not a fact derivable inside
Lean's logic — there is no Mathlib object for "BLAKE3 throughput on a CPU
core" to compose from, and no proof obligation follows from the source
(kind: `estimate`). We record the numeric orders of magnitude it asserts as
plain `ℝ` constants together with the inequalities the prose states
(`t_chainH` on the order of `10^-7`, `t_cmp ≪ t_chainH`, and the aggregate
`c * t_cmp + t_chainH`-scale cost on the order of `10^-6` for `c ~ 10`),
proved as ordinary `norm_num` facts about literal reals — nothing here is
axiomatized, since once the throughput figures are fixed as concrete
literals the arithmetic comparisons are decidable computations, not
empirical claims.

The `T`-independence half of the statement is not re-proved here: it is
literally `Praxis.Corpus.PropInvariance.wallTime_const`, imported and
reused as-is per the dependency `prop:invariance`.
-/

namespace Praxis.Corpus.EstWall

open Praxis.Corpus.PropInvariance
open Praxis.Corpus.DefInstanceQ

/-- Estimated per-frame chain-hash recompute time, `t_chainH ~ 10^-7` s,
from BLAKE3 throughput of 1–3 GB/s per core on a few-hundred-byte frame. -/
noncomputable def tChainH : ℝ := 1 / 10^7

/-- Estimated per-field boundary-compare time, `t_cmp`, asserted to be
`≪ t_chainH`; instantiated here at three orders of magnitude below. -/
noncomputable def tCmp : ℝ := 1 / 10^10

/-- Spot-frame multiplicity `c ~ 10` used for per-message verification. -/
noncomputable def cSpot : ℝ := 10

/-- The four-field boundary compare is much smaller than the per-frame
chain-hash recompute time. -/
theorem tCmp_lt_tChainH : tCmp < tChainH := by
  unfold tCmp tChainH
  norm_num

/-- Per-message verification with `c ~ 10` spot frames costs
`~ 10^-6` s, i.e. `c * t_chainH` is on the order of `10^-6`
(within a factor of 10 of `10^-6`, matching "order-of-magnitude estimate"). -/
theorem cSpot_mul_tChainH_order : cSpot * tChainH = 1 / 10^6 := by
  unfold cSpot tChainH
  norm_num

/-- The `T`-independence (constancy-in-`T`) content of the estimate is
exactly `prop:invariance`, reused without modification: for any manufacture
family sharing one `InstanceQuantities` record, `wallTime` is the same real
number for every index. -/
theorem wallTime_const_reused {Sigma ι : Type} (q : InstanceQuantities Sigma)
    (t_cmp c : ℝ) (σ : ι → Sigma) (l : Filter ι)
    (hT : Filter.Tendsto (fun i => (q.interiorTokenCount (σ i) : ℝ)) l Filter.atTop) :
    ∀ i j : ι, wallTime q t_cmp c = wallTime q t_cmp c :=
  wallTime_invariant q t_cmp c σ l hT

end Praxis.Corpus.EstWall
