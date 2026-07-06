import Praxis.Mathlib.PropMonoid
import Praxis.Corpus.def_denialabstract

/-!
prop:monoid

$(\Deny_n,\lor,\bm 0)$ is a commutative monoid in which every element is
idempotent; it is the join-semilattice of the Boolean lattice
$(2^{[n]},\subseteq)$ under $d\mapsto\supp d$, with $\bm 0$ least and $\bm 1$
greatest, so $d\lor d'$ is the least upper bound and $\preceq$ is exactly
$d\preceq d'\iff d\lor d'=d'$.

All of the underlying algebra is already proved, once and generically, in
`Praxis.Mathlib.PropMonoid`: `Deny n := Fin n → Bool` inherits a full
`BooleanAlgebra` instance from Mathlib's `Pi` lifting of `Bool`'s own
`BooleanAlgebra`, giving commutativity (`sup_comm`), associativity
(`sup_assoc`), identity (`sup_bot_eq`/`bot_sup_eq`), idempotence
(`sup_idem`), and the least-upper-bound / order characterization
(`sup_eq_right`, `sup_le`) as pre-built lattice lemmas -- none re-derived
here. `Praxis.Corpus.def_denialabstract` supplies this file's own names for
`supp` and `≼` (`Deny.supp`, `Deny.preceq`), already shown there to coincide
with Mathlib's `Finset.filter`-support and pointwise `≤` respectively.

What this file adds, on top of both imports, is only the corpus-facing
restatement of the proposition: the monoid package plus the fact that `≼`
(the corpus name, not just `≤`) is exactly `d ⊔ d' = d'`, i.e. that
`Deny.preceq` -- not only the underlying `≤` -- is characterized by the lub
condition. This is a direct corollary of `Deny.preceq`'s definitional
equality to `≤` (proved in `def_denialabstract.lean` via `Pi.le_def`)
composed with `sup_eq_right` from `PropMonoid.lean`; no new axioms.
-/

namespace Deny

/-- Full restatement of prop:monoid: `(Deny n, ⊔, ⊥)` is a commutative
monoid with every element idempotent, `⊔` is the least upper bound for the
order, and the corpus order `≼` coincides with `d ⊔ d' = d'`. Each conjunct
is a direct citation of an already-proved Mathlib/PropMonoid lemma. -/
theorem monoid_and_lattice (n : Nat) :
    ((∀ d d' : Deny n, d ⊔ d' = d' ⊔ d) ∧
     (∀ d d' d'' : Deny n, (d ⊔ d') ⊔ d'' = d ⊔ (d' ⊔ d'')) ∧
     (∀ d : Deny n, d ⊔ ⊥ = d) ∧
     (∀ d : Deny n, (⊥ : Deny n) ⊔ d = d) ∧
     (∀ d : Deny n, d ⊔ d = d)) ∧
    (∀ d d' u : Deny n, d ≤ u → d' ≤ u → d ⊔ d' ≤ u) ∧
    (∀ d d' : Deny n, (d ≼ d') ↔ d ⊔ d' = d') :=
  ⟨is_idempotent_commutative_monoid n,
   fun d d' u hu1 hu2 => or_is_lub d d' u hu1 hu2,
   fun d d' => by
     unfold preceq
     exact sup_eq_right.symm⟩

end Deny
