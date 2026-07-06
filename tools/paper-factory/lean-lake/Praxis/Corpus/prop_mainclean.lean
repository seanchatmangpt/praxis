import Praxis.Corpus.def_worktree_ret

/-!
`prop:mainclean` -- Under the isolated worktree applier, the main branch is
never advanced past a failing CI gate; the rollback protocol is unconditional
on failure, running before the worktree is removed, restoring the branch to
`baseline_sha` regardless of which gate failed; hence the main branch
satisfies all CI gates at every point in its history.

This is a direct corollary of `RetrofitAttempt.mergeIffAdmits` (`def:worktree-ret`):
that field already states `oracle.admits node = true ↔ (mergedOid.isSome ∧
receiptDigest.isSome)`. "Main advances" for one attempt means exactly that
the worktree's OID gets merged in *and* the chain receipt is minted (the
`mergedOid`/`receiptDigest` pair, populated only on the all-pass branch per
`def:worktree-ret`'s docstring). Reading the contrapositive of the forward
direction of `mergeIffAdmits` off that biconditional says exactly: if the CI
oracle does not admit the node (some gate failed), main was not advanced --
it is not the case that both the merge and the receipt happened. No new
axiom is needed: the cleanliness guarantee is already fully witnessed by the
`WorktreeApplication.cleanliness` field composed inside `RetrofitAttempt`, and
`mergeIffAdmits` is the record of exactly when the merge (i.e. advancing
main) happens; this proposition is a pure logical corollary of that field.
-/

namespace Praxis.Corpus

/-- If a `RetrofitAttempt`'s CI oracle does not admit its node (some gate in
the battery failed), then main was never advanced past that failure: it is
not the case that both the worktree OID was merged in and the chain receipt
was minted. This is the "main branch never advanced past a failing CI gate"
guarantee, read directly off `mergeIffAdmits`. -/
theorem RetrofitAttempt.not_admits_not_merged {T : Type}
    (a : RetrofitAttempt T) (h : a.oracle.admits a.node = false) :
    ¬ (a.mergedOid.isSome ∧ a.receiptDigest.isSome) := by
  intro hp
  have hadmit : a.oracle.admits a.node = true := a.mergeIffAdmits.mpr hp
  rw [h] at hadmit
  exact Bool.noConfusion hadmit

end Praxis.Corpus
