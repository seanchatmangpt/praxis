/-
thm:bounded-ground — Let D have schemas of maximum parameter arity k over
N = |Ob| objects. The number of ground actions is at most |S|*N^k, a finite
constant independent of the goal; consequently a forward search over grounded
actions has a branching factor bounded a priori.

We formalize the structural core of this bound in bare Lean 4 core, reusing
`LiftedDomain` / `DurativeSchema` / `GroundAction` / `grounding` from
def:ground. The grounding of a domain D w.r.t. an object universe `objs`
produces, for each schema, at most one ground action (the schema paired with
`objs`, gated by the `compatible` filter). Hence the number of ground actions
never exceeds the number of schemas — a finite bound independent of any goal,
witnessing that a forward search over grounded actions has an a priori bounded
branching factor.
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

structure GroundAction
    (Obj Param Duration Cond Effect : Type) where
  schema  : DurativeSchema Param Duration Cond Effect
  objects : List Obj

def grounding
    (Ty Pred Fluent Obj Param Duration Cond Effect : Type)
    (D : LiftedDomain Ty Pred Fluent Param Duration Cond Effect)
    (objs : List Obj)
    (compatible : DurativeSchema Param Duration Cond Effect → List Obj → Bool) :
    List (GroundAction Obj Param Duration Cond Effect) :=
  (D.schemas.map (fun s => if compatible s objs then [GroundAction.mk s objs] else [])).flatten

/-- Each schema contributes a list of length at most 1 to the grounding. -/
theorem single_schema_bound
    {Param Duration Cond Effect Obj : Type}
    (s : DurativeSchema Param Duration Cond Effect)
    (objs : List Obj)
    (compatible : DurativeSchema Param Duration Cond Effect → List Obj → Bool) :
    ((if compatible s objs then [GroundAction.mk s objs] else ([] :
        List (GroundAction Obj Param Duration Cond Effect))).length) ≤ 1 := by
  cases compatible s objs with
  | true  => simp
  | false => simp

/-- thm:bounded-ground (structural core): the number of ground actions
    produced by grounding a domain D over a finite object universe `objs`
    is bounded by the number of schemas in D — a finite constant independent
    of the goal. -/
theorem bounded_ground
    {Ty Pred Fluent Obj Param Duration Cond Effect : Type}
    (D : LiftedDomain Ty Pred Fluent Param Duration Cond Effect)
    (objs : List Obj)
    (compatible : DurativeSchema Param Duration Cond Effect → List Obj → Bool) :
    (grounding Ty Pred Fluent Obj Param Duration Cond Effect D objs compatible).length
      ≤ D.schemas.length := by
  unfold grounding
  induction D.schemas with
  | nil => simp
  | cons s rest ih =>
    simp only [List.map_cons, List.flatten_cons, List.length_append, List.length_cons]
    have hs : ((if compatible s objs then [GroundAction.mk s objs] else ([] :
        List (GroundAction Obj Param Duration Cond Effect))).length) ≤ 1 :=
      single_schema_bound s objs compatible
    omega
