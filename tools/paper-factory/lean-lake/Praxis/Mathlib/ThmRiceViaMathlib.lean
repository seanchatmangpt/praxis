import Mathlib.Computability.Halting

/-!
thm:rice, reformalized in the Mathlib lane -- the strongest pilot swap in
this pass.

Bare-core version (`tools/paper-factory/lean-pilot/thm_rice.lean`)
axiomatizes an entire minimal computability layer from scratch (`Obs`,
`Sim`+3 equivalence axioms, `Halts`+undecidability axiom, `bot`, `smn`+2
axioms -- 9 axioms total) and then proves the halting-problem reduction
by hand, as a genuine tactic proof, ~35 lines.

Mathlib already contains a real, published, from-first-principles proof
of Rice's theorem over actual partial recursive functions
(`Mathlib.Computability.Halting`, formalizing Carneiro 2019): a universal
partial recursive function, a genuine `Nat.Partrec.Code` type for
program codes, and `ComputablePred.rice₂`, whose statement is the exact
shape of the corpus's own `thm:rice`, just phrased over `Code`/`eval`
(real programs and real partial-function evaluation) instead of an
abstract, axiomatized `Obs`/`Sim`.

Composing the corpus's own theorem statement as a direct corollary of
`ComputablePred.rice₂` needs ZERO new axioms: `Code`, `eval`, and the
undecidability proof are all pre-built, not re-derived.
-/

open Nat.Partrec (Code)
open Nat.Partrec.Code (eval)

/-- The corpus's `thm:rice` shape (a semantic, non-trivial property of
observations is undecidable), instantiated concretely with `Obs := Code`
(real program codes) and `Sim cf cg := eval cf = eval cg` (real
partial-function equivalence, not an axiomatized relation) -- a direct
corollary of Mathlib's `ComputablePred.rice₂`, not a re-proof. -/
theorem rice_corpus_shape (C : Set Code)
    (hsem : ∀ cf cg, eval cf = eval cg → (cf ∈ C ↔ cg ∈ C))
    (hnontrivial : C ≠ ∅ ∧ C ≠ Set.univ) :
    ¬ ComputablePred fun c => c ∈ C := by
  intro h
  rcases (ComputablePred.rice₂ C hsem).mp h with hempty | huniv
  · exact hnontrivial.1 hempty
  · exact hnontrivial.2 huniv

/-- The halting problem itself, for the same concrete `Code`/`eval`
model, is likewise Mathlib's own pre-built `ComputablePred.halting_problem`
-- cited directly, not axiomatized as `Halts_undecidable` was in the
bare-core version. -/
theorem halting_problem_corpus_shape (n : ℕ) :
    ¬ ComputablePred fun c => (eval c n).Dom :=
  ComputablePred.halting_problem n
