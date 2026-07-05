-- thm:magnitude
-- If a system maintains Over = ∅ (the AntibodyInvariant), then every
-- emitted claim c is admissible: c.w ≤ ŵ_c, i.e. some receipt witness
-- attests a magnitude at least as large as the claimed one.
-- No claim can exceed what its mechanism can receipt.

structure Claim where
  phi : Prop
  w   : Nat

structure ReceiptWitness (c : Claim) where
  accepted     : Bool
  accepted_eq  : accepted = true
  attestedMag  : Nat

def Claim.admissible (c : Claim) : Prop :=
  ∃ rw : ReceiptWitness c, c.w ≤ rw.attestedMag

def Over (C : List Claim) (c : Claim) : Prop :=
  c ∈ C ∧ ¬ c.admissible

def AntibodyInvariant (C : List Claim) : Prop :=
  ∀ c, ¬ Over C c

theorem magnitude (C : List Claim) (h : AntibodyInvariant C) :
    ∀ c, c ∈ C → c.admissible := by
  intro c hc
  exact Classical.byContradiction (fun hna => h c ⟨hc, hna⟩)
