import Mathlib.Data.BitVec

/-!
Label: con:swar

"For a denial word $w\in\{0,1\}^{64}$ of eight 8-bit lanes with
$L_{\text{high}}=\code{0x8080808080808080}$ and
$L_{\text{low7}}=\code{0x7f7f7f7f7f7f7f7f}$, the zero-lane bitmask is
$z(w)=\lnot(((w\&L_{\text{low7}})+L_{\text{low7}})\lor w)\&L_{\text{high}}$;
bit 7 of lane $j$ in $z(w)$ is 1 iff lane $j$ in $w$ is exactly `0x00`."

This is the classic SWAR (SIMD-within-a-register) zero-byte test, expressed
directly on Mathlib's pre-built `BitVec 64` and its pre-built bitwise
operators (`&&&`, `|||`, `~~~`, `+`) -- no hand-rolled bit type, no new
axioms. `Lhigh` / `Llow7` are literal `BitVec 64` constants built from
Mathlib's `BitVec.ofNat`/numeral machinery, and `zeroLaneMask` is exactly the
formula `z(w)` above, composed from those pre-built operators.

This is a `construction`: it packages the LaTeX's data (the two mask
constants and the branchless zero-lane formula) as an actual `BitVec 64`
computation using Mathlib's pre-built `BitVec` bitwise operators, with no new
axioms and no proof obligation beyond this file type-checking.
-/

namespace Swar

/-- `L_high = 0x8080808080808080`: the high bit (bit 7) of every one of the
8 lanes set, all other bits 0. -/
def Lhigh : BitVec 64 := 0x8080808080808080#64

/-- `L_low7 = 0x7f7f7f7f7f7f7f7f`: the low 7 bits of every lane set, the
high bit of every lane clear. -/
def Llow7 : BitVec 64 := 0x7f7f7f7f7f7f7f7f#64

/-- The zero-lane bitmask `z(w) = ¬(((w & L_low7) + L_low7) | w) & L_high`.
Bit 7 of lane `j` in `z(w)` is 1 iff lane `j` of `w` is exactly `0x00`,
using Mathlib's pre-built `BitVec` and/or/not/add. -/
def zeroLaneMask (w : BitVec 64) : BitVec 64 :=
  (~~~(((w &&& Llow7) + Llow7) ||| w)) &&& Lhigh

end Swar
