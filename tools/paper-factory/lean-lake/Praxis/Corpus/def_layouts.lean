import Mathlib.Tactic

/-!
`def:layouts`: the two competing 64-bit-word memory layouts for a batch of agents.

- Array-of-structures (AoS): each agent's 8-bit byte is stored contiguously; a 64-bit
  word holds 8 agents' bytes (one word = 8 agents x 1 byte each).
- Structure-of-arrays (SoA / bit-plane): each lane `l` is its own bitset `P_l ∈ {0,1}^N`;
  a 64-bit word of `P_l` holds the lane-`l` bit of 64 agents (one word per lane, packing
  64 agents' worth of that single bit).

Both layouts are just different ways of composing `BitVec` words, so no axioms are
needed: `BitVec 8`, `BitVec 64`, and `Fin`-indexed functions from core/Mathlib already
capture the bit-level structure exactly. This mirrors `Praxis/Mathlib/DefReceipt.lean`,
which replaced an opaque `Bits256` axiom with the real `BitVec 256`.
-/

/-- AoS layout: a 64-bit word packs 8 agents, each contributing one 8-bit byte
(one full "structure" per agent, laid out contiguously). -/
structure AoSWord where
  /-- the 8-bit byte belonging to agent `i` (i : Fin 8) inside this word -/
  byte : Fin 8 → BitVec 8

/-- SoA / bit-plane layout: a 64-bit word packs one bit per agent, for a single lane;
64 agents' worth of that lane's bit fit in one `BitVec 64` word. -/
structure SoAWord where
  /-- the bitset `P_l` restricted to this word: the lane-`l` bit of each of the 64 agents -/
  plane : BitVec 64

/-- A lane index `l` picks out one bit-plane `P_l : {0,1}^N`; here we model `P_l` for a
batch of `N` agents as a function from agent index to `Bool`, matching `{0,1}^N`. -/
def BitPlane (N : Nat) : Type := Fin N → Bool

/-- The two named layouts as a closed enumeration of which packing scheme is in use. -/
inductive Layout : Type where
  | AoS
  | SoA
  deriving DecidableEq, Repr, Fintype
