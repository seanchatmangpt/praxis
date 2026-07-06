import Praxis.Corpus.prop_fold

/-!
prop:nodrift, reformalized in the Mathlib lane.

"A receipt emitted by `LawObject::receipt_with_record` and later recomputed by
`ReceiptRecord::recompute_chain_hash` both route through `build_admission_frame` and
`chain_from_frame`; for identical stored fields they compute identical `h_+`, so any
disagreement is attributable to a changed stored field, never divergent code paths."

The Rust-level claim is: two call sites (`receipt_with_record`, `recompute_chain_hash`)
are *not* two independent implementations of "compute h_+" -- they are two callers of
the *same* two-stage pipeline `build_admission_frame` then `chain_from_frame`. Formalized
generically (over abstract types for stored fields, frames, and digests, and abstract
functions for the two named stages, exactly mirroring the two named Rust functions),
the proposition is: since both code paths are literally the same composed function
`chain_from_frame ∘ build_admission_frame` applied to the stored fields, equal stored
fields give a bit-identical `h_+` (by `congrArg`, i.e. Lean functions are deterministic
-- the same fact `prop:fold` already used for `chain`), and contrapositively, any
*disagreement* in the emitted `h_+` can only be witnessed by the stored-fields argument
itself differing, never by the two call sites running different code (there is only
one function here, not two).

No new axiom is introduced: `build_admission_frame` and `chain_from_frame` are kept as
universally-quantified opaque functions (the proposition holds for whatever those two
functions concretely are), and the proof uses only core's `congrArg` / `mt`, the same
toolkit `prop_fold.chain_deterministic` already used. This is deliberately generic
rather than an `axiom` because the claim is a structural fact about "two callers of one
composed function," true for *any* choice of the two stage functions -- axiomatizing it
would hide a theorem that composition-of-shared-functions already proves for free.
-/

variable {StoredFields Frame Digest : Type}

/-- The single two-stage pipeline both Rust call sites route through: stored fields are
first turned into an admission frame (`build_admission_frame`), then the frame is turned
into the chain digest `h_+` (`chain_from_frame`). Both `receipt_with_record` and
`recompute_chain_hash` are callers of exactly this one function, not two separate
implementations. -/
def computeHPlus (buildAdmissionFrame : StoredFields → Frame)
    (chainFromFrame : Frame → Digest) : StoredFields → Digest :=
  chainFromFrame ∘ buildAdmissionFrame

/-- Clause 1 (positive form): for identical stored fields, the shared pipeline computes
an identical `h_+`. This is `congrArg` applied to the single composed function
`computeHPlus` -- Lean functions are deterministic, so equal inputs to the *same*
function give bit-identical outputs, exactly as `prop_fold.chain_deterministic` used for
`chain`. -/
theorem nodrift_same_fields (buildAdmissionFrame : StoredFields → Frame)
    (chainFromFrame : Frame → Digest) {s₁ s₂ : StoredFields} (h : s₁ = s₂) :
    computeHPlus buildAdmissionFrame chainFromFrame s₁
      = computeHPlus buildAdmissionFrame chainFromFrame s₂ :=
  congrArg (computeHPlus buildAdmissionFrame chainFromFrame) h

/-- Clause 2 (the "no drift" contrapositive): if the two computed `h_+` values disagree,
the stored fields must have differed -- the disagreement can never be attributed to the
two call sites running divergent code, because both call sites are the *same* function
`computeHPlus`. -/
theorem nodrift_disagreement_implies_field_change
    (buildAdmissionFrame : StoredFields → Frame) (chainFromFrame : Frame → Digest)
    {s₁ s₂ : StoredFields}
    (hdisagree : computeHPlus buildAdmissionFrame chainFromFrame s₁
      ≠ computeHPlus buildAdmissionFrame chainFromFrame s₂) :
    s₁ ≠ s₂ :=
  mt (nodrift_same_fields buildAdmissionFrame chainFromFrame) hdisagree

/-- Combined statement of `prop:nodrift`: the two named Rust code paths reduce to one
shared pipeline `computeHPlus`, so (1) identical stored fields give an identical `h_+`,
and (2) any disagreement in `h_+` is attributable to a changed stored field, never to
divergent code paths. -/
theorem prop_nodrift (buildAdmissionFrame : StoredFields → Frame)
    (chainFromFrame : Frame → Digest) {s₁ s₂ : StoredFields} :
    (s₁ = s₂ → computeHPlus buildAdmissionFrame chainFromFrame s₁
        = computeHPlus buildAdmissionFrame chainFromFrame s₂)
      ∧ (computeHPlus buildAdmissionFrame chainFromFrame s₁
          ≠ computeHPlus buildAdmissionFrame chainFromFrame s₂ → s₁ ≠ s₂) :=
  ⟨nodrift_same_fields buildAdmissionFrame chainFromFrame,
   nodrift_disagreement_implies_field_change buildAdmissionFrame chainFromFrame⟩
