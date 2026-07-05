/-
def:domain — A lifted domain is a tuple D = (T, P, F, S): a finite type hierarchy T,
a finite set P of predicate symbols, a finite set F of numeric function symbols
(fluents), and a finite set S of action schemas; a durative schema carries typed
parameters, a duration constraint, a condition C_s, and an effect E_s.

We model this abstractly in bare Lean 4 core (no mathlib): finiteness is captured
via `List` carriers (a concrete finite representation), and a durative action
schema is a structure bundling typed parameters, a duration constraint, a
condition, and an effect, each left abstract as opaque payload types supplied
by the domain.
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
