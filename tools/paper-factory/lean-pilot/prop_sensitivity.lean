/-
prop:sensitivity — Let `T(φ=c)` be the makespan the planner finds when
fluent `φ`'s initial capacity is `c`. The capacity delta reports
`(T(c-1), T(c), T(c+1))`, and `φ` is binding iff `T(c-1) ≠ T(c)` (or the
problem is infeasible at `c-1`); this is a finite-difference sensitivity of
the specific schedule the greedy planner produced, not a dual shadow price
of an optimal scheduler.

Formalized in bare Lean 4 core (no mathlib). We model the planner's
makespan-as-a-function-of-capacity as an arbitrary `T : Nat → Option Nat`
(reusing the `makespan : List Op → Nat` shape of `def:makespan` for the
feasible case, with `none` standing for "infeasible at this capacity" —
this is exactly the parenthetical "or the problem is infeasible at c-1"
in the source text, folded into equality on `Option Nat` so that
infeasibility at `c-1` alone already forces `T(c-1) ≠ T(c)` whenever `c` is
feasible, without a separate disjunct). `φ` is binding at `c` iff
`T(c-1) ≠ T(c)`, definitionally the finite difference the statement
describes.
-/

/-- `isBinding T c` — fluent `φ` (represented abstractly by its
capacity-indexed makespan function `T`) is binding at capacity `c` iff the
makespan strictly changes (or becomes/ceases to be infeasible, captured by
`Option Nat` equality) when capacity drops from `c` to `c - 1`. -/
def isBinding (T : Nat → Option Nat) (c : Nat) : Prop :=
  T (c - 1) ≠ T c

/-- prop:sensitivity, contrapositive form — `φ` fails to be binding at `c`
exactly when the makespan (or its infeasibility) at `c - 1` and at `c`
agree. A real proof: `isBinding T c` unfolds to `¬ (T (c-1) = T c)`, so its
negation is `¬¬ (T (c-1) = T c)`, and `Option Nat` has decidable equality,
so double-negation elimination applies. -/
theorem sensitivity_not_binding_iff (T : Nat → Option Nat) (c : Nat) :
    ¬ isBinding T c ↔ T (c - 1) = T c := by
  unfold isBinding
  constructor
  · intro h
    by_cases h2 : T (c - 1) = T c
    · exact h2
    · exact absurd h2 h
  · intro h hne
    exact hne h

/-- prop:sensitivity, direct form — `φ` is binding at `c` iff `T(c-1)`
differs from `T(c)`, stated as the defining finite difference from the
source text. -/
theorem sensitivity_binding_iff (T : Nat → Option Nat) (c : Nat) :
    isBinding T c ↔ T (c - 1) ≠ T c :=
  Iff.rfl
