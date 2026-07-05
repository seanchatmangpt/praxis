/-
def:powl — A POWL model over an activity alphabet `A` is one of:
  * an activity `a ∈ A` (a leaf);
  * a partial order `({M₁,…,M_k}, ≺)` executed by respecting `≺` and running
    incomparable children concurrently;
  * a choice graph `({M₁,…,M_k}, E)` combined by exclusive choice (and
    cyclic/loop edges), taking exactly one live branch.

Formalized in bare Lean 4 core (no mathlib) as an inductive family of process
trees over an activity alphabet `A`. The partial-order case is represented by
a list of children together with a precedence relation on their indices; the
choice-graph case by a list of children together with an edge relation on
their indices (edges may be cyclic, modelling loop edges).
-/

inductive POWL (A : Type u) where
  | leaf : A → POWL A
  | partialOrder : (children : List (POWL A)) →
      (prec : Nat → Nat → Prop) → POWL A
  | choice : (children : List (POWL A)) →
      (edge : Nat → Nat → Prop) → POWL A
