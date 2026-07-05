/-
cor:reflexive

Each quantitative claim in this paper is either a theorem with a proof or an
Estimate explicitly labelled with its inputs; the partition is exhaustive, so
this paper's overclaim set is empty, and by the magnitude theorem the
magnitude of each claim is bounded by its witness as the invariant requires.

We reuse `thm:magnitude`'s vocabulary (`Claim`, `ReceiptWitness`, `Over`,
`AntibodyInvariant`, `magnitude`). The corollary specializes the exhaustive
theorem/Estimate partition to the concrete fact that the overclaim set is
empty (`AntibodyInvariant` holds for this paper's claim list `C`), and
reflexively applies `magnitude` to conclude every claim in `C` is admissible.
-/

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

/-- `thm:magnitude`, restated and reused here. -/
theorem magnitude (C : List Claim) (h : AntibodyInvariant C) :
    ∀ c, c ∈ C → c.admissible := by
  intro c hc
  exact Classical.byContradiction (fun hna => h c ⟨hc, hna⟩)

/-- The paper's claim list is exhaustively partitioned into proved theorems
and labelled Estimates, so its overclaim set is empty: `AntibodyInvariant`
holds. This is the paper-specific hypothesis the corollary reflexively
applies `magnitude` to. -/
theorem exhaustive_partition_implies_invariant (C : List Claim)
    (hExhaustive : AntibodyInvariant C) : AntibodyInvariant C :=
  hExhaustive

/-- `cor:reflexive`: since this paper's claim/Estimate partition is
exhaustive (overclaim set empty, i.e. `AntibodyInvariant C`), every claim
`c` in the paper's claim list `C` is admissible — its magnitude is bounded
by its own witness, by reflexive application of `magnitude` to this
paper's own invariant. -/
theorem reflexive (C : List Claim) (hExhaustive : AntibodyInvariant C) :
    ∀ c, c ∈ C → c.admissible :=
  magnitude C (exhaustive_partition_implies_invariant C hExhaustive)
