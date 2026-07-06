import Praxis.Corpus.def_chain

/-!
prop:fold, reformalized in the Mathlib lane.

"$h_n$ is a deterministic function of $(\Genesis,\fr_1,\dots,\fr_n)$, computable by a
single left fold; equal inputs give bit-identical $h_n$."

`def:chain` already defines `chain ledger := chainCommitments ledger :=
(ledger : List Frame).foldl chainStep genesis`. So the two clauses of this proposition
are:

1. `chain` *is* a single left fold over `(genesis, fr_1, ..., fr_n)` -- this holds
   definitionally (`rfl`) by unfolding `chain`/`chainCommitments`, no new axiom needed.
2. Determinism / "equal inputs give bit-identical outputs" -- `chain` is an ordinary
   total Lean function `Ledger → Digest`, so this is exactly `congrArg chain`, a
   one-line consequence of Lean being a functional (deterministic) language; no
   Mathlib lemma or axiom is needed beyond core's `congrArg`.

No new axioms are introduced by this file.
-/

/-- Clause 1: `chain` computes its output by a single left fold of `chainStep`
starting from `genesis`, run over the list of frames `(fr_1, ..., fr_n)`. This is
definitional truth about `def:chain`'s construction, not an extra assumption. -/
theorem chain_is_left_fold (ledger : Ledger) :
    chain ledger = (ledger : List Frame).foldl chainStep genesis := rfl

/-- Clause 2: determinism -- equal inputs `(Genesis, fr_1, ..., fr_n)` (i.e. equal
ledgers, since `genesis` is a fixed constant) give a bit-identical `h_n`. Since
`chain` is an ordinary deterministic Lean function, this is immediate from
`congrArg`. -/
theorem chain_deterministic {l₁ l₂ : Ledger} (h : l₁ = l₂) :
    chain l₁ = chain l₂ :=
  congrArg chain h

/-- Combined statement of `prop:fold`: `chain` is computed by a single left fold,
and is a deterministic function of the ledger (equal inputs give bit-identical
outputs). -/
theorem prop_fold (ledger : Ledger) (l₁ l₂ : Ledger) (h : l₁ = l₂) :
    chain ledger = (ledger : List Frame).foldl chainStep genesis ∧ chain l₁ = chain l₂ :=
  ⟨chain_is_left_fold ledger, chain_deterministic h⟩
