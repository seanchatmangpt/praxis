import Mathlib.Tactic
import Praxis.Corpus.def_frame

/-!
`def:body`: the hash body `body(fr) ∈ {0,1}^(8·99)` is the 99-byte serialization produced
by `to_hash_bytes`; it is a total, injective function of the semantic fields of `fr` (all
fields except the structural `pad`).

`Frame` (from `def:frame`) has no separate `pad` field -- the 5 bytes of interior padding
noted there are a `repr(C, align(64))` *compiler layout* artifact of the host-language
struct, not a Lean field, so "all fields except `pad`" is simply "all fields of `Frame`".

The exact byte-for-byte mapping from `Frame`'s abstract fields to the concrete 99-byte
wire format produced by `to_hash_bytes` (a real serializer implemented outside Lean, e.g.
choice of endianness, which sub-ranges of which fields are dropped/repacked, etc.) is an
implementation detail of that external function, not a fact recoverable from the abstract
field *types* alone -- exactly analogous to why `Praxis/Mathlib/DefReceipt.lean` keeps the
hash function itself opaque while making everything it operates on concrete. We therefore
keep `body` as an axiomatized total function `Frame → BitVec (8 * 99)` together with its
injectivity, rather than fabricating a concrete byte-packing that would not actually match
`to_hash_bytes`'s real, external definition.
-/

/-- The hash body type: `{0,1}^(8·99)`, i.e. 99 bytes, as a `BitVec`. -/
def HashBody : Type := BitVec (8 * 99)

/-- `body`: the (total) serialization of a frame's semantic fields into its 99-byte
hash body, as produced by `to_hash_bytes`. Kept axiomatized -- see module doc for why the
concrete byte layout of the external serializer is not reconstructed here. -/
axiom body : Frame → HashBody

/-- `body` is injective: distinct frames (differing in any semantic field) serialize to
distinct hash bodies. Part of the axiomatized specification of `to_hash_bytes`, for the
same reason `body` itself is axiomatized. -/
axiom body_injective : Function.Injective body
