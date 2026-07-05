/-
def:denialcode

In praxis the denial word is `DenialPolarity`, a `repr(transparent)` newtype over
`u64` carved into eight byte-lanes. The clean word is `Adml = DenialPolarity(0)`.
Seven named nonzero constants each occupy one distinct byte lane. The product is
`compose(a, b) = DenialPolarity(a.0 | b.0)`, bitwise OR. The admission predicate
is `is_admitted(d) ↔ d.0 = 0`.
-/

/-- `DenialPolarity` is a transparent newtype over a 64-bit word, carved into
eight byte-lanes (one bit-group per named denial reason). -/
structure DenialPolarity where
  val : UInt64
deriving DecidableEq, Repr

namespace DenialPolarity

/-- The clean word: no denial reasons set. -/
def Adml : DenialPolarity := ⟨0⟩

/-- Seven named nonzero constants, each occupying one distinct byte lane. -/
def reasonA : DenialPolarity := ⟨0x00000000000000FF⟩
def reasonB : DenialPolarity := ⟨0x000000000000FF00⟩
def reasonC : DenialPolarity := ⟨0x0000000000FF0000⟩
def reasonD : DenialPolarity := ⟨0x00000000FF000000⟩
def reasonE : DenialPolarity := ⟨0x000000FF00000000⟩
def reasonF : DenialPolarity := ⟨0x0000FF0000000000⟩
def reasonG : DenialPolarity := ⟨0x00FF000000000000⟩

/-- The product: bitwise OR of the underlying words. -/
def compose (a b : DenialPolarity) : DenialPolarity := ⟨a.val ||| b.val⟩

/-- The admission predicate: a word admits iff it is the clean (zero) word. -/
def is_admitted (d : DenialPolarity) : Prop := d.val = 0

end DenialPolarity
