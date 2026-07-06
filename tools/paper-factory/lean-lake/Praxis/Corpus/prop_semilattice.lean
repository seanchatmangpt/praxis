import Praxis.Corpus.con_denial

/-!
Label: prop:semilattice

"$(\{0,1\}^n,\lor,\bm0)$ is a bounded join-semilattice; consequently adding
obligations can only enlarge the denial, never shrink it."

`Deny n = Fin n → Bool` already carries a full `BooleanAlgebra` instance
(`Praxis.Mathlib.PropMonoid`, imported transitively via `Praxis.Corpus.con_denial`),
and every `BooleanAlgebra` is in particular a `SemilatticeSup` with `⊥` as the
bottom element -- i.e. exactly a *bounded join-semilattice*: `⊔` is
associative, commutative, idempotent, and `⊥` is the identity/least element.
This is not re-derived here; it is the content of `Deny.is_idempotent_commutative_monoid`
in `Praxis.Mathlib.PropMonoid`, confirmed below by `inferInstance`.

The consequential clause -- "adding obligations can only enlarge the denial,
never shrink it" -- is the monotonicity of the join: for the total denial
`d(o) = ⨆ d_i(o)` (`Deny.denial`, from `con:denial`), adjoining one more
obligation's lane map `e` to the running total `d` replaces it with `d ⊔ e`,
and `d ≤ d ⊔ e` always holds in a `SemilatticeSup` (Mathlib's `le_sup_left`).
Dually the new obligation's own contribution never exceeds the enlarged
total (`le_sup_right`). This is a genuine proof obligation (`proposition`),
discharged by pre-built Mathlib lattice lemmas -- no new axioms, no `sorry`.
-/

namespace Deny

/-- `Deny n` is a bounded join-semilattice: `⊔` (disjunction) is the join and
`⊥` (the all-zero word) is the least element, both witnessed by the
pre-built `BooleanAlgebra` (hence `SemilatticeSup` + `OrderBot`) instance on
`Fin n → Bool` -- confirmed, not assumed. -/
example (n : Nat) : SemilatticeSup (Deny n) := inferInstance

example (n : Nat) : OrderBot (Deny n) := inferInstance

/-- Adjoining one more obligation's lane map `e` to a running total denial
`d` can only *enlarge* the denial (in the `≼` / `≤` order on `Deny n`), never
shrink it: the old total `d` is always `≤` the new total `d ⊔ e`. -/
theorem denial_grows {n : Nat} (d e : Deny n) : d ≤ d ⊔ e :=
  le_sup_left

/-- Symmetrically, the freshly-added obligation's own lane map `e` is also
never larger than the enlarged total: `e ≤ d ⊔ e`. -/
theorem denial_grows' {n : Nat} (d e : Deny n) : e ≤ d ⊔ e :=
  le_sup_right

/-- The bundled proposition: as a consequence of `Deny n` being a bounded
join-semilattice (witnessed above by `SemilatticeSup`/`OrderBot` instances,
which are structures, not propositions, so are checked separately rather
than conjoined here), joining in one more obligation's denial can only
enlarge the running total denial, never shrink it (both directions of
monotonicity). -/
theorem denial_monotone (n : Nat) :
    (∀ d e : Deny n, d ≤ d ⊔ e) ∧ (∀ d e : Deny n, e ≤ d ⊔ e) :=
  ⟨fun d e => denial_grows d e, fun d e => denial_grows' d e⟩

end Deny
