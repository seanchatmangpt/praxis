/-
def:ground — Given a domain D and a problem (Ob, m0, nu0, gamma), the grounding
substitutes every type-compatible object tuple into each schema's parameters,
producing a finite set of ground actions; a grounded state is a pair (m, nu) of
a set m of true ground atoms and a fluent valuation nu.

We model this abstractly in bare Lean 4 core (no mathlib), reusing the
`LiftedDomain` / `DurativeSchema` vocabulary from def:domain. Objects, ground
atoms, and fluent values are left as opaque payload types. A ground action is
a schema whose parameters have been instantiated by a concrete object tuple
(here: a list of objects, one per parameter slot). A grounded state bundles a
finite set of true ground atoms (as a `List`) with a fluent valuation, i.e. a
total function from fluents to values.
-/

structure DurativeSchema (Param Duration Cond Effect : Type) where
  parameters : List Param
  duration   : Duration
  condition  : Cond
  effect     : Effect

structure LiftedDomain
    (Ty Pred Fluent Param Duration Cond Effect : Type) where
  types      : List Ty
  predicates : List Pred
  fluents    : List Fluent
  schemas    : List (DurativeSchema Param Duration Cond Effect)

/-- A ground action: a lifted schema together with a concrete object tuple
    (one object per parameter slot) substituted into it. -/
structure GroundAction
    (Obj Param Duration Cond Effect : Type) where
  schema  : DurativeSchema Param Duration Cond Effect
  objects : List Obj

/-- The grounding of a domain w.r.t. a finite object universe: the finite set
    of ground actions obtained by pairing each schema (from the domain) with
    every type-compatible object tuple drawn from `objs`. The concrete
    type-compatibility filter is left abstract as `compatible`; the result is
    the list of ground actions built from the domain's schemas. -/
def grounding
    (Ty Pred Fluent Obj Param Duration Cond Effect : Type)
    (D : LiftedDomain Ty Pred Fluent Param Duration Cond Effect)
    (objs : List Obj)
    (compatible : DurativeSchema Param Duration Cond Effect → List Obj → Bool) :
    List (GroundAction Obj Param Duration Cond Effect) :=
  (D.schemas.map (fun s => if compatible s objs then [GroundAction.mk s objs] else [])).flatten

/-- A grounded state: a pair (m, nu) of a finite set `m` of true ground atoms
    and a fluent valuation `nu` assigning a value to every fluent. -/
structure GroundState (Atom Fluent Val : Type) where
  atoms      : List Atom
  valuation  : Fluent → Val
