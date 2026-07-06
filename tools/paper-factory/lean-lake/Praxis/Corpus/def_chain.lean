import Praxis.Corpus.def_body
import Praxis.Corpus.def_genesis

/-!
def:chain, reformalized in the Mathlib lane.

"For a frame `fr` with predecessor commitment `h_-`, the successor commitment is
`h_+ = chainH(h_- ++ body(fr))`; a chain `Ledger = (fr_1,...,fr_n)` has `h_0 = Genesis`
and `h_t = chainH(h_{t-1} ++ body(fr_t))`, terminal value `h_n`."

Composed from pieces already established in this pilot rather than fresh axioms:

* `Digest`, `chainH` reused from `Praxis.Corpus.def_contentaddr` (via `def_genesis`'s
  import chain) -- the same BLAKE3 chain hash, not a second axiom.
* `HashBody`, `body` reused from `Praxis.Corpus.def_body` -- the 99-byte frame
  serialization, not redeclared.
* `genesis` reused from `Praxis.Corpus.def_genesis` for the base case `h_0`.
* The one genuinely new piece needed here is the byte-level concatenation
  `h_- ++ body(fr)` that `chainH` (a `ByteArray → Digest` axiom) is applied to.
  Rather than axiomatizing "the concatenation", we build it: `bitVecToBytesLE`
  is a fully concrete, computable little-endian byte-packing of any `BitVec (8*n)`
  into a `ByteArray` of `n` bytes, using only core's `UInt8.ofNat`/`ByteArray.mk`.
  This mirrors `def_genesis`'s `zeros32`/`domainSeedBytes`: a concrete, computable
  helper, not an axiom -- and it is intentionally left unspecified whether it matches
  any *external* wire format bit-for-bit (same caveat as `def_body`'s `body`), since
  only `chainH`'s *argument* needs to be a `ByteArray` for the definition to
  type-check; no proof obligation depends on the exact byte order chosen.
-/

/-- Little-endian packing of a `BitVec (8 * n)` into a `ByteArray` of `n` bytes,
built from core's `UInt8.ofNat` and `ByteArray.mk` -- fully concrete and computable,
no axiom. -/
def bitVecToBytesLE (n : Nat) (v : BitVec (8 * n)) : ByteArray :=
  ByteArray.mk ((List.range n).toArray.map (fun i => UInt8.ofNat (v.toNat / 256 ^ i)))

/-- One chain step: `h_+ = chainH(h_- ++ body(fr))`, built from the reused `chainH`
and `body`, concatenated via `bitVecToBytesLE`. -/
noncomputable def chainStep (hMinus : Digest) (fr : Frame) : Digest :=
  chainH (bitVecToBytesLE 32 hMinus ++ bitVecToBytesLE 99 (body fr))

/-- A ledger `Ledger = (fr_1, ..., fr_n)` is a list of frames. -/
def Ledger : Type := List Frame

/-- The chain's running commitments `h_0, h_1, ..., h_n`, with `h_0 = genesis` and
`h_t = chainStep h_{t-1} fr_t`, folded left-to-right over the ledger. -/
noncomputable def chainCommitments (ledger : Ledger) : Digest :=
  (ledger : List Frame).foldl chainStep genesis

/-- The terminal chain value `h_n` for a ledger, i.e. `chainCommitments` applied to
the whole ledger -- the `def:chain` construction's final commitment. -/
noncomputable def chain (ledger : Ledger) : Digest :=
  chainCommitments ledger
