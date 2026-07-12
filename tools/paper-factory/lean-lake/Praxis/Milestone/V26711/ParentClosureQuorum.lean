import Mathlib.Data.Finset.Card
import Praxis.Milestone.V26711.ClosureModel

/-!
# PROJ-769 / PRD v26.7.11 §9 — Parent Closure: `quorum(q)`

Target 5 of the 9 declared Lean/Lake formalization targets at `PRD.md:1035-1043`:
"parent closure for quorum."

PRD §9 (`docs/jira/v26.7.11/PRD.md:517-521`), verbatim:

> For `quorum(q)`:
> `Close(W) ⟺ |{c∈C(W):TerminalAdmitted(c)}| ≥ q`

## Real correspondence

Models `crates/praxis-graphlaw/src/chatman/closure.rs`'s `ClosureLaw::Quorum(q)`
arm of `RecursiveSocketClosure::is_closed` (`closure.rs:494-503`): counts admitted
children via a `Finset`/iterator filter and compares against `q`, exactly
`quorumClosed` below.

No axioms.
-/

variable {ι : Type}

/-- `Close(W)` under `quorum(q)` (PRD §9, `closure.rs`'s `ClosureLaw::Quorum(q)`
arm): at least `q` declared children are `TerminalAdmitted`. -/
def quorumClosed (q : Nat) (children : Finset ι) (s : ι → ChildState) : Prop :=
  q ≤ (children.filter (fun c => s c = ChildState.Admitted)).card

/-- `thm:parent_closure_quorum_persists`: once `quorum(q)` closure holds, it
persists under any further pointwise state upgrade — the admitted-count can only
grow (never shrink) as states move forward under `ChildState.le`, so `quorumClosed`
is monotone exactly like `all_required`'s own persistence theorem. -/
theorem quorumClosed_persists {q : Nat} {children : Finset ι} {s s' : ι → ChildState}
    (h : quorumClosed q children s) (hle : statePointwiseLe s s') :
    quorumClosed q children s' := by
  have hsub : children.filter (fun c => s c = ChildState.Admitted) ⊆
      children.filter (fun c => s' c = ChildState.Admitted) := by
    intro c hc
    obtain ⟨hcmem, hcs⟩ := Finset.mem_filter.mp hc
    refine Finset.mem_filter.mpr ⟨hcmem, ?_⟩
    have hmono : (s c).le (s' c) := hle c
    rw [hcs] at hmono
    cases hs' : s' c with
    | Admitted => rfl
    | Open => simp [hs', ChildState.le] at hmono
    | Observed => simp [hs', ChildState.le] at hmono
  exact h.trans (Finset.card_le_card hsub)
