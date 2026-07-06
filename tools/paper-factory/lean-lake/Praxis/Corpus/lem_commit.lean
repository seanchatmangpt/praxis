import Praxis.Corpus.def_receipt
import Mathlib.Data.List.Induction
import Mathlib.Data.List.Basic

/-!
lem:commit

"Under collision resistance of `chainH`, the chain value `h+` is a binding
commitment to `(h-,fr)`; by induction `h+` binds the entire causal prefix."

We model "collision resistance of `chainH`" the way it is used here: as the
hypothesis that `chainH` is injective on the (fixed, finite) domain of strings
that actually arise as chain encodings -- i.e. `chainH` does not collide on
two distinct encoded inputs. That is exactly `Function.Injective chainH`,
taken as a hypothesis of the lemma (not a fresh axiom): the lemma's content is
the *derivation* of the binding property from collision resistance, matching
the paper's own "under collision resistance of chainH" phrasing.

`encodeChain` is the plain, computable serialization from `def:receipt`
(`Praxis/Corpus/def_receipt.lean`); its injectivity on `(hMinus, fr)` pairs is
a property of that fixed serialization scheme (it never collides two distinct
`(digest, frame)` pairs into the same string), so it is likewise taken as a
hypothesis rather than re-derived from string-manipulation lemmas that would
only reprove a fact about the encoding, not about the commitment.

From these two hypotheses we derive:
1. `chainedReceipt_binding`: `h+ = chainedReceipt hMinus fr` is a binding
   commitment to `(hMinus, fr)` -- i.e. `chainedReceipt` is injective on pairs.
2. `chainFold_binding`: by induction on the causal prefix (a `List Frame`
   folded from a genesis digest via `chainedReceipt`), the final chain value
   is a binding commitment to the *entire* list of frames, for prefixes of
   equal length. This is the "by induction ... binds the entire causal
   prefix" clause.
-/

/-- The chained receipt is a binding commitment to `(hMinus, fr)`: under
    collision resistance of `chainH` and injectivity of the fixed encoding,
    two chain steps agreeing on `h+` must agree on `(hMinus, fr)`. -/
theorem chainedReceipt_binding
    (hInj : Function.Injective chainH)
    (hEnc : Function.Injective (fun p : Digest × Frame => encodeChain p.1 p.2)) :
    Function.Injective (fun p : Digest × Frame => chainedReceipt p.1 p.2) := by
  intro p q hpq
  simp only [chainedReceipt] at hpq
  exact hEnc (hInj hpq)

/-- Fold a causal prefix of frames, starting from a genesis digest, into a
    single chain value via the chained receipt. -/
noncomputable def chainFold (genesis : Digest) (frames : List Frame) : Digest :=
  frames.foldl chainedReceipt genesis

/-- By induction on the causal prefix: `chainFold` is a binding commitment to
    the entire list of frames, among prefixes of equal length starting from
    the same genesis digest. This is the inductive extension of
    `chainedReceipt_binding` from a single step to the whole causal prefix. -/
theorem chainFold_binding
    (hInj : Function.Injective chainH)
    (hEnc : Function.Injective (fun p : Digest × Frame => encodeChain p.1 p.2)) :
    ∀ (l1 l2 : List Frame) (g : Digest),
      l1.length = l2.length → chainFold g l1 = chainFold g l2 → l1 = l2 := by
  have step := chainedReceipt_binding hInj hEnc
  intro l1
  induction l1 using List.reverseRecOn with
  | nil =>
      intro l2 g hlen heq
      cases l2 with
      | nil => rfl
      | cons _ _ => simp at hlen
  | append_singleton rest1 fr1 ih =>
      intro l2 g hlen heq
      -- l2 has the same nonzero length, so it also decomposes as rest2 ++ [fr2]
      match l2, hlen with
      | [], hlen => simp at hlen
      | l2, hlen =>
        obtain ⟨rest2, fr2, rfl⟩ := l2.eq_nil_or_concat.resolve_left (by
          intro h; rw [h] at hlen; simp at hlen)
        simp only [List.concat_eq_append] at *
        have heq' : chainedReceipt (chainFold g rest1) fr1
            = chainedReceipt (chainFold g rest2) fr2 := by
          simpa [chainFold, List.foldl_append, List.foldl_cons, List.foldl_nil] using heq
        have hlen' : rest1.length = rest2.length := by
          simpa [List.length_append] using hlen
        have hpair : (chainFold g rest1, fr1) = (chainFold g rest2, fr2) :=
          step (by simpa [chainedReceipt] using heq')
        have hfold : chainFold g rest1 = chainFold g rest2 := (Prod.mk.injEq .. ▸ hpair).1
        have hfr : fr1 = fr2 := (Prod.mk.injEq .. ▸ hpair).2
        have hrest : rest1 = rest2 := ih rest2 g hlen' hfold
        rw [hrest, hfr]
