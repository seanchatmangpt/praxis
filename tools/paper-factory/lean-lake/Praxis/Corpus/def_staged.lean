import Mathlib.Data.Fin.Basic
import Mathlib.Order.SetNotation

/-!
# `def:staged` — Staged validators

A staged validator is a pipeline `V = (V₁, …, V_k)` where each stage `Vᵢ : S → {pass} ∪ {rejectᵢ}`
tests a decidable invariant `Iᵢ ⊆ S`; the pipeline accepts `s` iff `s ∈ ⋂ᵢ Iᵢ`, reporting on
rejection the least `i` with `s ∉ Iᵢ`. Stage `Vᵢ` is sound if `Vᵢ(s) = rejectᵢ ⇒ s ∉ Iᵢ` and
complete if `s ∉ Iᵢ ⇒ Vᵢ(s) = rejectᵢ`.

We model the per-stage outcome type `{pass} ∪ {rejectᵢ}` as `Option Unit` (`none` = pass,
`some ()` = reject at that stage), and represent each stage directly as its decidable invariant
`Iᵢ : S → Prop` together with a `DecidablePred` instance, since the outcome of stage `i` is
determined by whether `s ∈ Iᵢ`. This is a faithful, decidability-preserving encoding, not an
axiomatized abstraction: everything below is built from core `Prop`/`Decidable`/`Fin`/`Option`
machinery already in Mathlib/core, no new axioms are introduced.
-/

namespace Praxis.Corpus.DefStaged

/-- A staged validator pipeline over state space `S` with `k` stages: each stage `i : Fin k`
is given by its decidable invariant `inv i : S → Prop`. -/
structure StagedValidator (S : Type*) (k : ℕ) where
  /-- The invariant tested by stage `i`. -/
  inv : Fin k → S → Prop
  /-- Each stage's invariant is decidable, so the stage can actually be run as a validator. -/
  dec : ∀ i, DecidablePred (inv i)

namespace StagedValidator

variable {S : Type*} {k : ℕ} (V : StagedValidator S k)

/-- The pipeline accepts `s` iff `s` lies in every stage's invariant, i.e. `s ∈ ⋂ᵢ Iᵢ`. -/
def accepts (s : S) : Prop := ∀ i, V.inv i s

/-- The per-stage outcome: `none` for pass, `some ()` for `rejectᵢ`. -/
noncomputable def stageOutcome (i : Fin k) (s : S) : Option Unit :=
  @ite _ (V.inv i s) (V.dec i s) none (some ())

/-- On rejection the pipeline reports the least stage index `i` with `s ∉ Iᵢ`, when one exists:
we scan the stages `0, 1, …, k-1` in order (via `List.finRange k`) and return the first one whose
outcome is a rejection. -/
def firstRejection (s : S) : Option (Fin k) :=
  (List.finRange k).find? (fun i => ! @decide (V.inv i s) (V.dec i s))

/-- Stage `i` is *sound*: reporting `rejectᵢ` implies `s` genuinely fails invariant `Iᵢ`. -/
def Sound (i : Fin k) : Prop :=
  ∀ s, V.stageOutcome i s = some () → ¬ V.inv i s

/-- Stage `i` is *complete*: genuinely failing invariant `Iᵢ` implies stage `i` reports `rejectᵢ`. -/
def Complete (i : Fin k) : Prop :=
  ∀ s, ¬ V.inv i s → V.stageOutcome i s = some ()

end StagedValidator

end Praxis.Corpus.DefStaged
