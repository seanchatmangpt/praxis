/-
def:body

The hash body body(fr) ∈ {0,1}^(8·99) is the 99-byte serialization produced by
`to_hash_bytes`; it is a total, injective function of the semantic fields of `fr`
(all fields except the structural `pad`).

Formalized here as a total function from `Frame` (the semantic fields) to a fixed-width
bit vector of 8·99 = 792 bits. The semantic fields are first packed losslessly into a
single natural number via nested pairing (`pairN`, which is injective on its domain),
then that natural number is taken modulo 2^792 to land in the fixed-width codomain.
-/

/-- Cantor pairing function (injective on `Nat × Nat`), since bare Lean 4 core has no
    `pairN`. -/
def pairN (a b : Nat) : Nat :=
  (a + b) * (a + b + 1) / 2 + b

/- def:frame (dependency, reproduced verbatim from def_frame.lean, already kernel-verified) -/
structure Frame where
  instruction_id : UInt64
  fired_mask     : UInt64
  denial         : UInt8
  obj_refs       : Fin 8 → UInt64
  ts_ns          : UInt64
  activity_idx   : UInt32
  node_kind      : UInt8
  prior_hash     : UInt64

/-- Pack the eight `obj_refs` entries into one natural number via repeated pairing. -/
def packObjRefs (f : Fin 8 → UInt64) : Nat :=
  pairN (f 0).toNat
    (pairN (f 1).toNat
      (pairN (f 2).toNat
        (pairN (f 3).toNat
          (pairN (f 4).toNat
            (pairN (f 5).toNat
              (pairN (f 6).toNat (f 7).toNat))))))

/-- Total packing of all semantic fields of a `Frame` (everything except structural
    padding) into a single natural number, via nested `pairN`. -/
def Frame.toHashNat (fr : Frame) : Nat :=
  pairN fr.instruction_id.toNat
    (pairN fr.fired_mask.toNat
      (pairN fr.denial.toNat
        (pairN (packObjRefs fr.obj_refs)
          (pairN fr.ts_ns.toNat
            (pairN fr.activity_idx.toNat
              (pairN fr.node_kind.toNat fr.prior_hash.toNat))))))

/-- The hash body: a total function of the semantic fields of `fr`, landing in the
    fixed-width codomain `{0,1}^(8·99)`. -/
def body (fr : Frame) : BitVec (8 * 99) :=
  BitVec.ofNat (8 * 99) fr.toHashNat
