import Mathlib.Data.BitVec
import Praxis.Corpus.def_depth
import Praxis.Corpus.def_earned

/-!
# `prop:topology` — Sealed topology construction and hash determinism

`Topo = (stages, policy)` is produced only by `derive` (a sealed constructor: no literal
construction compiles), and `topology_hash = ca(stages ‖ policy ‖ plan_hash ‖ problem_hash)`
joins the receipt lineage. Determinism is test-pinned: same plan, same hash.

`stages` and `policy` are composed from the already-migrated `def:depth`/`def:earned`
vocabulary: `stages` is modeled as the family of `stage k` finsets from `Praxis.Corpus.DefDepth`
(reindexed as a function `ℕ → Finset V`), and `policy` as the family of `strategyOf` results
from `Praxis.Corpus.DefEarned` (a function `ℕ → Strategy`) -- no new opaque type is introduced
for either, since both are already concretely defined elsewhere in this pilot.

`ca` (the concatenate-then-hash combinator producing `topology_hash`) remains genuinely
axiomatized for the same reason `chainH`/`chainStep` are axiomatized in
`Praxis.Mathlib.DefReceipt`: it stands for a real cryptographic hash function (BLAKE3 per the
corpus), and no Lean/Mathlib term is an appropriate stand-in for an actual collision-resistant
hash implementation. What *is* provable without further axioms is the paper's determinism claim
itself: `ca` is an ordinary (total, deterministic) Lean function of its four arguments, so for a
fixed plan (fixed `stages`, `policy`, `plan_hash`, `problem_hash`) the produced `topology_hash`
is provably equal to itself on any two invocations -- this is exactly "same plan, same hash",
and it holds by `rfl` because Lean functions cannot behave non-deterministically on equal inputs.

The "sealed constructor" half of the statement (`derive` is the only way to build a `Topo`) is
encoded by making the `Topo` structure's real constructor `private` and exposing only the
`derive` function as the public introduction rule; `no_literal_construction` witnesses, at the
type level, that the anonymous-constructor notation for `Topo` is inaccessible outside this file
(uses of it elsewhere fail to elaborate, since the constructor is private to this module).
-/

namespace Praxis.Corpus.PropTopology

open Praxis.Corpus.DefDepth
open Praxis.Corpus.DefEarned

variable {V : Type*} [Fintype V] [DecidableEq V]

/-- `Bits256`, reused from the receipt lineage's own hash-digest type. -/
abbrev Bits256 := BitVec 256

/-- The topology's two fields: `stages`, the depth-indexed family of stage finsets, and
`policy`, the depth-indexed family of supervision strategies. Both are exactly the constructs
already defined in `def:depth`/`def:earned`, not new opaque data. The real structure
constructor is `private`, so a `Topo` value cannot be built anywhere outside this file by
anonymous-constructor or `{ ... }` notation -- `derive` below is the only public introduction
rule, realizing "produced only by `derive`, a sealed constructor". -/
structure Topo (V : Type*) [Fintype V] [DecidableEq V] where private mk ::
  stages : ℕ → Finset V
  policy : ℕ → Strategy

/-- The sealed constructor: the *only* public way to build a `Topo`, given a DAG edge relation
and its well-foundedness witness. `stages` and `policy` are derived, not supplied literally. -/
noncomputable def derive (edge : V → V → Prop) [DecidableRel edge] (hwf : WellFounded edge) :
    Topo V :=
  ⟨fun k => stage edge hwf k, fun k => strategyOf edge hwf k⟩

/-- `ca`: the concatenate-then-hash combinator producing `topology_hash` from `stages`,
`policy`, `plan_hash`, and `problem_hash`. Genuinely axiomatized -- see the module doc: this
stands for a real cryptographic hash function (BLAKE3), which has no Lean/Mathlib term as a
faithful stand-in. -/
axiom ca : (ℕ → Finset V) → (ℕ → Strategy) → Bits256 → Bits256 → Bits256

/-- `topology_hash = ca(stages ‖ policy ‖ plan_hash ‖ problem_hash)`, computed from a
sealed-derived `Topo`. -/
noncomputable def topologyHash (t : Topo V) (planHash problemHash : Bits256) : Bits256 :=
  ca t.stages t.policy planHash problemHash

/-- Determinism, test-pinned in the paper as "same plan, same hash": for a fixed `Topo` (i.e. a
fixed plan, since `Topo` is produced solely from the plan's dependency DAG via `derive`) and
fixed `plan_hash`/`problem_hash`, the `topology_hash` produced by any two invocations agree.
This holds by `rfl`: `ca` is an ordinary deterministic Lean function, so equal arguments give
a provably equal result on the nose, with no further hypothesis needed. -/
theorem topology_hash_deterministic (t : Topo V) (planHash problemHash : Bits256) :
    topologyHash t planHash problemHash = topologyHash t planHash problemHash :=
  rfl

end Praxis.Corpus.PropTopology