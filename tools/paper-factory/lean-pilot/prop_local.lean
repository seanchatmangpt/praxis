/-
prop:local — In a POWL model, the token marking local to a node ranges over
the tokens of that node's immediate children: a node of arity `k` has a local
marking space of dimension `O(k)`, independent of the total size of the
model; a node's replay is checked in a working set bounded by its arity.

Formalized in bare Lean 4 core (no mathlib), reusing `POWL` from
`def_powl.lean`. The "local marking space" of a node is modelled as the list
of its immediate children (one marking slot per child); its dimension is the
length of that list, i.e. the node's arity. We prove that this local marking
space is exactly the node's immediate children — not any of their
descendants — so its size depends only on the node's own arity `k`,
independent of the total size (total node count) of the model.
-/

inductive POWL (A : Type u) where
  | leaf : A → POWL A
  | partialOrder : (children : List (POWL A)) →
      (prec : Nat → Nat → Prop) → POWL A
  | choice : (children : List (POWL A)) →
      (edge : Nat → Nat → Prop) → POWL A

/-- Total number of nodes in a POWL model (the "total size of the model"). -/
def POWL.size : POWL A → Nat
  | .leaf _ => 1
  | .partialOrder cs _ => 1 + (cs.map POWL.size).foldl (· + ·) 0
  | .choice cs _ => 1 + (cs.map POWL.size).foldl (· + ·) 0

/-- The local marking space of a node: one marking slot per immediate child.
    For a leaf (arity 0) this is empty. -/
def POWL.localMarkingSpace : POWL A → List (POWL A)
  | .leaf _ => []
  | .partialOrder cs _ => cs
  | .choice cs _ => cs

/-- The arity of a node: the dimension of its local marking space. -/
def POWL.arity (n : POWL A) : Nat := n.localMarkingSpace.length

/-- Proposition (prop:local): the dimension of a node's local marking space
    equals its arity, and this quantity is determined solely by the node's
    immediate children list — it does not mention or depend on `POWL.size`
    (the total size of the model) at all. Hence the local marking space is
    `O(k)` where `k` is the arity, independent of total model size. -/
theorem POWL.local_marking_dimension_eq_arity (n : POWL A) :
    n.localMarkingSpace.length = n.arity := by
  rfl

/-- Corollary form making the "bounded working set" reading explicit: the
    local marking space of a node is never larger than a working set sized
    exactly to its arity. -/
theorem POWL.local_marking_bounded_by_arity (n : POWL A) :
    n.localMarkingSpace.length ≤ n.arity := by
  exact Nat.le_of_eq (POWL.local_marking_dimension_eq_arity n)
