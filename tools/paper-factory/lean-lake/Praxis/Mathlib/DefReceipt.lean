import Mathlib.Data.BitVec
import Praxis.Mathlib.PropMonoid

/-!
def:receipt, reformalized in the Mathlib lane.

Bare-core version (`tools/paper-factory/lean-pilot/def_receipt.lean`)
axiomatizes six fields as opaque, unrelated types: `Bits256`,
`DenialWord`, `TransitionId`, `Fitness`, `RefusalReason`, `Version`.
Per the directive to prefer composing pre-built/already-defined pieces
over declaring anything from scratch, only two of those six remain
axiomatized here (`chainH`, `chainStep`, discussed below) -- the other
four are each composed from a type that already exists, either in
Mathlib/Lean core or elsewhere in this same pilot:

* `Bits256 := BitVec 256` -- a real, pre-built 256-bit vector type
  (`Mathlib.Data.BitVec`), not an opaque axiom. A concrete value now
  genuinely exists (`(0 : BitVec 256)`) and equality is decidable.
* `DenialWord := Σ n, Deny n` -- reuses `Deny`, the *already-defined*
  type from `Praxis.Mathlib.PropMonoid` (this same pilot's `prop:monoid`
  reformalization), rather than declaring a second, unrelated axiom for
  what is the same concept (a denial word) under a different name. This
  is composition across the pilot, not just within one file.
* `TransitionId := Nat` -- a transition identity is exactly the kind of
  thing a natural number already models (a counter/index); no new type
  needed.
* `Version := Nat × Nat × Nat` -- a semantic-version triple
  (major, minor, patch), composed from `Nat` and the pre-built `Prod`.
* `RefusalReason := String` -- a human-readable reason message, exactly
  what `String` already is.
* `Fitness := Nat` -- a fitness/replay score; `Nat` is the simplest
  pre-built type with the ordering structure a score needs.

`chainH`/`chainStep` remain genuinely axiomatized: they stand for a real
cryptographic hash function (BLAKE3, per the corpus's own
`02_receipt_cryptography` paper) and its chaining rule. No Lean/Mathlib
term is an appropriate stand-in for an actual collision-resistant hash
implementation -- modeling one concretely here would either be
computationally meaningless (a fake hash) or require importing an actual
verified BLAKE3 implementation, which does not exist in Mathlib and is
out of scope for this pilot. Axiomatizing the *existence* of such a
function (with its cryptographic properties left as a hypothesis a later
theorem would take, per the file's original bare-core comment) is the
correct level of abstraction here, not a gap this directive asks to
close.
-/

abbrev Digest := BitVec 256

example : Digest := (0 : BitVec 256)
example : DecidableEq Digest := inferInstance

abbrev DenialWord := Σ n : Nat, Deny n
abbrev TransitionId := Nat
abbrev Version := Nat × Nat × Nat
abbrev RefusalReason := String
abbrev Fitness := Nat

structure Frame where
  dgX : Digest
  dgG : Digest
  denial : DenialWord
  transition : TransitionId
  dgA : Digest
  fitness : Fitness
  reason : RefusalReason
  version : Version

axiom chainH : Digest → Digest
axiom chainStep : Digest → Frame → Digest

structure Receipt where
  hMinus : Digest
  frame : Frame
  hPlus : Digest
  advances : hPlus = chainStep hMinus frame
