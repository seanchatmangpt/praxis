/-
prop:footprint — Fleet byte-footprint identity.

A population of `N = 10^10` agents, each an 8-bit byte, occupies
`N × 8` bits = `N` bytes = `10^10` bytes = `10 GB` in either layout;
this is an exact arithmetic identity, not an estimate.

We state it over `Nat`: with `N := 10 ^ 10` agents and 8 bits per
agent's status byte (`con:agent8`'s `StatusByte`, one `Bool` per
`Lane`, 8 lanes), the total bit count `N * 8` divided back down by 8
recovers exactly `N` bytes, and that byte count is exactly `10`
gigabytes when a gigabyte is taken as `10^9` bytes.
-/

/-- The fleet size from `con:agent8`'s worked example: ten billion agents. -/
def NAgents : Nat := 10 ^ 10

/-- Bits per agent: one bit per lane of `con:agent8`'s 8-lane `StatusByte`. -/
def BitsPerAgent : Nat := 8

/-- One gigabyte, in bytes. -/
def GB : Nat := 10 ^ 9

/-- The footprint identity: `N` agents at 8 bits each occupy exactly
`N` bytes (bit count divides evenly back down to the agent count),
and that byte count is exactly `10` gigabytes — an exact arithmetic
identity, not an estimate. -/
theorem footprint :
    NAgents * BitsPerAgent / BitsPerAgent = NAgents ∧
    NAgents = 10 * GB := by
  constructor
  · decide
  · decide
