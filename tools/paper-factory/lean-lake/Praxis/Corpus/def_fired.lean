import Praxis.Corpus.def_denialcode
import Mathlib.Data.UInt
import Mathlib.Data.Nat.Bitwise

/-!
# def:fired

`firedmap : Deny → {0,1}^8` sends a word `d` to the bit vector whose bit `j` equals the
nonzero-indicator of byte lane `j` of `d`:
`firedmap(d)_j = 1[(d.0 >> 8j) & 0xFF ≠ 0]`,
computed branchlessly via `1[x ≠ 0] = (x | (-x)) >> 63` on each lane.

We model `{0,1}^8` as `BitVec 8` (Mathlib/core's fixed-width bit-vector type), and compute
each output bit `j` from the `UInt64` word underlying `DenialPolarity` by shifting lane `j`
into the low byte, masking with `0xFF`, and then applying the branchless nonzero-indicator
`x ↦ (x ||| (-x)) >>> 63` to the (sign-extended-as-unsigned) shifted value before testing the
result against `0`. This is a direct transcription of the branchless formula in the statement;
no proof obligation attaches to a `definition`.
-/

namespace DenialPolarity

/-- Branchless nonzero-indicator on a `UInt64`: `1[x ≠ 0] = (x | (-x)) >>> 63`, which is `1`
whenever `x ≠ 0` and `0` when `x = 0`, matching the statement's specified computation. -/
def branchlessNonzeroIndicator (x : UInt64) : UInt64 :=
  (x ||| (0 - x)) >>> 63

/-- Bit `j` of `firedmap d`: the nonzero-indicator of byte lane `j` of `d.val`, i.e.
`1[(d.0 >> 8j) & 0xFF ≠ 0]`, computed via the branchless formula above. -/
def firedBit (d : DenialPolarity) (j : Fin 8) : Bool :=
  let lane : UInt64 := (d.val >>> (UInt64.ofNat (8 * j.val))) &&& 0xFF
  branchlessNonzeroIndicator lane = 1

/-- `firedmap : Deny → {0,1}^8`, sending `d` to the bit vector whose bit `j` is
`firedBit d j`, the nonzero-indicator of byte lane `j` of `d`. Packed as a `Nat` (sum of
`2^j` over the set bits) and reinterpreted as a `BitVec 8`. -/
def firedmap (d : DenialPolarity) : BitVec 8 :=
  BitVec.ofNat 8 ((List.finRange 8).foldl
    (fun acc j => if firedBit d j then acc + 2 ^ j.val else acc) 0)

end DenialPolarity
