import Praxis.Corpus.def_oracle

/-!
# def:diff

For `f, g : D → R` targeting the same specification `S`, the differential
oracle is `Ω_{f,g}(x) = [f(x) = g(x)]`; a test passes at `x` iff
`Ω_{f,g}(x) = 1`.

We reuse `Praxis.Corpus.Oracle` from `def:oracle`: the differential oracle is
just the boolean equality test `f x = g x` (via `DecidableEq R`, Mathlib's
standard hypothesis for a decidable/computable equality predicate), packaged
with the two implementations being compared and their shared carrier `X`.
-/

namespace Praxis.Corpus

variable {D R : Type*} [DecidableEq R]

/-- The differential oracle `Ω_{f,g}(x) = [f(x) = g(x)]` comparing two
implementations `f g : D → R` against the same specification `S`, on a finite
carrier `X ⊆ D`. Built from `Oracle` by setting the oracle predicate to the
decidable equality test between `f` and `g`. -/
def diffOracle (spec : D → R → Prop) (f g : D → R) (carrier : Finset D) :
    Oracle D R where
  spec := spec
  impl := f
  carrier := carrier
  Ω := fun x => decide (f x = g x)
  witnesses := ∀ x ∈ carrier, (decide (f x = g x) = true ↔ spec x (g x))

end Praxis.Corpus
