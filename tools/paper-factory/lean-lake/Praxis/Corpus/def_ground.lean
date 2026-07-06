import Mathlib.Data.Fintype.Basic
import Mathlib.Data.Finset.Basic
import Mathlib.Data.Rat.Defs
import Praxis.Corpus.def_domain


/-!
# def:ground — Grounding a lifted domain against an object universe

Given a domain `D` and a problem `(Ob, m₀, ν₀, γ)`, the grounding substitutes every
type-compatible object tuple into each schema's parameters, producing a finite set of
ground actions; a grounded state is a pair `(m, ν)` of a set `m` of true ground atoms
and a fluent valuation `ν`.

We reuse `Praxis.Corpus.DefDomain.LiftedDomain` for `D`. The object universe `Ob` is
modelled as a `Fintype` (finiteness reused from Mathlib rather than hand-rolled). A
ground action is a schema paired with a parameter-list-length tuple of objects (one
object per formal parameter); the set of all ground actions for a schema is finite
because both the schema's parameter list and `Ob` are finite (`Fintype`/`Finset`
machinery from Mathlib, not reconstructed here). A ground atom is a predicate symbol
applied to a tuple of objects (arity again given by a `List Ty`-style parameter list,
kept abstract as in `def:domain`); a grounded state bundles a `Finset` of true ground
atoms together with a fluent valuation `ν : GroundFluent → ℚ` (rationals stand in for
the source's unspecified numeric value domain — no concrete numeric type is fixed by
the thesis text, so we pick Mathlib's `ℚ` as the standard finite/discrete choice used
elsewhere in this corpus rather than inventing a bespoke one).
-/

namespace Praxis.Corpus.DefGround

open Praxis.Corpus.DefDomain

/-- A ground atom: a predicate symbol from `D.Pred` applied to a finite tuple of
objects drawn from the object universe `Ob`, one per argument position. The argument
count is left abstract via a `List Unit`-shaped arity witness `arity`, mirroring how
`DurativeSchema.params : List Ty` fixes a schema's arity in `def:domain`. -/
structure GroundAtom (D : LiftedDomain) (Ob : Type) where
  pred : D.Pred
  args : List Ob

/-- A ground action: an action schema `s : D.Schema` together with one concrete
object, drawn from `Ob`, substituted for each of its typed parameters
(`D.schemaData s |>.params`). -/
structure GroundAction (D : LiftedDomain) (Ob : Type) where
  schema : D.Schema
  args : List Ob
  /-- The substituted object tuple has the same length as the schema's typed
  parameter list, i.e. every parameter received exactly one object. -/
  arity_eq : args.length = (D.schemaData schema).params.length

/-- A ground fluent: a fluent symbol from `D.Fluent` applied to a finite tuple of
objects, analogous to `GroundAtom` for predicates. -/
structure GroundFluent (D : LiftedDomain) (Ob : Type) where
  fluent : D.Fluent
  args : List Ob

/-- A grounded state `(m, ν)`: a finite set `m` of true ground atoms together with a
fluent valuation `ν` assigning a rational value to every ground fluent. Finiteness of
`m` uses Mathlib's `Finset` (requires decidable equality on ground atoms, discharged
via the `DecidableEq` instance parameter). -/
structure GroundedState (D : LiftedDomain) (Ob : Type) [DecidableEq (GroundAtom D Ob)] where
  /-- The set `m` of currently-true ground atoms. -/
  trueAtoms : Finset (GroundAtom D Ob)
  /-- The fluent valuation `ν`. -/
  val : GroundFluent D Ob → ℚ

/-- A planning problem over domain `D`: a finite object universe `Ob`, an initial
grounded state `(m₀, ν₀)`, and a rational discount factor `γ`. -/
structure Problem (D : LiftedDomain) where
  Ob : Type
  [obFintype : Fintype Ob]
  [atomDecEq : DecidableEq (GroundAtom D Ob)]
  initTrueAtoms : Finset (GroundAtom D Ob)
  initVal : GroundFluent D Ob → ℚ
  discount : ℚ

end Praxis.Corpus.DefGround
