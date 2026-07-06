import Mathlib.Tactic

/-!
`est:demand` — aggregate admission-demand estimate.

For a population `N ∈ [10^9, 10^12]` agents each re-admitted at rate
`ρ ∈ [1, 10^2] s^-1`, aggregate admission demand `Λ = N * ρ` lies in
`[10^9, 10^14]` decisions/s.

This is a concrete real-number bound, so it is stated as a genuine theorem
(not an asserted axiom) and discharged with `nlinarith` from the two
hypothesis intervals — no Mathlib gap, this is elementary interval
arithmetic on ordered fields.
-/

theorem est_demand (N ρ : ℝ)
    (hN1 : (10:ℝ) ^ 9 ≤ N) (hN2 : N ≤ (10:ℝ) ^ 12)
    (hρ1 : (1:ℝ) ≤ ρ) (hρ2 : ρ ≤ (10:ℝ) ^ 2) :
    (10:ℝ) ^ 9 ≤ N * ρ ∧ N * ρ ≤ (10:ℝ) ^ 14 := by
  have hN0 : (0:ℝ) ≤ N := le_trans (by norm_num) hN1
  have hρ0 : (0:ℝ) ≤ ρ := le_trans (by norm_num) hρ1
  constructor
  · nlinarith [mul_le_mul hN1 hρ1 (by norm_num : (0:ℝ) ≤ 1) hN0]
  · nlinarith [mul_le_mul hN2 hρ2 hρ0 (by norm_num : (0:ℝ) ≤ (10:ℝ) ^ 12)]
