/-
Label: cor:onlybit
Kind: corollary

A planetary control plane that re-admits its population continuously
cannot be built on comprehension: no attainable accelerator fleet closes
the demand gap; it can be built on the sweep (bit-parallel admission),
whose cost is dominated by a single pass; comprehension is reserved for
sampled audit.

We reuse thm:afford's central estimates and its two proven facts:
  comp_underprovisioned : lambdaCompCentral * 10^3 ≤ demandCentral
  bit_meets_demand      : demandCentral ≤ serverSupply
and draw the corollary that comprehension cannot be scaled up (even by an
attainable 10^2-fold accelerator fleet) to close the gap, while the
bit-parallel path already meets demand on a single facility.
-/

def lambdaCompCentral : Nat := 10 ^ 8
def NCentral : Nat := 10 ^ 10
def rhoCentral : Nat := 10
def demand (N rho : Nat) : Nat := N * rho
def demandCentral : Nat := demand NCentral rhoCentral
def serverSupply : Nat := 10 ^ 11

theorem comp_underprovisioned :
    lambdaCompCentral * 10 ^ 3 ≤ demandCentral := by decide

theorem bit_meets_demand :
    demandCentral ≤ serverSupply := by decide

/-- An "attainable" comprehension accelerator fleet is capped at a
    100-fold speed-up over the central comprehension rate. -/
def attainableCompFleet : Nat := lambdaCompCentral * 10 ^ 2

/-- Corollary: even an attainable (100x) comprehension accelerator fleet
    still fails to close the demand gap, whereas bit-parallel admission
    already meets demand on a single facility — so only the bit-parallel
    (sweep) construction is viable. -/
theorem onlybit :
    attainableCompFleet < demandCentral ∧ demandCentral ≤ serverSupply := by
  constructor
  · show lambdaCompCentral * 10 ^ 2 < demandCentral
    decide
  · exact bit_meets_demand
