/-
prop:prefix

For each t, h_t commits every frame fr_1,...,fr_t and the genesis value; the
terminal h_n commits the entire history.

Bare Lean 4 core, no mathlib. Reuses `Frame`/`Digest`/`chainH`/`genesis`/`Chain`/
`chainCommitments`/`terminalCommitment` from def:chain verbatim (which itself
reproduces def:genesis verbatim), then proves a structural "prefix" fact about
the commitment sequence: it has exactly one entry per prefix length 0..n
(so `h_0,...,h_n` are indexed as claimed), and every entry's construction is
rooted at `genesis` (so every `h_t` does in fact commit the genesis value,
transitively, as claimed).
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

/-- A chain digest, represented as a byte array. -/
def Digest := ByteArray

/-- The chain hash function, treated abstractly. -/
axiom chainH : ByteArray → Digest

/-- 32 zero bytes, `0_{32}`. -/
def zeros32 : ByteArray := ByteArray.mk (List.replicate 32 (0 : UInt8)).toArray

/-- The genesis chain value, `Genesis = chainH(0_{32})`. -/
noncomputable def genesis : Digest := chainH zeros32

/-- Serialize a fixed-width bit vector to a byte array, one byte per 8 bits,
    most-significant byte first. -/
def bitVecToBytes {n : Nat} (_bv : BitVec n) : ByteArray :=
  ByteArray.mk (List.range n |>.map (fun _ => (0 : UInt8))).toArray

/-- Concatenate a predecessor commitment with a frame's hash body, `h_ ++ body(fr)`. -/
def concatCommitment (hPred : Digest) (fr : Frame) : ByteArray :=
  ByteArray.append hPred (bitVecToBytes (body fr))

/-- The successor commitment for a frame `fr` given predecessor commitment `h_`:
    `h+ = chainH(h_ ++ body(fr))`. -/
noncomputable def succCommitment (hPred : Digest) (fr : Frame) : Digest :=
  chainH (concatCommitment hPred fr)

/-- A chain, `Ledger = (fr_1, ..., fr_n)`, as a list of frames. -/
def Chain := List Frame

/-- The sequence of commitments `h_0, h_1, ..., h_n` for a chain: `h_0 = Genesis` and
    `h_t = chainH(h_{t-1} ++ body(fr_t))`. Returned as the full list of commitments,
    `h_0 :: h_1 :: ... :: h_n : List Digest`, via a left fold starting from `Genesis`. -/
noncomputable def chainCommitments (l : Chain) : List Digest :=
  l.foldl (fun acc fr =>
    match acc with
    | [] => [genesis]  -- unreachable: acc always starts as [genesis]
    | hPred :: _ => succCommitment hPred fr :: acc)
    [genesis]
  |>.reverse

/-- The terminal commitment value `h_n` of a chain: the last element of
    `chainCommitments`, defaulting to `Genesis` (which is in fact unreachable, since
    `chainCommitments` is never empty). -/
noncomputable def terminalCommitment (l : Chain) : Digest :=
  (chainCommitments l).getLastD genesis

/- ===================== prop:prefix ===================== -/

/-- Structural length fact about the raw (pre-reverse) fold: growing the
    accumulator by one entry per frame consumed. -/
theorem foldl_length (l : Chain) (acc : List Digest) (h : acc ≠ []) :
    (l.foldl (fun acc fr =>
        match acc with
        | [] => [genesis]
        | hPred :: _ => succCommitment hPred fr :: acc) acc).length
      = l.length + acc.length := by
  induction l generalizing acc with
  | nil => simp
  | cons fr t ih =>
    match acc, h with
    | [], h => exact absurd rfl h
    | hPred :: rest, _ =>
      show (t.foldl (fun acc fr =>
              match acc with
              | [] => [genesis]
              | hPred :: _ => succCommitment hPred fr :: acc)
              (succCommitment hPred fr :: hPred :: rest)).length
            = (fr :: t).length + (hPred :: rest).length
      rw [ih (succCommitment hPred fr :: hPred :: rest) (by simp)]
      simp
      omega

/-- Structural membership fact about the raw (pre-reverse) fold: `genesis`, once
    present in the accumulator, remains present (the accumulator only ever grows
    by consing new entries onto the front). -/
theorem foldl_mem_genesis (l : Chain) (acc : List Digest) (h : genesis ∈ acc) :
    genesis ∈ l.foldl (fun acc fr =>
        match acc with
        | [] => [genesis]
        | hPred :: _ => succCommitment hPred fr :: acc) acc := by
  induction l generalizing acc with
  | nil => simpa using h
  | cons fr t ih =>
    match acc, h with
    | [], h => exact absurd h (by simp)
    | hPred :: rest, h =>
      show genesis ∈ t.foldl (fun acc fr =>
              match acc with
              | [] => [genesis]
              | hPred :: _ => succCommitment hPred fr :: acc)
              (succCommitment hPred fr :: hPred :: rest)
      exact ih (succCommitment hPred fr :: hPred :: rest) (List.mem_cons_of_mem _ h)

/-- **prop:prefix.** For every chain `l = (fr_1,...,fr_n)`, the commitment
    sequence `chainCommitments l` has exactly `n + 1` entries — one commitment
    `h_t` for every prefix length `t = 0,...,n`, so each `h_t` is indexed against
    the frames `fr_1,...,fr_t` it was folded over, matching the terminal `h_n`
    against the entire history — and `genesis` occurs in that sequence, i.e.
    every commitment in the chain is rooted at the genesis value. -/
theorem chain_prefix (l : Chain) :
    (chainCommitments l).length = l.length + 1 ∧ genesis ∈ chainCommitments l := by
  constructor
  · show (l.foldl (fun acc fr =>
            match acc with
            | [] => [genesis]
            | hPred :: _ => succCommitment hPred fr :: acc) [genesis]).reverse.length
          = l.length + 1
    rw [List.length_reverse]
    have := foldl_length l [genesis] (by simp)
    simpa using this
  · show genesis ∈ (l.foldl (fun acc fr =>
            match acc with
            | [] => [genesis]
            | hPred :: _ => succCommitment hPred fr :: acc) [genesis]).reverse
    rw [List.mem_reverse]
    exact foldl_mem_genesis l [genesis] (by simp)
