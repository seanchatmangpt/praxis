import Mathlib.Order.BooleanAlgebra.Basic
import Mathlib.Data.Fin.Basic

/-!
prop:monoid, reformalized in the Mathlib lane.

Bare-core version (`tools/paper-factory/lean-pilot/prop_monoid.lean`) proves
commutativity, associativity, identity, and idempotence of `Deny n`'s `or`
operation directly, by hand, via `funext` + `simp [Bool.or_comm]` etc. --
each property re-derived from scratch for this specific type.

Here, `Deny n := Fin n → Bool` is recognized as an instance of a structure
Mathlib already builds and proves properties of once, generically: `Bool`
is a `BooleanAlgebra` (`Mathlib.Order.BooleanAlgebra.Basic`), and Mathlib's
`Pi` instances lift any `BooleanAlgebra` pointwise across a function type,
so `Fin n → Bool` gets a full `BooleanAlgebra` instance -- and with it,
`⊔` (= our `or`), `⊥` (= our `zero`), and every lattice law (commutativity,
associativity, absorption, idempotence) as pre-built lemmas
(`sup_comm`, `sup_assoc`, `sup_idem`, `bot_sup_eq`) -- proved once in
Mathlib for every `BooleanAlgebra`, not re-derived here.

This is the concrete difference the pivot away from bare core makes: the
five bare-core theorems below are each one line citing an existing Mathlib
lemma, not a `funext`-and-`simp` proof written from scratch.
-/

abbrev Deny (n : Nat) := Fin n → Bool

namespace Deny

-- `Fin n → Bool` already has a `BooleanAlgebra` instance via Mathlib's `Pi`
-- lifting of `Bool`'s own `BooleanAlgebra` instance -- confirmed available,
-- not assumed.
example (n : Nat) : BooleanAlgebra (Deny n) := inferInstance

theorem or_comm {n : Nat} (d d' : Deny n) : d ⊔ d' = d' ⊔ d :=
  sup_comm d d'

theorem or_assoc {n : Nat} (d d' d'' : Deny n) : (d ⊔ d') ⊔ d'' = d ⊔ (d' ⊔ d'') :=
  sup_assoc d d' d''

theorem or_zero {n : Nat} (d : Deny n) : d ⊔ ⊥ = d :=
  sup_bot_eq d

theorem zero_or {n : Nat} (d : Deny n) : ⊥ ⊔ d = d :=
  bot_sup_eq d

theorem or_idem {n : Nat} (d : Deny n) : d ⊔ d = d :=
  sup_idem d

/-- The full commutative-monoid-with-idempotence package, exactly matching
the bare-core file's bundled statement, now proved by five one-line
citations of pre-built Mathlib lattice lemmas. -/
theorem is_idempotent_commutative_monoid (n : Nat) :
    (∀ d d' : Deny n, d ⊔ d' = d' ⊔ d) ∧
    (∀ d d' d'' : Deny n, (d ⊔ d') ⊔ d'' = d ⊔ (d' ⊔ d'')) ∧
    (∀ d : Deny n, d ⊔ ⊥ = d) ∧
    (∀ d : Deny n, (⊥ : Deny n) ⊔ d = d) ∧
    (∀ d : Deny n, d ⊔ d = d) :=
  ⟨or_comm, or_assoc, or_zero, zero_or, or_idem⟩

/-- The join-semilattice / partial-order characterization is likewise
pre-built: Mathlib's `BooleanAlgebra` already carries a `PartialOrder`
whose `≤` is defined so that `d ⊔ d' = d'` is exactly `d ≤ d'`
(`sup_eq_right`), so `le_refl`, `le_antisymm`, `le_trans`, and the
least-upper-bound property are all inherited from the `PartialOrder`
instance rather than proved by hand as in the bare-core version. -/
example {n : Nat} (d d' : Deny n) : d ≤ d' ↔ d ⊔ d' = d' :=
  sup_eq_right.symm

theorem or_is_lub {n : Nat} (d d' u : Deny n) (hu1 : d ≤ u) (hu2 : d' ≤ u) :
    d ⊔ d' ≤ u :=
  sup_le hu1 hu2

end Deny
