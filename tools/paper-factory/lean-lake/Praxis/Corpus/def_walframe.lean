import Mathlib.Data.BitVec

/-!
def:walframe, reformalized in the Mathlib lane.

A WAL (write-ahead-log) frame journaling a memoized DAG node is a
length-prefixed record:
  WAL_Frame = ⟨ length, chainH(payload), payload ⟩
allowing recovery to discard torn frames (a frame whose recorded
length/hash doesn't match its actual payload is truncated/corrupt and
is dropped during replay).

Composed from pre-built pieces rather than fresh axioms:
* `length : Nat` -- a byte/word count is exactly what `Nat` models.
* `payload : String` -- the serialized bytes of the frame; `String`
  (Lean's builtin sequence-of-characters type) is the natural stand-in
  for a serialized byte payload, matching `RefusalReason := String` in
  `Praxis/Mathlib/DefReceipt.lean`.
* `Digest := BitVec 256` -- reused verbatim from `DefReceipt.lean`
  rather than re-declaring a second 256-bit digest type.

`chainH` remains genuinely axiomatized, again reused from
`DefReceipt.lean`'s own justification: it stands for a real
cryptographic hash function (BLAKE3), and no Lean/Mathlib term is an
appropriate concrete stand-in for that without importing an actual
verified hash implementation, which is out of scope here.
-/

abbrev Digest := BitVec 256

axiom chainH : String → Digest

structure WalFrame where
  length  : Nat
  digest  : Digest
  payload : String
  wf      : digest = chainH payload
