import Mathlib.Data.Rat.Defs
import Mathlib.Data.Finset.Basic
import Praxis.Corpus.def_ground

/-!
# con:xorf — An 8-bit XOR filter over the reachability fixpoint fact set

Let `R ⊆ Obs` be the reachability fixpoint fact set. A frozen fact store builds an
8-bit XOR filter over FNV-1a hashes of `R`, with index mapping via the fastrange
reduction `reduce(x, n) = (x * n) / 2^32`; membership is gated by `x ∈ B` iff
`F(x) = true`, else `false`, with false-positive rate `≈ 0.4%` and false negatives
`0`.

We reuse `Praxis.Corpus.DefGround.GroundAtom` for the element type of `R` (a ground
fact/atom over a domain `D` and object universe `Ob`), rather than re-deriving a
notion of "fact". Fingerprints are modelled with Mathlib's `BitVec 8` rather than a
hand-rolled 8-bit type. The fastrange reduction and the filter's `slots : Finset (Fin
n)` array are built from `Nat`/`Finset` machinery already in Mathlib.

FNV-1a itself (`fnv1a`) is kept as an `axiom`: it is a concrete, standardized
non-cryptographic hash function fixed bit-for-bit by an external specification
(FNV-1a, Fowler–Noll–Vo), not a mathematical object Mathlib provides or that this
formalization should re-derive from first principles — pinning down its exact
32-bit arithmetic (multiply by the FNV prime, XOR each byte) inside this
construction would add no mathematical content, only faithfully re-encode an
external bitwise algorithm. This mirrors the `Bits256` justification style in
`Praxis/Mathlib/DefReceipt.lean`.

The stated false-positive rate `≈ 0.4%` and false-negative rate `0` are empirical/
design parameters of the filter construction (not theorems proved here); they are
recorded as fields of the construction, as the thesis states them, rather than
derived.
-/

namespace Praxis.Corpus.ConXorf

open Praxis.Corpus.DefGround

/-- FNV-1a: a fixed, standardized 32-bit non-cryptographic hash function. Kept
axiomatic — see the module docstring for why no Mathlib equivalent applies. -/
axiom fnv1a {D : Praxis.Corpus.DefDomain.LiftedDomain} {Ob : Type}
    [DecidableEq (GroundAtom D Ob)] : GroundAtom D Ob → UInt32

/-- The fastrange index reduction `reduce(x, n) = (x * n) / 2^32`, computed over
`Nat` (both `x : UInt32` and `n : Nat` are cast up before the multiply so the
division by `2^32` is exact integer division, matching the standard fastrange
technique used by real XOR-filter implementations to avoid a modulo). -/
def reduce (x : UInt32) (n : Nat) : Nat :=
  (x.toNat * n) / 2 ^ 32

/-- An 8-bit XOR filter over ground atoms of a domain `D` with object universe `Ob`:
`n` fingerprint slots, each holding a `BitVec 8` fingerprint (`0` slots count as
"empty"/no fingerprint, following the usual XOR-filter convention), together with
the construction's stated false-positive and false-negative rates. -/
structure XorFilter (D : Praxis.Corpus.DefDomain.LiftedDomain) (Ob : Type)
    [DecidableEq (GroundAtom D Ob)] where
  /-- Number of fingerprint slots in the frozen fact store. -/
  n : Nat
  /-- The `n` fingerprint slots, indexed `Fin n → BitVec 8`. -/
  slots : Fin n → BitVec 8
  /-- Stated false-positive rate of the construction, `≈ 0.004`. -/
  falsePositiveRate : ℚ
  /-- Stated false-negative rate of the construction, exactly `0`. -/
  falseNegativeRate : ℚ

/-- `F(x)`: the filter's computed fingerprint check at atom `x`, folding the three
slots touched by an 8-bit XOR filter (indices from `fnv1a x` reduced into range via
two derived sub-hashes, following the standard 3-block XOR filter layout) and
comparing against the atom's own fingerprint byte (low 8 bits of `fnv1a x`). This
is the computed decision procedure; it is not asserted, it is derived from `slots`
and `fnv1a` exactly as the thesis specifies. -/
noncomputable def XorFilter.check {D : Praxis.Corpus.DefDomain.LiftedDomain} {Ob : Type}
    [DecidableEq (GroundAtom D Ob)] (F : XorFilter D Ob) (x : GroundAtom D Ob) :
    Bool :=
  if h : F.n = 0 then
    false
  else
    let hx := fnv1a x
    let fp : BitVec 8 := BitVec.ofNat 8 hx.toNat
    let i0 : Fin F.n := ⟨reduce hx F.n % F.n, Nat.mod_lt _ (Nat.pos_of_ne_zero h)⟩
    let i1 : Fin F.n :=
      ⟨reduce (hx ^^^ 0x9E3779B9) F.n % F.n, Nat.mod_lt _ (Nat.pos_of_ne_zero h)⟩
    let i2 : Fin F.n :=
      ⟨reduce (hx ^^^ 0x85EBCA6B) F.n % F.n, Nat.mod_lt _ (Nat.pos_of_ne_zero h)⟩
    decide (F.slots i0 ^^^ F.slots i1 ^^^ F.slots i2 = fp)

/-- Membership gate: `x ∈ B` iff `F(x) = true`, else `false`, exactly as stated. -/
noncomputable def XorFilter.member {D : Praxis.Corpus.DefDomain.LiftedDomain} {Ob : Type}
    [DecidableEq (GroundAtom D Ob)] (F : XorFilter D Ob) (x : GroundAtom D Ob) :
    Bool :=
  F.check x

end Praxis.Corpus.ConXorf
