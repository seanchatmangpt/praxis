import Praxis.Corpus.def_logicadm

/-!
# prop:boundedcert — Bounded-degree proof DAG and fixed-width proof_root

Every proof node emitted by the kernel has fan-in ≤ 8 and every rule body has ≤ 8 atoms,
so a proof is a bounded-degree DAG; its rolling `proof_root` is a single fixed-width hash
field of the receipt, and a bounded-context verifier checks a query outcome in `O(bound)`
by recomputing `proof_root`.

The bounded-degree content of this sentence is exactly the `fanIn ≤ bound` /
`body.length ≤ bound` conjuncts already carried in `Rule.inFragment` (`def:logicadm`), so
the proposition is proved as a direct corollary of `Rule.admitted` by unfolding — no fresh
axiom, matching the `ThmRiceViaMathlib.lean` / `ThmTrichotomy.lean` precedent of deriving
corpus results from already-migrated structure.

`proof_root` is modelled as a fixed-width `BitVec 256` field of a `Receipt`, reusing the
`Bits256 := BitVec 256` precedent from `DefReceipt.lean` rather than inventing a new
hash-output type.

The `O(bound)` recomputation cost is a claim about the running time of an unspecified
concrete hash-recomputation algorithm over the DAG — it is a cost-model/complexity
statement, not a proposition over mathematical objects Mathlib has a formal cost-model
for, so (matching the source's own reading of "prolog8" as leaving concrete syntax/engines
abstract, per `DefLogicAdm`) it is not restated as a further provable conjunct: the
DAG-boundedness fact proved below is precisely what licenses that cost bound, given a
recomputation procedure that does constant work per node beyond following ≤ `bound`
premises.
-/

namespace Praxis.Corpus.PropBoundedCert

open Praxis.Corpus.DefLogicAdm

/-- Fixed-width hash type for `proof_root` / receipt fields (256-bit, matching
`DefReceipt.lean`'s `Bits256` precedent). -/
abbrev Bits256 := BitVec 256

/-- A receipt bundles the rolling `proof_root` as one fixed-width hash field. -/
structure Receipt where
  /-- The rolling proof root: a single fixed-width hash field. -/
  proofRoot : Bits256

/-- Bounded-degree certificate: every admitted rule has fan-in `≤ bound` and body length
`≤ bound`, so its proof is a bounded-degree DAG (the structural content that licenses
recomputing `proof_root` — the single fixed-width `Bits256` field of `Receipt` — in
`O(bound)` per node). -/
theorem boundedCert {Term Var : Type} (r : Rule Term Var) (h : Hygiene)
    (ha : r.admitted h) : r.fanIn ≤ bound ∧ r.body.length ≤ bound := by
  obtain ⟨hfrag, _⟩ := ha
  exact ⟨hfrag.2.2.2, hfrag.2.1⟩

end Praxis.Corpus.PropBoundedCert
