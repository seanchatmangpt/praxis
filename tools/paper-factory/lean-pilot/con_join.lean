/-
con:join — Grounding with type constraints is a relational join: for a schema
parameter `v` of type `τ`, the admissible objects are
`{o : TypeIndex::satisfies(SymId(o), SymId(τ))}`, walking the parent chain to
depth bounded by the height of `T`, reducing effective branching from `N^k`
to `∏_i |{o : type(o) ≤ τ_i}|`.

We model this abstractly in bare Lean 4 core (no mathlib), reusing `SymId`
from def:symdict. `TypeIndex` is the abstract parent-chain satisfies-check
(taking a fuel bound for the walk, since we have no well-founded height proof
available here), and `admissible` filters a candidate object list down to
those objects whose type satisfies the target type — i.e. the per-parameter
join operand. `joinDomain` then takes the list of per-parameter admissible
sets (one per schema parameter) as the relational join itself: a tuple of
bindings is admissible iff each component lies in its own admissible set,
which is exactly `List.all` over the per-position membership check, giving
the `∏_i |{o : type(o) ≤ τ_i}|` branching instead of `N^k`.
-/

def SymId := { n : Nat // n > 0 }

instance : BEq SymId where
  beq a b := a.1 == b.1

/-- Abstract type index: given a fuel bound (walking the parent chain to
depth bounded by the height of the type hierarchy `T`), decide whether the
object's symbol id satisfies (is-a, up to that many parent steps) the
target type's symbol id. Left as an opaque predicate supplied by the
domain/runtime (`pddl-index`'s `TypeIndex::satisfies`). -/
structure TypeIndex where
  satisfies : (fuel : Nat) → SymId → SymId → Bool

/-- The admissible objects for a schema parameter of type `τ`: filter the
candidate object list down to those satisfying `τ` under the type index,
walking the parent chain up to `fuel` steps. -/
def admissible (ti : TypeIndex) (fuel : Nat) (objs : List SymId) (τ : SymId) :
    List SymId :=
  objs.filter (fun o => ti.satisfies fuel o τ)

/-- Grounding a schema's typed parameters as a relational join: given the
object universe, one target type per parameter, and a type index, compute
the per-parameter admissible sets (one join operand per parameter). -/
def paramDomains (ti : TypeIndex) (fuel : Nat) (objs : List SymId)
    (paramTypes : List SymId) : List (List SymId) :=
  paramTypes.map (admissible ti fuel objs)

/-- A binding tuple (one object per parameter) is admissible for the join
iff each component lies in its own parameter's admissible set — this is the
relational join itself, replacing the naive `N^k` enumeration with
`∏_i |{o : type(o) ≤ τ_i}|` by construction (each factor list is already
restricted before the tuples are ever formed). -/
def joinAdmissible (domains : List (List SymId)) (binding : List SymId) : Bool :=
  (domains.zip binding).all (fun (dom, o) => dom.elem o)

/-- The full join: all admissible binding tuples for a schema's parameters,
given the object universe, per-parameter types, and the type index. -/
def joinDomain (ti : TypeIndex) (fuel : Nat) (objs : List SymId)
    (paramTypes : List SymId) : List (List SymId) × (List SymId → Bool) :=
  let domains := paramDomains ti fuel objs paramTypes
  (domains, joinAdmissible domains)
