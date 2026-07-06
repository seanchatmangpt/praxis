import Mathlib.Data.BitVec
import Praxis.Corpus.con_denial

/-!
Label: con:agent8

"Project each agent to a status byte $b\in\{0,1\}^8$ whose lanes name
admitted, evidence-ok, within-budget, authority-bound, healthy, conformant,
receipted, replayable. A fleet of $N$ agents is a bit-packed vector; a
$64$-bit word holds $8$ agents; a fleet admission sweep is the branchless
mask reduction of the denial construction applied word-parallel."

`\{0,1\}^8` is exactly `Deny 8` (`Fin 8 → Bool`, from `Praxis.Mathlib.PropMonoid`,
imported transitively via `Praxis.Corpus.con_denial`) -- the same status-byte
shape already used for the per-obligation lane words in `con:denial`, not a
new axiomatized "byte" type. Each of the 8 lanes names one obligation
(admitted, evidence-ok, within-budget, authority-bound, healthy, conformant,
receipted, replayable); a `false` in lane `i` means "denied on lane `i`",
matching `con:denial`'s convention that `⊥ : Deny 8` is the clean/admitted
byte.

A fleet of `N` agents is `Fin N → Deny 8`, a bit-packed vector of status
bytes (bit-packing made concrete, not merely asserted, by `packByte` /
`packWord` below, which fold each `Deny 8` into an actual `BitVec 8` via
Mathlib's pre-built `BitVec.ofFn` -- no hand-rolled bit-twiddling).

A `64`-bit word holding `8` agents is `BitVec 64`, built by concatenating
8 packed `BitVec 8` status bytes with Mathlib's pre-built `BitVec.append`
(`++`), reusing the existing `BitVec` monoid structure rather than defining
a new packing primitive from scratch.

The fleet admission sweep is the *word-parallel* application of the
`con:denial` construction: instead of running `Deny.denial` once per agent
and then looping to combine results, `fleetDenial8` takes the `Finset.sup`
(same pre-built Mathlib finite-lattice operation `con:denial` already uses)
of the 8 agents' status bytes directly -- a single branchless reduction
across the whole word, matching "the branchless mask reduction of the
denial construction applied word-parallel". `wordAdmits8` restates
admission as `wordOf8 ... = 0#64`, the word-level analogue of `con:denial`'s
`denialAdmits`.

This is a `construction`: it packages the LaTeX's data (status byte, fleet
vector, packed word, word-parallel admission sweep) from pre-built
Mathlib/core machinery (`Deny` from `con:denial`, `BitVec.ofFn`,
`BitVec.append`, `Finset.sup`, `BooleanAlgebra`'s `⊥`), with no new axioms
and no proof obligation beyond this file type-checking.
-/

namespace Agent8

/-- The status byte: 8 lanes, one per obligation (admitted, evidence-ok,
within-budget, authority-bound, healthy, conformant, receipted, replayable).
Exactly `Deny 8` from `con:denial` -- not a new type. -/
abbrev StatusByte := Deny 8

/-- A fleet of `N` agents, each carrying a status byte: a bit-packed vector
in the abstract (function) sense; `packByte`/`packWord8` below make the
literal bit-packing concrete. -/
abbrev Fleet (N : Nat) := Fin N → StatusByte

/-- Pack one agent's status byte (`Deny 8`, i.e. `Fin 8 → Bool`) into a real
`BitVec 8`, via core's pre-built `BitVec.ofBoolListLE` applied to the list of
the 8 lane bits (`List.ofFn`, also pre-built). -/
def packByte (b : StatusByte) : BitVec 8 :=
  (List.length_ofFn (f := b)) ▸ BitVec.ofBoolListLE (List.ofFn b)

/-- Pack 8 agents' status bytes into a single 64-bit word by concatenating
their packed bytes with Mathlib's pre-built `BitVec.append` (`++`), left to
right. A `64`-bit word holding `8` agents, made literal. -/
def wordOf8 (f : Fin 8 → StatusByte) : BitVec 64 :=
  packByte (f 0) ++ packByte (f 1) ++ packByte (f 2) ++ packByte (f 3) ++
  packByte (f 4) ++ packByte (f 5) ++ packByte (f 6) ++ packByte (f 7)

/-- The fleet admission sweep: the branchless mask reduction of `con:denial`
(`Deny.denial`, itself a `Finset.univ.sup` over the lanes of one agent)
applied word-parallel, i.e. `Finset.univ.sup` over the 8 agents in one word,
combining their status bytes lane-by-lane in a single pass. -/
def fleetDenial8 (f : Fin 8 → StatusByte) : StatusByte :=
  Finset.univ.sup f

/-- Word-level admission: the packed word is all-zero, the `BitVec 64`
analogue of `con:denial`'s `denialAdmits` (`denial gs o = ⊥`). -/
def wordAdmits8 (f : Fin 8 → StatusByte) : Prop :=
  wordOf8 f = (0#64 : BitVec 64)

end Agent8
