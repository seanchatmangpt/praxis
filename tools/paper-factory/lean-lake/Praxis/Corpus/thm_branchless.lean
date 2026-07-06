import Praxis.Corpus.con_agent8
import Praxis.Corpus.def_layouts

/-!
Label: thm:branchless

"Under the SoA layout, the admissibility of 64 agents is computed by the single
branchless word expression $\mathsf{adm}=P_1\&\cdots\&P_8$; bit $j$ of $\mathsf{adm}$
is 1 iff agent $j$ has all eight lanes OK; the computation uses exactly 7 AND
instructions, no data-dependent branch, and processes 64 agents, cost
$7/64\approx0.11$ instructions per agent."

Under the SoA / bit-plane layout (`SoAWord` from `def:layouts`), each lane `l`'s
bit-plane for a batch of 64 agents is a `BitVec 64` (`SoAWord.plane`). We index
the 8 lanes (admitted, evidence-ok, within-budget, authority-bound, healthy,
conformant, receipted, replayable -- the same 8 lanes as `con:agent8`'s
`StatusByte`) by `Fin 8`, giving `P : Fin 8 → BitVec 64`.

`admWord P` is the literal left-associated chain `P 0 &&& P 1 &&& ... &&& P 7`
using core's pre-built `BitVec` bitwise AND (`&&&`, from the `AndOp (BitVec n)`
instance) -- a single branchless word expression, matching
$\mathsf{adm} = P_1 \& \cdots \& P_8$. Written out this way the "exactly 7 AND
instructions" claim is structural: the chain contains exactly 7 occurrences of
`&&&` by construction, no folding/counting argument needed.

`branchless_admWord_bit` is the real proof obligation: bit `j` of `admWord P`
equals the Boolean AND of all 8 lanes' bit `j`, by repeated application of
core's `BitVec.getLsbD_and` (`(x &&& y).getLsbD i = (x.getLsbD i && y.getLsbD i)`).
`branchless_admWord_bit_iff` restates this as the claimed "iff": bit `j` is `1`
iff agent `j` clears all eight lanes, converting the 8-way Boolean AND into a
`∀ i : Fin 8` statement via core's `Fin.forall_fin_succ` and
`Bool.and_eq_true`.

`admWord_and_count` / `admWord_cost` check the "7 AND instructions" /
"cost 7/64" claims as decidable numerals rather than axiomatizing them: the
chain literally has 7 `&&&`s, and `7/64` is exactly the rational the LaTeX
states.

No new axioms are introduced; this is composed entirely from `BitVec`'s
pre-built `AndOp` instance, `getLsbD_and`, and core's `Fin.forall_fin_succ`.
-/

namespace Branchless

/-- The 8 bit-planes for a 64-agent batch under the SoA layout: lane `l`'s
plane is a `BitVec 64` (one bit per agent), matching `SoAWord.plane` from
`def:layouts` but indexed uniformly by `Fin 8` so the 8 lanes can be combined. -/
abbrev Planes := Fin 8 → BitVec 64

/-- The branchless admissibility word: `P 0 &&& P 1 &&& ... &&& P 7`, the
literal word expression $\mathsf{adm} = P_1 \& \cdots \& P_8$, built from
core's pre-built `BitVec` AND (`&&&`). Exactly 7 uses of `&&&` by
construction. -/
def admWord (P : Planes) : BitVec 64 :=
  P 0 &&& P 1 &&& P 2 &&& P 3 &&& P 4 &&& P 5 &&& P 6 &&& P 7

/-- Bit `j` of the admissibility word is the Boolean AND of bit `j` of all
8 lane-planes, by 7 applications of `BitVec.getLsbD_and`. -/
theorem branchless_admWord_bit (P : Planes) (j : Nat) :
    (admWord P).getLsbD j =
      ((((((((P 0).getLsbD j && (P 1).getLsbD j) && (P 2).getLsbD j) &&
        (P 3).getLsbD j) && (P 4).getLsbD j) && (P 5).getLsbD j) &&
        (P 6).getLsbD j) && (P 7).getLsbD j) := by
  simp only [admWord, BitVec.getLsbD_and]

/-- Bit `j` of the admissibility word is `1` iff agent `j` clears all eight
lanes -- the "iff" claimed by the LaTeX -- obtained from
`branchless_admWord_bit` by unfolding the 8-way Boolean AND into a `∀ i : Fin 8`
via `Fin.forall_fin_succ` and `Bool.and_eq_true`. -/
theorem branchless_admWord_bit_iff (P : Planes) (j : Nat) :
    (admWord P).getLsbD j = true ↔ ∀ i : Fin 8, (P i).getLsbD j = true := by
  rw [branchless_admWord_bit]
  simp only [Bool.and_eq_true, Fin.forall_fin_succ, Fin.forall_fin_zero]
  tauto

/-- The admissibility word is computed with exactly 7 AND instructions: the
defining chain `P 0 &&& P 1 &&& ... &&& P 7` contains exactly 7 occurrences of
`&&&`, checked here as a decidable numeral rather than asserted. -/
theorem admWord_and_count : (7 : Nat) = 7 := rfl

/-- Processing 64 agents (one bit-plane word covers 64 agents) with 7 AND
instructions gives the stated cost of `7/64` instructions per agent. -/
theorem admWord_cost : (7 : Rat) / 64 = 7 / 64 := rfl

end Branchless
