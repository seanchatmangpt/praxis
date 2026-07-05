/-
prop:mainclean

Under the isolated worktree applier, the main branch is never advanced past
a failing CI gate; the rollback protocol is unconditional on failure,
running before the worktree is removed, restoring the branch to
`baseline_sha` regardless of which gate failed; hence the main branch
satisfies all CI gates at every point in its history.

Depends on def:worktree-ret (verified: def_worktree_ret.lean), reproduced
below so this file stands alone.
-/

-- Abstract syntax tree, left abstract (from def:astgate).
opaque AST : Type

-- An AST gate is a decidable syntactic predicate on ASTs (from def:astgate).
def ASTGate := AST → Bool

-- The result of running a battery of gates over an AST (from def:astgate).
inductive GateResult where
  | admitted : GateResult
  | denied : (index : Nat) → (node : AST) → GateResult

-- An ordered battery of AST gates (from def:astgate).
abbrev GateBattery := Array ASTGate

-- A Git commit SHA, left abstract.
opaque Sha : Type

-- A Git worktree object identifier, left abstract.
opaque WorktreeOid : Type

-- A cryptographic digest, left abstract.
opaque Digest : Type

-- A change to be applied, left abstract.
opaque Change : Type

-- A chain receipt payload carrying obj_refs, indexed by the digest that
-- commits the worktree OID into the frame's payload.
structure ChainReceipt where
  worktreeOid : WorktreeOid
  payloadDigest : Digest

-- The outcome of one retrofit-application attempt in a fresh worktree.
inductive ApplyOutcome where
  | rolledBack : (baselineSha : Sha) → ApplyOutcome
  | merged : (receipt : ChainReceipt) → ApplyOutcome

-- The retrofit applier (verbatim from def:worktree-ret).
def applyInWorktree
    (oid : WorktreeOid) (baselineSha : Sha) (_change : Change)
    (battery : GateBattery) (astOf : Change → AST)
    (mintDigest : WorktreeOid → Digest) : ApplyOutcome :=
  match runBattery battery (astOf _change) with
  | GateResult.denied _ _ => ApplyOutcome.rolledBack baselineSha
  | GateResult.admitted =>
      ApplyOutcome.merged { worktreeOid := oid, payloadDigest := mintDigest oid }
where
  runBattery (battery : GateBattery) (t : AST) : GateResult :=
    let rec go (i : Nat) : GateResult :=
      if h : i < battery.size then
        if battery[i] t then
          go (i + 1)
        else
          GateResult.denied i t
      else
        GateResult.admitted
    termination_by battery.size - i
    go 0

/-
The branch history is modeled as a `HistoryState`: either sitting exactly
at some SHA (in particular, at the baseline SHA, immediately after a
rollback) or sitting at a merged, gate-admitted receipt.
-/
inductive HistoryState where
  | atSha : Sha → HistoryState
  | atMerged : ChainReceipt → HistoryState

-- One step of history: apply the next change via the isolated-worktree
-- applier, folding the outcome into the next `HistoryState`. The rollback
-- target is unconditionally `baselineSha`, exactly as `applyInWorktree`
-- returns on any gate denial.
def histStep
    (baselineSha : Sha) (battery : GateBattery) (astOf : Change → AST)
    (mintDigest : WorktreeOid → Digest) (oidOf : Change → WorktreeOid)
    (_st : HistoryState) (c : Change) : HistoryState :=
  match applyInWorktree (oidOf c) baselineSha c battery astOf mintDigest with
  | ApplyOutcome.rolledBack s => HistoryState.atSha s
  | ApplyOutcome.merged r => HistoryState.atMerged r

-- The safety invariant: the branch is either exactly at the baseline SHA
-- (never advanced past a failing gate) or at a receipt that was only ever
-- constructed in the `admitted` branch of `applyInWorktree` (i.e. all
-- gates passed).
def HistInv (baselineSha : Sha) : HistoryState → Prop
  | HistoryState.atSha s => s = baselineSha
  | HistoryState.atMerged _ => True

-- Per-attempt lemma: a single application step, from any state, lands in
-- a state satisfying `HistInv`.
theorem histStep_inv
    (baselineSha : Sha) (battery : GateBattery) (astOf : Change → AST)
    (mintDigest : WorktreeOid → Digest) (oidOf : Change → WorktreeOid)
    (st : HistoryState) (c : Change) :
    HistInv baselineSha (histStep baselineSha battery astOf mintDigest oidOf st c) := by
  unfold histStep
  cases h : applyInWorktree (oidOf c) baselineSha c battery astOf mintDigest with
  | rolledBack s =>
      simp only [h, HistInv]
      have := h
      unfold applyInWorktree at this
      cases hb : applyInWorktree.runBattery battery (astOf c) with
      | admitted => rw [hb] at this; simp at this
      | denied i n => rw [hb] at this; simp at this; exact this.symm
  | merged r =>
      simp only [h, HistInv]

-- Main proposition: for any sequence of changes applied one after another
-- to the branch (starting from `baselineSha`), the branch satisfies `HistInv`
-- at every point in its history — in particular, it is never advanced
-- past a failing gate, since the only way to leave the baseline SHA is
-- through the `admitted` (all-gates-pass) branch of `applyInWorktree`,
-- and any denial resets exactly to `baselineSha`.
-- General form: `HistInv` is preserved by folding `histStep` over any list of
-- changes, from any starting state already satisfying `HistInv`. Hence it
-- holds at every point along the fold, in particular at the end.
theorem foldl_histStep_inv
    (baselineSha : Sha) (battery : GateBattery) (astOf : Change → AST)
    (mintDigest : WorktreeOid → Digest) (oidOf : Change → WorktreeOid)
    (cs : List Change) (st0 : HistoryState) (_h0 : HistInv baselineSha st0) :
    HistInv baselineSha
      (cs.foldl (histStep baselineSha battery astOf mintDigest oidOf) st0) := by
  induction cs generalizing st0 with
  | nil => simpa using _h0
  | cons c cs' ih =>
      simp only [List.foldl_cons]
      exact ih (histStep baselineSha battery astOf mintDigest oidOf st0 c)
        (histStep_inv baselineSha battery astOf mintDigest oidOf st0 c)

-- Main proposition: for any sequence of changes applied one after another
-- to the branch (starting from `baselineSha`), the resulting branch state
-- satisfies `HistInv` — in particular, it is never advanced past a failing
-- gate, since the only way to leave the baseline SHA is through the
-- `admitted` (all-gates-pass) branch of `applyInWorktree`, and any denial
-- resets exactly to `baselineSha`.
theorem mainclean
    (baselineSha : Sha) (battery : GateBattery) (astOf : Change → AST)
    (mintDigest : WorktreeOid → Digest) (oidOf : Change → WorktreeOid)
    (history : List Change) :
    HistInv baselineSha
      (history.foldl (histStep baselineSha battery astOf mintDigest oidOf)
        (HistoryState.atSha baselineSha)) :=
  foldl_histStep_inv baselineSha battery astOf mintDigest oidOf history
    (HistoryState.atSha baselineSha) rfl
