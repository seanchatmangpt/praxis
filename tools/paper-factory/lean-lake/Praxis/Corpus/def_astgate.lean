/-
Label: def:astgate
Kind: definition

An AST gate is a syntactic predicate `g : T → {0,1}` on an abstract syntax tree `T`,
computable in `O(|T|)` time; the retrofit applier gates every proposed change through
an ordered battery `(g₁,…,gₘ)`: the change is admitted iff all gates pass,
`⋀ᵢ gᵢ(T) = 1`; a failing gate returns a denial with the gate index and offending
AST node.

We model the AST gate as a computable `Bool`-valued predicate (the Lean-level
analogue of `{0,1}`) over an abstract, caller-supplied AST type `T`. Complexity
(`O(|T|)`) is a runtime property of the supplied implementation, not something
Lean's type system tracks, so it is left as a documented expectation on
`ASTGate.gate` rather than an axiom: no Mathlib structure captures "this
concrete function runs in linear time" as a type, and asserting it as an axiom
would not correspond to anything checkable in the kernel.
-/

namespace Praxis.Corpus

/-- An AST gate: a syntactic, computable predicate on an abstract syntax tree `T`. -/
structure ASTGate (T : Type) where
  /-- The gate predicate itself, `g : T → {0,1}` (represented as `Bool`). -/
  gate : T → Bool

/-- An ordered battery of gates `(g₁, …, gₘ)` through which every proposed change
is routed. -/
def GateBattery (T : Type) := List (ASTGate T)

/-- The change is admitted iff every gate in the battery passes:
`⋀ᵢ gᵢ(T) = 1`. -/
def GateBattery.admits {T : Type} (b : GateBattery T) (t : T) : Bool :=
  b.all (fun g => g.gate t)

/-- A denial identifies the index of the first failing gate together with the
offending AST node. -/
structure Denial (T : Type) where
  /-- Index of the failing gate in the battery. -/
  index : Nat
  /-- The AST node on which the gate failed. -/
  node : T

/-- If the battery does not admit `t`, produce the denial witnessing the first
failing gate (by position in the battery) and the offending node. `none` if
every gate passes. -/
def GateBattery.denial {T : Type} (b : GateBattery T) (t : T) : Option (Denial T) :=
  (b.findIdx? (fun g => !g.gate t)).map (fun i => ⟨i, t⟩)

end Praxis.Corpus
