import Praxis.Corpus.def_gitlock

/-!
# `def:gitlock-op`

For named resource `L` at HEAD OID `H`, the CAS lock is acquired by
`git update-ref refs/locks/L H 0`, atomically creating `refs/locks/L` pointing
to `H` iff the reference is absent; deletion on drop releases the lock.

We package the two lock operations from `def:gitlock`
(`Praxis.Corpus.gitlockAcquire`, `Praxis.Corpus.gitlockRelease`) into a single
inductive `GitLockOp` type describing which operation is requested, plus a
`step` function giving its semantics as a state transformer on `GitLockStore`.
This is exactly the CAS-map structure already modeled in `def:gitlock`; no new
axiom is introduced, since `GitLockOp` is a plain two-constructor inductive
(the Lean/Mathlib analogue of an enum) and `step` is composed directly from
`gitlockAcquire`/`gitlockRelease`.
-/

namespace Praxis.Corpus

/-- A requested git-lock operation: acquire resource `L` at OID `H`, or
release resource `L`. -/
inductive GitLockOp where
  | acquire (L H : String)
  | release (L : String)
  deriving Repr, DecidableEq

/-- Semantics of a `GitLockOp` as a (possibly failing) transformer on the
ref-namespace store. `acquire` is the CAS operation from `def:gitlock`
(`none` on conflict); `release` always succeeds. -/
def GitLockOp.step (store : GitLockStore) : GitLockOp → Option GitLockStore
  | .acquire L H => gitlockAcquire store L H
  | .release L => some (gitlockRelease store L)

end Praxis.Corpus
