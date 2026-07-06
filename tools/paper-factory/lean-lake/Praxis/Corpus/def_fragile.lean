import Mathlib.Data.Fintype.Basic

/-!
# `def:fragile` — Fragile preconditions

A precondition predicate `p` of capability `c` is fragile iff no capability in the problem
produces `p`. The initial state is a one-time gift: if a fragile fact is lost mid-run, nothing
in the plan can lawfully re-produce it — whereas a fact with even one producer is recoverable
by restarting that producer.

We model the finite set of capabilities as a `Fintype C`, predicates as a type `P`, and the
"produces" relation as `produces : C → P → Prop` (capability `c` produces predicate `p`).
`fragile produces p` holds iff no capability produces `p`. This is a plain existential/decidable
predicate over a `Fintype`, built entirely from core logic and Mathlib's `Fintype`/`Decidable`
machinery; no axioms are introduced.
-/

namespace Praxis.Corpus.DefFragile

variable {C : Type*} [Fintype C] {P : Type*}

/-- `p` is fragile with respect to `produces` iff no capability `c` produces `p`. -/
def fragile (produces : C → P → Prop) (p : P) : Prop :=
  ∀ c : C, ¬ produces c p

/-- Decidability of `fragile` when `produces` is decidable, via Mathlib's `Fintype` quantifier
instance (a finite `∀` over a decidable predicate is decidable). -/
instance fragile.decidable (produces : C → P → Prop) (p : P)
    [DecidablePred fun c => produces c p] : Decidable (fragile produces p) :=
  Fintype.decidableForallFintype

end Praxis.Corpus.DefFragile
