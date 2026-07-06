import Praxis.Corpus.thm_typestate

/-!
# cor:subcat

The sub-category of admissible morphisms a program may actually perform is precisely `Life`;
illegal-transition-is-a-type-error is a static fact about inhabited hom-sets, not a runtime check.

This is a direct corollary of `Praxis.Corpus.typestate`: for any two stages `X Y : Life`, either
`Y` is reachable from `X` and the hom-set `X ⟶ Y` is (uniquely) inhabited by an actual composite
of the generating arrows `judge / admit / receipt` -- i.e. the admissible morphisms are exactly
the morphisms of `Life` -- or `Y` is unreachable and `X ⟶ Y` is empty, so the "illegal transition"
is not something checked at run time: it is simply a type with no term, decided once and for all
by the shape of the `Life` quiver.
-/

namespace Praxis.Corpus

open CategoryTheory LifeObj

/-- **Illegal transitions are type errors, not runtime checks.** If `Y` is unreachable from `X`
(`¬ rank X ≤ rank Y`), the hom-set `X ⟶ Y` in `Life` is empty: there is no well-typed
stage-changing program realizing that transition, so admissibility is decided statically by
inhabitedness of the hom-set rather than by any check performed while the program runs. -/
theorem illegal_transition_is_type_error (X Y : Life) (h : ¬ rank X ≤ rank Y) :
    IsEmpty (X ⟶ Y) :=
  ((typestate X Y).resolve_left (fun hp => h hp.1)).2

/-- **The admissible sub-category is `Life`.** Every well-typed stage-changing morphism `X ⟶ Y`
is a composite of the generating arrows `judge`, `admit`, `receipt` of `Life` -- i.e. the
morphisms a program may actually perform between stages `X` and reachable `Y` are precisely the
morphisms of `Life`, with no further restriction needed. -/
theorem admissible_is_life (X Y : Life) (h : rank X ≤ rank Y) :
    ∃ f : X ⟶ Y, ∀ g : X ⟶ Y, g = f :=
  ((typestate X Y).resolve_right (fun hq => hq.1 h)).2

end Praxis.Corpus
