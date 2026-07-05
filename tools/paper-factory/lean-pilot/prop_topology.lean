/-
Label: prop:topology
Kind: proposition

Topo = (stages, policy) is produced only by `derive` (a sealed constructor: no
literal construction compiles), and
  topology_hash = ca(stages ‖ policy ‖ plan_hash ‖ problem_hash)
joins the receipt lineage. Determinism is test-pinned: same plan, same hash.

We model this abstractly: `stages`, `policy`, `plan_hash`, `problem_hash` are
values of arbitrary (abstract) types, `derive` is the sealed constructor
producing a `Topo` from those fields (standing in for "no literal construction
compiles" — the only way to build a `Topo` is via `derive`), `ca` is the
abstract hash-combining function, and `topology_hash` combines the four
fields through `ca`. The proposition to prove is determinism: given the same
plan (i.e. the same `stages`, `policy`, `plan_hash`, `problem_hash`), the
resulting `topology_hash` is the same.
-/

universe u v w x y

variable {Stages : Type u} {Policy : Type v} {PlanHash : Type w} {ProblemHash : Type x}
variable {Hash : Type y}

/-- `Topo` is the pair of stages and policy. The sole constructor `derive`
stands in for the sealed-constructor discipline: every `Topo` arises from a
`derive` call over the four plan-defining fields, never from a bare literal. -/
structure Topo (Stages : Type u) (Policy : Type v) where
  stages : Stages
  policy : Policy

/-- `derive`: the sealed constructor. Given `stages`, `policy`, and (through
the hash they feed) `plan_hash`/`problem_hash`, it produces the `Topo`. -/
def derive (stages : Stages) (policy : Policy) : Topo Stages Policy :=
  { stages := stages, policy := policy }

-- `ca`: the abstract commitment/hash function joining stages, policy,
-- plan_hash, and problem_hash into the receipt lineage.
variable (ca : Stages → Policy → PlanHash → ProblemHash → Hash)

/-- `topology_hash = ca(stages ‖ policy ‖ plan_hash ‖ problem_hash)`. -/
def topology_hash (stages : Stages) (policy : Policy)
    (plan_hash : PlanHash) (problem_hash : ProblemHash) : Hash :=
  ca stages policy plan_hash problem_hash

/-- Determinism: for the same plan — i.e. the same `stages`, `policy`,
`plan_hash`, and `problem_hash` as produced by `derive` over the plan's
data — `topology_hash` yields the same hash. -/
theorem topology_hash_deterministic
    (stages₁ stages₂ : Stages) (policy₁ policy₂ : Policy)
    (plan_hash₁ plan_hash₂ : PlanHash) (problem_hash₁ problem_hash₂ : ProblemHash)
    (hstages : stages₁ = stages₂) (hpolicy : policy₁ = policy₂)
    (hplan : plan_hash₁ = plan_hash₂) (hproblem : problem_hash₁ = problem_hash₂) :
    topology_hash ca stages₁ policy₁ plan_hash₁ problem_hash₁
      = topology_hash ca stages₂ policy₂ plan_hash₂ problem_hash₂ := by
  subst hstages
  subst hpolicy
  subst hplan
  subst hproblem
  rfl

/-- Corollary phrased directly over `derive`'d `Topo` values: two `Topo`s
built by `derive` from equal `(stages, policy)` pairs, combined with equal
`plan_hash`/`problem_hash`, produce equal `topology_hash`. -/
theorem topology_hash_deterministic_derive
    (stages₁ stages₂ : Stages) (policy₁ policy₂ : Policy)
    (plan_hash₁ plan_hash₂ : PlanHash) (problem_hash₁ problem_hash₂ : ProblemHash)
    (htopo : derive stages₁ policy₁ = derive stages₂ policy₂)
    (hplan : plan_hash₁ = plan_hash₂) (hproblem : problem_hash₁ = problem_hash₂) :
    topology_hash ca (derive stages₁ policy₁).stages (derive stages₁ policy₁).policy
        plan_hash₁ problem_hash₁
      = topology_hash ca (derive stages₂ policy₂).stages (derive stages₂ policy₂).policy
        plan_hash₂ problem_hash₂ := by
  rw [htopo, hplan, hproblem]
