import Praxis.Milestone.V26711.ClosureModel
import Praxis.Milestone.V26711.ParentClosureAllRequired

/-!
# PROJ-769 / PRD v26.7.11 §19.7 — Idempotent Duplicate-Result Transition

Target 6 of the 9 declared Lean/Lake formalization targets at `PRD.md:1035-1043`:
"idempotent duplicate-result transition."

PRD §19.7 (`docs/jira/v26.7.11/PRD.md:910-912`), verbatim:

> Given the same correlated result twice, the second return SHALL not duplicate
> consequence or advance the workflow twice.

## Real correspondence

Models `crates/praxis-graphlaw/src/chatman/closure.rs`'s
`RecursiveSocketClosure::admit`, whose own doc comment states: "Idempotent:
admitting an already-`Admitted` child is a no-op (PRD §19.7, duplicate result)" —
`admit`'s body is an unconditional `*state = Admitted`, i.e. a constant-valued
transition at the target child. `admitAt` below is `updateAt` (from
`ClosureModel`) specialized to that same constant transition.

No axioms.
-/

variable {ι : Type} [DecidableEq ι]

/-- The `admit` transition (`closure.rs::admit`): unconditionally sets the target
child's state to `Admitted`, regardless of its prior state. -/
def admitTransition (_ : ChildState) : ChildState := ChildState.Admitted

/-- `admitAt c s`: the whole-map state after admitting child `c` once, from state
`s` (`RecursiveSocketClosure::admit`'s effect, generalized to the shared
`ClosureModel`). -/
def admitAt (s : ι → ChildState) (c : ι) : ι → ChildState :=
  updateAt s c admitTransition

/-- `thm:idempotent_duplicate_result` (state form): admitting the same child `c` a
second time produces the exact same whole-map state as admitting it once — "the
second return SHALL not ... advance the workflow twice," formalized as literal
state-map equality between the single- and double-admit outcomes. -/
theorem admitAt_idempotent (s : ι → ChildState) (c : ι) :
    admitAt (admitAt s c) c = admitAt s c := by
  funext x
  by_cases hx : x = c
  · subst hx; simp [admitAt, updateAt, admitTransition]
  · simp [admitAt, updateAt, hx]

/-- Corollary: since the second `admit` produces byte-for-byte the same state map
as the first, any closure verdict computed from that state map (e.g. `all_required`,
PRD §9) is likewise unaffected by the duplicate — "the second return SHALL not
duplicate consequence." -/
theorem allRequiredClosed_admitAt_idempotent (children : Finset ι) (s : ι → ChildState)
    (c : ι) :
    allRequiredClosed children (admitAt (admitAt s c) c) ↔
      allRequiredClosed children (admitAt s c) := by
  rw [admitAt_idempotent]
