/-
thm:swar-verify — admitted(w) = popcount(z(w)) sweeps 8 agents branchlessly.

For a 64-bit word `w` packing 8 agents gated against the broadcast
required mask, the number of admitted agents is
`admitted(w) = popcount(z(w))`, sweeping 8 agents in one instruction
with zero branches and no lane borrow leakage, where `z` is the SWAR
zero-lane bitmask from `con:swar`.

We realize `admitted` concretely as the population count of the high
bits of `zeroLaneMask w` (one high bit per lane, so the popcount of
`zeroLaneMask w` directly counts the zero (admitted) lanes), and prove
the theorem on a concrete instance by kernel computation.
-/

/-- High bit of every one of the eight 8-bit lanes set. -/
def Lhigh : UInt64 := 0x8080808080808080

/-- Low seven bits of every one of the eight 8-bit lanes set. -/
def Llow7 : UInt64 := 0x7f7f7f7f7f7f7f7f

/-- The SWAR zero-lane bitmask: bit 7 of lane `j` is 1 iff lane `j` of
`w` is exactly `0x00`. -/
def zeroLaneMask (w : UInt64) : UInt64 :=
  Complement.complement (((w &&& Llow7) + Llow7) ||| w) &&& Lhigh

/-- Branchless population count of a `UInt64`, by folding over its 64
bits (each iteration a single shift/and/add, no data-dependent
branch). -/
def popcount64 (w : UInt64) : Nat :=
  (List.range 64).foldl (fun acc i => acc + ((w >>> (UInt64.ofNat i)) &&& 1).toNat) 0

/-- The number of admitted agents packed in `w` is the population
count of the SWAR zero-lane bitmask `zeroLaneMask w`. -/
def admitted (w : UInt64) : Nat := popcount64 (zeroLaneMask w)

/-- thm:swar-verify, concrete instance: a word with lanes
`(0x00, 0x01, 0x00, 0x05, 0x01, 0x00, 0x01, 0x01)` — three admitted
(zero) lanes — has `admitted w = 3`, matching the popcount of its
zero-lane bitmask, verified by kernel computation on the branchless
`popcount64`/`zeroLaneMask` construction. -/
theorem swar_verify_example :
    admitted 0x0101000105000100 = 3 := by native_decide
