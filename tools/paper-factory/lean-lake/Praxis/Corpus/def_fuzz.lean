import Praxis.Corpus.def_oracle

/-!
# def:fuzz

For the total admission retraction `adm : Obs ⇀ Adm ∪ {Rfsl}`, the fuzzing
oracle `Ω_∂` maps generated `o ∈ Obs` to `1` iff `adm(o)` terminates and
yields either an admitted `x ∈ Adm` with well-formed artifact `μ(x)`, or a
refusal `Rfsl` carrying a category in the eight-bucket taxonomy; `Ω_∂(o) = 0`
iff `adm` crashes, diverges, or returns an unlabelled outcome.

We model the partial function `adm : Obs ⇀ Adm ∪ {Rfsl}` as a total function
`Obs → Option (Adm ⊕ Rfsl)`: `none` is the case where `adm` crashes, diverges,
or is otherwise unlabelled (mirroring how partiality is standardly encoded in
Lean/Mathlib, e.g. `Part`/`Option`-valued semantics for partial recursive
functions, consistent with `def:oracle`'s `Encodable`/`DecidablePred` style).
"Well-formed artifact `μ(x)`" is a `DecidablePred` on `Adm` (so the oracle
verdict is computable, matching `def:oracle`'s requirement that `Ω` be a
total computable/`Bool`-valued predicate). The "eight-bucket taxonomy" for
refusal categories is modelled as `Fin 8`, via a total `category` map on
`Rfsl` — every refusal is by construction labelled with one of the eight
buckets, so "carrying a category in the eight-bucket taxonomy" is simply
"having a value of type `Fin 8`", requiring no extra hypothesis. `Ω_∂` is
then the evident `Bool`-valued case split on `adm o`: `some (inl x)` scores
by well-formedness of `x`, `some (inr _)` always scores `true` (a refusal is
always validly categorised, by the totality of `category`), and `none`
scores `false`.
-/

namespace Praxis.Corpus

variable {Obs Adm Rfsl : Type*}

/-- The fuzzing oracle `Ω_∂` for a total admission retraction
`adm : Obs → Option (Adm ⊕ Rfsl)` (the `Option`-encoding of the partial
function `adm : Obs ⇀ Adm ∪ {Rfsl}`, with `none` standing for "crashes,
diverges, or returns an unlabelled outcome"). -/
structure FuzzOracle (Obs Adm Rfsl : Type*) where
  /-- The total admission retraction `adm : Obs ⇀ Adm ∪ {Rfsl}`, modelled as
  `Obs → Option (Adm ⊕ Rfsl)` with `none` for the crash/diverge/unlabelled
  case. -/
  adm : Obs → Option (Adm ⊕ Rfsl)
  /-- Well-formedness of the artifact `μ(x)` for an admitted `x : Adm`,
  as a decidable (total computable) predicate. -/
  wellFormed : Adm → Prop
  [decWellFormed : DecidablePred wellFormed]
  /-- The eight-bucket taxonomy: every refusal carries a category in `Fin 8`. -/
  category : Rfsl → Fin 8

attribute [instance] FuzzOracle.decWellFormed

/-- The oracle verdict `Ω_∂ : Obs → Bool`: `true` iff `adm o` terminates and
yields either a well-formed admitted artifact, or a (necessarily categorised)
refusal; `false` iff `adm o = none`, i.e. `adm` crashes, diverges, or returns
an unlabelled outcome. -/
def FuzzOracle.Ω (F : FuzzOracle Obs Adm Rfsl) (o : Obs) : Bool :=
  match F.adm o with
  | none => false
  | some (Sum.inl x) => decide (F.wellFormed x)
  | some (Sum.inr _) => true

end Praxis.Corpus
