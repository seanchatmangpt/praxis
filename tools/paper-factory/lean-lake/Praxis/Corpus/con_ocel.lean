import Mathlib.Tactic
import Praxis.Corpus.def_frame

/-!
`con:ocel`: the map `Ω : Ledger → OCEL2.0` sends each frame to an OCEL event with event id
`instruction_id`, activity the label of `activity_idx`, timestamp `ts_ns`, and E2O qualifiers
to `object_ids`.

We model a single OCEL 2.0 event as a record with an event id, an activity label (a `String`,
since activities in OCEL are names drawn from a finite label alphabet indexed by
`activity_idx : BitVec 32`), a timestamp, and the list of related object ids (the E2O
qualifiers), reusing exactly the field types already established in `def:frame`
(`Praxis/Corpus/def_frame.lean`): `BitVec 64` for ids/timestamps, `ObjRef` for object handles.
No axioms are needed: this is a plain data construction (a structure plus a total function
into it), matching the four Mathlib-composition examples already in this repo.
-/

/-- A minimal OCEL 2.0 event record: event id, activity label, timestamp, and the list of
related object ids (E2O qualifiers). `Ledger` here is modeled as (a stream of) `Frame`s, and
`OCEL2.0` as (a stream of) `OCELEvent`s; `Ω` acts frame-by-frame. -/
structure OCELEvent : Type where
  /-- the OCEL event id, taken from `instruction_id` -/
  event_id : BitVec 64
  /-- the OCEL activity label, taken from the (named) label of `activity_idx` -/
  activity : String
  /-- the OCEL event timestamp, taken from `ts_ns` -/
  timestamp : BitVec 64
  /-- the OCEL E2O qualifiers: the list of related object ids, taken from `obj_refs` -/
  object_ids : List ObjRef

/-- `activityLabel`: the label of an `activity_idx`. In the absence of a concrete finite
naming table for activity indices (that table is deployment/pack configuration, not a
mathematical object), we use the decimal string of the underlying index as the canonical
label, which is total and injective on `BitVec 32`, faithfully modeling "the label of
`activity_idx`" as a function `BitVec 32 → String`. -/
def activityLabel (idx : BitVec 32) : String :=
  toString idx.toNat

/-- `Ω : Frame → OCELEvent`: the construction sending each frame to an OCEL event with
event id `instruction_id`, activity the label of `activity_idx`, timestamp `ts_ns`, and
E2O qualifiers to `object_ids` (here, the eight `obj_refs` slots collected as a list). -/
def Ω (fr : Frame) : OCELEvent where
  event_id := fr.instruction_id
  activity := activityLabel fr.activity_idx
  timestamp := fr.ts_ns
  object_ids := (List.finRange 8).map fr.obj_refs
