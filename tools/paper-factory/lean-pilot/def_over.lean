-- def:over
-- The overclaim set of a system emitting claims C is
-- Over = { c ∈ C : ¬∃ r(c) with V(Proj(r(c))) = 1 and w_c ≤ ŵ_c }
-- i.e. the claims in C that are not admissible (cf. def:claim).
-- The antibody invariant is Over = ∅, the docs-exceed-mechanism defect
-- class enforced as a gate.
--
-- Bare Lean 4 core, no mathlib: a system's emitted claim set C is modeled
-- as `List Claim`; the overclaim set is the sub-predicate on membership in
-- C conjoined with non-admissibility.

structure Claim where
  phi : Prop
  w   : Nat

structure ReceiptWitness (c : Claim) where
  accepted     : Bool
  accepted_eq  : accepted = true
  attestedMag  : Nat

def Claim.admissible (c : Claim) : Prop :=
  ∃ rw : ReceiptWitness c, c.w ≤ rw.attestedMag

-- Overclaim set, relative to the emitted claim set `C`.
def Over (C : List Claim) (c : Claim) : Prop :=
  c ∈ C ∧ ¬ c.admissible

-- The antibody invariant: no claim in C is an overclaim.
def AntibodyInvariant (C : List Claim) : Prop :=
  ∀ c, ¬ Over C c
