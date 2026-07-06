import Praxis.Mathlib.PropMonoid
import Mathlib.Data.Finset.Basic
import Mathlib.Data.Fintype.Basic

/-!
def:denialabstract

Fix `n : ℕ` lanes; the abstract denial word space is `Deny n = {0,1}^n` with
componentwise disjunction `∨` and the all-zero word `0`; lane `i` is set in
`d` when `d i = 1`, `supp d = {i : d i = 1}`, and `d ≼ d'` means
`∀ i, d i ≤ d' i`.

`Deny n` itself (`Fin n → Bool`, with `⊔` = disjunction and `⊥` = the
all-zero word) is already defined and given a full `BooleanAlgebra`
instance in `Praxis.Mathlib.PropMonoid` -- reused here via `import` rather
than redefined, per the corpus vocabulary-reuse rule.

What remains to formalize from this statement, on top of that import, is
just: (1) "lane `i` is set in `d`" as a boolean predicate, (2) `supp d` as
the finite set of set lanes, and (3) the pointwise order `≼`, all built
from pre-built Mathlib machinery -- no new axioms.
-/

namespace Deny

/-- Lane `i` is set in `d`. -/
def IsSet {n : Nat} (d : Deny n) (i : Fin n) : Prop := d i = true

/-- `supp d = {i : d i = 1}`, the finite support of `d`, built from
Mathlib's `Finset.filter` over the ambient `Fin n` (via `Finset.univ`) --
no new set-theoretic machinery needed. -/
def supp {n : Nat} (d : Deny n) : Finset (Fin n) :=
  Finset.univ.filter (fun i => d i = true)

/-- `d ≼ d'` means `∀ i, d i ≤ d' i`, i.e. the pointwise order already
carried by `Deny n`'s `BooleanAlgebra`/`PartialOrder` instance (`Bool`'s
`≤` has `false ≤ true`, matching "`0 ≤ 1`" componentwise). This is
definitionally Mathlib's `Pi.le_def` order, exposed here under the
corpus's own name. -/
def preceq {n : Nat} (d d' : Deny n) : Prop := d ≤ d'

@[inherit_doc] scoped infix:50 " ≼ " => preceq

/-- Sanity check: `≼` unfolds to exactly the pointwise `≤` from the
statement, confirmed rather than assumed. -/
example {n : Nat} (d d' : Deny n) : (d ≼ d') ↔ ∀ i, d i ≤ d' i := by
  unfold preceq
  exact Pi.le_def

end Deny
