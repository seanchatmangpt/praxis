/-
Label: def:earned
Kind: definition

Stage S_k supervises RestForOne iff some v ∈ S_k has a nonempty transitive
dependent set D(v); otherwise OneForOne. The cohort of v is {v} ∪ D(v)
under RestForOne and {v} under OneForOne.

We reuse the `Dag` structure from def:depth. The transitive dependent set
D(v) is the set of vertices reachable from v by following edges forward
(u → v edges represent "u feeds v", so v's dependents are vertices w such
that there is a directed path v → ... → w). We define reachability as an
inductive relation `Reaches G v w` (bare Lean core has no transitive
closure operator, so we build it directly), then `D G v` is the predicate
picking out proper dependents (w reachable from v with w ≠ v is not
required by the source text, but we follow the natural reading: D(v) is
all vertices reachable from v via at least one edge step).

A stage `S : V → Prop` (e.g. `Dag.Stage G d k` from def:depth) supervises
RestForOne iff some v satisfying S has a nonempty D(v); otherwise it
supervises OneForOne. The cohort of v is {v} ∪ D(v) under RestForOne,
and just {v} under OneForOne — represented here as the predicate
`Cohort` parameterized by which supervision strategy applies.
-/

structure Dag (V : Type) where
  E : V → V → Prop

/-- `Reaches G v w`: there is a nonempty directed path of edges from `v`
to `w` in `G` (at least one edge step). -/
inductive Dag.Reaches {V : Type} (G : Dag V) : V → V → Prop where
  | step  : ∀ {v w}, G.E v w → Dag.Reaches G v w
  | trans : ∀ {v u w}, G.E v u → Dag.Reaches G u w → Dag.Reaches G v w

/-- The transitive dependent set `D(v)`: all vertices reachable from `v`
by following at least one edge. -/
def Dag.D {V : Type} (G : Dag V) (v : V) : V → Prop :=
  fun w => G.Reaches v w

/-- `D(v)` is nonempty: some vertex is a dependent of `v`. -/
def Dag.HasDependents {V : Type} (G : Dag V) (v : V) : Prop :=
  ∃ w, G.D v w

/-- A supervision strategy: RestForOne (kill the whole cohort) or
OneForOne (restart just the failed vertex). -/
inductive Strategy where
  | RestForOne
  | OneForOne

/-- Stage `S` (a predicate on vertices, e.g. `Dag.Stage G d k`) supervises
`RestForOne` iff some vertex in the stage has a nonempty dependent set;
otherwise it supervises `OneForOne`. Expressed as a relation between a
stage and the strategy it selects (rather than a computed function, since
`∃` is not decidable in bare Lean core without classical axioms). -/
def Dag.Supervises {V : Type} (G : Dag V) (S : V → Prop) : Strategy → Prop
  | Strategy.RestForOne => ∃ v, S v ∧ G.HasDependents v
  | Strategy.OneForOne  => ¬ ∃ v, S v ∧ G.HasDependents v

/-- The cohort of `v` under a given strategy: `{v} ∪ D(v)` under
RestForOne, and just `{v}` under OneForOne. -/
def Dag.Cohort {V : Type} (G : Dag V) (strat : Strategy) (v : V) : V → Prop :=
  match strat with
  | Strategy.RestForOne => fun w => w = v ∨ G.D v w
  | Strategy.OneForOne  => fun w => w = v
