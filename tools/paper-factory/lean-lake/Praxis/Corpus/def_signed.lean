import Mathlib.Data.BitVec

/-!
def:signed

A signed receipt is the triple `SignedReceipt = ⟨h_hex, s, k⟩` where
`s = Sign_sk(h_hex)` is an Ed25519 signature and `k` the hex verifying
key; it is self-contained since `s` and `k` travel with `h_hex`.

Composition over axiomatization:

* `hHex : String` -- the hex-encoded digest is exactly a string of hex
  characters, so `String` (already built-in) is the right carrier, no
  new opaque type needed.
* `sig : BitVec 512` -- an Ed25519 signature is a fixed 64-byte (512-bit)
  value; `BitVec 512` from `Mathlib.Data.BitVec` is a real, already-built
  bit-vector type, giving decidable equality and a genuine inhabitant
  (`0`), unlike an opaque axiomatized type.
* `key : BitVec 256` -- an Ed25519 verifying key is a fixed 32-byte
  (256-bit) value, likewise modeled by the pre-built `BitVec 256`
  (matching `Digest` in `Praxis/Mathlib/DefReceipt.lean`, the same
  concept used there for a hash digest).

No axioms are introduced: `Sign_sk` itself is not modeled here because
this statement only defines the *shape* of a signed receipt (the triple
and its self-containedness), not the signing algorithm; the algorithm
would only need axiomatizing if a later theorem quantified over it,
which is out of scope for this definition.
-/

structure SignedReceipt where
  hHex : String
  sig : BitVec 512
  key : BitVec 256

/-- A signed receipt is self-contained: the triple already carries its
own signature and verifying key alongside the hash, so no external
lookup is required to reconstruct any of the three fields. -/
example (r : SignedReceipt) : SignedReceipt := ⟨r.hHex, r.sig, r.key⟩
