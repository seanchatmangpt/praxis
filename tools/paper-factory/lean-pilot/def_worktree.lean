/-
def:worktree

An applier isolates modifications by spawning a decoupled working tree via
`git worktree add temp_path phase_branch`, running validation gates and
executing the auto-rollback protocol `git reset --hard baseline_sha` on
failure to guarantee main branch cleanliness.

We model this as a structure recording the identifying data of such an
isolated worktree operation: the temporary path, the phase branch, the
baseline commit to roll back to, and whether validation succeeded (which
determines whether the rollback protocol fires).
-/

structure Worktree where
  tempPath     : String
  phaseBranch  : String
  baselineSha  : String
  validationOk : Bool

/-- The rollback protocol fires exactly when validation failed. -/
def Worktree.needsRollback (w : Worktree) : Bool :=
  ! w.validationOk
