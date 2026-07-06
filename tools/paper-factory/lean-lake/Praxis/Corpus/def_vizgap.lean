import Praxis.Corpus.def_residual

/-!
def:vizgap

A visual gap report is the output of `measure_gap`: for each reconcilable dimension
`i ∈ {1,...,k}`, the residual `r_i = measured_i - midpoint(target_i)` and the dominant
dimension `i* = argmax_i |r_i|`, together with a rendered diff block; the report has
`k` residual values and one dominant index, size `O(k)` independent of the interior.

We reuse `Praxis.Corpus.DefResidual`'s `Vec`, `residual`, and `dominantDim` rather than
redefining the residual/argmax machinery. The report bundles: the residual vector (`k`
real values), the dominant index (one `Fin (n+1)`), and a rendered diff block. The
rendered diff block is an external, artifact-specific rendering artifact (e.g. a
formatted text/HTML diff) with no general mathematical content Mathlib could supply,
so it is modeled abstractly as an opaque type, matching the justification style of
`Praxis/Corpus/def_residual.lean`'s `RepairBand`.
-/

namespace Praxis.Corpus.DefVizgap

open Praxis.Corpus.DefResidual

/-- The rendered diff block: an external, artifact-specific rendering output (e.g. a
formatted text/HTML diff) with no general mathematical content Mathlib could supply, so
modeled abstractly rather than reused from Mathlib. -/
axiom DiffBlock : Type

/-- A visual gap report for `n + 1` reconcilable dimensions: the residual vector
(`k = n + 1` real values), the dominant dimension index, and a rendered diff block.
Size `O(k)`, independent of the interior, since it stores exactly the residual vector,
one index, and one diff block. -/
structure VizGap (n : ℕ) where
  /-- The residual vector `r_i = measured_i - midpoint(target_i)` for each dimension. -/
  residuals : Vec (n + 1)
  /-- The dominant dimension `i* = argmax_i |r_i|`. -/
  dominant : Fin (n + 1)
  /-- The rendered diff block accompanying the report. -/
  diff : DiffBlock

/-- Construct a visual gap report from `measure_gap`'s inputs: the measured and
target-midpoint vectors, and an externally-rendered diff block. Reuses `residual` and
`dominantDim` from `Praxis.Corpus.DefResidual` rather than recomputing the argmax by
hand. -/
noncomputable def measureGap {n : ℕ} (measured midpoint : Vec (n + 1)) (diff : DiffBlock) :
    VizGap n :=
  { residuals := residual measured midpoint
    dominant := dominantDim measured midpoint
    diff := diff }

end Praxis.Corpus.DefVizgap
