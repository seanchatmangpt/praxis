-- def:claim
-- A claim is a pair c = (phi_c, w_c) where phi_c is a proposition and
-- w_c ∈ ℝ_{≥0} its asserted magnitude. A receipt witness for c is a receipt
-- r(c) whose boundary projection is accepted (V(Proj(r(c))) = 1) and whose
-- committed fields entail an attested magnitude ŵ_c. A claim is admissible
-- iff it has a receipt witness with w_c ≤ ŵ_c.
--
-- Bare Lean 4 core, no mathlib: nonnegative magnitudes are modeled as `Nat`
-- (nonnegativity is definitional rather than an explicit hypothesis, since
-- there is no `ℝ_{≥0}` type available without mathlib).

structure Claim where
  phi : Prop
  w   : Nat

structure ReceiptWitness (c : Claim) where
  -- boundary projection acceptance: V(Proj(r(c))) = 1, encoded as a boolean flag
  accepted     : Bool
  accepted_eq  : accepted = true
  -- attested magnitude ŵ_c entailed by the receipt's committed fields
  attestedMag  : Nat

def Claim.admissible (c : Claim) : Prop :=
  ∃ rw : ReceiptWitness c, c.w ≤ rw.attestedMag
