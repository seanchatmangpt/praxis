import Mathlib.Data.Finset.Card
import Praxis.Milestone.V26711.ClosureModel

/-!
# PROJ-769 / PRD v26.7.11 §9 — Parent Closure: `all_required`

Target 4 of the 9 declared Lean/Lake formalization targets at `PRD.md:1035-1043`:
"parent closure for `all_required`."

PRD §9 (`docs/jira/v26.7.11/PRD.md:511-515`), verbatim:

> For `all_required`:
> `Close(W) ⟺ ∀c∈C(W), TerminalAdmitted(c)`

## Real correspondence

Models `crates/praxis-graphlaw/src/chatman/closure.rs`'s `ClosureLaw::AllRequired`
arm of `RecursiveSocketClosure::is_closed` (`closure.rs:490-493`):
`Ok(self.children.values().all(|s| s.is_terminal_admitted()))` — `allRequiredClosed`
below is that same universal quantifier over a `Finset` of children, restated over
`Praxis.Milestone.V26711.ClosureModel`'s shared `ChildState` model.

No axioms.
-/

variable {ι : Type}

/-- `Close(W)` under `all_required` (PRD §9, `closure.rs`'s `ClosureLaw::AllRequired`
arm): every declared child is `TerminalAdmitted`. -/
def allRequiredClosed (children : Finset ι) (s : ι → ChildState) : Prop :=
  ∀ c ∈ children, s c = ChildState.Admitted

/-- `thm:parent_closure_all_required_persists`: once `all_required` closure holds,
it persists under any further pointwise state upgrade — admission is monotone
(`closure.rs`'s `observe`/`admit` never downgrade a child), so a closed parent under
`all_required` never becomes un-closed by more admission. -/
theorem allRequiredClosed_persists {children : Finset ι} {s s' : ι → ChildState}
    (h : allRequiredClosed children s) (hle : statePointwiseLe s s') :
    allRequiredClosed children s' := by
  intro c hc
  have hsc : s c = ChildState.Admitted := h c hc
  have hmono : (s c).le (s' c) := hle c
  rw [hsc] at hmono
  cases hs' : s' c with
  | Admitted => rfl
  | Open => simp [hs', ChildState.le] at hmono
  | Observed => simp [hs', ChildState.le] at hmono

/-- `all_required` is exactly `quorum(|C(W)|)` — the maximal quorum over the full
child set. Both directions are proved directly from the two PRD §9 formulas, not
merely declared: matches `closure.rs`'s own `Quorum(q)` construction constraint
(`q` ranges `1..=child_set.len()`, so `q = children.card` is exactly the
`all_required` boundary case). -/
theorem allRequiredClosed_iff_quorum_card {children : Finset ι} {s : ι → ChildState} :
    allRequiredClosed children s ↔
      children.card ≤ (children.filter (fun c => s c = ChildState.Admitted)).card := by
  constructor
  · intro h
    have heq : children.filter (fun c => s c = ChildState.Admitted) = children :=
      Finset.filter_true_of_mem h
    rw [heq]
  · intro h c hc
    have hsub : children.filter (fun c => s c = ChildState.Admitted) ⊆ children :=
      Finset.filter_subset (fun c => s c = ChildState.Admitted) children
    have hcardeq : children.filter (fun c => s c = ChildState.Admitted) = children :=
      Finset.eq_of_subset_of_card_le hsub h
    have hmem : c ∈ children.filter (fun c => s c = ChildState.Admitted) := by
      rw [hcardeq]; exact hc
    exact (Finset.mem_filter.mp hmem).2
