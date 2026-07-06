import Praxis.Corpus.def_chain

/-!
prop:prefix, reformalized in the Mathlib lane.

"For each $t$, $h_t$ commits every frame $\fr_1,\dots,\fr_t$ and the genesis
value; the terminal $h_n$ commits the entire history."

Formalized as: the running commitment after processing a ledger extended by
further frames is exactly what you get by continuing the fold from the
commitment already computed on the prefix. Concretely, for any two ledgers
`l1` (the prefix, frames `fr_1..fr_t`) and `l2` (the remaining frames
`fr_{t+1}..fr_n`):

  `chainCommitments (l1 ++ l2) = l2.foldl chainStep (chainCommitments l1)`

i.e. the prefix commitment `chainCommitments l1` (which itself, unfolding
`chainStep`'s definition, is built by folding `chainH` over the genesis value
and each of `fr_1,...,fr_t` in turn) is exactly the state threaded into
continuing the computation over the rest of the ledger -- so `h_t` really
does commit the genesis value and frames `fr_1,...,fr_t`, and is unaffected
by what comes after. Taking `l2 = []` recovers `chainCommitments l1 =
chainCommitments l1` (the prefix is its own terminal value), and taking
`l1 = ledger`, `l2 = []` recovers that `chain ledger = chainCommitments
ledger` commits the entire history.

No fresh axiom: this is a direct instance of the standard fold-append lemma
(`List.foldl_append`, from Lean 4 core / used pervasively in Mathlib) applied
to `chainCommitments`'s defining fold with `chainStep` as the step function
and `genesis` as the seed.
-/

/-- **prop:prefix.** The running chain commitment is prefix-stable: continuing
the fold over an appended tail `l2` starting from the prefix's commitment
`chainCommitments l1` gives the same result as folding the whole ledger
`l1 ++ l2` from genesis. Hence `h_t := chainCommitments l1` (for `l1` the
first `t` frames) commits exactly `fr_1,...,fr_t` and `genesis`, independent
of any frames appended afterward; in particular (`l1 := ledger`, `l2 := []`)
the terminal value `chain ledger` commits the entire history. -/
theorem prop_prefix (l1 l2 : List Frame) :
    chainCommitments (l1 ++ l2 : List Frame) = l2.foldl chainStep (chainCommitments l1) := by
  unfold chainCommitments
  exact List.foldl_append
