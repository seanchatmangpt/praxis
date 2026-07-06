import Mathlib.Tactic
import Praxis.Corpus.def_branch

/-!
`thm:gap`: `GeometryGap` is the value classification returns when no stored
branch matches; it is never a member of any stored list. Consequently no
branch ordering, addition, or deletion can capture, mask, or remove the gap
outcome: the map admits its own incompleteness by construction, not by
convention.

Formalization notes:
- Classification over an ordered branch list, "first-match-wins", is exactly
  `List.find?` (already core/Mathlib) followed by projecting the matched
  branch's class; we model the outcome as `Option FailureClass`, where `none`
  *is* the `GeometryGap` value -- it is a distinguished constructor of a sum
  type, not a value smuggled into `FailureClass` itself, so "never a member
  of any stored list" is true by construction (no `Branch` or `FailureClass`
  literal can ever equal `none`; `Option.some_ne_none` already gives this).
- The substantive content of the theorem is the other direction: `none` is
  produced *exactly* when every branch in the list fails to match, for *any*
  list (so no ordering/addition/deletion of branches -- i.e. no choice of
  `l`) can turn a genuine non-match into something other than the gap. This
  is `List.find?_eq_none` from core Lean's `Init.Data.List.Find`, reused
  directly rather than reproved.
-/

variable {Snapshot Payload : Type}

/-- First-match-wins classification: the class of the first branch in `l`
    whose signal conjunction matches `snap`, or `none` (the `GeometryGap`)
    if no branch matches. -/
def classify (l : List (Branch Snapshot Payload)) (snap : Snapshot) :
    Option FailureClass :=
  (l.find? (fun b => b.matches snap)).map (·.c)

/-- `thm:gap`: classification yields the gap (`none`) on `l` at `snap` iff no
    branch in `l` matches `snap` -- i.e. the gap outcome is exactly the
    "nothing matched" state, for every branch list `l` (no ordering,
    addition, or deletion of branches can mask or remove it), and by the
    same token the gap is never equal to a `some c` produced by any stored
    branch. -/
theorem thm_gap (l : List (Branch Snapshot Payload)) (snap : Snapshot) :
    classify l snap = none ↔ ∀ b ∈ l, ¬ b.matches snap := by
  unfold classify
  rw [Option.map_eq_none_iff, List.find?_eq_none]

/-- The gap outcome is never a stored classification value: whenever
    classification produces `none`, it is definitionally distinct from every
    `some c` a matching branch could have produced. -/
theorem thm_gap_ne_some (l : List (Branch Snapshot Payload)) (snap : Snapshot)
    (h : classify l snap = none) (c : FailureClass) : classify l snap ≠ some c := by
  rw [h]; exact (Option.some_ne_none c).symm
