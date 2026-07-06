import Mathlib.Data.Fintype.Basic
import Mathlib.Data.Finset.Basic
import Mathlib.Algebra.BigOperators.Group.Finset.Basic
import Mathlib.Data.Real.Basic
import Praxis.Corpus.def_balance

/-!
# prop:conserve — Conservation of a fluent under fully-completed durative draws

If every durative action drawing on fluent `φ` both decrements it by `r_j` at its
start and increments it by `r_j` back at its end, then for a finite family of
*completed* actions (each contributing both its start-decrement and its
end-increment), the net effect on `φ` is zero: the terminal valuation equals the
initial valuation, `ν_end(φ) = ν₀(φ)`. Scheduling only redistributes *when* `φ` is
held (captured by `Praxis.Corpus.DefBalance.freeLevel`), never *how much* exists in
total once all draws have completed.

We state this as plain arithmetic over Mathlib's `Finset.sum`: applying the
start-decrement `∑ r j` and then the matching end-increment `∑ r j` to `ν₀` returns
`ν₀`. This is exactly `sub_add_cancel` (`a - b + b = a`) instantiated at
`a = ν₀`, `b = ∑ r j`, so no bespoke axiom is needed — Mathlib's field/ring
arithmetic on `ℝ` already proves it.
-/

namespace Praxis.Corpus.PropConserve

open Finset

/-- Conservation: for a finite family of durative draws (rates `r j ≥ 0`, one per
completed action `j`) on a fluent with initial valuation `ν₀`, applying every
action's start-decrement `∑ r j` followed by its matching end-increment `∑ r j`
returns `ν₀` unchanged: `ν₀ - ∑ r j + ∑ r j = ν₀`. Thus the terminal valuation
equals the initial one, `ν_end(φ) = ν₀(φ)`. -/
theorem conserve {J : Type} [Fintype J] (nu0 : ℝ) (r : J → ℝ)
    (_rate_nonneg : ∀ j, 0 ≤ r j) :
    nu0 - ∑ j, r j + ∑ j, r j = nu0 :=
  sub_add_cancel nu0 (∑ j, r j)

end Praxis.Corpus.PropConserve
