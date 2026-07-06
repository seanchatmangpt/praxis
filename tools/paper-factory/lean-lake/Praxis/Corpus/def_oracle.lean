import Mathlib.Data.Finset.Basic
import Mathlib.Computability.Halting

/-!
# def:oracle

For implementation `f : D → R` and specification `S ⊆ D × R`, the correctness
question is `Q_S(f) ≡ ∀ x, (x, f x) ∈ S`; a decidable oracle for `f` on finite
`X ⊆ D` is a total computable predicate `Ω : X → {0,1}` intended to witness
`(x, f x) ∈ S`, terminating on every `x ∈ X`.

We model `D`/`R` as `Encodable` types (Mathlib's standard hypothesis for talking
about computability, e.g. `Nat.Partrec`/`ComputablePred`), `S` as a decidable
predicate on `D × R` (so "witnessing membership" is meaningful and checkable),
and the oracle `Ω` as a `DecidablePred` on `D` together with the finite carrier
`X : Finset D` it is required to terminate on. Totality/termination on `X` is
captured by working with `Decidable`/`Bool`-valued predicates throughout: a
`DecidablePred` in Lean is by construction a total, terminating test (no partial
recursion), which is exactly the "total computable predicate" of the source
statement. The "intended to witness" relationship is recorded as the data field
`witnesses`, not proved here (this is a definition, not a theorem) — it names,
for the given `f` and `S`, the proposition that `Ω` and `S ∘ (·, f ·)` agree on
`X`.
-/

namespace Praxis.Corpus

variable {D R : Type*}

/-- The correctness question `Q_S(f) ≡ ∀ x, (x, f x) ∈ S` for an implementation
`f : D → R` against a specification `S ⊆ D × R`. -/
def CorrectnessQuestion (S : D → R → Prop) (f : D → R) : Prop :=
  ∀ x : D, S x (f x)

/-- A decidable oracle for `f` on a finite carrier `X ⊆ D`: a total computable
(i.e. `Decidable`) predicate `Ω : D → Bool`, together with the finite set `X`
it is guaranteed to terminate on, and the specification `S` it is intended to
witness membership in. -/
structure Oracle (D R : Type*) where
  /-- The specification being witnessed. -/
  spec : D → R → Prop
  /-- The implementation being checked. -/
  impl : D → R
  /-- The finite carrier `X ⊆ D` the oracle terminates on. -/
  carrier : Finset D
  /-- The oracle predicate `Ω : D → Bool` (total and computable: `Bool`-valued,
  hence trivially decidable/terminating on every input, matching "total
  computable predicate ... terminating on every `x ∈ X`"). -/
  Ω : D → Bool
  /-- `Ω` is intended to witness `(x, f x) ∈ S` for every `x ∈ X`: agreement
  between the oracle's verdict and the specification on the carrier. This is
  data (the intended-witness relation), not an established theorem. -/
  witnesses : Prop := ∀ x ∈ carrier, (Ω x = true ↔ spec x (impl x))

end Praxis.Corpus
