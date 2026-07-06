import Mathlib.Data.Real.Basic

/-!
# def:instance-q — Instance quantities for an executed manufacture

For an executed manufacture `σ`, the source names three quantities:

* `T σ` — the interior token count: a *measured* count of logged records
  for the run, hence a natural number;
* `C_L` — the boundary field count of the receipt projection `Proj(σ)`: a
  *design constant* fixed by the receipt schema, hence also a natural
  number, independent of any particular `σ`;
* `t_chainH` — the per-frame recomputation wall-time: *estimated* from
  published hash throughput rather than measured, hence modelled as a
  nonnegative real (a duration), independent of any particular `σ`.

No bespoke numeric type is introduced: measured/designed counts are `ℕ`
and the estimated wall-time is `ℝ`, both supplied by Mathlib. `σ` itself
is left abstract (an arbitrary type of "executed manufactures"), since
this statement only names quantities attached to it, it does not specify
what a manufacture concretely is.
-/

namespace Praxis.Corpus.DefInstanceQ

/-- The three named instance quantities, bundled together, parametrised
by `Sigma`, the (left abstract) type of executed manufactures:
* `interiorTokenCount σ = T(σ)`, measured;
* `boundaryFieldCount = C_L`, a design constant (independent of `σ`);
* `perFrameRecomputeTime = t_chainH`, estimated from published hash
  throughput (independent of `σ`), hence a nonnegative real duration. -/
structure InstanceQuantities (Sigma : Type) where
  /-- `T(σ)`, the interior token count: measured, a count of logged
  records. -/
  interiorTokenCount : Sigma → ℕ
  /-- `C_L`, the boundary field count of the receipt projection
  `Proj(σ)`: a design constant. -/
  boundaryFieldCount : ℕ
  /-- `t_chainH`, the per-frame recomputation wall-time: estimated from
  published hash throughput, not measured. -/
  perFrameRecomputeTime : ℝ
  perFrameRecomputeTime_nonneg : 0 ≤ perFrameRecomputeTime

end Praxis.Corpus.DefInstanceQ
