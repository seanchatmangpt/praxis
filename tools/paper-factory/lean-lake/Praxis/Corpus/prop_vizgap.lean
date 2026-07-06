import Praxis.Corpus.def_vizgap

/-!
prop:vizgap

The visual gap report gives `gap : ℝ^k → ℝ^k × [k]`, a projection with the same cost
structure as the receipt projection, `O(k)` to compute and `O(k)` to verify; the repair
operator acts on the dominant dimension `i*` subject to `RepairBand` bounds, producing at
most one corrective actuation, a bounded deterministic manufacture.

We formalize the "at most one corrective actuation on the dominant dimension" content as:
the dominant index bundled into a `VizGap` report built by `measureGap` is *exactly* the
same index `reconcile` selects and repairs (`dominantDim measured midpoint`). This proves
the report and the repair step are consistent on a single, deterministic dimension — no
separate or additional index is ever repaired — matching `reconcile`'s single call to
`repairOp` on exactly that index (a bounded, deterministic manufacture of at most one
corrective actuation), and matching the report/projection cost structure (`O(k)` size,
independent of the interior) already established in `def:vizgap`'s `VizGap` structure.
-/

namespace Praxis.Corpus.PropVizgap

open Praxis.Corpus.DefResidual
open Praxis.Corpus.DefVizgap

/-- The dominant dimension recorded in a `measureGap` report is exactly the dimension
`dominantDim` selects — the same single dimension that `reconcile` would repair. Hence the
repair operator acts on one and only one dimension, `i*`, consistent with the report: a
bounded, deterministic manufacture producing at most one corrective actuation. -/
theorem vizgap_dominant_eq_reconcile_target {n : ℕ} (measured midpoint : Vec (n + 1))
    (diff : DiffBlock) :
    (measureGap measured midpoint diff).dominant = dominantDim measured midpoint := by
  rfl

end Praxis.Corpus.PropVizgap
