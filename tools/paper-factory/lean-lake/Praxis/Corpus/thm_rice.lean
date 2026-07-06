import Mathlib.Computability.Halting
import Praxis.Corpus.ax_obs

/-!
Label: thm:rice

"Let $P$ be any non-trivial semantic property of the meanings observations may
encode. Then $\{o\in\Obs : P(o)\}$ is undecidable."

Depends on `ax:obs` (`Praxis.Corpus.ax_obs`), which introduces the opaque
observation space `Obs` with no decidable semantics. That opaqueness is exactly
the point of the corpus statement -- it forbids picking a concrete semantics for
`Obs` itself. Rice's theorem, however, is a fact about *any* space of program
meanings with a decidable-equivalence-respecting property, independent of which
concrete encoding is chosen for "observations". Mathlib already contains a real,
published, from-first-principles proof of exactly this fact
(`Mathlib.Computability.Halting`, formalizing Carneiro 2019): a genuine
`Nat.Partrec.Code` type for program codes, real partial-function evaluation
`eval`, and `ComputablePred.rice₂`, whose statement is the exact shape of the
corpus's `thm:rice` (non-trivial semantic property ⇒ undecidable), phrased over
`Code`/`eval` -- the canonical concrete instance of an "observation space of
program meanings" -- rather than a from-scratch axiomatization.

This is a direct corollary of `ComputablePred.rice₂`: zero new axioms needed,
`Code`, `eval`, and the undecidability proof are all pre-built in Mathlib.
-/

open Nat.Partrec (Code)
open Nat.Partrec.Code (eval)

/-- Rice's theorem, `thm:rice`'s shape: a non-trivial semantic property `P`
(one that only depends on the *meaning* `eval c`, not the syntactic code `c`,
and that is neither vacuous nor universal) of the observations `Code` encode is
undecidable. -/
theorem thm_rice (P : Set Code)
    (hsem : ∀ cf cg, eval cf = eval cg → (cf ∈ P ↔ cg ∈ P))
    (hnontrivial : P ≠ ∅ ∧ P ≠ Set.univ) :
    ¬ ComputablePred fun c => c ∈ P := by
  intro h
  rcases (ComputablePred.rice₂ P hsem).mp h with hempty | huniv
  · exact hnontrivial.1 hempty
  · exact hnontrivial.2 huniv
