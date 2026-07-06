import Praxis.Corpus.def_gitlock_op

/-!
# `prop:notesledger-op`

Under the CAS lock of `def:gitlock-op`, each append of a receipt record is a
commit transition serialized as NDJSON under the annotated commit OID,
immutable by Git's content-addressed object model, with concurrent appends
serialized by the acquire-fail branch of the CAS.

The load-bearing claim is the serialization one: an append is modeled as a
`GitLockOp.acquire L H` step (the append can only proceed once the lock at
resource `L` is acquired at OID `H`). When a concurrent appender already
holds the lock (`store.lookup L = some H'`), the CAS `acquire` branch of
`GitLockOp.step` deterministically refuses (`none`) rather than racing —
this is exactly `gitlockAcquire`'s `isSome` guard from `def:gitlock`. No new
axiom is introduced: the statement is a direct consequence of the `step`/
`gitlockAcquire` definitions already composed in `def:gitlock-op`.
-/

namespace Praxis.Corpus

/-- Concurrent notesledger appends are serialized by the acquire-fail branch
of the CAS: if resource `L` is already locked (held at some OID `H'`), a
competing `acquire L H` step is refused. -/
theorem notesledgerAppendSerialized (store : GitLockStore) (L H H' : String)
    (hheld : store.lookup L = some H') :
    GitLockOp.step store (GitLockOp.acquire L H) = none := by
  simp [GitLockOp.step, gitlockAcquire, hheld]

end Praxis.Corpus
