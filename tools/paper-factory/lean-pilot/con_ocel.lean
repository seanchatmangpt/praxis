/-
con:ocel

The map Ω : Ledger → OCEL2.0 sends each frame to an OCEL event with event id
`instruction_id`, activity the label of `activity_idx`, timestamp `ts_ns`, and
E2O qualifiers to `object_ids`.

Formalized as: an `OcelEvent` record shape (event id, activity label, timestamp,
E2O qualifiers to object ids), together with the construction `omega` mapping a
`Frame` (def:frame) to its `OcelEvent`.
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

/-- An OCEL 2.0 event: id, activity label, timestamp, and E2O qualifiers
    (the related object ids). -/
structure OcelEvent where
  event_id  : UInt64
  activity  : UInt32
  timestamp : UInt64
  e2o       : Fin 8 → UInt64

/-- Ω : Frame → OcelEvent, sending each frame to its OCEL event. -/
def omega (fr : Frame) : OcelEvent :=
  { event_id  := fr.instruction_id
  , activity  := fr.activity_idx
  , timestamp := fr.ts_ns
  , e2o       := fr.obj_refs }
