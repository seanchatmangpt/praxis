/-
Label: thm:afford
Kind: theorem

For demand Λ_demand and supplies Λ_comp, Λ_bit: Λ_comp ~ 10^8 ≪ Λ_demand ~
10^11–10^12 ≲ Λ_bit ~ 10^11–10^13; comprehension-based admission is
under-provisioned by three to four orders of magnitude for a 10^10-agent
fleet, while bit-parallel admission meets demand on a single facility, and
the separation persists under an order-of-magnitude perturbation of any
single input.

We reuse the central-estimate values fixed in the three dependency files
(est:comp-supply's `lambdaCompCentral = 10^8`, est:demand's
`demand NCentral rhoCentral = 10^11`, est:bit-supply's
`serverSupply = 10^11`) and prove:

  (1) Λ_comp is under-provisioned relative to demand by at least three
      orders of magnitude: `lambdaCompCentral * 10^3 ≤ demandCentral`.
  (2) Bit-parallel supply meets demand on a single facility:
      `demandCentral ≤ serverSupply`.
  (3) The separation in (1) survives an order-of-magnitude perturbation of
      any single one of its three inputs (comp rate, fleet size, per-agent
      admission rate), each checked in isolation.
-/

/-- Comprehension-supply central estimate, ~10^8 decisions/s
    (est:comp-supply's `lambdaCompCentral`). -/
def lambdaCompCentral : Nat := 10 ^ 8

/-- Population and per-agent re-admission-rate central point
    (est:demand's `NCentral`, `rhoCentral`). -/
def NCentral : Nat := 10 ^ 10
def rhoCentral : Nat := 10

/-- Aggregate admission demand (est:demand's `demand`). -/
def demand (N rho : Nat) : Nat := N * rho

/-- Demand central estimate, ~10^11 decisions/s. -/
def demandCentral : Nat := demand NCentral rhoCentral

/-- Bit-parallel single-server supply, exactly 10^11 decisions/s
    (est:bit-supply's `serverSupply`). -/
def serverSupply : Nat := 10 ^ 11

/-- (1) Comprehension-based admission is under-provisioned relative to
    demand by at least three orders of magnitude. -/
theorem comp_underprovisioned :
    lambdaCompCentral * 10 ^ 3 ≤ demandCentral := by decide

/-- (2) Bit-parallel admission meets demand on a single facility. -/
theorem bit_meets_demand :
    demandCentral ≤ serverSupply := by decide

/-- Combined separation: Λ_comp ≪ Λ_demand ≲ Λ_bit at the stated central
    estimates, with the comprehension gap at least three orders. -/
theorem afford_separation :
    lambdaCompCentral * 10 ^ 3 ≤ demandCentral ∧ demandCentral ≤ serverSupply :=
  ⟨comp_underprovisioned, bit_meets_demand⟩

/-- (3a) Perturbing the comprehension rate up by one order of magnitude
    still leaves it under-provisioned by at least two orders. -/
theorem comp_underprovisioned_perturbed_rate_up :
    (lambdaCompCentral * 10) * 10 ^ 2 ≤ demandCentral := by decide

/-- (3b) Perturbing the fleet size (N) down by one order of magnitude
    still leaves comprehension under-provisioned by at least two orders. -/
theorem comp_underprovisioned_perturbed_N_down :
    lambdaCompCentral * 10 ^ 2 ≤ demand (NCentral / 10) rhoCentral := by decide

/-- (3c) Perturbing the fleet size (N) up by one order of magnitude
    still leaves comprehension under-provisioned by at least four orders. -/
theorem comp_underprovisioned_perturbed_N_up :
    lambdaCompCentral * 10 ^ 4 ≤ demand (NCentral * 10) rhoCentral := by decide

/-- (3d) Perturbing the per-agent re-admission rate (ρ) down by one order
    of magnitude still leaves comprehension under-provisioned by at least
    two orders. -/
theorem comp_underprovisioned_perturbed_rho_down :
    lambdaCompCentral * 10 ^ 2 ≤ demand NCentral (rhoCentral / 10) := by decide

/-- (3e) Perturbing the per-agent re-admission rate (ρ) up by one order of
    magnitude still leaves comprehension under-provisioned by at least
    four orders. -/
theorem comp_underprovisioned_perturbed_rho_up :
    lambdaCompCentral * 10 ^ 4 ≤ demand NCentral (rhoCentral * 10) := by decide
