import Praxis.Corpus.def_reachcone
import Mathlib.Data.Matrix.Basic
import Mathlib.Data.Matrix.Mul
import Mathlib.Algebra.Order.Pi

/-!
prop:conformembership

A trace with Parikh vector `x` is conformant only if its marking lies in
`P = {m₀ + N x : x ≥ 0} ∩ {m ≥ 0}`. If `m† ∉ P`, Farkas' lemma yields `y` with
`yᵀN ≤ 0` and `yᵀ(m† - m₀) > 0`: a certificate of nonconformance.

## Formalization note

This proposition packages two directions:

1. *Necessity* (`conformant → marking ∈ P`): immediate from the definition of
   `reachSet`/`reachCone` (`Praxis.Corpus.def_reachcone`) once we intersect
   with the nonnegativity constraint on markings — proved below as
   `conformant_mem_relaxedCone`.

2. *Certificate soundness* (existence of `y` with `yᵀN ≤ 0`,
   `yᵀ(m† - m₀) > 0` implies `m† ∉ P`): this is the operationally useful half
   of Farkas' lemma (checking a *given* certificate refutes membership), and
   is proved below as `nonconformance_of_certificate` by direct linear
   algebra (`Matrix.dotProduct`/`mulVec`), composing only pre-built Mathlib
   arithmetic — no new axiom.

   The *converse* existence claim — that such a `y` always exists whenever
   `m† ∉ P` — is the hard (Minkowski–Weyl) direction of finite-dimensional
   Farkas duality. Mathlib's only proved Farkas lemma,
   `ProperCone.hyperplane_separation_point`
   (`Mathlib/Analysis/Convex/Cone/Dual.lean`), separates a point from an
   *already-closed* proper cone; it does not itself establish that a
   finitely-generated cone `{N c : c ≥ 0}` is closed in finite dimension.
   That equivalence (`PointedCone.FG` cones are closed / dually finitely
   generated) is explicitly flagged as the Minkowski–Weyl theorem in
   Mathlib's own `Mathlib/Geometry/Convex/Cone/DualFinite.lean` docstring
   and is not yet proved there (only the converse `DualFG → FG` direction
   is). Composing the existence direction from Mathlib today would require
   reproving Minkowski–Weyl from scratch, which is out of scope for a
   single-statement migration; we therefore formalize the direction that
   *is* composable from present Mathlib content and record this gap
   honestly rather than axiomatizing the existential.
-/

namespace Praxis.Corpus

open scoped Matrix

variable {p ntrans : ℕ}

/-- The feasibility set `P = {m₀ + N x : x ≥ 0} ∩ {m ≥ 0}` from real
(rational-relaxed) Parikh vectors `x`, as a subset of `Fin p → ℝ`. -/
def relaxedCone (N : ReachMatrix p ntrans) (m0 : Marking p) : Set (Fin p → ℝ) :=
  reachCone p ntrans N m0 ∩ {m | 0 ≤ m}

/-- **Necessity**: a marking reached (over `ℝ`, relaxing integrality) by a
nonnegative combination `x ≥ 0` with `m = m₀ + N x ≥ 0` lies in `P`. This is
immediate from the definitions in `def:reachcone`. -/
theorem conformant_mem_relaxedCone (N : ReachMatrix p ntrans) (m0 : Marking p)
    (x : Fin ntrans → ℝ) (hx : 0 ≤ x)
    (m : Fin p → ℝ) (hm : m = (fun i => (m0 i : ℝ)) + (N.map (fun z : ℤ => (z : ℝ))).mulVec x)
    (hnonneg : 0 ≤ m) :
    m ∈ relaxedCone N m0 :=
  ⟨⟨x, hx, hm⟩, hnonneg⟩

/-- **Certificate soundness**: if `y` satisfies `yᵀ N ≤ 0` (i.e. `Nᵀ y ≤ 0`,
`y` nonincreasing along every column direction) and `yᵀ (m† - m₀) > 0`, then
`m†` cannot lie in the nonnegative cone `{m₀ + N c : c ≥ 0}` (a fortiori not
in `P ⊆` that cone). This is the half of Farkas' lemma used to *check* a
nonconformance certificate, proved directly from `Matrix.dotProduct` and
`Matrix.mulVec` algebra (no separating-hyperplane machinery needed for this
direction). -/
theorem nonconformance_of_certificate (N : ReachMatrix p ntrans) (m0 mdag : Marking p)
    (y : Fin p → ℝ)
    (hyN : (Nᵀ).map (fun z : ℤ => (z : ℝ)) *ᵥ y ≤ 0)
    (hgap : 0 < Matrix.dotProduct y (fun i => (mdag i : ℝ) - (m0 i : ℝ))) :
    (fun i => (mdag i : ℝ)) ∉ reachCone p ntrans N m0 := by
  rintro ⟨c, hc, hmdag⟩
  have hcert :
      Matrix.dotProduct y (fun i => (mdag i : ℝ) - (m0 i : ℝ))
        = Matrix.dotProduct ((N.map (fun z : ℤ => (z : ℝ)))ᵀ *ᵥ y) c := by
    have h1 : (fun i => (mdag i : ℝ) - (m0 i : ℝ))
        = (N.map (fun z : ℤ => (z : ℝ))).mulVec c := by
      funext i; simp [hmdag]
    rw [h1, Matrix.dotProduct_mulVec, Matrix.mulVec_transpose]
  have htrans : (Nᵀ).map (fun z : ℤ => (z : ℝ)) = (N.map (fun z : ℤ => (z : ℝ)))ᵀ := by
    ext i j; simp [Matrix.transpose_apply, Matrix.map_apply]
  rw [htrans] at hyN
  have hle : Matrix.dotProduct ((N.map (fun z : ℤ => (z : ℝ)))ᵀ *ᵥ y) c ≤ 0 := by
    have := Matrix.dotProduct_le_dotProduct_of_nonneg_right hyN hc
    simpa using this
  have : Matrix.dotProduct y (fun i => (mdag i : ℝ) - (m0 i : ℝ)) ≤ 0 := hcert ▸ hle
  exact absurd this (not_le.mpr hgap)

end Praxis.Corpus
