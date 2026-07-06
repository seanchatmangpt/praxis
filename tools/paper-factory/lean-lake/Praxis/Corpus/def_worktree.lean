import Mathlib.Data.String.Basic

/-!
`def:worktree` -- An applier isolates modifications by spawning a decoupled
working tree via `git worktree add temp_path phase_branch`, running validation
gates and executing the auto-rollback protocol `git reset --hard baseline_sha`
on failure to guarantee main branch cleanliness.

We model this as a record of the data an applier threads through one isolated
phase: the paths/branch/commit identifiers involved (represented as `String`,
since these are opaque VCS identifiers with no numeric structure Mathlib
already models), whether the validation gates passed, and whether the
auto-rollback protocol was invoked. `validated = false → rolledBack = true` is
the guarantee of main-branch cleanliness, expressed as a field of the
structure itself rather than left implicit.
-/

/-- An isolated worktree-based application phase: the identifiers naming the
decoupled tree (`tempPath`, `phaseBranch`), the commit it forked from
(`baselineSha`), whether its validation gates passed, and whether the
auto-rollback protocol (`git reset --hard baselineSha`) was executed. The
`cleanliness` field records the invariant that failing validation forces a
rollback, guaranteeing the main branch is left clean. -/
structure WorktreeApplication where
  tempPath     : String
  phaseBranch  : String
  baselineSha  : String
  validated    : Bool
  rolledBack   : Bool
  cleanliness  : validated = false → rolledBack = true
