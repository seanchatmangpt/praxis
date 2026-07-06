import Praxis.Corpus.def_denialcode

/-!
# def:ob

An obligation is a first-class, hashable value the payload must satisfy before admission;
the `Obligation` enum has exactly three kinds; each obligation `g` induces a lane map
`δ_g : Obs → Deny` returning `Adml` if `g` is satisfied by `o`, else `ℓ(g)`; for a set `G`
the total denial is `d_G(o) = ⋁_{i=1}^{m} δ_{g_i}(o)`.

`Obs`, the payload/observation space, is left abstract here: the thesis does not commit to
a concrete encoding for it (any payload type the caller supplies works), so it is a type
parameter rather than a fixed Mathlib type -- there is no pre-built Mathlib "the" observation
space to reuse. `Deny` is the already-migrated `DenialPolarity` from `def:denialcode`.
-/

/-- The `Obligation` enum has exactly three kinds, matching the thesis's fixed taxonomy. -/
inductive ObligationKind where
  | schema
  | policy
  | temporal
deriving DecidableEq, Repr

/-- An obligation is a first-class, hashable value: a `kind` tag plus a `Nat` identifier
(hashable via `DecidableEq`/`Repr`, standing in for the thesis's opaque hash). It carries
the byte-lane `ℓ(g)` it denies into on failure, and (abstractly, per `Obs`) a satisfaction
predicate the payload must meet before admission. -/
structure Obligation (Obs : Type) where
  kind   : ObligationKind
  id     : Nat
  lane   : DenialPolarity
  satisfies : Obs → Prop
  [dec : DecidablePred satisfies]

namespace Obligation

variable {Obs : Type} (g : Obligation Obs)

/-- The lane map `δ_g : Obs → Deny`: `Adml` if `g` is satisfied by `o`, else `ℓ(g)`. -/
def deltaG [DecidablePred g.satisfies] (o : Obs) : DenialPolarity :=
  if g.satisfies o then DenialPolarity.Adml else g.lane

/-- For a set (here: list) `G` of obligations, the total denial
`d_G(o) = ⋁_{i=1}^{m} δ_{g_i}(o)`, realized as the fold of `compose` (bitwise OR)
over each obligation's lane map, starting from the clean word `Adml`. -/
def totalDenial (G : List (Obligation Obs)) (o : Obs) : DenialPolarity :=
  G.foldr (fun g acc => DenialPolarity.compose (@deltaG Obs g g.dec o) acc) DenialPolarity.Adml

end Obligation
