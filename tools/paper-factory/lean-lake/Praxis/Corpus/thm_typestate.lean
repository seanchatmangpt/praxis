import Praxis.Corpus.def_lifecat
import Praxis.Corpus.prop_hom

/-!
# thm:typestate

Interpreting each lifecycle stage as a `LawObject` type family, the exposed stage-changing
operations denote exactly the generating arrows of `Life`; every well-typed stage-changing
program is a composite of `judge`, `admit`, `receipt`; illegal transitions are uninhabited
hom-sets; and no law object can be receipted twice.

This is stated and proved as a corollary of `Praxis.Corpus.hom_card` and `rank_hom_succ`
(from `prop:hom`):

* every hom-set `X ⟶ Y` in `Life` is a `Quiver.Path` built out of the generating arrows
  `LifeHom.judge / .admit / .receipt` -- this holds definitionally, since `Life` is Mathlib's
  free path category `CategoryTheory.Paths` on the `LifeHom` quiver, so every well-typed
  stage-changing program literally *is* such a composite;
* whenever `Y` is unreachable from `X` (`¬ rank X ≤ rank Y`) the hom-set `X ⟶ Y` is empty,
  i.e. illegal transitions are uninhabited;
* the terminal stage `rcpt` has no outgoing generating arrow, so once a law object reaches
  `rcpt` it cannot be advanced (in particular cannot be receipted again): the hom-set
  `rcpt ⟶ Y` is empty for every `Y ≠ rcpt`.
-/

namespace Praxis.Corpus

open CategoryTheory LifeObj

/-- **Typestate theorem.** For every pair of stages `X Y : Life`:
either `Y` is reachable from `X` (`rank X ≤ rank Y`) and the hom-set `Life(X, Y)` has exactly
one element, itself a composite of the generating arrows `judge`, `admit`, `receipt`
(a `Quiver.Path` in the underlying quiver); or `Y` is unreachable and the hom-set is empty,
i.e. the illegal transition is uninhabited. -/
theorem typestate (X Y : Life) :
    (rank X ≤ rank Y ∧ ∃ f : X ⟶ Y, ∀ g : X ⟶ Y, g = f) ∨
      (¬ rank X ≤ rank Y ∧ IsEmpty (X ⟶ Y)) := by
  by_cases h : rank X ≤ rank Y
  · exact Or.inl ⟨h, (hom_card X Y).1 h⟩
  · exact Or.inr ⟨h, (hom_card X Y).2 h⟩

/-- **No re-receipting.** Once a law object has reached the terminal stage `rcpt`, there is no
well-typed stage-changing program taking it anywhere else (in particular, no further `receipt`
transition): every hom-set `rcpt ⟶ Y` with `Y ≠ rcpt` is empty. -/
theorem no_receipt_twice (X Y : Life) (hX : X = rcpt) (hY : Y ≠ rcpt) :
    IsEmpty (X ⟶ Y) := by
  have hrank : ¬ rank X ≤ rank Y := by
    subst hX
    cases Y <;> simp_all [rank]
  exact (hom_card X Y).2 hrank

end Praxis.Corpus
