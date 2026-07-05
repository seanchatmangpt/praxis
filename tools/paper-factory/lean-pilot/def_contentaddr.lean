/-
Label: def:contentaddr
Kind: definition

The address of a stored object `b` is `content_address(b) = chainH(b)` in hex;
two byte-identical objects share an address, any change yields a different address.

We model bytes as `List Bool` (bitstrings) and the hash function `chainH` as an
abstract deterministic function into a hex-string type (`List Char` restricted
to hex digits, represented here simply as `String`). `content_address` is then
defined as that hash applied to the object.
-/

/-- A stored object, represented as an arbitrary bitstring. -/
abbrev Bytes := List Bool

/-- An abstract deterministic hash function into a hex-encoded digest string.
    Concrete instantiations (e.g. BLAKE3) are assumed to satisfy this shape. -/
opaque chainH : Bytes → String

/-- The content address of a stored object `b` is the hex digest of `chainH b`. -/
def content_address (b : Bytes) : String := chainH b

/-- Byte-identical objects share an address (definitional determinism of `chainH`). -/
theorem content_address_deterministic (b : Bytes) :
    content_address b = content_address b := rfl

/-- Two objects with the same address are related by `chainH` agreeing on them
    (restatement of the address-sharing property for identical inputs). -/
theorem content_address_eq_of_bytes_eq {b₁ b₂ : Bytes} (h : b₁ = b₂) :
    content_address b₁ = content_address b₂ := by
  rw [h]
