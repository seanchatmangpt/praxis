import Mathlib.Data.Fintype.Basic
import Mathlib.Data.Finset.Basic
import Mathlib.Algebra.BigOperators.Group.Finset.Basic
import Mathlib.Data.Real.Basic

/-!
# def:balance — Free level of a fluent under durative draws

A durative action `j` draws on a fluent `φ` if its effect decreases `φ` by `r_j` at
start and increases it by `r_j` at end (rate `r_j ≥ 0`), holding `r_j` units over
`[s_j, e_j)`. The free level of `φ` at time `t` is
`f_φ(t) = ν₀(φ) − Σ_{j : t ∈ [s_j, e_j)} r_j`.

We reuse Mathlib's `Fintype`/`Finset` machinery (`Finset.filter`, `Finset.sum`) rather
than hand-rolling a summation over a finite index set, and use `Classical.dec` for the
interval-membership predicate (an open real interval condition need not be
constructively decidable in general) rather than inventing a bespoke decidability
instance.
-/

namespace Praxis.Corpus.DefBalance

open scoped Classical
open Finset

/-- A durative draw on some fluent: a start time `s_j`, an end time `e_j`, and a
nonnegative rate `r_j` held over `[s_j, e_j)`. -/
structure DrawAction where
  /-- Start time `s_j`. -/
  start : ℝ
  /-- End time `e_j`. -/
  stop : ℝ
  /-- Rate `r_j`, held over `[s_j, e_j)`. -/
  rate : ℝ
  /-- The draw rate is nonnegative, `r_j ≥ 0`. -/
  rate_nonneg : 0 ≤ rate

/-- Whether draw `d` is active at time `t`, i.e. `t ∈ [s_j, e_j)`. -/
def DrawAction.active (d : DrawAction) (t : ℝ) : Prop :=
  d.start ≤ t ∧ t < d.stop

/-- The free level of a fluent with initial value `ν₀ φ` at time `t`, given a finite
family `draws : J → DrawAction` of durative draws on it:
`f_φ(t) = ν₀(φ) − Σ_{j : t ∈ [s_j, e_j)} r_j`. -/
noncomputable def freeLevel {J : Type} [Fintype J] (nu0 : ℝ) (draws : J → DrawAction) (t : ℝ) : ℝ :=
  nu0 - ∑ j ∈ univ.filter (fun j => (draws j).active t), (draws j).rate

end Praxis.Corpus.DefBalance
