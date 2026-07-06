import Mathlib.Data.Fintype.Basic
import Mathlib.Data.Finset.Basic

/-!
# def:domain — Lifted planning domain

A lifted domain is a tuple `D = (T, P, F, S)`: a finite type hierarchy `T`, a finite
set `P` of predicate symbols, a finite set `F` of numeric function symbols (fluents),
and a finite set `S` of action schemas; a durative schema carries typed parameters,
a duration constraint, a condition `C_s`, and an effect `E_s`.

We model the four carriers `T, P, F, S` as `Fintype`-equipped types (finiteness via
Mathlib's `Fintype` class, reused rather than hand-rolled), and each durative schema
as a record of a finite parameter list (typed by `T`), a duration constraint over the
ambient numeric fluents, a precondition, and an effect. The condition/effect languages
are left as abstract `Prop`-valued predicates over states — no concrete syntax for
conditions/effects is fixed by the source thesis text, so they are kept abstract
rather than invented here.
-/

namespace Praxis.Corpus.DefDomain

/-- A durative action schema over type hierarchy `Ty`, fluent symbols `Fl`, and an
abstract state space `St`: typed parameters, a duration constraint, a precondition,
and an effect. -/
structure DurativeSchema (Ty Fl St : Type) where
  /-- Finite list of typed parameters. -/
  params : List Ty
  /-- Duration constraint over the fluents, as an abstract numeric-valued predicate. -/
  duration : Fl → St → Prop
  /-- Precondition `C_s`. -/
  cond : St → Prop
  /-- Effect `E_s`, relating pre- and post-states. -/
  effect : St → St → Prop

/-- A lifted domain `D = (T, P, F, S)`: a finite type hierarchy `T`, a finite set `P`
of predicate symbols, a finite set `F` of numeric fluent symbols, and a finite set `S`
of action schemas (each a `DurativeSchema`) over an ambient state space `St`. -/
structure LiftedDomain where
  /-- Type hierarchy. -/
  Ty : Type
  [tyFintype : Fintype Ty]
  /-- Predicate symbols. -/
  Pred : Type
  [predFintype : Fintype Pred]
  /-- Numeric fluent symbols. -/
  Fluent : Type
  [fluentFintype : Fintype Fluent]
  /-- Ambient state space over which conditions/effects/durations are evaluated. -/
  State : Type
  /-- Action schemas. -/
  Schema : Type
  [schemaFintype : Fintype Schema]
  /-- Each schema's durative content. -/
  schemaData : Schema → DurativeSchema Ty Fluent State

end Praxis.Corpus.DefDomain
