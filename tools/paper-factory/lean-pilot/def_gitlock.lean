/-
def:gitlock

An atomic CAS lock for resource `L` at repository HEAD OID `H` is acquired by
`git update-ref refs/locks/L H 0`, which fails if `refs/locks/L` already exists
and succeeds atomically otherwise via POSIX rename; deletion on drop releases
the lock.

We model this abstractly: a lock registry is a finite partial map from
resource names to the OID that owns the lock. Acquiring is a CAS: it succeeds
(returning the updated registry) only when the resource is currently absent
(the `refs/locks/L` ref does not exist), and fails (leaving the registry
unchanged) otherwise. Releasing removes the entry unconditionally.
-/

/-- Resource names and OIDs are modeled as strings (ref paths / hex hashes). -/
abbrev Resource := String
abbrev OID := String

/-- A lock registry: which resources are currently locked, and by which OID. -/
def LockRegistry := Resource → Option OID

/-- The empty registry: no `refs/locks/*` refs exist. -/
def LockRegistry.empty : LockRegistry := fun _ => none

/-- Result of a CAS lock-acquire attempt. -/
inductive AcquireResult where
  | acquired (reg : LockRegistry) : AcquireResult
  | conflict : AcquireResult

/-- Atomically acquire the lock for `L` at HEAD OID `H` via
`git update-ref refs/locks/L H 0`: succeeds (creating the ref, i.e. extending
the registry) only if `refs/locks/L` currently does not exist; otherwise the
CAS fails and the registry is left untouched. -/
def acquire (reg : LockRegistry) (L : Resource) (H : OID) : AcquireResult :=
  match reg L with
  | some _ => AcquireResult.conflict
  | none   => AcquireResult.acquired (fun r => if r = L then some H else reg r)

/-- Release the lock for `L` by deleting `refs/locks/L`, regardless of whether
it was held; this models "deletion on drop". -/
def release (reg : LockRegistry) (L : Resource) : LockRegistry :=
  fun r => if r = L then none else reg r
