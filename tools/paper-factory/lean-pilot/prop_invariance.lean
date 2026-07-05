/-
prop:invariance

For manufactures {σ_i} sharing the frame schema with T(σ_i) → ∞, Ver_∂(σ_i) = Θ(CL)
is constant in i, and the estimated per-message wall-time CL·t_cmp + c·t_chainH does
not depend on T(σ_i).

Formalized content: the wall-time expression, built from the boundary field count
`CL` and the per-frame recomputation time `tChainH` of an `ExecutedManufacture`
(def:instance-q), is invariant across any two manufactures that share those two
design/estimated quantities — in particular it does not depend on the (possibly
divergent) interior token count `T`.
-/

structure ExecutedManufacture where
  T : Nat
  CL : Nat
  tChainH : Nat

def instanceQ (σ : ExecutedManufacture) : Nat × Nat × Nat :=
  (σ.T, σ.CL, σ.tChainH)

/-- Estimated per-message wall-time: CL · t_cmp + c · t_chainH. -/
def wallTime (tCmp c : Nat) (σ : ExecutedManufacture) : Nat :=
  σ.CL * tCmp + c * σ.tChainH

/-- The wall-time estimate depends only on `CL` and `tChainH`, hence is invariant
across manufactures sharing the frame schema regardless of how their interior
token count `T` behaves (e.g. as T(σᵢ) → ∞). -/
theorem invariance (tCmp c : Nat) (σ₁ σ₂ : ExecutedManufacture)
    (hCL : σ₁.CL = σ₂.CL) (hH : σ₁.tChainH = σ₂.tChainH) :
    wallTime tCmp c σ₁ = wallTime tCmp c σ₂ := by
  simp [wallTime, hCL, hH]
