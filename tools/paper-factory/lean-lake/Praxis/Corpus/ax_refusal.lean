/-!
ax:refusal, migrated to the Mathlib-linked lane.

Statement: there is a distinguished refusal value `Rfsl` and a space of
reasons `R_f`; a refusal is not the absence of an output but the pair
`(Rfsl, r)` with `r ∈ R_f` a machine-checkable reason, so admission is
total as a map into `Adm ∪ ({Rfsl} × R_f)`.

Per the directive to prefer pre-built Mathlib/core composition over
declaring things from scratch:

* The distinguished refusal marker `Rfsl` needs no axiom at all -- a
  single distinguished value is exactly what the pre-built `Unit` type
  (with its unique inhabitant `()`) already models. No new opaque type
  is introduced for it.
* The pairing `(Rfsl, r)` and the disjoint union `Adm ∪ ({Rfsl} × R_f)`
  are likewise not axiomatized: they are the pre-built `Prod` and `Sum`
  types from core, so "admission is total as a map into
  `Adm ⊕ (Unit × R_f)`" is realized literally as a genuine total
  Lean function type `α → Adm ⊕ (Unit × R_f)`, not asserted.

Two things remain axioms, because the source text leaves them
deliberately abstract with no intended concrete encoding:

* `Adm` -- the space of admissible outputs. The thesis never fixes what
  an admissible value *is* (it is domain-specific to whatever process is
  being admitted/refused); there is no single Mathlib type that could
  stand for an arbitrary admission space without inventing content the
  statement does not supply.
* `ReasonSpace` (playing the role of `R_f`) -- the space of
  machine-checkable refusal reasons. Like `Adm`, its internal structure
  (e.g. an error-code enum, a proof object, a string) is left
  unspecified by the statement itself, so it cannot be composed from an
  existing concrete Mathlib type without fabricating detail the source
  does not assert.
-/

/-- The space of admissible outputs. Left abstract: the statement does
not specify what an admissible value is, only that it is disjoint from
the refusal case. -/
axiom Adm : Type

/-- The space of machine-checkable refusal reasons `R_f`. Left abstract
for the same reason as `Adm`: the statement asserts only that reasons
are machine-checkable and drawn from some space, not their concrete
representation. -/
axiom ReasonSpace : Type

/-- The distinguished refusal marker `Rfsl`. Not an axiom: a single
distinguished value is exactly `Unit`'s unique inhabitant. -/
abbrev Rfsl : Type := Unit

/-- A refusal is the pair `(Rfsl, r)` with `r : ReasonSpace` a
machine-checkable reason -- composed from core's `Prod`, not asserted
as a new axiom. -/
abbrev Refusal : Type := Rfsl × ReasonSpace

/-- Admission is total as a map into `Adm ∪ ({Rfsl} × R_f)`: realized
literally as core's `Sum` type, so any total function `α → AdmissionResult`
witnesses that every input is either admitted or refused with a reason,
never silently dropped. -/
abbrev AdmissionResult : Type := Adm ⊕ Refusal

/-- A refusal is not the absence of output: `Refusal` is inhabited
whenever `ReasonSpace` is, via `((), r)`, and it injects into
`AdmissionResult` via `Sum.inr`, witnessing totality of admission. -/
example (r : ReasonSpace) : AdmissionResult := Sum.inr ((), r)
