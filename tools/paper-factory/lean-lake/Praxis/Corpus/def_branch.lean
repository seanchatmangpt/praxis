import Mathlib.Tactic
import Praxis.Corpus.def_classes

/-!
`def:branch`: A branch is `(c, Σ, r)`: a class, a conjunction of signal predicates
over a crash snapshot, and a lawful response
`r ∈ {Restart, Park(ρ), Refuse(core), Escalate}`.

The geometry `Geo` maps each node to an ordered branch list; classification is
first-match-wins; `geometry_hash = ca(Geo ‖ topology_hash)`.

This file formalizes only the `Branch` record itself (the per-branch datum), not
the geometry map or its hash -- those are separate downstream statements.

Design notes on reuse vs. axiomatization:
- The response alternatives `{Restart, Park(ρ), Refuse(core), Escalate}` form a
  finite closed sum type with two payload-carrying constructors, exactly what
  Lean's native `inductive` gives (as in `def:classes`); no Mathlib composition
  needed beyond `deriving DecidableEq`.
- `ρ` (park payload) and `core` (refusal core payload) are left as an abstract
  type parameter `Payload` rather than axiomatized as opaque types: the thesis
  text does not fix their concrete representation at this point in the corpus,
  so introducing a bespoke opaque `axiom Rho : Type` would assert structure the
  source doesn't commit to. Parameterizing is the honest formalization and
  matches how `DefReceipt.lean` composed fields from `Nat`/`String`/`Prod`/`Sigma`
  instead of axiomatizing them.
- The "conjunction of signal predicates over a crash snapshot" `Σ` is modeled as
  `List (Snapshot → Bool)`, reusing Lean's/Mathlib's existing `List` and function
  types rather than inventing a bespoke predicate-conjunction structure; the
  conjunction itself is the pointwise `List.all` fold, already provided by core.
-/

/-- The lawful response alternatives attached to a branch, parameterized by the
    payload type carried by `Park` and `Refuse` (the thesis leaves `ρ` and
    `core` abstract at this point in the corpus). -/
inductive Response (Payload : Type) : Type where
  | Restart
  | Park (ρ : Payload)
  | Refuse (core : Payload)
  | Escalate
  deriving DecidableEq

/-- A branch `(c, Σ, r)`: a failure class, a conjunction (as a list, folded by
    `List.all`) of signal predicates over a crash snapshot, and a lawful
    response. -/
structure Branch (Snapshot Payload : Type) : Type where
  c : FailureClass
  sigma : List (Snapshot → Bool)
  r : Response Payload

/-- The conjunction of signal predicates in `sigma` (the thesis's `Σ`),
    evaluated at a given crash snapshot: true iff every predicate in the
    list holds. -/
def Branch.matches {Snapshot Payload : Type} (b : Branch Snapshot Payload)
    (snap : Snapshot) : Bool :=
  b.sigma.all (fun p => p snap)
