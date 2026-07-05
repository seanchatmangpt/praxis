-- prop:totalaccounting
-- Every node of every supervised run carries exactly one disposition
-- (Completed(r), Parked, SkippedBy, GaveUp); no silent rows exist
-- (test-pinned: |dispositions| = |V|).
--
-- Formalization: a "disposition" is one of four cases, `Completed` carrying a
-- lawful response payload. A supervised run assigns each node of `V` exactly
-- one disposition via a total function `disp : V → Disposition` (totality of
-- the function *is* the "exactly one disposition, no silent rows" clause).
-- The accounting identity is that the list of dispositions obtained by
-- running `disp` over the node list has the same length as the node list.

inductive Response where
  | Restart
  | Park
  | Refuse
  | Escalate

inductive Disposition where
  | Completed (r : Response)
  | Parked
  | SkippedBy
  | GaveUp

-- A supervised run: a list of nodes `V` (over an arbitrary carrier type `Node`)
-- together with a total disposition assignment.
structure SupervisedRun (Node : Type) where
  V : List Node
  disp : Node → Disposition

-- The recorded disposition rows for a run.
def SupervisedRun.dispositions {Node : Type} (run : SupervisedRun Node) : List Disposition :=
  run.V.map run.disp

-- Total accounting: every node contributes exactly one disposition row, so the
-- number of recorded dispositions equals the number of nodes — no silent rows.
theorem prop_totalaccounting {Node : Type} (run : SupervisedRun Node) :
    run.dispositions.length = run.V.length := by
  simp [SupervisedRun.dispositions]
