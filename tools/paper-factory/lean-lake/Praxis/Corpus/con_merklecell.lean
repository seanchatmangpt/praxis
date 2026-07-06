import Mathlib.Data.BitVec

/-!
con:merklecell, migrated to the Mathlib-linked lane.

Statement: let `M_{g,i}` be the terminal hash of member `i` in group `g`.
The Group Receipt hash is `H_g = chainH(sort({M_{g,i}}))`, and the Cell
Receipt hash is the rolling fold over `G` group hashes:
`H_cell = chainH(H_1 ‖ H_2 ‖ ... ‖ H_G)`.

`Digest` is composed from the pre-built `BitVec 256` (Mathlib
`Mathlib.Data.BitVec`), exactly as in `Praxis/Mathlib/DefReceipt.lean`,
rather than an opaque axiomatized type.

`chainH` remains axiomatized: it stands for the real cryptographic hash
function (BLAKE3, chained/rolling, per the corpus's own receipt
cryptography paper) applied to a list of digests. No Lean/Mathlib term
is an appropriate stand-in for an actual collision-resistant hash
implementation over `List Digest` -- modeling one concretely would
either be computationally meaningless (a fake hash) or require
importing a verified BLAKE3 implementation, which does not exist in
Mathlib and is out of scope here. This matches the justification given
for `chainH`/`chainStep` in `Praxis/Mathlib/DefReceipt.lean`.

Sorting a finite collection of digests is genuinely pre-built: rather
than axiomatizing "sort", we use core's `List.mergeSort` with the
digest's total order given by comparing the underlying `Nat` values
(`BitVec.toNat`), which is decidable.
-/

abbrev Digest := BitVec 256

/-- Terminal hash of member `i` in group `g`, i.e. `M_{g,i}`. Indexed by
two `Nat`s (group id, member id) rather than an opaque pair type, since
that is exactly what a bare pair of indices already models. -/
abbrev MemberHash := Nat → Nat → Digest

/-- The real, chained cryptographic hash of a list of digests. Axiomatized
for the reason discussed above: it stands for BLAKE3 chaining, not a
Lean-modelable pure function. -/
axiom chainH : List Digest → Digest

/-- Decidable total order on digests via the underlying natural number,
used only to sort a finite collection before folding -- not part of the
cryptographic content. -/
def digestLe (a b : Digest) : Bool := a.toNat ≤ b.toNat

/-- Sort a finite list of member hashes for group `g` (pre-built
`List.mergeSort`, no new sorting axiom needed). -/
def sortedMembers (members : List Digest) : List Digest :=
  members.mergeSort digestLe

/-- Group Receipt hash: `H_g = chainH(sort({M_{g,i}}))`. -/
noncomputable def groupReceipt (members : List Digest) : Digest :=
  chainH (sortedMembers members)

/-- Cell Receipt hash: the rolling fold over `G` group hashes,
`H_cell = chainH(H_1 ‖ H_2 ‖ ... ‖ H_G)`. -/
noncomputable def cellReceipt (groupHashes : List Digest) : Digest :=
  chainH groupHashes

example : Digest := (0 : BitVec 256)
noncomputable example (gs : List (List Digest)) : Digest :=
  cellReceipt (gs.map groupReceipt)
