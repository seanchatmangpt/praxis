import Mathlib.Data.BitVec

/-!
def:receipt

"Fix a collision-resistant hash `chainH`; the frame is
`fr = <theta, chainH(b)>` and the chained receipt is
`h+ = chainH(h- ++ fr)`; the receipt is the `<= CL`-chunk tuple
(verdict, h+, Fitness, reason)."

Per the mandatory-composition directive, every field that already has a
suitable pre-built Mathlib/Lean-core type is composed from that type
rather than declared as a fresh opaque axiom:

* `Digest := BitVec 256` -- a real, pre-built 256-bit vector type
  (`Mathlib.Data.BitVec`), matching `Praxis/Mathlib/DefReceipt.lean`'s
  own choice for the same concept.
* `Theta := String` -- `theta` here is the frame's descriptive/label
  component; `String` is the simplest pre-built type for that role.
* `Block := String` -- `b`, the block being hashed into the frame, is
  likewise modeled as an opaque byte/text payload via `String`.
* `Verdict := Bool` -- the statement's "verdict" is a binary
  accept/refuse outcome, exactly what `Bool` already models.
* `Fitness := Nat` -- a fitness score, the simplest pre-built type with
  the ordering structure a score needs (matches `DefReceipt.lean`).
* `Reason := String` -- a human-readable refusal/verdict reason.

`chainH` remains genuinely axiomatized: it stands for a real
collision-resistant cryptographic hash function (BLAKE3, per the
corpus's own receipt-cryptography paper). No Lean/Mathlib term is an
appropriate stand-in for an actual collision-resistant hash
implementation -- modeling one concretely here would either be
computationally meaningless (a fake hash) or require importing a real
verified BLAKE3 implementation, which does not exist in Mathlib and is
out of scope here. Axiomatizing the *existence* of such a function is
the correct level of abstraction, not a gap this directive asks to
close.
-/

abbrev Digest := BitVec 256
abbrev Theta := String
abbrev Block := String
abbrev Verdict := Bool
abbrev Fitness := Nat
abbrev Reason := String

/-- A collision-resistant hash, fixed once and for all. -/
axiom chainH : String → Digest

/-- `fr = <theta, chainH(b)>`. -/
structure Frame where
  theta : Theta
  dg : Digest

/-- Serialize a digest and a frame into the string `chainH` chains over
    (`h- ++ fr`). This is a plain, computable encoding step, not part of
    the axiomatized hash itself. -/
def encodeChain (hMinus : Digest) (fr : Frame) : String :=
  toString hMinus ++ fr.theta ++ toString fr.dg

/-- `h+ = chainH(h- ++ fr)`, the chained receipt digest. -/
noncomputable def chainedReceipt (hMinus : Digest) (fr : Frame) : Digest :=
  chainH (encodeChain hMinus fr)

/-- The receipt is the tuple (verdict, h+, Fitness, reason). -/
structure Receipt where
  verdict : Verdict
  hPlus : Digest
  fitness : Fitness
  reason : Reason

example : Digest := (0 : BitVec 256)
example : DecidableEq Digest := inferInstance
