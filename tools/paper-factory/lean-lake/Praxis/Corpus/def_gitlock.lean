import Mathlib.Data.Finmap

/-!
# `def:gitlock`

An atomic CAS lock for resource `L` at repository HEAD OID `H` is acquired by
`git update-ref refs/locks/L H 0`, which fails if `refs/locks/L` already exists
and succeeds atomically otherwise via POSIX `rename`; deletion on drop releases
the lock.

We model the ref namespace as a finite partial map from resource names to the
OID that currently holds the lock (`Finmap` from Mathlib, i.e. a computable
finite partial function `String →. String`). Acquiring the lock is exactly a
compare-and-set: it succeeds (returning the updated store) only when the key
is currently absent, mirroring `git update-ref`'s refusal to overwrite an
existing ref without `--force`. Releasing the lock is deletion of that key.

No axiom is introduced: `Finmap` and its `lookup`/`insert`/`erase` operations
already exist in Mathlib and are literally the CAS-map structure being
modeled here.
-/

namespace Praxis.Corpus

/-- The ref-namespace store: a finite partial map from resource name to the
OID currently holding its lock. -/
abbrev GitLockStore := Finmap (fun _ : String => String)

/-- Attempt to acquire the lock for resource `L` at HEAD OID `H` against the
current store. Fails (`none`) if `refs/locks/L` already exists; otherwise
succeeds atomically, returning the store with the new lock ref inserted. -/
def gitlockAcquire (store : GitLockStore) (L H : String) : Option GitLockStore :=
  if store.lookup L |>.isSome then
    none
  else
    some (store.insert L H)

/-- Release the lock for resource `L` by deleting `refs/locks/L`. -/
def gitlockRelease (store : GitLockStore) (L : String) : GitLockStore :=
  store.erase L

end Praxis.Corpus
