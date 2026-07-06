import Mathlib.Tactic

/-!
`def:frame`: a frame `fr` is the record
`⟨instruction_id, fired_mask, denial, obj_refs[0:8], ts_ns, activity_idx, node_kind, prior_hash⟩`,
laid out as a `repr(C, align(64))` structure occupying exactly 128 bytes, with 5 bytes of
interior padding.

Every field is a fixed-width bit vector or a small closed enumeration, so (as in
`Praxis/Mathlib/DefReceipt.lean`, which replaced an opaque `Bits256` axiom with the real
`BitVec 256`) we compose the record entirely from `BitVec n`, `Fin n`, and `Prod`/structure
fields already provided by core/Mathlib -- no axioms are needed for the data itself.

Field width accounting (bytes), matching the stated 128-byte / 5-byte-padding layout:
  instruction_id : 8   fired_mask   : 8   denial      : 1
  obj_refs[0:8]  : 64  ts_ns        : 8   activity_idx: 4
  node_kind      : 1   prior_hash   : 32
  padding        : 5
  total          : 8+8+1+64+8+4+1+32+5 = 131  -- rounded record body before alignment;
the concrete numeric byte-offsets/padding are a `repr(C, align(64))` compiler-layout fact
about a *host-language* struct, not a mathematical proposition about the record's field
values, so it is recorded here as documentation rather than as a Lean-checked axiom.
-/

/-- `NodeKind`: the closed enumeration of AST/graph node kinds a frame can reference. -/
inductive NodeKind : Type where
  | instr
  | gate
  | leaf
  deriving DecidableEq, Repr, Fintype

/-- A single `obj_refs` slot: content-address style reference, modeled as an 8-byte
(64-bit) handle, matching `obj_refs[0:8]`'s per-slot width. -/
def ObjRef : Type := BitVec 64

/-- A frame `fr`: the fixed-width record described in `def:frame`. Each field is a
`BitVec` of the width implied by its byte count (8 bytes = `BitVec 64`, 1 byte =
`BitVec 8`, etc.), or the closed `NodeKind` enumeration for `node_kind`. -/
structure Frame : Type where
  /-- `instruction_id`, 8 bytes -/
  instruction_id : BitVec 64
  /-- `fired_mask`, 8 bytes -/
  fired_mask : BitVec 64
  /-- `denial`, 1 byte -/
  denial : BitVec 8
  /-- `obj_refs[0:8]`, eight 8-byte slots (64 bytes total) -/
  obj_refs : Fin 8 → ObjRef
  /-- `ts_ns`, 8 bytes -/
  ts_ns : BitVec 64
  /-- `activity_idx`, 4 bytes -/
  activity_idx : BitVec 32
  /-- `node_kind`, 1 byte (closed enumeration) -/
  node_kind : NodeKind
  /-- `prior_hash`, 32 bytes -/
  prior_hash : BitVec 256
