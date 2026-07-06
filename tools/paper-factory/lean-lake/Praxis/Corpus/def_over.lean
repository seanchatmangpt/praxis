import Praxis.Corpus.def_claim

/-!
def:over

"The overclaim set of a system emitting claims `𝒞` is
`Over = { c ∈ 𝒞 : ¬∃ r(c) with V(Proj(r(c))) = 1 and w_c ≤ ŵ_c }`;
the antibody invariant is `Over = ∅`, the `docs-exceed-mechanism` defect
class enforced as a gate."

Composition over fresh axioms:

* `𝒞`, the system's set of emitted claims, is exactly Mathlib/core's
  `Set Claim` -- `Claim` itself already migrated in `def:claim`
  (`Praxis/Corpus/def_claim.lean`). No new "claim collection" type is
  needed beyond `Set`.
* The condition "no receipt witness `r(c)` with `V(Proj(r(c))) = 1` and
  `w_c ≤ ŵ_c`" is precisely the negation of `Claim.Admissible`, already
  defined in `def:claim` in terms of `HasReceiptWitnessAttesting`. So
  `Over` is just `Set.sep` (equivalently `{c ∈ 𝒞 | ¬c.Admissible}`) over
  the existing `Admissible` predicate -- no new machinery, just
  set-builder notation composed with the prior definition.
* The "antibody invariant" `Over = ∅` is not itself part of this
  definition's type -- it is a downstream property (a gate condition) to
  be *stated* about a given `𝒞`, not baked into the definition of `Over`.
  We give it below as `Over.AntibodyInvariant`, a `Prop`-valued predicate
  on `𝒞`, so future migrations can invoke it as a hypothesis or goal
  without re-deriving `Over`.

What is *not* captured concretely here: as in `def:claim`, the receipt
witness apparatus (`r(c)`, `Proj`, `V`) remains the abstract
`HasReceiptWitnessAttesting` predicate from `def:claim`; this definition
composes on top of it rather than re-abstracting it.
-/

/-- The overclaim set: claims in `𝒞` that are *not* admissible, i.e. for
which no receipt witness attests a sufficient magnitude. -/
def Over (𝒞 : Set Claim) : Set Claim :=
  {c ∈ 𝒞 | ¬ c.Admissible}

/-- The antibody invariant: the overclaim set of `𝒞` is empty (the
`docs-exceed-mechanism` defect class, enforced as a gate). -/
def Over.AntibodyInvariant (𝒞 : Set Claim) : Prop :=
  Over 𝒞 = ∅
