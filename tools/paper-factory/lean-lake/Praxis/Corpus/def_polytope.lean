import Praxis.Corpus.def_reachcone

/-!
# def:polytope

The marking polytope of a net is the relaxation
`Poly = {m₀ + N x : x ≥ 0} ∩ {m ≥ 0} ⊆ ℝ₊^p`, the continuous translated
cone whose integer points over-approximate the reachable set.

Composition over new axioms: reuses `def:reachcone`'s own `reachCone`
(the translated real cone `{m₀ + N c : c ≥ 0}`) and simply intersects it
with the nonnegative orthant `{v : Fin p → ℝ | 0 ≤ v}`, both pre-built
`Set` operations on Mathlib's `Matrix`/pointwise-order machinery. No new
axiom is declared.
-/

namespace Praxis.Corpus

variable (p ntrans : ℕ)

/-- **def:polytope**: the marking polytope `Poly = (m₀ + cone(N)) ∩ ℝ₊^p`,
the relaxation of the reachable set to nonnegative real combinations of
the net's translated cone, intersected with the nonnegative orthant. -/
def markingPolytope (N : ReachMatrix p ntrans) (m0 : Marking p) : Set (Fin p → ℝ) :=
  reachCone p ntrans N m0 ∩ { v : Fin p → ℝ | 0 ≤ v }

end Praxis.Corpus
