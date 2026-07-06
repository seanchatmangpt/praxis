import Praxis.Corpus.con_agent8

/-!
Label: prop:footprint

"A population of $N=10^{10}$ agents, each an 8-bit byte, occupies $N\times8$
bits $=N$ bytes $=10^{10}$ bytes $=10\,\mathrm{GB}$ in either layout; this is
an exact arithmetic identity, not an estimate."

This is a `proposition`: a real arithmetic identity about the fleet
population size `N` from `con:agent8` (`Agent8.Fleet N`, `N` agents each
carrying one `StatusByte`, i.e. one 8-bit byte). No new axioms are needed --
the claim is pure `Nat` arithmetic (`N * 8` bits `= N` bytes, and
`N = 10^10` bytes `= 10` GB under the exact byte-per-`10^9` convention the
LaTeX uses), discharged by `decide`/`norm_num` on concrete numerals, matching
"an exact arithmetic identity, not an estimate."
-/

namespace Agent8

/-- The population size from the LaTeX: `N = 10^10` agents. -/
def N : Nat := 10 ^ 10

/-- Each agent is one `StatusByte` (`con:agent8`), i.e. one 8-bit byte, so a
fleet `Agent8.Fleet N` occupies exactly `N * 8` bits. This is definitional
bookkeeping: the bit count of a fleet of `N` agents, each an 8-bit byte. -/
def fleetBits (n : Nat) : Nat := n * 8

/-- The exact arithmetic identity: for `N = 10^10` agents each an 8-bit byte,
the bit count `N × 8` divided back down by 8 bits/byte recovers exactly `N`
bytes, and that byte count is exactly `10^10` bytes `= 10` GB (at `10^9`
bytes/GB, the exact decimal convention the LaTeX statement uses). Every
conjunct is a closed numeral computation, decided by `decide`. -/
theorem footprint :
    fleetBits N / 8 = N ∧ N = 10 ^ 10 ∧ N / 10 ^ 9 = 10 := by
  refine ⟨?_, rfl, ?_⟩ <;> decide

end Agent8
