import Mathlib.Data.String.Basic
import Praxis.Corpus.def_astgate
import Praxis.Corpus.def_worktree

/-!
`def:worktree-ret` -- The retrofit applier isolates each application attempt in
a fresh Git worktree, applies the change, runs the CI oracle `(g₁,…,gₘ)`
(`cargo build`, `cargo test`, `cargo clippy`), and on any gate failure executes
`git reset --hard baseline_sha`; on all-pass the worktree change is merged and
the chain receipt for the application is minted with the worktree OID
committed into the frame's `obj_refs` payload digest.

We compose this from vocabulary already migrated in this corpus rather than
re-deriving it: `WorktreeApplication` (`def:worktree`) already carries the
isolated-worktree identifiers, the `validated`/`rolledBack` fields and the
`validated = false → rolledBack = true` cleanliness guarantee; `GateBattery`
(`def:astgate`) already models the CI oracle as an ordered battery of
computable predicates whose conjunction determines admission. This
definition packages a `WorktreeApplication` with the AST/CI type `T` it was
run against, the gate battery that served as the oracle, and (on the
all-pass/merge branch) the worktree object identifier (the "worktree OID")
committed into the chain receipt's payload digest.

The worktree OID and the receipt payload digest are both represented as
`String`: they are opaque, content-addressed VCS/hash identifiers with no
numeric structure Mathlib already models (the same choice `WorktreeApplication`
makes for its own identifier fields), so no Mathlib type is dropped in favor
of an axiom here -- everything is plain data composed from `String`, `Bool`,
`Option`, and the two already-migrated corpus structures.
-/

namespace Praxis.Corpus

/-- One retrofit-applier attempt: a `WorktreeApplication` (the isolated
worktree, its identifiers, and its validated/rolledBack cleanliness guarantee
from `def:worktree`) together with the CI oracle `(g₁,…,gₘ)` it was run
through (`GateBattery T` from `def:astgate`) and the AST/CI witness `node` the
oracle was evaluated against.

`mergedOid` and `receiptDigest` are populated only on the all-pass/merge
branch: if the battery admits `node`, the worktree's object identifier is
merged in (`mergedOid`) and the chain receipt for the application is minted
with that OID committed into the frame's `obj_refs` payload digest
(`receiptDigest`); on any gate failure (mirroring the applier's
`git reset --hard baseline_sha`) both are `none`, consistent with
`application.cleanliness` forcing `rolledBack = true`. -/
structure RetrofitAttempt (T : Type) where
  /-- The isolated worktree phase this attempt ran in, with its
  validated/rolledBack cleanliness guarantee. -/
  application    : WorktreeApplication
  /-- The CI oracle `(g₁,…,gₘ)` (e.g. `cargo build`, `cargo test`,
  `cargo clippy`) the change was run through. -/
  oracle         : GateBattery T
  /-- The AST/CI witness the oracle was evaluated against. -/
  node           : T
  /-- The worktree object identifier merged in, present only when the oracle
  admits `node`. -/
  mergedOid      : Option String
  /-- The chain receipt's `obj_refs` payload digest, minted with `mergedOid`
  committed into it; present only when the oracle admits `node`. -/
  receiptDigest  : Option String
  /-- The merge/receipt data is present exactly when the oracle admits the
  node: an all-pass run merges and mints a receipt, and a failing run leaves
  both absent (mirroring the auto-rollback). -/
  mergeIffAdmits :
    oracle.admits node = true ↔ (mergedOid.isSome ∧ receiptDigest.isSome)

end Praxis.Corpus
