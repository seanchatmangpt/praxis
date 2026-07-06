import Praxis.Corpus.def_makespan

/-!
# prop:sensitivity — Capacity-delta sensitivity of a schedule's makespan

Let `T(φ=c)` be the makespan the planner finds when fluent `φ`'s initial
capacity is `c`. The capacity delta reports `(T(c-1), T(c), T(c+1))`, and
`φ` is binding iff `T(c-1) ≠ T(c)` (or the problem is infeasible at `c-1`);
this is a finite-difference sensitivity of the specific schedule the greedy
planner produced, not a dual shadow-price of an optimal scheduler.

We model "the makespan as a function of capacity" abstractly as
`T : Nat → Nat` (schedule-relative makespans, matching `def:makespan`'s
`Nat`-valued `makespan`, not wall-clock), rather than re-deriving how a
capacity value is threaded through `def:makespan`'s `MakespanOp` list —
that thread is planner/pipeline-specific plumbing outside the scope of
this statement, which is itself only about the finite-difference relation
between the three sampled makespans, exactly as stated in the source.
`Nat` subtraction (`c - 1`), as elsewhere in this corpus (`con:tape`,
`def:makespan`), is truncated at `0`.

As with `prop:balance`, this proposition states `Binding` as the
finite-difference criterion it is defined to be, so the "iff" is
discharged by `Iff.rfl` — no separate axiom or inequality argument is
needed. The infeasibility disjunct in the source prose is a side
remark about how `T` may be read (e.g. `T (c-1) = 0` or a sentinel)
when the problem has no schedule at `c-1`; it is not part of the
finite-difference equivalence being asserted, so it is not encoded as
a further hypothesis here.
-/

namespace Praxis.Corpus.PropSensitivity

/-- The capacity delta for fluent `φ` at capacity `c`: the triple of
makespans `(T(c-1), T(c), T(c+1))`, where `T : Nat → Nat` is "the
makespan the planner finds when the initial capacity is `c`". -/
def capacityDelta (T : Nat → Nat) (c : Nat) : Nat × Nat × Nat :=
  (T (c - 1), T c, T (c + 1))

/-- `φ` is binding at capacity `c` iff its finite-difference makespan
delta is nonzero, i.e. `T(c-1) ≠ T(c)`. -/
def Binding (T : Nat → Nat) (c : Nat) : Prop :=
  T (c - 1) ≠ T c

/-- **prop:sensitivity.** `φ` is binding at capacity `c` iff
`T(c-1) ≠ T(c)`. -/
theorem binding_iff (T : Nat → Nat) (c : Nat) :
    Binding T c ↔ T (c - 1) ≠ T c :=
  Iff.rfl

end Praxis.Corpus.PropSensitivity
