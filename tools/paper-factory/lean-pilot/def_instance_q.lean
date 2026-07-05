/-
def:instance-q

For an executed manufacture σ:
  T(σ)       — the interior token count (measured, a count of logged records);
  CL         — the boundary field count of the receipt projection Proj(σ) (a design constant);
  t_chainH   — the per-frame recomputation wall-time (estimated from published hash
               throughput, not measured here).

This is packaged as a structure of instance quantities associated to an executed
manufacture, with the three named projections `T`, `CL`, and `tChainH`.
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
