/-
def:worktree-ret

The retrofit applier isolates each application attempt in a fresh Git
worktree, applies the change, runs the CI oracle (g_1,...,g_m) (cargo
build, cargo test, cargo clippy), and on any gate failure executes
`git reset --hard baseline_sha`; on all-pass the worktree change is
merged and the chain receipt for the application is minted with the
worktree OID committed into the frame's obj_refs payload digest.
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

-- The outcome of one retrofit-application attempt in a fresh worktree:
-- either the gate battery failed (the worktree is reset to the baseline
-- SHA and no receipt is minted), or all gates passed, the change was
-- merged, and a chain receipt was minted.
inductive ApplyOutcome where
  | rolledBack : (baselineSha : Sha) → ApplyOutcome
  | merged : (receipt : ChainReceipt) → ApplyOutcome

-- The retrofit applier: given a fresh worktree (identified by its OID),
-- a baseline SHA to reset to on failure, a change to apply, a gate
-- battery serving as the CI oracle, and a function producing the AST of
-- the applied change plus a way to mint the payload digest from the
-- worktree OID, produces the apply outcome.
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
