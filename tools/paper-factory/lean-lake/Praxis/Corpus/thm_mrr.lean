import Mathlib.Data.Fintype.Pi
import Mathlib.Data.Fintype.Sets
import Mathlib.Data.Finset.Lattice.Fold
import Mathlib.Algebra.Order.BigOperators.Group.Finset
import Mathlib.Data.Real.Basic

/-!
# thm:mrr — Linear decomposition of joint plan optimization over independent accounts

Let `A` be a (finite) set of independent client accounts, `realized a t` the revenue of
account `a` under target stage `t`, and `T a := lawful_targets(a)` its evidence-gated,
nonempty, finite set of target stages. A *joint plan* is a choice, for every account
`a ∈ A`, of one of its lawful targets — formalized as an element of
`Fintype.piFinset (fun a : ↥A => T a)`, the Cartesian-product `Finset` over the finite
subtype `↥A` (finite via the standard `Fintype ↥A` instance attached to any `Finset A`).

The statement is:
`max over joint plans of (sum over a ∈ A of realized(a, plan a))
   = sum over a ∈ A of (max over t ∈ T a of realized(a, t))`

We formalize `max` over a nonempty finite set as Mathlib's `Finset.sup'`, and prove the
equality directly from two off-the-shelf `Finset.sup'` facts already in Mathlib:
`Finset.le_sup'` (a value at a witness is `≤` the sup) and `Finset.exists_mem_eq_sup'`
(the sup is attained at some witness) — no new order-theoretic machinery, no axioms.
Both sides are summed over `A.attach` (the standard finite-subtype indexing for a
`Finset`), which is the natural indexing for a per-account sum here; the informal
`Ω(∏_a |T_a|) → O(|A|·|T_a|)` complexity reduction quoted in the source is the reading
of this equality (the RHS is computed by `|A|` independent per-account maximizations
rather than a max over the exponentially large product space) and is not itself a
separate formal claim.
-/

namespace Praxis.Corpus.ThmMrr

open Finset

variable {ι : Type*} [DecidableEq ι] {A : Finset ι} {T : ι → Finset ℝ}

/-- **thm:mrr.** The joint plan optimization over independent accounts decomposes
linearly: the max (over all lawful joint plans, one target per account) of the summed
realized revenue equals the sum (over accounts `a ∈ A`) of the per-account maximal
realized revenue over `a`'s lawful target stages. -/
theorem mrr (hT : ∀ a ∈ A, (T a).Nonempty) (realized : ι → ℝ → ℝ) :
    (Fintype.piFinset (fun a : A => T (a : ι))).sup'
        (Fintype.piFinset_nonempty.mpr (fun a : A => hT a a.2))
        (fun g => ∑ a ∈ A.attach, realized (a : ι) (g a))
      = ∑ a ∈ A.attach, (T (a : ι)).sup' (hT a a.2) (realized a) := by
  have hcard : ∀ a : A, (T (a : ι)).Nonempty := fun a => hT a a.2
  refine le_antisymm ?_ ?_
  · -- Every joint plan's realized sum is bounded by the sum of per-account maxima:
    -- pointwise `realized a (g a) ≤ (T a).sup' (hcard a) (realized a)` since `g a ∈ T a`.
    refine Finset.sup'_le _ _ ?_
    intro g hg
    refine Finset.sum_le_sum ?_
    intro a _
    exact Finset.le_sup' (realized (a : ι)) (Fintype.mem_piFinset.mp hg a)
  · -- Build the maximizing joint plan by choosing, for each account, a witness
    -- attaining that account's per-account max, then evaluate the LHS there.
    choose b hb hbeq using fun a : A => Finset.exists_mem_eq_sup' (hcard a) (realized a)
    have hb_mem : b ∈ Fintype.piFinset (fun a : A => T (a : ι)) := Fintype.mem_piFinset.mpr hb
    calc ∑ a ∈ A.attach, (T (a : ι)).sup' (hT a a.2) (realized a)
        = ∑ a ∈ A.attach, realized (a : ι) (b a) := Finset.sum_congr rfl (fun a _ => hbeq a)
      _ ≤ (Fintype.piFinset (fun a : A => T (a : ι))).sup'
            (Fintype.piFinset_nonempty.mpr hcard)
            (fun g => ∑ a ∈ A.attach, realized (a : ι) (g a)) :=
          Finset.le_sup' _ hb_mem

end Praxis.Corpus.ThmMrr
