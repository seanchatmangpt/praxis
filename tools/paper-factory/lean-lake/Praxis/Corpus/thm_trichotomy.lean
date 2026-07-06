import Praxis.Corpus.def_queryresult

/-!
# thm:trichotomy — Answered / Denied / Invalid trichotomy

For a query `q` evaluated against an admitted kernel, exactly one of `Answered` (a
positive proof DAG with fan-in ≤ 8 plus receipt), `Denied` (a negative proof plus
receipt), or `Invalid` (a `RejectionCode`, no proof) holds.

`QueryResult` was migrated (`def:queryresult`) as a plain 3-constructor inductive type,
so "exactly one of the three shapes holds" is exactly the exhaustiveness +
mutual-exclusivity of that inductive's constructors — no external Mathlib lemma or
fresh axiom is needed; `cases`/pattern-matching on the inductive discharges it
directly, matching the `ThmRiceViaMathlib.lean` precedent of proving the corpus
theorem as a direct corollary of already-migrated structure rather than axiomatizing
the conclusion. The bound `≤ 8` and the "positive/negative" reading of proof polarity
are exactly the `bound`/`positive` fields already carried in `DefLogicAdm`/
`DefQueryResult`, so no new vocabulary is introduced here.
-/

namespace Praxis.Corpus.ThmTrichotomy

open Praxis.Corpus.DefQueryResult

/-- Exactly one of `Answered`, `Denied`, `Invalid` holds of any `QueryResult`: the
three disjuncts are jointly exhaustive (every result is one of the three shapes) and
pairwise exclusive (no result is two shapes at once). -/
theorem trichotomy {ProofNode Receipt : Type} (r : QueryResult ProofNode Receipt) :
    ((∃ ds, r = QueryResult.Answered ds) ∨
      (∃ d, r = QueryResult.Denied d) ∨
      (∃ c, r = QueryResult.Invalid c)) ∧
    (¬ ((∃ ds, r = QueryResult.Answered ds) ∧ (∃ d, r = QueryResult.Denied d))) ∧
    (¬ ((∃ ds, r = QueryResult.Answered ds) ∧ (∃ c, r = QueryResult.Invalid c))) ∧
    (¬ ((∃ d, r = QueryResult.Denied d) ∧ (∃ c, r = QueryResult.Invalid c))) := by
  refine ⟨?_, ?_, ?_, ?_⟩
  · cases r with
    | Answered ds => exact Or.inl ⟨ds, rfl⟩
    | Denied d => exact Or.inr (Or.inl ⟨d, rfl⟩)
    | Invalid c => exact Or.inr (Or.inr ⟨c, rfl⟩)
  · rintro ⟨⟨ds, hA⟩, ⟨d, hD⟩⟩; rw [hA] at hD; exact absurd hD (by simp)
  · rintro ⟨⟨ds, hA⟩, ⟨c, hI⟩⟩; rw [hA] at hI; exact absurd hI (by simp)
  · rintro ⟨⟨d, hD⟩, ⟨c, hI⟩⟩; rw [hD] at hI; exact absurd hI (by simp)

end Praxis.Corpus.ThmTrichotomy
