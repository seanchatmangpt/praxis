/-
  est:comp-supply

  A comprehension-based admission decision costs ~10^-2 -- 10^0 s of accelerator
  time, i.e. ~1 -- 10^2 decisions/s per accelerator; a planetary fleet of
  ~10^6 -- 10^7 accelerators yields a comprehension-supply ceiling
  Λ_comp ≲ 10^7 -- 10^9 decisions/s, central estimate ~10^8 decisions/s.

  This is an order-of-magnitude engineering estimate, not a theorem: we
  formalize it as a definition recording the stated bounds (per-accelerator
  decision rate, fleet size, resulting throughput range, and central point
  estimate), and check by `decide` that the recorded numbers are internally
  consistent (each range is nonempty, low ≤ high, and the central estimate
  lies within the derived throughput range).
-/

/-- Per-accelerator decision-rate range, in decisions/s: [10^0, 10^2]. -/
def perAcceleratorRateLow  : Nat := 1
def perAcceleratorRateHigh : Nat := 10 ^ 2

/-- Planetary accelerator-fleet size range: [10^6, 10^7]. -/
def fleetSizeLow  : Nat := 10 ^ 6
def fleetSizeHigh : Nat := 10 ^ 7

/-- Comprehension-supply ceiling Λ_comp, in decisions/s: [10^7, 10^9]. -/
def lambdaCompLow  : Nat := 10 ^ 7
def lambdaCompHigh : Nat := 10 ^ 9

/-- Central estimate: ~10^8 decisions/s. -/
def lambdaCompCentral : Nat := 10 ^ 8

/-- Sanity: the per-accelerator rate range is nonempty and ordered. -/
theorem perAcceleratorRate_ordered :
    perAcceleratorRateLow ≤ perAcceleratorRateHigh := by decide

/-- Sanity: the fleet-size range is nonempty and ordered. -/
theorem fleetSize_ordered :
    fleetSizeLow ≤ fleetSizeHigh := by decide

/-- Sanity: the throughput ceiling range is nonempty and ordered. -/
theorem lambdaComp_ordered :
    lambdaCompLow ≤ lambdaCompHigh := by decide

/-- Sanity: the low end of the throughput range is at least the product of
    the low ends of the per-accelerator rate and fleet size (10^6 · 1 = 10^6
    ≤ 10^7), i.e. the stated ceiling is not weaker than the naive low-low
    product. -/
theorem lambdaCompLow_dominates_naive_low :
    perAcceleratorRateLow * fleetSizeLow ≤ lambdaCompLow := by decide

/-- Sanity: the high end of the throughput range is at least the product of
    the high ends of the per-accelerator rate and fleet size
    (10^2 · 10^7 = 10^9 ≤ 10^9). -/
theorem lambdaCompHigh_matches_naive_high :
    perAcceleratorRateHigh * fleetSizeHigh ≤ lambdaCompHigh := by decide

/-- Sanity: the central estimate lies within the stated throughput ceiling
    range. -/
theorem lambdaCompCentral_in_range :
    lambdaCompLow ≤ lambdaCompCentral ∧ lambdaCompCentral ≤ lambdaCompHigh := by
  decide
