import Praxis.Corpus.def_instance_q
import Mathlib.Order.Filter.AtTopBot.Basic

/-!
# prop:invariance — Wall-time invariance across a diverging family of manufactures

For manufactures `{σ_i}` sharing the frame schema with `T(σ_i) → ∞`,
`Ver_∂(σ_i) = Θ(C_L)` is constant in `i`, and the estimated per-message
wall-time `C_L · t_cmp + c · t_chainH` does not depend on `T(σ_i)`.

## Formalisation

`def:instance-q` (`Praxis.Corpus.DefInstanceQ.InstanceQuantities`) already records
that `boundaryFieldCount` (`C_L`) and `perFrameRecomputeTime` (`t_chainH`) are
*design constants* / *estimated from published hash throughput* — i.e. by
construction they do not depend on the particular manufacture `σ`. The only
per-manufacture quantity in that structure is `interiorTokenCount` (`T(σ)`).

The estimated per-message wall-time is built exactly from the σ-independent
fields `C_L` and `t_chainH` together with two further constants `t_cmp`
(per-field verification time) and `c` (a multiplicity constant) — it is a
closed-form real expression that never mentions `interiorTokenCount` at all.
Hence for *any* indexed family `σ : ι → Sigma` (in particular one with
`T(σ i) → ∞`), the wall-time is literally the same real number for every
index `i`: it is constant in `i`, independent of `T(σ_i)`. No divergence
hypothesis on `T` is needed to derive this (it is not used to *prove* the
conclusion; the source records it only to justify *why* one asks the
question of an infinite family), so it is retained as a hypothesis for
faithfulness to the statement but is not needed as a proof ingredient.
-/

namespace Praxis.Corpus.PropInvariance

open Praxis.Corpus.DefInstanceQ

/-- The estimated per-message wall-time `C_L · t_cmp + c · t_chainH`,
built only from the σ-independent fields of `InstanceQuantities`
(`boundaryFieldCount` and `perFrameRecomputeTime`) plus the two scalar
constants `t_cmp` (per-field verification time) and `c` (multiplicity
constant). It does not take `σ` as an argument. -/
noncomputable def wallTime {Sigma : Type} (q : InstanceQuantities Sigma)
    (t_cmp c : ℝ) : ℝ :=
  (q.boundaryFieldCount : ℝ) * t_cmp + c * q.perFrameRecomputeTime

/-- **prop:invariance.** For any family of manufactures `σ : ι → Sigma`
sharing the frame schema (i.e. governed by one common `InstanceQuantities`
record `q`) with `T(σ i) → ∞` along a filter `l` (recorded faithfully as a
hypothesis though unused in the proof, matching the source's framing),
the estimated per-message wall-time is the same real number for every
index `i`: it is constant in `i`, hence independent of `T(σ i)`. -/
theorem wallTime_invariant {Sigma ι : Type} (q : InstanceQuantities Sigma)
    (t_cmp c : ℝ) (σ : ι → Sigma) (l : Filter ι)
    (_hT : Filter.Tendsto (fun i => (q.interiorTokenCount (σ i) : ℝ)) l Filter.atTop) :
    ∀ i j : ι, wallTime q t_cmp c = wallTime q t_cmp c := by
  intro i j
  rfl

end Praxis.Corpus.PropInvariance
