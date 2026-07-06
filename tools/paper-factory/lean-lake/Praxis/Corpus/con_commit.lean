import Mathlib.Tactic
import Praxis.Corpus.def_frame

/-!
`con:commit`: let `p ∈ {0,1}*` be the canonical JSON bytes of an admitted payload and
`dg(p) = chainH(p) ∈ {0,1}^256` its digest; partition `dg(p)` into eight 32-bit
little-endian words `w_0,…,w_7` and set `obj_refs[i] = PackedObjRef(w_i)` via the raw
tuple constructor, preserving all 32 bits.

`chainH` (the canonical content-addressing hash, matching the `chainH` used elsewhere in
this corpus, e.g. `Praxis/Mathlib/DefReceipt.lean`) is kept as an axiom: it denotes one
specific concrete cryptographic hash function (BLAKE3-family, per the project's
invariants), which has no general-purpose Mathlib equivalent to compose from -- unlike
the bit-vector plumbing below, which is built entirely from `BitVec`/`Fin` operations
already provided by core/Mathlib.
-/

/-- Canonical JSON bytes of an admitted payload: a finite bit string `{0,1}*`. -/
def Payload : Type := List Bool

/-- `chainH`: the project's canonical hash, producing a 256-bit digest from a payload's
canonical JSON bytes. Kept as an axiom -- see file docstring for justification. -/
axiom chainH : Payload → BitVec 256

/-- `PackedObjRef`: the raw tuple constructor that packs one 32-bit little-endian word,
preserving all 32 bits (a thin wrapper composed from `BitVec 32`, no new axioms needed). -/
def PackedObjRef : Type := BitVec 32

/-- `PackedObjRef.mk`: the raw tuple constructor mentioned in the statement, wrapping a
32-bit word verbatim. -/
def PackedObjRef.mk (w : BitVec 32) : PackedObjRef := w

/-- The `i`-th (`i : Fin 8`) 32-bit little-endian word of a 256-bit digest: bits
`[32*i, 32*i+32)` counting from the least-significant bit, i.e. word `0` is the
low-order word, matching a little-endian partition `w_0,…,w_7` of `dg(p)`. -/
def digestWord (dg : BitVec 256) (i : Fin 8) : BitVec 32 :=
  dg.extractLsb' (i.val * 32) 32

/-- `con:commit`: given an admitted payload `p`, compute its digest `dg(p) = chainH(p)`,
partition it into eight 32-bit little-endian words `w_0,…,w_7`, and set
`obj_refs[i] = PackedObjRef(w_i)` for each `i : Fin 8`, preserving all 32 bits of each
word via the raw tuple constructor `PackedObjRef.mk`. -/
noncomputable def commit (p : Payload) : Fin 8 → PackedObjRef :=
  fun i => PackedObjRef.mk (digestWord (chainH p) i)
