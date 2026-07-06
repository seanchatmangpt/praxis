import Praxis.Corpus.def_depth
import Mathlib.Logic.Relation

/-!
# `def:earned` — Stage supervision strategy and cohorts

Stage `S_k` supervises `RestForOne` iff some `v ∈ S_k` has a nonempty transitive dependent
set `D(v)`; otherwise `OneForOne`. The cohort of `v` is `{v} ∪ D(v)` under `RestForOne` and
`{v}` under `OneForOne`.

`D(v)` (the transitive dependents of `v`) is built from Mathlib's `Relation.TransGen` applied
to the same `edge` relation used in `def:depth`, filtered over the ambient `Fintype`. Since
`Relation.TransGen edge` has no automatic `DecidablePred` instance for a generic relation, we
use classical choice (`Classical.dec`) to decide membership — this is a `Prop`-level
definition, not a computable algorithm, so classical decidability introduces no axiom beyond
`Classical.choice`/`propext`/`Quot.sound`, the standard Mathlib foundation.
-/

namespace Praxis.Corpus.DefEarned

open Praxis.Corpus.DefDepth

variable {V : Type*} [Fintype V] [DecidableEq V]

/-- The transitive dependent set `D(v) = {u : v →⁺ u}`, i.e. all vertices reachable from `v`
by one or more `edge` steps (Mathlib's `Relation.TransGen`). -/
noncomputable def D (edge : V → V → Prop) (v : V) : Finset V :=
  @Finset.filter V (fun u => Relation.TransGen edge v u)
    (fun u => Classical.propDecidable _) Finset.univ

/-- The two supervision strategies. -/
inductive Strategy
  | restForOne
  | oneForOne
  deriving DecidableEq, Repr

/-- Stage `S_k` supervises `RestForOne` iff some `v ∈ S_k` has a nonempty transitive dependent
set `D(v)`; otherwise it supervises `OneForOne`. -/
noncomputable def strategyOf (edge : V → V → Prop) [DecidableRel edge] (hwf : WellFounded edge)
    (k : ℕ) : Strategy :=
  haveI := Classical.propDecidable (∃ v ∈ stage edge hwf k, (D edge v).Nonempty)
  if ∃ v ∈ stage edge hwf k, (D edge v).Nonempty then
    Strategy.restForOne
  else
    Strategy.oneForOne

/-- The cohort of `v`: `{v} ∪ D(v)` under `RestForOne`, `{v}` alone under `OneForOne`. -/
noncomputable def cohort (edge : V → V → Prop) (v : V) (s : Strategy) : Finset V :=
  match s with
  | Strategy.restForOne => insert v (D edge v)
  | Strategy.oneForOne => {v}

end Praxis.Corpus.DefEarned
