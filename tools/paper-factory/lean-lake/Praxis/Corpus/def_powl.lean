import Mathlib.Order.Basic
import Mathlib.Data.List.Basic
import Mathlib.Data.Fin.Basic

/-!
# def:powl

A POWL model over an activity alphabet `A` is one of:
- an activity `a ∈ A` (a leaf);
- a partial order `({M₁,…,M_k}, ≺)` executed by respecting `≺` and running
  incomparable children concurrently;
- a choice graph `({M₁,…,M_k}, E)` combined by exclusive choice (and
  cyclic/loop edges), taking exactly one live branch.

We encode the ordering/edge relation on a node's children as a relation on
`Nat` positions into the (recursively defined) list of child models,
rather than on `Fin children.length` -- the kernel's nested-inductive
positivity check rejects a `Fin` index whose bound is itself the
inductive's own local `children` argument, since `Fin`'s parameter would
then contain a local variable of the datatype being defined. Indexing by
raw `Nat` (with out-of-range positions simply vacuous/ignored) sidesteps
that restriction while keeping the same mathematical content: a relation
on the positions of the children list. No Mathlib type already captures
this specific ternary process-tree shape (activity / ordered-parallel /
choice-with-cycles), so it is introduced here as a genuine new inductive
definition rather than an axiom -- the recursion itself is discharged
structurally via `List`, which Lean/Mathlib already knows is a positive
(and hence admissible) functor.
-/

inductive POWL (A : Type u) : Type u where
  /-- A leaf: a single activity from the alphabet. -/
  | activity : A → POWL A
  /-- A partial-order block: children executed respecting a precedence
  relation `≺` on their (list) positions, with incomparable children run
  concurrently. -/
  | partialOrder :
      (children : List (POWL A)) →
      (prec : Nat → Nat → Prop) →
      POWL A
  /-- A choice graph: children combined by exclusive choice, with an edge
  relation `E` on (list) positions (allowing cyclic/loop edges), of which
  exactly one live branch is taken. -/
  | choiceGraph :
      (children : List (POWL A)) →
      (edges : Nat → Nat → Prop) →
      POWL A
