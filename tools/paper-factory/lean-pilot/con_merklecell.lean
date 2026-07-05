/-
con:merklecell (construction)

Let `M_{g,i}` be the terminal hash of member `i` in group `g`. The Group Receipt
hash is `H_g = chainH(sort({M_{g,i}}))`, and the Cell Receipt hash is the rolling
fold over `G` group hashes: `H_cell = chainH(H_1 ‖ H_2 ‖ ... ‖ H_G)`.

We work in bare Lean 4 core (no mathlib). Hashes are modeled abstractly as an
opaque type `Hash` with an abstract chaining primitive `chainH : List Hash → Hash`
(the "chain" of a hash sequence, standing in for BLAKE3-style rolling folds).
Sorting is modeled via an abstract total order on `Hash` (a `LT`/decidable
comparator) and `List.mergeSort`-free bare insertion sort implemented from
scratch, since mathlib's sort combinators are unavailable.
-/

/-- An abstract hash value, represented concretely by a natural number so that
the opaque chaining/ordering primitives below have a nonempty carrier type. -/
def Hash : Type := Nat

instance : Inhabited Hash := ⟨(0 : Nat)⟩

/-- An abstract rolling chain-hash primitive over a list of hashes, standing in
for a BLAKE3-style fold `chainH(x_1 ‖ x_2 ‖ ... ‖ x_n)`. -/
opaque chainH : List Hash → Hash

/-- An abstract total order used to sort member hashes before chaining, so the
Group Receipt hash is well-defined independent of member enumeration order. -/
opaque hashLe : Hash → Hash → Bool

/-- Bare insertion of one hash into an already-sorted list, ordered by `hashLe`. -/
partial def insertSorted (x : Hash) (l : List Hash) : List Hash :=
  match l with
  | [] => [x]
  | y :: ys => if hashLe x y then x :: y :: ys else y :: insertSorted x ys

/-- Bare insertion sort over hash lists (no mathlib `List.sort` available). -/
partial def sortHashes (l : List Hash) : List Hash :=
  match l with
  | [] => []
  | x :: xs => insertSorted x (sortHashes xs)

/-- The terminal hash of member `i` in group `g`, i.e. `M_{g,i}`, represented
as a list of per-group member-hash lists indexed by group. -/
abbrev MemberHashes := List (List Hash)

/-- The Group Receipt hash `H_g = chainH(sort({M_{g,i}}))` for a single group's
member hashes. -/
def groupReceipt (members : List Hash) : Hash :=
  chainH (sortHashes members)

/-- The Group Receipt hashes `H_1, ..., H_G` for all groups in the cell. -/
def groupReceipts (cell : MemberHashes) : List Hash :=
  cell.map groupReceipt

/-- The Cell Receipt hash `H_cell = chainH(H_1 ‖ H_2 ‖ ... ‖ H_G)`, the rolling
fold over the `G` group hashes. -/
def cellReceipt (cell : MemberHashes) : Hash :=
  chainH (groupReceipts cell)
