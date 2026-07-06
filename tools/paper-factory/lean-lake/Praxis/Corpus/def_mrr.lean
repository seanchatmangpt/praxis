import Mathlib.Data.Finset.Lattice.Fold
import Mathlib.Data.Finset.BooleanAlgebra
import Mathlib.Algebra.BigOperators.Group.Finset.Basic
import Mathlib.Data.Real.Basic

/-!
`def:mrr` -- Maximum Reachable Revenue.

Given a finite set `A` of client accounts, a finite nonempty type `Plan` of
candidate plans (each plan selects, for every account, one of its
evidence-gated lawful target stages), and a `realized` function giving the
revenue an account contributes under a given plan, the Maximum Reachable
Revenue is the supremum over all plans of the total realized revenue summed
over the accounts.

We model this directly with Mathlib's `Finset.sup'` (max over a nonempty
finite index set), which is the standard Mathlib idiom for "max over a
finite search space of a real-valued objective" -- no bespoke max/argmax
machinery needed.
-/

open Finset

/-- Maximum Reachable Revenue: the greatest total realized revenue, over
accounts `Aset : Finset A`, achievable by any plan in the finite nonempty
plan space `Plan`. `realized plan a` is the revenue realized by account `a`
under `plan` (already incorporating the evidence-gated lawful target stage
that `plan` assigns to `a`). -/
def MRR {A : Type*} {Plan : Type*} [Fintype Plan] [Nonempty Plan]
    (Aset : Finset A) (realized : Plan → A → ℝ) : ℝ :=
  Finset.univ.sup' Finset.univ_nonempty (fun plan => Aset.sum (realized plan))
