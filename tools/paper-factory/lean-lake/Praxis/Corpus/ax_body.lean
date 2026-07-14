import Mathlib.Tactic
import Praxis.Corpus.def_frame

/-!
`def:body`'s external-serializer axioms, split out of `def_body.lean`.

Both axioms below stand in for `to_hash_bytes`, a real serializer implemented outside
Lean (choice of endianness, which sub-ranges of which fields are dropped/repacked,
etc.). `body`'s exact byte-for-byte mapping is an implementation detail of that
external function, not a fact recoverable from `Frame`'s abstract field *types* alone
-- exactly analogous to why `Praxis/Mathlib/DefReceipt.lean` keeps the hash function
itself opaque while making everything it operates on concrete.

`body_injective` was audited (2026-07) for a real in-Lean proof: since `body` is
declared here as an *axiom* -- an opaque total function with no Lean-visible
definition, not a `def` unfolding to a concrete byte-packing -- there is no term or
equation available to case on, so `Function.Injective body` is not derivable from
`Frame`'s structure by any tactic; it is exactly as external as `body` itself, and
is kept axiomatized for the same reason, not reclassified as provable.
-/

/-- The hash body type: `{0,1}^(8·99)`, i.e. 99 bytes, as a `BitVec`. -/
def HashBody : Type := BitVec (8 * 99)

/-- `body`: the (total) serialization of a frame's semantic fields into its 99-byte
hash body, as produced by `to_hash_bytes`. Kept axiomatized -- see module doc for why the
concrete byte layout of the external serializer is not reconstructed here. -/
axiom body : Frame → HashBody

/-- `body` is injective: distinct frames (differing in any semantic field) serialize to
distinct hash bodies. Genuinely external -- `body` itself is an opaque axiom with no
Lean-visible definition to prove injectivity from, so this is part of the axiomatized
specification of `to_hash_bytes`, not a fact derivable in-Lean. See module doc. -/
axiom body_injective : Function.Injective body
