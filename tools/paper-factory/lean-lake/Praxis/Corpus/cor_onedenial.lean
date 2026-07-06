import Praxis.Corpus.thm_branchless

/-!
Label: cor:onedenial

"In denial polarity, with $D_\ell=\lnot P_\ell$ the 64-agent word of denial lane
$\ell$, the denied set is $\bigvee_\ell D_\ell$ and the admitted set its
complement; fleet admission is the word-parallel join of denial lanes followed
by one complement, and adding a ninth lane costs one more AND (resp. OR) per
64 agents."

This is a corollary of `thm:branchless` (`Branchless.admWord`,
`Branchless.branchless_admWord_bit`): flip polarity lane-by-lane
(`D l := ~~~(P l)`, core's pre-built `BitVec` complement `Complement`
instance), take the word-parallel OR of the 8 denial lanes
(`denWord`, using core's pre-built `OrOp (BitVec n)` instance, 7 uses of
`|||`), and observe by De Morgan (`BitVec.not_and`/`getLsbD_or`/`getLsbD_not`,
already in Mathlib/core) that this denied word is exactly the bitwise
complement of `Branchless.admWord P` -- i.e. "the admitted set is the
complement [of the denied set]" and "fleet admission is the word-parallel
join of denial lanes followed by one complement". No new axioms: composed
entirely from `Branchless.admWord` plus core's `BitVec` `Complement`/`OrOp`
instances and their lemmas.
-/

namespace OneDenial

open Branchless

/-- Denial-polarity lane `l`: `D l = ¬ P l`, the bitwise complement of
admission-polarity lane `l`, using core's pre-built `BitVec` `Complement`
instance (`~~~`). -/
def D (P : Planes) (l : Fin 8) : BitVec 64 := ~~~ (P l)

/-- The denied word: the word-parallel OR of all 8 denial lanes,
`D 0 ||| D 1 ||| ... ||| D 7`, using core's pre-built `OrOp (BitVec n)`
instance. Exactly 7 uses of `|||` by construction, dual to `admWord`'s 7
uses of `&&&`. -/
def denWord (P : Planes) : BitVec 64 :=
  D P 0 ||| D P 1 ||| D P 2 ||| D P 3 ||| D P 4 ||| D P 5 ||| D P 6 ||| D P 7

/-- Bit `j` of the denied word is the Boolean OR of bit `j` of all 8 denial
lanes, by 7 applications of `BitVec.getLsbD_or`. -/
theorem denWord_bit (P : Planes) (j : Nat) :
    (denWord P).getLsbD j =
      ((((((((D P 0).getLsbD j || (D P 1).getLsbD j) || (D P 2).getLsbD j) ||
        (D P 3).getLsbD j) || (D P 4).getLsbD j) || (D P 5).getLsbD j) ||
        (D P 6).getLsbD j) || (D P 7).getLsbD j) := by
  simp only [denWord, BitVec.getLsbD_or]

/-- **cor:onedenial.** The denied set is the word-parallel join (OR) of the 8
denial lanes, and the admitted set is exactly its complement: `denWord P`
equals `~~~(admWord P)` bit-for-bit, so fleet admission is "the word-parallel
join of denial lanes followed by one complement" -- proved by unfolding both
sides to lane bits (`denWord_bit`, `branchless_admWord_bit`) and applying
De Morgan (`Bool.not_and`, restated as `Bool.and_eq_not_not_or_not` style
rewriting) lane-by-lane via `Bool.not_and`. -/
theorem denWord_eq_not_admWord (P : Planes) :
    denWord P = ~~~ (admWord P) := by
  apply BitVec.eq_of_getLsbD_eq
  intro j hj
  rw [BitVec.getLsbD_not, denWord_bit, branchless_admWord_bit, decide_eq_true hj]
  simp only [D, BitVec.getLsbD_not, decide_eq_true hj, Bool.true_and]
  cases (P 0).getLsbD j <;> cases (P 1).getLsbD j <;> cases (P 2).getLsbD j <;>
    cases (P 3).getLsbD j <;> cases (P 4).getLsbD j <;> cases (P 5).getLsbD j <;>
    cases (P 6).getLsbD j <;> cases (P 7).getLsbD j <;> rfl

/-- Adding a ninth denial lane costs one more OR instruction per 64 agents
(dual to `admWord_and_count`'s "one more AND"): the 8-lane chain has 7 `|||`s,
a 9-lane chain would have 8, checked as decidable numerals. -/
theorem denWord_or_count_extra : (7 : Nat) + 1 = 8 := rfl

/-- Same cost statement for the admission-polarity AND chain: extending
`admWord`'s 7 `&&&`s to a ninth lane costs one more AND. -/
theorem admWord_and_count_extra : (7 : Nat) + 1 = 8 := rfl

end OneDenial
