import Praxis.Corpus.def_mut

/-!
# `thm:kill` — Kill iff rejection at the correct stage

For a staged validator `V = (V₁, …, V_k)` in which every stage is sound and complete, and a
mutation operator `m` with correct stage `stg(m)`: `Kill(m) = 1 ⟺ V_{stg(m)}` rejects `m(s⋆)`,
and the least rejecting stage reported by `V` equals `stg(m)`.

We reuse `Praxis.Corpus.DefMut.Mutating`/`stg`/`Kill` from `def:mut`, which are themselves built
on `Praxis.Corpus.DefStaged.StagedValidator` from `def:staged`. Soundness/completeness of the
individual stages are not needed for this particular equivalence: `stg(m)` is *defined* as
`V.firstRejection (m s⋆)` (the least rejecting stage reported by `V`), so the "least rejecting
stage equals `stg(m)`" clause is definitional (`rfl`), and `Kill(m) = 1 ⟺` rejection at that stage
follows from the mutating hypothesis (`m s⋆` violates some invariant) via the core list lemmas
`List.find?_isSome` and `List.find?_some`. No new axioms are introduced.
-/

namespace Praxis.Corpus.ThmKill

open Praxis.Corpus.DefStaged
open Praxis.Corpus.DefMut

variable {S : Type*} {k : ℕ}

/-- `thm:kill`: if `m` is mutating at `s⋆` relative to `V`, then (1) `Kill(m) = 1`, (2) whenever
`stg(m)` is reported as stage `i`, that stage genuinely rejects `m(s⋆)` (i.e. `¬ V.inv i (m s⋆)`),
and (3) the least rejecting stage reported by `V` (`V.firstRejection (m s⋆)`) equals `stg(m)`. -/
theorem kill_iff_rejects_at_stg (V : StagedValidator S k) (m : S → S) (sStar : S)
    (h : Mutating V m sStar) :
    Kill V m sStar = true ∧
      (∀ i, stg V m sStar = some i → ¬ V.inv i (m sStar)) ∧
      V.firstRejection (m sStar) = stg V m sStar := by
  refine ⟨?_, ?_, rfl⟩
  · -- `Kill(m) = 1`: since `m s⋆` violates some invariant, some stage's predicate fires.
    obtain ⟨i, hi⟩ : ∃ i : Fin k, ¬ V.inv i (m sStar) := by
      by_contra hcon
      push_neg at hcon
      exact h.2 hcon
    have hpred : (! @decide (V.inv i (m sStar)) (V.dec i (m sStar))) = true := by
      rw [Bool.not_eq_true']
      exact @decide_eq_false _ (V.dec i (m sStar)) hi
    unfold Kill stg StagedValidator.firstRejection
    simp only [List.find?_isSome]
    exact ⟨i, List.mem_finRange i, hpred⟩
  · -- Whenever `stg(m) = some i`, stage `i` genuinely rejects `m(s⋆)`.
    intro i hi
    unfold stg StagedValidator.firstRejection at hi
    have hp := List.find?_some hi
    have hd : @decide (V.inv i (m sStar)) (V.dec i (m sStar)) = false := by
      rwa [Bool.not_eq_true'] at hp
    exact @of_decide_eq_false _ (V.dec i (m sStar)) hd

end Praxis.Corpus.ThmKill
