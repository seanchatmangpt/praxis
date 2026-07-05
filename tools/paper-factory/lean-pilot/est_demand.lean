/-
Label: est:demand
Kind: estimate

For a population N ∈ [10^9, 10^12] agents each re-admitted at rate ρ ∈ [1, 10^2] s^-1,
aggregate admission demand is Λ_demand = N·ρ ∈ [10^9, 10^14] decisions/s,
central estimate ~10^11–10^12 decisions/s for N ~ 10^10, ρ ~ 10.
-/

/-- Population bounds: N ∈ [10^9, 10^12]. -/
def NLo : Nat := 10 ^ 9
def NHi : Nat := 10 ^ 12

/-- Per-agent re-admission rate bounds: ρ ∈ [1, 10^2] (s^-1, dimension elided). -/
def rhoLo : Nat := 1
def rhoHi : Nat := 10 ^ 2

/-- Aggregate admission demand Λ_demand = N · ρ. -/
def demand (N rho : Nat) : Nat := N * rho

/-- Central-estimate point: N ~ 10^10, ρ ~ 10. -/
def NCentral : Nat := 10 ^ 10
def rhoCentral : Nat := 10

/-- The demand estimate is bracketed: 10^9 ≤ Λ_demand ≤ 10^14 whenever
    N and ρ lie in their stated ranges. -/
theorem demand_bounds (N rho : Nat)
    (hNlo : NLo ≤ N) (hNhi : N ≤ NHi)
    (hrlo : rhoLo ≤ rho) (hrhi : rho ≤ rhoHi) :
    10 ^ 9 ≤ demand N rho ∧ demand N rho ≤ 10 ^ 14 := by
  constructor
  · calc (10:Nat) ^ 9 = NLo * rhoLo := by decide
      _ ≤ N * rho := Nat.mul_le_mul hNlo hrlo
  · calc demand N rho = N * rho := rfl
      _ ≤ NHi * rhoHi := Nat.mul_le_mul hNhi hrhi
      _ = 10 ^ 14 := by decide

/-- The central-estimate values fall in [10^9, 10^14] and equal 10^11 exactly. -/
theorem demand_central : demand NCentral rhoCentral = 10 ^ 11 := by decide
