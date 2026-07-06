import Mathlib.Data.BitVec

/-!
def:contentaddr, reformalized in the Mathlib lane.

"The address of a stored object $b$ is
$\code{content\_address}(b)=\chainH(b)$ in hex; two byte-identical objects
share an address, any change yields a different address."

Composed from pre-built pieces rather than fresh axioms wherever possible:

* A stored object is a byte string, modeled by core Lean's `ByteArray`
  (`Init.Data.ByteArray`), not a new opaque type.
* `Digest := BitVec 256`, the same real 256-bit vector type used by
  `Praxis.Mathlib.DefReceipt` (`def:receipt`) for hash outputs -- reused
  here rather than re-declared, since it is the same concept.
* The "in hex" rendering is composed from core's own `Nat.toDigits 16`
  applied to `BitVec.toNat`, not a fresh hex-encoding axiom -- hex display
  of a natural number is already a built-in Lean function.
* `chainH : ByteArray -> Digest` remains the one genuine axiom, exactly as
  in `Praxis.Mathlib.DefReceipt`: it stands for the real BLAKE3
  cryptographic hash function, which has no Mathlib/Lean-core
  implementation and is out of scope for this pilot to construct. Its
  defining properties -- that byte-identical inputs share an address, and
  that (assuming no collision) any byte change yields a different address
  -- are exactly what a genuine hash function satisfies definitionally
  (`chainH` is a total function of `b`, so `chainH b₁ = chainH b₂`
  whenever `b₁ = b₂` is `congrArg chainH`, i.e. free; the converse,
  collision-resistance, is the cryptographic hypothesis a later theorem
  would take, not something to axiomatize as an unconditional fact here).
-/

abbrev Digest := BitVec 256

/-- The one genuine axiom: a real cryptographic hash function
(BLAKE3, per the corpus), from a byte string to a 256-bit digest. -/
axiom chainH : ByteArray → Digest

/-- Hex rendering of a digest, composed from core's `Nat.toDigits`. -/
noncomputable def contentAddressHex (b : ByteArray) : List Char :=
  Nat.toDigits 16 (chainH b).toNat

/-- `content_address(b) = chainH(b)`, as a digest (the hex string above is
just its display form). -/
noncomputable def contentAddress (b : ByteArray) : Digest :=
  chainH b

/-- Byte-identical objects share an address: immediate from `chainH` being
a total function of its argument. -/
theorem contentAddress_congr {b₁ b₂ : ByteArray} (h : b₁ = b₂) :
    contentAddress b₁ = contentAddress b₂ :=
  congrArg contentAddress h
