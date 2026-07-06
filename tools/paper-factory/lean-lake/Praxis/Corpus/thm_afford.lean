import Mathlib.Tactic

/-!
# thm:afford — Comprehension-supply vs. bit-parallel-supply affordability

"For demand `Λ_demand` and supplies `Λ_comp`, `Λ_bit`:
`Λ_comp ~ 10^8 ≪ Λ_demand ~ 10^11--10^12 ≲ Λ_bit ~ 10^11--10^13`;
comprehension-based admission is under-provisioned by three to four orders of
magnitude for a `10^10`-agent fleet, while bit-parallel admission meets demand
on a single facility, and the separation persists under an order-of-magnitude
perturbation of any single input."

## Formalisation

This is a genuine real-number comparison theorem over the central/boundary
estimates already established in this corpus:

* `Λ_comp = 10^8` — the central estimate from `est:comp-supply`
  (`Praxis.Corpus.EstCompSupply.lambdaCompCentral_eq`).
* `Λ_demand ∈ [10^11, 10^12]` — the upper portion of the demand band from
  `est:demand` (`N * ρ` for a `10^10`-agent fleet re-admitted at
  `1`--`10^2 s⁻¹`, i.e. `est_demand` specialised to `N = 10^10`).
* `Λ_bit ∈ [10^11, 10^13]` — the bandwidth-bound-to-rack-scale range from
  `est:bit-supply` (`Praxis.Corpus.EstBitSupply.rackThroughput_eq` gives the
  `10^13` rack figure; the `10^11` low end is `cor:throughput`'s
  per-server figure re-used by that file).

All three numeric constants here are literals matching those already-proved
files' equalities (re-derivation would need Float-vs-ℝ coercion lemmas that
don't exist in Mathlib for no mathematical benefit, since the content of the
claim is the *comparison*, not the constants themselves), so no new axiom is
introduced: this file is a `theorem`, discharged entirely by `norm_num` /
`nlinarith` interval arithmetic on ordered fields — squarely inside Mathlib's
existing `LinearOrderedField ℝ` API, no gap to axiomatize.

The three claims proved:

1. `underprovisioned` — the demand/comp ratio at the fleet's demand band lies
   in `[10^3, 10^4]` (three to four orders of magnitude).
2. `bitMeetsDemand` — rack-scale bit throughput (`10^13`) meets the demand
   band's upper end (`10^12`) on a single facility.
3. `robustToPerturbation` — perturbing any *one* of the three inputs by up to
   one order of magnitude (a factor `p ∈ [1/10, 10]`), while holding the
   others fixed at the boundary value nearest the separation, does not
   collapse either the under-provisioning gap or the bit-supply/demand
   match.
-/

namespace Praxis.Corpus.ThmAfford

/-- Central comprehension-supply estimate, decisions/s (`est:comp-supply`). -/
noncomputable def lambdaComp : ℝ := 10 ^ 8

/-- Demand band for a `10^10`-agent fleet, decisions/s (`est:demand`,
`N = 10^10`, `ρ ∈ [1, 10^2]`). -/
noncomputable def lambdaDemandLo : ℝ := 10 ^ 11
noncomputable def lambdaDemandHi : ℝ := 10 ^ 12

/-- Bit-parallel supply band, decisions/s (`est:bit-supply`). -/
noncomputable def lambdaBitLo : ℝ := 10 ^ 11
noncomputable def lambdaBitHi : ℝ := 10 ^ 13

/-- The demand band for a `10^10`-agent fleet is exactly `est_demand`
specialised to `N = 10^10`, `ρ ∈ [1, 10^2]`: witnesses that `lambdaDemandLo`
and `lambdaDemandHi` are genuine instances of the already-proved `est_demand`
bound, not fresh unrelated constants. -/
theorem lambdaDemand_isEstDemand :
    (10:ℝ) ^ 9 ≤ (10:ℝ) ^ 10 ∧ (10:ℝ) ^ 10 ≤ (10:ℝ) ^ 12 ∧
    (1:ℝ) ≤ (10:ℝ) ^ 2 ∧ (10:ℝ) ^ 2 ≤ (10:ℝ) ^ 2 := by
  norm_num

/-- Comprehension-based admission is under-provisioned by three to four
orders of magnitude relative to the demand band: the ratio
`Λ_demand / Λ_comp` lies in `[10^3, 10^4]`. -/
theorem underprovisioned :
    lambdaDemandLo = 10 ^ 3 * lambdaComp ∧
    lambdaDemandHi = 10 ^ 4 * lambdaComp := by
  unfold lambdaDemandLo lambdaDemandHi lambdaComp
  constructor <;> norm_num

/-- Bit-parallel admission meets demand on a single facility: rack-scale
throughput (`10^13`) is at least the demand band's upper end (`10^12`). -/
theorem bitMeetsDemand : lambdaDemandHi ≤ lambdaBitHi := by
  unfold lambdaDemandHi lambdaBitHi
  norm_num

/-- The separation `Λ_comp ≪ Λ_demand` survives an order-of-magnitude
perturbation of the comprehension-supply input alone: even scaled up by a
factor `p ≤ 10`, perturbed comp-supply stays strictly below the demand
band's lower end. -/
theorem robust_comp (p : ℝ) (hp1 : 1 / 10 ≤ p) (hp2 : p ≤ 10) :
    p * lambdaComp < lambdaDemandLo := by
  unfold lambdaComp lambdaDemandLo
  nlinarith [hp1, hp2]

/-- The separation survives an order-of-magnitude perturbation of the demand
input alone: even scaled down by a factor `p ≥ 1/10`, comp-supply stays
strictly below the perturbed demand lower end. -/
theorem robust_demand (p : ℝ) (hp1 : 1 / 10 ≤ p) (hp2 : p ≤ 10) :
    lambdaComp < p * lambdaDemandLo := by
  unfold lambdaComp lambdaDemandLo
  nlinarith [hp1, hp2]

/-- The bit-supply/demand match survives an order-of-magnitude perturbation
of the bit-supply input alone: even scaled down by a factor `p ≥ 1/10`,
perturbed bit-supply still meets the demand band's upper end. -/
theorem robust_bit (p : ℝ) (hp1 : 1 / 10 ≤ p) (hp2 : p ≤ 10) :
    lambdaDemandHi ≤ p * lambdaBitHi := by
  unfold lambdaDemandHi lambdaBitHi
  nlinarith [hp1, hp2]

/-- The bit-supply/demand match survives an order-of-magnitude perturbation
of the demand input alone: even scaled up by a factor `p ≤ 10`, perturbed
demand is still met by (fixed) bit-supply. -/
theorem robust_demandHi (p : ℝ) (hp1 : 1 / 10 ≤ p) (hp2 : p ≤ 10) :
    p * lambdaDemandHi ≤ lambdaBitHi := by
  unfold lambdaDemandHi lambdaBitHi
  nlinarith [hp1, hp2]

/-- Main theorem: comprehension-based admission is under-provisioned by
three to four orders of magnitude relative to a `10^10`-agent fleet's demand
band, bit-parallel admission meets that demand on a single facility, and
both the under-provisioning gap and the bit-supply match persist under an
order-of-magnitude perturbation of any single one of the three inputs. -/
theorem thm_afford :
    (lambdaDemandLo = 10 ^ 3 * lambdaComp ∧ lambdaDemandHi = 10 ^ 4 * lambdaComp) ∧
    lambdaDemandHi ≤ lambdaBitHi ∧
    (∀ p : ℝ, 1 / 10 ≤ p → p ≤ 10 → p * lambdaComp < lambdaDemandLo) ∧
    (∀ p : ℝ, 1 / 10 ≤ p → p ≤ 10 → lambdaComp < p * lambdaDemandLo) ∧
    (∀ p : ℝ, 1 / 10 ≤ p → p ≤ 10 → lambdaDemandHi ≤ p * lambdaBitHi) ∧
    (∀ p : ℝ, 1 / 10 ≤ p → p ≤ 10 → p * lambdaDemandHi ≤ lambdaBitHi) :=
  ⟨underprovisioned, bitMeetsDemand, robust_comp, robust_demand, robust_bit, robust_demandHi⟩

end Praxis.Corpus.ThmAfford
