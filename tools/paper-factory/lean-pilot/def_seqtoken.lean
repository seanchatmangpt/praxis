/- def:seqtoken
   The lifecycle process is modeled as a safe Petri net (3-Node SEQ POWL
   token game) over the marking
     M = (TOK_START, TOK_JUDGED, TOK_ADMITTED, TOK_DONE),
   with fitness Fitness ∈ [0,1] in Q16.16.

   We model the four places of the safe Petri net as an inductive type
   `Place`, a marking as a function `Place → Bool` (safe net: at most one
   token per place), and the fitness value as a Q16.16 fixed-point number,
   represented as a natural number `num` together with a proof that
   `num ≤ 65536` (the fixed-point encoding of the closed interval [0,1]
   with denominator 2^16 = 65536). -/

/-- The four places of the 3-Node SEQ POWL token game. -/
inductive Place where
  | TOK_START    : Place
  | TOK_JUDGED   : Place
  | TOK_ADMITTED : Place
  | TOK_DONE     : Place

open Place

/-- A marking of the safe Petri net: for each place, whether it currently
    holds a token (safe net ⇒ capacity 1 per place). -/
def Marking := Place → Bool

/-- The initial marking: only `TOK_START` is occupied. -/
def Marking.start : Marking
  | TOK_START    => true
  | TOK_JUDGED   => false
  | TOK_ADMITTED => false
  | TOK_DONE     => false

/-- Q16.16 fixed-point fitness value in the closed interval [0,1],
    represented as a numerator over the fixed denominator 2^16 = 65536. -/
structure Fitness where
  num  : Nat
  le   : num ≤ 65536

/-- Fitness 0 (the bottom of the [0,1] range). -/
def Fitness.zero : Fitness := ⟨0, Nat.zero_le _⟩

/-- Fitness 1 (the top of the [0,1] range, i.e. 65536 / 65536). -/
def Fitness.one : Fitness := ⟨65536, Nat.le_refl _⟩

/-- The full state of the token game: a marking together with its
    associated fitness value. -/
structure SeqToken where
  marking : Marking
  fitness : Fitness

/-- The canonical initial state of the 3-Node SEQ POWL token game. -/
def SeqToken.init : SeqToken := ⟨Marking.start, Fitness.zero⟩
