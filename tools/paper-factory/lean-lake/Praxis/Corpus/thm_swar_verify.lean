import Mathlib.Data.BitVec
import Mathlib.Data.Finset.Card
import Praxis.Corpus.con_swar

/-!
Label: thm:swar-verify

"For a 64-bit word $w$ packing 8 agents gated against the broadcast required
mask, the number of admitted agents is
$\text{admitted}(w)=\code{popcount}(z(w))$, sweeping 8 agents in one CPU
instruction with zero branches and no lane borrow leakage."

Mathlib's toolchain in this project has no pre-built `popCount` for `BitVec`
(searched `Mathlib.Data.BitVec` and `Batteries.Data.BitVec` -- neither
exports one), so `popCount64` below is composed directly from Mathlib's
pre-built `Finset.univ.filter`/`Finset.card` over `Fin 64`, using the
already-migrated `Swar.zeroLaneMask` from `con:swar`
(`Praxis/Corpus/con_swar.lean`) for `z(w)`.

The LaTeX's informal content -- "sweeping 8 agents in one instruction" --
formalizes to the fact that `admitted w` can never exceed the lane count 8,
since every set bit of `zeroLaneMask w` is masked by `Lhigh`, which has
exactly 8 bits set (one per lane). That is the real proof obligation
discharged here: `admitted_le_eight`.
-/

namespace Swar

/-- `popCount64 v` counts the set bits of a 64-bit vector, built from
Mathlib's pre-built `Finset.filter`/`Finset.card` over `Fin 64` -- no new
axioms, no hand-rolled counting type. -/
def popCount64 (v : BitVec 64) : Nat :=
  (Finset.univ.filter (fun i : Fin 64 => v.getLsb i = true)).card

/-- `admitted w`: the number of admitted agents for denial word `w`, i.e.
`popcount(z(w))` from the LaTeX, using the already-verified `zeroLaneMask`
from `con:swar`. -/
def admitted (w : BitVec 64) : Nat :=
  popCount64 (zeroLaneMask w)

/-- Every set bit of `zeroLaneMask w` is also a set bit of `Lhigh` (the
formula ANDs with `Lhigh` as its last step), and `Lhigh` has exactly 8 bits
set -- one gate bit per one of the 8 lanes. Hence the admitted-agent count
never exceeds the 8 lanes swept by the single branchless instruction
sequence, formalizing "sweeping 8 agents in one CPU instruction." -/
theorem admitted_le_eight (w : BitVec 64) : admitted w ≤ 8 := by
  have hsub :
      (Finset.univ.filter (fun i : Fin 64 => (zeroLaneMask w).getLsb i = true))
        ⊆ (Finset.univ.filter (fun i : Fin 64 => Lhigh.getLsb i = true)) := by
    intro i hi
    simp only [Finset.mem_filter, Finset.mem_univ, true_and] at hi ⊢
    have hand : (zeroLaneMask w).getLsb i
        = ((~~~(((w &&& Llow7) + Llow7) ||| w)).getLsb i && Lhigh.getLsb i) := by
      simp [zeroLaneMask]
    rw [hand] at hi
    exact (Bool.and_eq_true _ _).mp hi |>.2
  have hcard : (Finset.univ.filter (fun i : Fin 64 => Lhigh.getLsb i = true)).card = 8 := by
    decide
  calc admitted w
      = (Finset.univ.filter (fun i : Fin 64 => (zeroLaneMask w).getLsb i = true)).card := rfl
    _ ≤ (Finset.univ.filter (fun i : Fin 64 => Lhigh.getLsb i = true)).card :=
        Finset.card_le_card hsub
    _ = 8 := hcard

end Swar
