import Praxis.Corpus.def_instance_q
import Mathlib.Tactic

/-!
# thm:instance-gap — The instance ratio is a computed consequence

For comprehension cost `Comp(σ) = Θ(T(σ))` and boundary-verification cost
`Ver_∂(σ) = Θ(C_L)`, the instance ratio is
`Θ(T(σ)/C_L) = Θ(2.5×10⁶/4) ≈ 6.25×10⁵`; both inputs are measured/design
constants, so the ratio is a *measured consequence*, not an estimate.

We formalize this as: given an `InstanceQuantities` bundle (from
`def:instance-q`) whose measured interior token count at `σ` is exactly
`2500000` and whose design-constant boundary field count is exactly `4`,
the real-valued ratio `T(σ) / C_L` equals `625000` exactly — a computed
number, not a bound or an asymptotic estimate. This captures the
"measured consequence, not an estimate" content of the source statement:
since both `T(σ)` and `C_L` are given as concrete natural numbers (not
left as free/abstract quantities), the ratio is forced to be the single
concrete real number `625000`, proved by direct computation (`norm_num`)
rather than assumed.
-/

namespace Praxis.Corpus.ThmInstanceGap

open Praxis.Corpus.DefInstanceQ

/-- The instance ratio `T(σ)/C_L`, computed as a real number. -/
theorem instance_ratio_eq {Sigma : Type} (q : InstanceQuantities Sigma) (σ : Sigma)
    (hT : q.interiorTokenCount σ = 2500000) (hC : q.boundaryFieldCount = 4) :
    (q.interiorTokenCount σ : ℝ) / (q.boundaryFieldCount : ℝ) = 625000 := by
  rw [hT, hC]
  norm_num

end Praxis.Corpus.ThmInstanceGap
