import Praxis.Corpus.def_polytope

/-!
# def:conf

A trace with final marking `m†` and transition-count (Parikh) vector `x`
conforms to a net with initial marking `m₀` if `m† ∈ Poly` and the `x`
witnessing it is realized by a genuine firing ordering (i.e. `x` is a
nonnegative integer Parikh vector, `def:reachcone`'s `IsParikh`, with
`m† = m₀ + N x`, `def:reachcone`'s `reachSet` equation); it fails
membership if `m† ∉ Poly`.

Composition over new axioms: reuses `def:polytope`'s `markingPolytope`
(cast to `ℝ`) for the `Poly`-membership half, and `def:reachcone`'s
`IsParikh`/`ReachMatrix.mulVec` equation for the "genuine firing
ordering" half — both pre-built from earlier corpus definitions and
Mathlib's `Matrix`/order machinery. No new axiom is declared.
-/

namespace Praxis.Corpus

variable (p ntrans : ℕ)

/-- **def:conf**: a trace with final marking `mDagger` and Parikh vector
`x` conforms to the net `(N, m0)` iff `mDagger` (cast to `ℝ`) lies in the
marking polytope `Poly`, and `x` is a genuine nonnegative-integer firing
vector realizing `mDagger = m0 + N x`. It fails membership whenever the
real-cast marking is outside `Poly`. -/
def Conforms (N : ReachMatrix p ntrans) (m0 : Marking p)
    (mDagger : Marking p) (x : Fin ntrans → ℤ) : Prop :=
  (fun i => (mDagger i : ℝ)) ∈ markingPolytope p ntrans N m0 ∧
    IsParikh ntrans x ∧ mDagger = m0 + N.mulVec x

end Praxis.Corpus
