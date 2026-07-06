import Mathlib.Data.Fin.Basic
import Mathlib.Analysis.SpecialFunctions.Pow.Real
import Mathlib.Data.Finset.Lattice.Fold

/-!
def:residual

For environmental observations `O` and target artifact `A`, the measurement `μop(O)`
returns a residual vector `R ∈ ℝ^k` with `r_i = measured_i - midpoint(target_i)`;
reconciliation selects the dominant dimension `argmax_i |r_i|` and applies a repair
operator subject to `RepairBand` limits.

We model the residual vector as a function `Fin k → ℝ` (reusing core's function-space
representation of finite-dimensional real vectors rather than inventing a bespoke vector
type). `measured` and `midpoint` are likewise `Fin k → ℝ`. The dominant-dimension
selection reuses Mathlib's `Finset.exists_max_image` machinery already available on
`Fin k` (a `Fintype`), applied to the nonempty `Finset.univ`, instead of hand-rolling an
argmax search. The concrete measurement/target-midpoint functions (`μop`, `target`) and
the `RepairBand`-constrained repair operator are external, artifact-specific processes
with no general mathematical content Mathlib could supply, so they remain axiomatized
(as opaque functions), matching the justification style of
`Praxis/Corpus/def_sandbox.lean`.
-/

namespace Praxis.Corpus.DefResidual

/-- A real-valued vector indexed by `Fin k`, reusing core's function-space representation
of finite-dimensional real vectors rather than inventing a new vector type. -/
abbrev Vec (k : ℕ) := Fin k → ℝ

/-- The residual vector: componentwise difference between the measured value and the
midpoint of the target band, `r_i = measured_i - midpoint(target_i)`. -/
def residual {k : ℕ} (measured midpoint : Vec k) : Vec k :=
  fun i => measured i - midpoint i

/-- A repair band: the allowed correction limits for a single residual dimension,
external artifact-specific tolerance data with no general mathematical content, so
modeled abstractly rather than reused from Mathlib. -/
axiom RepairBand : Type

/-- The external repair operator, constrained by a `RepairBand`, that maps a chosen
residual dimension and its value to a corrective action. Axiomatized: this represents a
concrete artifact-repair process (e.g. a physical or software correction procedure), not
a mathematical predicate Mathlib could supply. -/
axiom repairOp {k : ℕ} : RepairBand → Fin k → ℝ → Unit

/-- The dominant residual dimension: the index `i` maximizing `|r_i|`, obtained via
Mathlib's `Finset.exists_max_image` over the nonempty `Finset.univ : Finset (Fin (n+1))`
(nonempty by construction, indexing the vector by `n+1` rather than an arbitrary `k` so
nonemptiness holds definitionally), rather than a hand-rolled argmax search. -/
noncomputable def dominantDim {n : ℕ} (measured midpoint : Vec (n + 1)) : Fin (n + 1) :=
  (Finset.exists_max_image (Finset.univ : Finset (Fin (n + 1)))
      (fun i => |residual measured midpoint i|)
      ⟨(0 : Fin (n + 1)), Finset.mem_univ _⟩).choose

/-- Reconciliation: select the dominant residual dimension and apply the repair operator
(under the given `RepairBand`) to that dimension's residual value. -/
noncomputable def reconcile {n : ℕ} (band : RepairBand) (measured midpoint : Vec (n + 1)) :
    Unit :=
  let i := dominantDim measured midpoint
  repairOp band i (residual measured midpoint i)

end Praxis.Corpus.DefResidual
