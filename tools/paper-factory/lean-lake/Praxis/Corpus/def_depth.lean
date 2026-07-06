import Mathlib.Data.Fintype.Basic
import Mathlib.Data.Finset.Lattice.Fold
import Mathlib.Order.WellFounded

/-!
# `def:depth` — Plan dependency-DAG depth and stages

Let `Dag = (V, E)` be the plan's data-dependency DAG (edge `u → v` iff an add-effect of `u`
feeds a precondition of `v`); define `d(v) = 0` for sources and `d(v) = 1 + max_{u → v} d(u)`
otherwise. A stage `S_k = {v : d(v) = k}`.

We model the vertex set `V` as a `Fintype` (a plan has finitely many actions) and the edge
relation `edge : V → V → Prop` as a well-founded relation (acyclicity of the dependency DAG,
i.e. no vertex depends on itself through a chain of edges). `depth` is then defined by
well-founded recursion directly on this relation, taking the `Finset.sup` (max, with `⊥ = 0`
default) of the depths of all predecessors `u` with `edge u v`, plus one whenever some
predecessor exists. `stage k` is the finite set of vertices at depth exactly `k`.

This is built entirely from core `WellFounded.fix` and Mathlib's `Finset.sup`/`Fintype`
machinery; no new axioms are introduced.
-/

namespace Praxis.Corpus.DefDepth

variable {V : Type*} [Fintype V] [DecidableEq V]

/-- The depth function `d` on a data-dependency DAG: `d(v) = 0` if `v` has no predecessor
(a source), and `d(v) = 1 + max_{u → v} d(u)` otherwise. Defined by well-founded recursion on
the edge relation, whose well-foundedness encodes acyclicity of the DAG. -/
noncomputable def depth (edge : V → V → Prop) [DecidableRel edge] (hwf : WellFounded edge) :
    V → ℕ :=
  hwf.fix fun v ih =>
    let preds : Finset V := Finset.univ.filter (fun u => edge u v)
    preds.attach.sup (fun u => ih u.1 (Finset.mem_filter.mp u.2).2) +
      (if preds.Nonempty then 1 else 0)

/-- Stage `S_k = {v : d(v) = k}`, the finite set of vertices at depth exactly `k`. -/
noncomputable def stage (edge : V → V → Prop) [DecidableRel edge] (hwf : WellFounded edge)
    (k : ℕ) : Finset V :=
  Finset.univ.filter (fun v => depth edge hwf v = k)

end Praxis.Corpus.DefDepth
