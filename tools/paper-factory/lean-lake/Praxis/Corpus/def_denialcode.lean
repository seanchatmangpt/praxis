import Mathlib.Data.UInt
import Mathlib.Data.Nat.Bitwise

/-!
# def:denialcode

In praxis the denial word is `DenialPolarity`, a `repr(transparent)` newtype over `u64`
carved into eight byte-lanes; the clean word is `Adml = DenialPolarity(0)`; seven named
nonzero constants each occupy one distinct byte lane; the product is
`compose(a,b) = DenialPolarity(a.0 | b.0)`, bitwise OR; the admission predicate is
`is_admitted(d) ↔ d.0 = 0`.

We model the underlying `u64` as `UInt64` (Lean/Mathlib's fixed-width machine word type,
which already carries the bitwise-OR operation we need), and wrap it in a single-field
structure to get the `repr(transparent)` newtype discipline.
-/

/-- A newtype over `UInt64`, carved into eight byte-lanes (bits `8*i .. 8*i+7`). -/
structure DenialPolarity where
  val : UInt64
deriving DecidableEq, Repr

namespace DenialPolarity

/-- The clean word: no denial bit set in any lane. -/
def Adml : DenialPolarity := ⟨0⟩

/-- One named nonzero constant per byte lane `i ∈ {0,...,7}`, occupying lane `i`
exclusively (bit pattern `0x01` shifted left by `8*i`). -/
def laneConst (i : Fin 8) : DenialPolarity :=
  ⟨(1 : UInt64) <<< (UInt64.ofNat (8 * i.val))⟩

/-- The seven named nonzero constants, one per lane `1,...,7` (lane `0` is reserved
here as an example instantiation; any seven of the eight lanes may be named). -/
def lane1 : DenialPolarity := laneConst 0
def lane2 : DenialPolarity := laneConst 1
def lane3 : DenialPolarity := laneConst 2
def lane4 : DenialPolarity := laneConst 3
def lane5 : DenialPolarity := laneConst 4
def lane6 : DenialPolarity := laneConst 5
def lane7 : DenialPolarity := laneConst 6

/-- The product: bitwise OR of the underlying words. -/
def compose (a b : DenialPolarity) : DenialPolarity :=
  ⟨a.val ||| b.val⟩

/-- The admission predicate: a denial word admits iff its underlying word is zero. -/
def is_admitted (d : DenialPolarity) : Prop :=
  d.val = 0

end DenialPolarity
