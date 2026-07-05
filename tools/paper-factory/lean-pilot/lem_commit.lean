/-
lem:commit (00_foundations / projection_thesis)

Original (LaTeX):
  Under collision resistance of $\chainH$, the chain value $h_+$ is a binding
  commitment to $(h_-,\mathsf{fr})$; by induction $h_+$ binds the entire
  causal prefix.

Formalization plan: reuse def:receipt's `Digest`, `Frame`, `chainStep`
verbatim (copied in below so this file type-checks standalone, exactly as
in def_receipt.lean). Collision resistance of the chain step is modeled as
the hypothesis that `chainStep` is injective as a function of its pair of
arguments (hMinus, frame): two distinct (prior, frame) pairs never produce
the same advanced digest. The lemma has two parts, matching the two
sentences of the LaTeX:

  1. `chainStep_binds` — one step of chaining is a binding commitment: the
     resulting digest determines the prior digest and frame that produced
     it.
  2. `chain_binds_prefix` — by induction over a causal chain (a list of
     frames folded via `chainStep`, of the same length on both sides), the
     final digest determines the entire causal prefix: the initial digest
     and the full list of frames.

`foldChain` is defined so that the *first* frame in the list is the
outermost (most recently applied) `chainStep`, which is what lets the
inductive step reduce directly to `chainStep_binds` on the head frame and
the fold of the tail.
-/

axiom Bits256 : Type
abbrev Digest := Bits256

axiom DenialWord : Type
axiom TransitionId : Type
axiom Fitness : Type
axiom RefusalReason : Type
axiom Version : Type

structure Frame where
  dgX : Digest
  dgG : Digest
  denial : DenialWord
  transition : TransitionId
  dgA : Digest
  fitness : Fitness
  reason : RefusalReason
  version : Version

axiom chainStep : Digest → Frame → Digest

/-- Collision resistance of the chain step, as a hypothesis a theorem may
    assume: no two distinct (prior digest, frame) pairs collide to the same
    advanced digest. -/
def CollisionResistant : Prop :=
  ∀ h1 f1 h2 f2, chainStep h1 f1 = chainStep h2 f2 → h1 = h2 ∧ f1 = f2

/-- Part 1: under collision resistance, one chain step is a binding
    commitment to `(h-, fr)` — the advanced digest determines the prior
    digest and frame. -/
theorem chainStep_binds (hcr : CollisionResistant)
    (h1 f1 h2 f2 : _) (heq : chainStep h1 f1 = chainStep h2 f2) :
    h1 = h2 ∧ f1 = f2 :=
  hcr h1 f1 h2 f2 heq

/-- Fold a list of frames into a running digest starting from `h0`; the
    head of the list is the outermost (last-applied) `chainStep`. -/
noncomputable def foldChain (h0 : Digest) : List Frame → Digest
  | [] => h0
  | fr :: frs => chainStep (foldChain h0 frs) fr

/-- Part 2: by induction, under collision resistance the final digest of a
    causal chain binds the entire prefix — for chains of matching length,
    the initial digest and the full list of frames are uniquely
    determined by the final digest. -/
theorem chain_binds_prefix (hcr : CollisionResistant) :
    ∀ (frs1 frs2 : List Frame) (h1 h2 : Digest),
      frs1.length = frs2.length →
      foldChain h1 frs1 = foldChain h2 frs2 →
      h1 = h2 ∧ frs1 = frs2 := by
  intro frs1
  induction frs1 with
  | nil =>
      intro frs2 h1 h2 hlen heq
      cases frs2 with
      | nil => exact ⟨heq, rfl⟩
      | cons fr2 frs2' => simp at hlen
  | cons fr1 frs1' ih =>
      intro frs2 h1 h2 hlen heq
      cases frs2 with
      | nil => simp at hlen
      | cons fr2 frs2' =>
          simp only [foldChain] at heq
          have hstep := chainStep_binds hcr (foldChain h1 frs1') fr1
            (foldChain h2 frs2') fr2 heq
          obtain ⟨hfold, hfr⟩ := hstep
          have hlen' : frs1'.length = frs2'.length := by
            simpa using hlen
          obtain ⟨hh, hfrs⟩ := ih frs2' h1 h2 hlen' hfold
          exact ⟨hh, by rw [hfr, hfrs]⟩
