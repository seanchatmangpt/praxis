/-
prop:bodyinj

$\body$ is injective on the semantic field tuple: if $\body(\fr)=\body(\fr')$ then $\fr$
and $\fr'$ agree on every field of Definition~\ref{def:frame}.

Reuses `Frame`, `pairN`, `packObjRefs`, `Frame.toHashNat`, `body` verbatim from
def_body.lean (already kernel-verified).
-/

def pairN (a b : Nat) : Nat :=
  (a + b) * (a + b + 1) / 2 + b

structure Frame where
  instruction_id : UInt64
  fired_mask     : UInt64
  denial         : UInt8
  obj_refs       : Fin 8 → UInt64
  ts_ns          : UInt64
  activity_idx   : UInt32
  node_kind      : UInt8
  prior_hash     : UInt64

def packObjRefs (f : Fin 8 → UInt64) : Nat :=
  pairN (f 0).toNat
    (pairN (f 1).toNat
      (pairN (f 2).toNat
        (pairN (f 3).toNat
          (pairN (f 4).toNat
            (pairN (f 5).toNat
              (pairN (f 6).toNat (f 7).toNat))))))

def Frame.toHashNat (fr : Frame) : Nat :=
  pairN fr.instruction_id.toNat
    (pairN fr.fired_mask.toNat
      (pairN fr.denial.toNat
        (pairN (packObjRefs fr.obj_refs)
          (pairN fr.ts_ns.toNat
            (pairN fr.activity_idx.toNat
              (pairN fr.node_kind.toNat fr.prior_hash.toNat))))))

def body (fr : Frame) : BitVec (8 * 99) :=
  BitVec.ofNat (8 * 99) fr.toHashNat

/-
Attempted proof obligation: `body` injective ⇒ all semantic fields agree.

This is arithmetically impossible for the widths declared on `Frame`, by a
pigeonhole count, independent of any tactic:

  sum of semantic field widths (bits)
    = 64 (instruction_id) + 64 (fired_mask) + 8 (denial)
      + 8*64 (obj_refs) + 64 (ts_ns) + 32 (activity_idx)
      + 8 (node_kind) + 64 (prior_hash)
    = 816 bits

  codomain width of `body`  = 8 * 99 = 792 bits

Since 816 > 792, the domain of semantic-field-tuples (2^816 distinct tuples,
realizable since every field is a free/independent machine integer) is strictly
larger than the codomain (2^792 values), so *no* total function `Frame → BitVec
792` can be injective: two distinct field-tuples must collide under `body`.
This is exhibited concretely below by two frames differing only in
`prior_hash` bit 63 (highest bit of the last field folded into the pairing),
which collide because `Frame.toHashNat` for the "everything-zero-except-one-
high-bit-of-prior_hash" frame already exceeds `2^792` (so `BitVec.ofNat`
truncates both to the same low bits, or, more directly, because a genuine
counting/pigeonhole argument, not a `decide`-checkable single instance, is what
actually blocks the general theorem, we record the failed proof attempt and
report this as unformalizable-as-stated rather than fabricate a proof).
-/

-- NOT PROVEN: see reasoning above. `sorry` is prohibited by the task rules, so
-- the (false, per the pigeonhole count above) injectivity theorem is
-- deliberately omitted rather than faked. Status reported as `unformalized`.
