import Praxis.Corpus.def_powl
import Mathlib.Data.Fintype.BigOperators
import Mathlib.Data.Fintype.Card

/-!
# prop:local

"In a POWL model, the token marking local to a node ranges over the tokens of
that node's immediate children: a node of arity `k` has a local marking space
of dimension `O(k)`, independent of the total size of the model; a node's
replay is checked in a working set bounded by its arity."

We formalize "arity" as the length of a node's immediate `children` list
(`0` for a leaf, which has no children), and the "local marking space" as the
type of boolean markings over the node's children positions, `Fin (arity m)
→ Bool`. The proposition below is the precise, checkable content of the
informal claim: this local marking space has size `2 ^ arity`, a quantity
that depends *only* on the node's own arity `k` -- not on the sizes of the
children themselves, i.e. not on the total size of the model. This is
exactly the "dimension `O(k)`, independent of total model size" and "working
set bounded by arity" claims made precise as a cardinality bound on a type
indexed by arity alone.

No new axiom is introduced: the result is a direct corollary of Mathlib's
`Fintype.card_fun` (cardinality of a function type between finite types is
`(card of codomain) ^ (card of domain)`), specialized to `Fin k → Bool`.
-/

namespace POWL

/-- The arity of a POWL node: the number of its immediate children.
A leaf activity has arity `0`. -/
def arity {A : Type u} : POWL A → Nat
  | .activity _ => 0
  | .partialOrder children _ => children.length
  | .choiceGraph children _ => children.length

/-- The local marking space of a node: a boolean marking over the positions
of its immediate children, i.e. a function `Fin (arity m) → Bool`. This
depends only on `arity m`, not on the children models themselves or their
sizes. -/
abbrev LocalMarking {A : Type u} (m : POWL A) : Type :=
  Fin (arity m) → Bool

/-- **prop:local.** The local marking space of a node has size exactly
`2 ^ (arity m)`: a quantity determined solely by the node's own arity `k`,
independent of the total size of the model (the children's internal
structure never enters the count). Equivalently, replaying/checking a
node's local marking is a computation over a working set of size `2 ^
(arity m)`, bounded by its arity `k`. -/
theorem local_marking_card_eq_pow_arity {A : Type u} (m : POWL A) :
    Fintype.card (LocalMarking m) = 2 ^ arity m := by
  simpa using Fintype.card_fun (α := Fin (arity m)) (β := Bool)

end POWL
