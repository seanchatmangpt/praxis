import Praxis.Corpus.def_logicadm

/-!
# def:queryresult — `Kernel::query` result type

`Kernel::query` returns a `QueryResult`, one of `Answered (Vec<Decision>)`,
`Denied (Box<Decision>)`, or `Invalid (RejectionCode)`; a `Decision` carries a proof
(a `Vec<ProofNode>` DAG, positive or negative) and a receipt sufficient for
deterministic replay.

We reuse `Praxis.Corpus.DefLogicAdm.RejectionCode` (already migrated for `def:logicadm`)
verbatim for the `Invalid` case rather than re-declaring a fresh rejection enum, since the
thesis text does not distinguish a separate rejection vocabulary for query results.

The proof DAG (`Vec<ProofNode>`) and the receipt are both left as abstract parameters
(`ProofNode` and `Receipt` type variables) rather than invented concrete encodings: the
source fixes no concrete node/receipt syntax here (the receipt's concrete structure, e.g.
BLAKE3/genesis-folded, is defined elsewhere in the corpus and is orthogonal to this
result-shape definition). `List` (Mathlib-free, from core) models `Vec`, matching the
`def:logicadm` precedent of using `List.length`-bounded records without pulling in
Mathlib machinery for a plain sequence type. The polarity of a proof ("positive or
negative") is modeled as a `Bool` flag alongside the DAG, since no concrete polarity
encoding is fixed by the source either.
-/

namespace Praxis.Corpus.DefQueryResult

open Praxis.Corpus.DefLogicAdm

/-- A `Decision`: a proof (a DAG of `ProofNode`s, with an overall polarity — `true` for
positive, `false` for negative) together with a `Receipt` sufficient for deterministic
replay. `ProofNode` and `Receipt` are left abstract: the source gives no concrete
node/receipt syntax at this definition site. -/
structure Decision (ProofNode Receipt : Type) where
  /-- The proof DAG: a sequence of proof nodes. -/
  proof : List ProofNode
  /-- Overall polarity of the proof: `true` = positive, `false` = negative. -/
  positive : Bool
  /-- The receipt sufficient for deterministic replay. -/
  receipt : Receipt

/-- The result of `Kernel::query`: either `Answered` with a list of `Decision`s,
`Denied` with a single `Decision`, or `Invalid` with a `RejectionCode` (reusing the
rejection vocabulary already migrated for `def:logicadm`). -/
inductive QueryResult (ProofNode Receipt : Type) where
  /-- The query was answered: a list of supporting decisions. -/
  | Answered : List (Decision ProofNode Receipt) → QueryResult ProofNode Receipt
  /-- The query was denied: a single decision witnessing the denial. -/
  | Denied : Decision ProofNode Receipt → QueryResult ProofNode Receipt
  /-- The query itself was inadmissible; carries why. -/
  | Invalid : RejectionCode → QueryResult ProofNode Receipt

end Praxis.Corpus.DefQueryResult
