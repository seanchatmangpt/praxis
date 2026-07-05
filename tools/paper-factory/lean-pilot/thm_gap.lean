-- thm:gap
-- GeometryGap is the value classification returns when no stored branch matchB;
-- it is never a member of any stored list. Consequently no branch ordering,
-- addition, or deletion can capture, mask, or remove the gap outcome: the map
-- admits its own incompleteness by construction, not by convention.

inductive FailureClass where
  | LogicFault
  | BudgetBreach
  | AuthorityVacuum
  | TransientFault
  | Stall
  | StarvedInput
  | CertifiedUnsat
  | GeometryGap

opaque CrashSnapshot : Type
opaque SignalPredicate : Type
opaque ParkPayload : Type
opaque CoreDump : Type

inductive Response where
  | Restart
  | Park (ρ : ParkPayload)
  | Refuse (core : CoreDump)
  | Escalate

structure Branch where
  cls : FailureClass
  sigma : List SignalPredicate
  r : Response

-- Classification is first-match-wins over an arbitrary ordered branch list,
-- where `matchB` decides whether a branch's signal conjunction fires on a
-- given crash snapshot. When no branch in the list matchB, the outcome is
-- the sentinel GeometryGap — it is never read off a stored branch, only
-- produced by exhausting the list.
def classify (matchB : Branch → CrashSnapshot → Bool)
    (bs : List Branch) (snap : CrashSnapshot) : FailureClass :=
  match bs with
  | [] => FailureClass.GeometryGap
  | b :: rest => if matchB b snap then b.cls else classify matchB rest snap

-- thm:gap. If no branch in the list matchB the snapshot (whatever the
-- ordering, and however many branches have been inserted or deleted), the
-- classifier falls through to GeometryGap. The gap outcome cannot be
-- captured, masked, or removed by any arrangement of the branch list itself:
-- it is reached exactly when the list's own matchB are exhausted.
theorem thm_gap (matchB : Branch → CrashSnapshot → Bool)
    (bs : List Branch) (snap : CrashSnapshot)
    (hnone : ∀ b ∈ bs, matchB b snap = false) :
    classify matchB bs snap = FailureClass.GeometryGap := by
  induction bs with
  | nil => rfl
  | cons b rest ih =>
    have hb : matchB b snap = false := hnone b (List.Mem.head rest)
    have hrest : ∀ b' ∈ rest, matchB b' snap = false :=
      fun b' hb' => hnone b' (List.Mem.tail b hb')
    unfold classify
    rw [hb]
    simp
    exact ih hrest
