/-
est:sweep — Full admission sweep cost estimate.

A full admission sweep reads all 8 planes once (10 GB) and performs
7N/64 ≈ 1.1×10^9 AND instructions for N = 10^10; at streaming bandwidth
Bw ~ 10^2..4×10^2 GB/s, t_sweep ≈ 10 GB / Bw ≈ 25..100 ms; the
instruction count is exact, the wall-time is an order-of-magnitude
estimate.

This is an `estimate`: the numeric claims are recorded as Nat/rational
quantities and the *exact* instruction-count claim is checked by
kernel computation (`decide`). The wall-time range is a derived
order-of-magnitude estimate and is stated as a bounding inequality on
rationals, not a proof obligation beyond type-checking + one decidable
numeric fact.
-/

/-- Number of admissible-agent slots swept (10^10, as in the thesis). -/
def N : Nat := 10 ^ 10

/-- Bytes read per plane sweep: 8 planes, giving 10 GB total
(the thesis's round figure for the read volume). -/
def bytesRead : Nat := 10 * 1000 * 1000 * 1000

/-- Number of branchless AND instructions for a full sweep: 7 AND-folds
(8 lanes → 7 pairwise ANDs, cf. `admWord` in thm:branchless) applied
per 64-wide word, i.e. `7 * N / 64`. -/
def instrCount : Nat := 7 * N / 64

/-- The exact instruction-count claim of the thesis: `7N/64 ≈ 1.1×10^9`
for `N = 10^10`. We record the exact quotient and check it lands in the
stated order-of-magnitude band `[10^9, 1.2×10^9]`. -/
theorem instrCount_order_of_magnitude :
    1000000000 ≤ instrCount ∧ instrCount ≤ 1200000000 := by
  decide

/-- Streaming bandwidth range, in GB/s: 10^2 to 4×10^2. -/
def bwLow : Nat := 100
def bwHigh : Nat := 400

/-- Sweep wall-time bound, in milliseconds, as `bytesRead / bandwidth`
converted from GB/(GB/s) to ms. At `bwHigh` GB/s the sweep takes
`bytesRead / bwHigh` seconds; at `bwLow` GB/s it takes
`bytesRead / bwLow` seconds. We record both endpoints (in ms) and
check they match the thesis's order-of-magnitude estimate 25–100 ms. -/
def tSweepMsAtBw (bw : Nat) : Nat := (bytesRead * 1000) / (bw * 1000000000)

theorem tSweep_range :
    tSweepMsAtBw bwHigh = 25 ∧ tSweepMsAtBw bwLow = 100 := by
  decide
