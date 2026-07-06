import Praxis.Corpus.def_reachcone

/-!
# prop:cone

Every marking `m` reachable from `m₀` satisfies the state equation for some
`x ∈ ℤ_{≥0}^{|T|}` (`reachSet`), hence lies in the integer points of
`m₀ + cone(N)` (`reachCone`) intersected with `ℤ^p`; the state equation is
necessary for reachability but not sufficient (the converse containment does
not hold in general).

Composition over new axioms: this is proved directly from `def:reachcone`'s
own `reachSet`/`reachCone`/`IsParikh`, using Mathlib's `RingHom.map_mulVec`
(cast commutes with matrix-vector product along the ring homomorphism
`Int.castRingHom ℝ`) together with `Int.cast_nonneg` (cast preserves the
`0 ≤ ·` order pointwise). No new axiom is declared.
-/

namespace Praxis.Corpus

variable (p ntrans : ℕ)

/-- The integer-to-real cast of a marking, coordinatewise. -/
def castMarking (m : Marking p) : Fin p → ℝ := fun i => (m i : ℝ)

/-- **prop:cone**: every reachable marking's real cast lies in the
translated cone `m₀ + cone(N)`. That is, the state equation
(witnessed by `reachSet`'s existential `x ≥ 0` with `m = m₀ + N x`) is
*necessary* for reachability: reachability implies membership in the cone. -/
theorem reachSet_subset_castMarking_reachCone
    (N : ReachMatrix p ntrans) (m0 : Marking p) :
    ∀ m ∈ reachSet p ntrans N m0, castMarking p m ∈ reachCone p ntrans N m0 := by
  rintro m ⟨x, hx, rfl⟩
  refine ⟨fun t => (x t : ℝ), ?_, ?_⟩
  · intro t
    show (0 : ℝ) ≤ (x t : ℝ)
    exact_mod_cast hx t
  · funext i
    have h := RingHom.map_mulVec (Int.castRingHom ℝ) N x i
    simp only [Int.coe_castRingHom] at h
    simpa [castMarking, Function.comp_def] using h

end Praxis.Corpus
