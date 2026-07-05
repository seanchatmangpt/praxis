-- meas:recoveryinvisible
-- On the lawobject plan with two injected transient crashes at judge: both
-- crashes land in named branches (TransientFault, then Stall), the run
-- completes, and the final root hash is byte-identical to the crash-free
-- run's. Park-then-re-admit heals to the same identity; machine death
-- composes: kill-9 mid-restart-loop, WAL recovery, identical receipt.
--
-- Formalization (measurement, no proof obligation): we model a crash branch
-- as one of the named outcomes, a run as either crash-free or subject to a
-- sequence of injected crash branches, and "recovery is invisible" as the
-- Prop that the root hash produced is independent of which (possibly empty)
-- sequence of crash branches was injected along the way. This mirrors
-- prop:totalaccounting's use of a total function into a closed outcome type.

inductive CrashBranch where
  | TransientFault
  | Stall

-- A run of the lawobject plan, parameterized by an arbitrary hash carrier
-- `Hash` and an arbitrary node/state carrier `Node`.
structure LawObjectRun (Node Hash : Type) where
  -- crash branches injected during this run, in order (empty = crash-free)
  injected : List CrashBranch
  -- the root hash the run settles on after completion / recovery
  rootHash : Node → Hash

-- Two runs "cohere" when, despite differing injected crash sequences, they
-- settle on the same root hash for every node — i.e. recovery is invisible
-- at the level of the final receipt.
def RecoveryInvisible {Node Hash : Type}
    (crashFree : LawObjectRun Node Hash)
    (crashed : LawObjectRun Node Hash) : Prop :=
  ∀ n : Node, crashFree.rootHash n = crashed.rootHash n

-- The two-crash instance named in the statement: TransientFault then Stall,
-- injected at judge, versus the crash-free baseline.
def twoCrashInstance : List CrashBranch :=
  [CrashBranch.TransientFault, CrashBranch.Stall]

-- Well-formedness sanity check for the measurement: the named instance has
-- exactly the two branches described (both crashes land, in order).
example : twoCrashInstance.length = 2 := by decide
