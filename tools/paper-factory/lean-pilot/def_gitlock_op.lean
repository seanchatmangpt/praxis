/-
def:gitlock-op

For named resource `L` at HEAD OID `H`, the CAS lock is acquired by
`git update-ref refs/locks/L H 0`, atomically creating `refs/locks/L` pointing
to `H` iff the reference is absent; deletion on drop releases the lock.

This models the single combined "lock operation" interface built from the
`acquire`/`release` primitives of `def:gitlock`: an inductive type of the two
possible lock operations, together with a step function applying an operation
to a registry.
-/

abbrev Resource := String
abbrev OID := String

def LockRegistry := Resource → Option OID

def LockRegistry.empty : LockRegistry := fun _ => none

inductive AcquireResult where
  | acquired (reg : LockRegistry) : AcquireResult
  | conflict : AcquireResult

def acquire (reg : LockRegistry) (L : Resource) (H : OID) : AcquireResult :=
  match reg L with
  | some _ => AcquireResult.conflict
  | none   => AcquireResult.acquired (fun r => if r = L then some H else reg r)

def release (reg : LockRegistry) (L : Resource) : LockRegistry :=
  fun r => if r = L then none else reg r

/-- The combined gitlock operation: either a CAS-acquire at a given OID, or a
release, for a named resource. -/
inductive LockOp where
  | acquireOp (L : Resource) (H : OID) : LockOp
  | releaseOp (L : Resource) : LockOp

/-- Apply a `LockOp` to a registry, producing the resulting registry. An
acquire that conflicts leaves the registry unchanged (the CAS fails). -/
def LockOp.step (reg : LockRegistry) : LockOp → LockRegistry
  | LockOp.acquireOp L H =>
      match acquire reg L H with
      | AcquireResult.acquired reg' => reg'
      | AcquireResult.conflict => reg
  | LockOp.releaseOp L => release reg L
