/- def:layouts
The array-of-structures (AoS) layout stores each agent's 8-bit byte contiguously,
a 64-bit word holding 8 agents' bytes; the structure-of-arrays (SoA / bit-plane)
layout stores each lane ℓ as its own bitset P_ℓ ∈ {0,1}^N, a 64-bit word of P_ℓ
holding the lane-ℓ bit of 64 agents. -/

-- An agent's state is 8 lanes (bits), each lane a Bool.
def Agent := Fin 8 → Bool

-- AoS layout: each 64-bit word packs 8 agents' worth of bytes contiguously.
structure AoSWord where
  agents : Fin 8 → Agent

-- SoA / bit-plane layout: each lane ℓ has its own bitset P_ℓ over N agents;
-- a 64-bit word of P_ℓ packs the lane-ℓ bit of 64 agents.
def Bitset (N : Nat) := Fin N → Bool

structure SoAWord where
  lane : Fin 8 → Bitset 64
