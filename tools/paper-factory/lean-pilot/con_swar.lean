/-
con:swar — SWAR (SIMD-within-a-register) zero-lane bitmask.

For a denial word `w : UInt64` of eight 8-bit lanes with
`Lhigh = 0x8080808080808080` and `Llow7 = 0x7f7f7f7f7f7f7f7f`, the
zero-lane bitmask is

  z(w) = ¬(((w &&& Llow7) + Llow7) ||| w) &&& Lhigh

Bit 7 of lane `j` in `z(w)` is 1 iff lane `j` in `w` is exactly `0x00`.

This reuses the fixed 8-lane, 64-bit word shape from `con:agent8`
(`Word64 = Fin 8 → Lane → Bool`, one 8-bit status byte per packed
agent) but here realizes the word concretely as a machine `UInt64` and
defines the classic branchless SWAR zero-byte test on it.

This is a *construction*: the only proof obligation is that the file
type-checks.
-/

/-- High bit of every one of the eight 8-bit lanes set. -/
def Lhigh : UInt64 := 0x8080808080808080

/-- Low seven bits of every one of the eight 8-bit lanes set. -/
def Llow7 : UInt64 := 0x7f7f7f7f7f7f7f7f

/-- The SWAR zero-lane bitmask: bit 7 of lane `j` is 1 iff lane `j` of
`w` is exactly `0x00`. -/
def zeroLaneMask (w : UInt64) : UInt64 :=
  Complement.complement (((w &&& Llow7) + Llow7) ||| w) &&& Lhigh

/-- Sanity check: the all-zero word has every high lane bit set. -/
example : zeroLaneMask 0 = Lhigh := by native_decide

/-- Sanity check: a word with no zero lanes (every lane `0x01`) has
zero-lane bitmask `0`. -/
example : zeroLaneMask 0x0101010101010101 = 0 := by native_decide
