import Praxis.Corpus.thm_kill

/-!
# `cor:diag` — Diagnostic content of `thm:kill`; survival certifies under-commitment

Under the Mutant Kill Theorem (`thm:kill`), the index of the killing stage certifies which
law-bearing invariant a mutation broke; a non-empty surviving set that is not equivalent-mutants
is a proof that the frame under-commits, the Faithful Projection converse.

We split this into the two halves the vocabulary of `def:staged`/`def:mut`/`thm:kill` already
supports:

1. *Diagnostic half* — if `m` is mutating at `s⋆` and `stg V m sStar = some i`, then stage `i`
   genuinely rejects `m(s⋆)` (`¬ V.inv i (m sStar)`): the reported stage index certifies exactly
   which invariant `Iᵢ` was violated. This is `thm:kill`'s clause `(2)`, restated here as the
   standalone corollary that gives the index its diagnostic reading.
2. *Converse half* — if `m(s⋆)` **survives** (`Kill V m sStar = false`, i.e. no stage rejects
   it), then `m(s⋆)` is accepted by every stage of `V` (`V.accepts (m sStar)`). A survivor is
   exactly a state the pipeline could not distinguish from a valid one; when `m` is genuinely a
   mutation (not an equivalent-mutant, i.e. semantically identical to `s⋆` for the properties
   `V` is meant to police), this acceptance is precisely the sense in which `V`'s invariants
   *under-commit*: they fail to pin down enough of `S` to reject a state that should have been
   rejected. `V.accepts (m sStar)` is the Faithful-Projection-converse witness: `m(s⋆)` projects
   faithfully through every stage of `V` even though (for a non-equivalent mutant) it should not.

Both halves are proved from the existing `Praxis.Corpus.DefStaged`/`Praxis.Corpus.DefMut`/
`Praxis.Corpus.ThmKill` machinery and core `List`/`Option`/`Bool` lemmas — no new axioms are
introduced.
-/

namespace Praxis.Corpus.CorDiag

open Praxis.Corpus.DefStaged
open Praxis.Corpus.DefMut
open Praxis.Corpus.ThmKill

variable {S : Type*} {k : ℕ}

/-- Diagnostic half of `cor:diag`: whenever `m` is mutating at `s⋆` and the reported stage is
`i`, stage `i` genuinely violates `m(s⋆)` — the index certifies which invariant broke. -/
theorem stage_certifies_violation (V : StagedValidator S k) (m : S → S) (sStar : S)
    (h : Mutating V m sStar) (i : Fin k) (hi : stg V m sStar = some i) :
    ¬ V.inv i (m sStar) :=
  (kill_iff_rejects_at_stg V m sStar h).2.1 i hi

/-- Converse half of `cor:diag`: if `m(s⋆)` survives `V` (`Kill V m sStar = false`, no stage
rejects it), then `m(s⋆)` is accepted by every stage of `V`. A surviving mutant is thus exactly a
state the frame's invariants fail to separate from a valid one — the Faithful Projection
converse witnessing that `V` under-commits whenever `m` is not an equivalent-mutant. -/
theorem survivor_accepted (V : StagedValidator S k) (m : S → S) (sStar : S)
    (hsurv : Kill V m sStar = false) :
    V.accepts (m sStar) := by
  unfold Kill stg StagedValidator.firstRejection at hsurv
  rw [Option.isSome_eq_false_iff, Option.isNone_iff_eq_none, List.find?_eq_none] at hsurv
  intro i
  have hnp := hsurv i (List.mem_finRange i)
  by_contra hcon
  have hpred : (! @decide (V.inv i (m sStar)) (V.dec i (m sStar))) = true := by
    rw [Bool.not_eq_true']
    exact @decide_eq_false _ (V.dec i (m sStar)) hcon
  simp [hpred] at hnp

end Praxis.Corpus.CorDiag
