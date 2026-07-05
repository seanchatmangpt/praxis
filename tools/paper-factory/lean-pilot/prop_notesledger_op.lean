/-
prop:notesledger-op

Under the CAS lock of Definition def:gitlock-op, each append of a receipt
record is a commit transition serialized as NDJSON under the annotated commit
OID, immutable by Git's content-addressed object model, with concurrent
appends serialized by the acquire-fail branch of the CAS.

We model "serialized by the acquire-fail branch of the CAS" as: whenever the
lock for a resource `L` is already held (some prior OID is recorded), a
concurrent append attempt — represented as `LockOp.acquireOp L H'` for any
candidate OID `H'` — is rejected by the CAS conflict branch and leaves the
lock registry completely unchanged. This is exactly the serialization
guarantee: a second, concurrent appender cannot silently overwrite the first.
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

inductive LockOp where
  | acquireOp (L : Resource) (H : OID) : LockOp
  | releaseOp (L : Resource) : LockOp

def LockOp.step (reg : LockRegistry) : LockOp → LockRegistry
  | LockOp.acquireOp L H =>
      match acquire reg L H with
      | AcquireResult.acquired reg' => reg'
      | AcquireResult.conflict => reg
  | LockOp.releaseOp L => release reg L

/-- Concurrent-append serialization: if resource `L` is already locked
(holds some OID `H0`), then a concurrent acquire attempt at any OID `H'`
is rejected by the CAS conflict branch and leaves the whole registry
unchanged — i.e. the ledger append is serialized, not silently raced. -/
theorem notesledger_op_serializes
    (reg : LockRegistry) (L : Resource) (H0 H' : OID)
    (hlocked : reg L = some H0) :
    LockOp.step reg (LockOp.acquireOp L H') = reg := by
  simp only [LockOp.step, acquire, hlocked]
