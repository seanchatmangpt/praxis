/-
Label: def:depth
Kind: definition

Let Dag = (V, E) be the plan's data-dependency DAG (edge u → v iff an
add-effect of u feeds a precondition of v); define d(v) = 0 for sources
and d(v) = 1 + max_{u → v} d(u) otherwise. A stage S_k = {v : d(v) = k}.

We model the DAG abstractly by a vertex type `V` and an edge relation
`E : V → V → Prop`. Rather than compute `d` (which would require
well-foundedness machinery from mathlib), we characterize it as a
predicate on candidate depth functions `d : V → Nat`, and define the
stage `S k` as the set of vertices at depth `k` under such a `d`.
-/

structure Dag (V : Type) where
  E : V → V → Prop

/-- `v` is a source of the DAG: no edge feeds into it. -/
def Dag.IsSource {V : Type} (G : Dag V) (v : V) : Prop :=
  ¬ ∃ u, G.E u v

/-- `n` is an upper bound realized by some predecessor's depth, i.e.
`n = 1 + d u` for some edge `u → v`, and `n` is the maximum such value:
every predecessor `u'` of `v` has `d u' ≤ n - 1`. -/
def Dag.IsMaxPredDepth {V : Type} (G : Dag V) (d : V → Nat) (v : V) (n : Nat) : Prop :=
  (∃ u, G.E u v ∧ d u + 1 = n) ∧ (∀ u, G.E u v → d u + 1 ≤ n)

/-- `d` is a valid depth function for `G`: sources get depth 0, and every
non-source vertex gets one more than the maximum depth of its predecessors. -/
def Dag.IsDepth {V : Type} (G : Dag V) (d : V → Nat) : Prop :=
  ∀ v, (G.IsSource v → d v = 0) ∧ (¬ G.IsSource v → G.IsMaxPredDepth d v (d v))

/-- The stage `S k`: all vertices whose depth under `d` equals `k`.
(Bare Lean core has no `Set` type, so a stage is represented as the
predicate `V → Prop` cutting out that subset.) -/
def Dag.Stage {V : Type} (_G : Dag V) (d : V → Nat) (k : Nat) : V → Prop :=
  fun v => d v = k
