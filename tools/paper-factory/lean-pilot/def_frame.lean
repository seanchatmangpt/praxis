/-
def:frame

A frame `fr` is the record
  ⟨instruction_id, fired_mask, denial, obj_refs[0:8], ts_ns, activity_idx, node_kind, prior_hash⟩,
laid out as a repr(C, align(64)) structure occupying exactly 128 bytes, with 5 bytes of
interior padding.

This is a definitional (data-layout) statement, formalized here as the logical record
shape of a frame: the fields and their bit-widths, independent of the physical memory
layout / padding concerns (which are a representation detail, not a mathematical one).
-/

structure Frame where
  instruction_id : UInt64
  fired_mask     : UInt64
  denial         : UInt8
  obj_refs       : Fin 8 → UInt64
  ts_ns          : UInt64
  activity_idx   : UInt32
  node_kind      : UInt8
  prior_hash     : UInt64
