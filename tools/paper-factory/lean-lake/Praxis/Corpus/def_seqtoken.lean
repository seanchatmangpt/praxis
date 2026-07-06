import Praxis.Corpus.def_lifecat

/-!
# def:seqtoken

The lifecycle process is modeled as a safe Petri net (3-Node SEQ POWL token game)
over the marking `M = (TOK_START, TOK_JUDGED, TOK_ADMITTED, TOK_DONE)`, with fitness
`Fitness ∈ [0,1]` in Q16.16.

We reuse `LifeObj` from `def:lifecat` (`raw`, `val`, `admd`, `rcpt`) as the four
places of the safe net -- `TOK_START/TOK_JUDGED/TOK_ADMITTED/TOK_DONE` are exactly
those four places under new names, so the marking is a function `LifeObj → Bool`
(safe net: each place holds 0 or 1 token), reusing the quiver's object type instead
of re-declaring a fresh four-constructor inductive.

`Fitness` in Q16.16 fixed point is realized as a subtype of `Int` constrained to the
integer range `[0, 65536]` (i.e. `0/65536` to `65536/65536 = 1` in Q16.16), reusing
Mathlib's `Int` and its order rather than hand-rolling a fixed-point number type.
No axiom is needed: both the marking and the fitness score are plain data composed
from Mathlib primitives (`Bool`, `Int`, subtype `Set.Icc`-style bounds).
-/

namespace Praxis.Corpus

open LifeObj

/-- A marking of the safe 3-Node SEQ POWL token game: for each of the four places
(`TOK_START = raw`, `TOK_JUDGED = val`, `TOK_ADMITTED = admd`, `TOK_DONE = rcpt`),
whether it currently holds a token. Reuses `LifeObj` from `def:lifecat` as the
place index rather than declaring a new four-constructor type. -/
def Marking : Type := LifeObj → Bool

/-- The Q16.16 fixed-point encoding of `Fitness ∈ [0,1]`: an integer numerator in
`[0, 65536]` over the implicit denominator `65536 = 2^16`, so `value = 65536`
represents fitness `1` and `value = 0` represents fitness `0`. -/
structure Fitness : Type where
  value : Int
  nonneg : 0 ≤ value
  le_one : value ≤ 65536
deriving DecidableEq

/-- A state of the token game: a marking `M` together with its fitness score. -/
structure SeqTokenState : Type where
  marking : Marking
  fitness : Fitness

end Praxis.Corpus
