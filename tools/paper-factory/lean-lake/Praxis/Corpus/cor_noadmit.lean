import Praxis.Corpus.thm_rice

/-!
Label: cor:noadmit

"There is no algorithm that admits an observation by deciding a non-trivial
semantic property of what it denotes; any total admission procedure decides a
syntactic, decidable surrogate instead."

Direct corollary of `thm_rice` (`Praxis.Corpus.thm_rice`): an "admission
procedure" is exactly a total (computable) decision procedure over the
observation encoding `Code`; "deciding a non-trivial semantic property" is
exactly `ComputablePred fun c => c ∈ P` for a semantic (`eval`-respecting),
non-trivial `P`. `thm_rice` already shows this is impossible. The corollary's
second clause ("decides a syntactic, decidable surrogate instead") is the
contrapositive reading: any predicate that *is* `ComputablePred` cannot be
semantic and non-trivial, i.e. it must fail `hsem` or triviality -- exactly a
syntactic surrogate rather than the semantic property itself. No new axioms:
this reuses `thm_rice`'s statement and Mathlib's `ComputablePred.rice₂` proof
verbatim.
-/

open Nat.Partrec (Code)
open Nat.Partrec.Code (eval)

/-- `cor:noadmit`: no total admission procedure can decide a non-trivial
semantic property `P` of what an observation (`Code`) denotes. -/
theorem cor_noadmit (P : Set Code)
    (hsem : ∀ cf cg, eval cf = eval cg → (cf ∈ P ↔ cg ∈ P))
    (hnontrivial : P ≠ ∅ ∧ P ≠ Set.univ) :
    ¬ ComputablePred fun c => c ∈ P :=
  thm_rice P hsem hnontrivial
