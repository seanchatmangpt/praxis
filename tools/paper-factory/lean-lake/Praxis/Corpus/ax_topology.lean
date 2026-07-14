import Mathlib.Data.BitVec
import Praxis.Corpus.def_earned

/-!
`prop:topology`'s external-hash axiom, split out of `prop_topology.lean`.

`ca`: the concatenate-then-hash combinator producing `topology_hash` from `stages`,
`policy`, `plan_hash`, and `problem_hash`. Genuinely axiomatized for the same reason
`chainH`/`chainStep` are axiomatized in `Praxis.Mathlib.DefReceipt`: it stands for a real
cryptographic hash function (BLAKE3 per the corpus), and no Lean/Mathlib term is an
appropriate stand-in for an actual collision-resistant hash implementation. This is a
genuine external-system axiom, not a placeholder for something provable in-Lean, so it
is declared here rather than inline in `prop_topology.lean`.
-/

namespace Praxis.Corpus.PropTopology

open Praxis.Corpus.DefEarned

/-- `Bits256`, reused from the receipt lineage's own hash-digest type. -/
abbrev Bits256 := BitVec 256

/-- `ca`: the concatenate-then-hash combinator producing `topology_hash` from `stages`,
`policy`, `plan_hash`, and `problem_hash`. Genuinely axiomatized -- see the module doc: this
stands for a real cryptographic hash function (BLAKE3), which has no Lean/Mathlib term as a
faithful stand-in. -/
axiom ca {V : Type*} : (ℕ → Finset V) → (ℕ → Strategy) → Bits256 → Bits256 → Bits256

end Praxis.Corpus.PropTopology
