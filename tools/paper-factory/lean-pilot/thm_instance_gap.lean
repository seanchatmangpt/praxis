/-
thm:instance-gap

For comprehension cost Comp(σ) = Θ(T(σ)) and boundary-verification cost
Ver_∂(σ) = Θ(CL), the instance ratio is Θ(T(σ)/CL) = Θ(2.5×10^6 / 4) ≈ 6.25×10^5;
both inputs are measured/design constants, so the ratio is a measured consequence,
not an estimate.

This is formalized as a concrete arithmetic fact about the instance quantities
of `def:instance-q`: for the executed manufacture with T = 2500000 and CL = 4,
the ratio T/CL computed via `instanceQ` is exactly 625000.
-/

structure ExecutedManufacture where
  /-- interior token count: a count of logged records, measured on σ -/
  T : Nat
  /-- boundary field count of the receipt projection Proj(σ): a design constant -/
  CL : Nat
  /-- per-frame recomputation wall-time, estimated from published hash throughput -/
  tChainH : Nat

def instanceQ (σ : ExecutedManufacture) : Nat × Nat × Nat :=
  (σ.T, σ.CL, σ.tChainH)

/-- The witness instance: T(σ) = 2.5×10^6 (measured), CL = 4 (design constant),
    t_chainH left as 0 (not needed for this ratio). -/
def witnessManufacture : ExecutedManufacture :=
  { T := 2500000, CL := 4, tChainH := 0 }

theorem instance_gap :
    (instanceQ witnessManufacture).1 / (instanceQ witnessManufacture).2.1 = 625000 := by
  decide
