import Praxis.Corpus.def_mrr
import Mathlib.Data.Fintype.Pi
import Mathlib.Algebra.Order.BigOperators.Group.Finset

/-!
`thm:mrrsep` -- MRR separability under independent accounts.

If the accounts in `A` are independent -- i.e. the joint plan space factors as
one independent per-account target choice `A → T` rather than a single
monolithic `Plan` -- then the Maximum Reachable Revenue equals the sum, over
accounts, of each account's own maximum realized revenue over its lawful
target choices. This is exactly the reduction of the joint plan search
(exponential in the number of accounts, since a monolithic `Plan` ranges over
all of `Fintype.card T ^ |A|` joint assignments) to a linear number of
independent per-account maximizations (one `sup'` over `T` for each
`a ∈ Aset`), matching the informal statement's exponential-to-linear claim.

`T` plays the role of the (shared) type of lawful target stages, and `f a t`
is the revenue realized by account `a` when assigned target `t`
(`lawful_targets(a)` is implicitly `Finset.univ : Finset T`, i.e. every
target is available to weigh, with unlawful ones modelled as contributing
`-∞`/very negative `f a t`; no further generality is lost since Mathlib's
`Finset.sup'` over `Finset.univ` already covers "max over the (finite)
choice set").
-/

open Finset

/-- Independent-account separability of `MRR`: when the joint plan type is
the product `A → T` of independent per-account target choices, `MRR` over
that independent plan space equals the sum over accounts of each account's
own best realized revenue. -/
theorem MRR_separable {A T : Type*} [Fintype A] [DecidableEq A] [Fintype T] [Nonempty T]
    (Aset : Finset A) (f : A → T → ℝ) :
    MRR Aset (fun (p : A → T) (a : A) => f a (p a))
      = Aset.sum (fun a => Finset.univ.sup' Finset.univ_nonempty (f a)) := by
  unfold MRR
  apply le_antisymm
  · -- every joint plan's total is dominated by the sum of per-account maxima
    apply Finset.sup'_le
    intro p _
    apply Finset.sum_le_sum
    intro a _
    exact Finset.le_sup' (f a) (Finset.mem_univ (p a))
  · -- the per-account maxima are jointly achievable by one plan (independence)
    have hchoice : ∀ a : A, ∃ t : T, f a t = Finset.univ.sup' Finset.univ_nonempty (f a) := by
      intro a
      obtain ⟨t, _, ht⟩ := Finset.exists_mem_eq_sup' Finset.univ_nonempty (f a)
      exact ⟨t, ht.symm⟩
    choose g hg using hchoice
    calc Aset.sum (fun a => Finset.univ.sup' Finset.univ_nonempty (f a))
        = Aset.sum (fun a => f a (g a)) := by
          apply Finset.sum_congr rfl
          intro a _
          exact (hg a).symm
      _ ≤ Finset.univ.sup' Finset.univ_nonempty
            (fun p : A → T => Aset.sum (fun a => f a (p a))) :=
          Finset.le_sup' (fun p : A → T => Aset.sum (fun a => f a (p a))) (Finset.mem_univ g)
