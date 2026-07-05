/-
def:regimes (00_foundations / projection_thesis) — formalized as an inductive
type of audit regimes together with their asymptotic cost, as a function of
`n`, the number of committed frames in a receipt chain (see def:receipt in
def_receipt.lean for the notion of a committed `Frame`/`Receipt`).

Original (LaTeX):
  For a trace of n committed frames: boundary projection inspection (any
  verifier, per message) costs O(CL); single frame recomputation (spot check)
  costs O(1) per frame; checkpointed/indexed replay (sampled audit) costs
  O(log n) or bounded path; full chain replay (full audit) costs O(n).

This is a DEFINITION, not a theorem — there is no proof obligation, only the
requirement that it type-checks as a well-formed Lean structure/inductive
type. Asymptotic cost classes (O(CL), O(1), O(log n), O(n)) are not modeled
as real complexity-theoretic bounds here (that would require analysis
machinery not available in bare Lean core without mathlib); instead each
regime is tagged with an abstract `Cost` shape parametrized by `n`, mirroring
the LaTeX's four cases exactly. `CL` (the per-message boundary-projection
cost, presumably "check length" or similar from the surrounding thesis
material) is modeled abstractly, matching how `Digest`, `Fitness`, etc. were
modeled abstractly in def:receipt.
-/

/-- The four audit regimes named in the LaTeX, indexed by the trace length
    `n` (number of committed frames). -/
inductive Regime (n : Nat) where
  | boundaryProjection   -- any verifier, per message
  | spotCheck            -- single frame recomputation
  | sampledAudit         -- checkpointed/indexed replay
  | fullAudit            -- full chain replay
  deriving Repr, DecidableEq

/-- Abstract per-message boundary-projection cost, `CL` in the LaTeX. -/
axiom CL : Nat

/-- The asymptotic cost associated to a regime, as a natural-number bound
    standing in for the LaTeX's O(-) classes:
    boundaryProjection ↦ CL, spotCheck ↦ 1 (per frame), sampledAudit ↦
    log2 n (or a bounded path, modeled the same way), fullAudit ↦ n. -/
noncomputable def Regime.cost {n : Nat} : Regime n → Nat
  | .boundaryProjection => CL
  | .spotCheck => 1
  | .sampledAudit => Nat.log2 n
  | .fullAudit => n

/-- Sanity check: full audit cost is exactly the trace length. -/
example (n : Nat) : (Regime.fullAudit (n := n)).cost = n := rfl

/-- Sanity check: spot check cost is constant, independent of `n`. -/
example (n : Nat) : (Regime.spotCheck (n := n)).cost = 1 := rfl
