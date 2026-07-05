/-
con:commit

Let p ∈ {0,1}* be the canonical JSON bytes of an admitted payload and
dg(p) = chainH(p) ∈ {0,1}^256 its digest; partition dg(p) into eight 32-bit
little-endian words w_0,...,w_7 and set obj_refs[i] = PackedObjRef(w_i) via
the raw tuple constructor, preserving all 32 bits.

Construction: given an abstract 256-bit digest represented as a function
from `Fin 8` to `UInt32` (its eight little-endian 32-bit words), build the
`obj_refs` field of a `def:frame` `Frame` by widening each 32-bit word into
the 64-bit `PackedObjRef` slot, preserving all 32 bits (no truncation).
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

/-- A digest, given as its 8 little-endian 32-bit words. -/
def Digest := Fin 8 → UInt32

/-- The raw tuple constructor `PackedObjRef` : widen a 32-bit word into the
64-bit slot, preserving all 32 bits. -/
def PackedObjRef (w : UInt32) : UInt64 :=
  UInt64.ofNat w.toNat

/-- Commit construction: build the `obj_refs` array of a frame from a digest,
by applying `PackedObjRef` word-wise. -/
def commit (dg : Digest) (instruction_id fired_mask ts_ns prior_hash : UInt64)
    (denial node_kind : UInt8) (activity_idx : UInt32) : Frame :=
  { instruction_id := instruction_id
    fired_mask     := fired_mask
    denial         := denial
    obj_refs       := fun i => PackedObjRef (dg i)
    ts_ns          := ts_ns
    activity_idx   := activity_idx
    node_kind      := node_kind
    prior_hash     := prior_hash }
