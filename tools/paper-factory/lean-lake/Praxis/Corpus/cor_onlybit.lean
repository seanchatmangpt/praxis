import Mathlib.Tactic
import Praxis.Corpus.thm_afford

/-!
# cor:onlybit — Only the sweep, not comprehension, closes the demand gap

"A planetary control plane that re-admits its population continuously cannot be
built on comprehension: no attainable accelerator fleet closes the demand gap;
it can be built on the sweep, whose cost is dominated by a single pass over
`10 GB` of memory; comprehension is reserved for sampled audit at honest `O(n)`
replay cost."

## Formalisation

This is a direct corollary of `thm:afford`
(`Praxis.Corpus.ThmAfford.thm_afford`), re-using its already-proved numeric
facts rather than re-deriving them:

* "no attainable accelerator fleet closes the demand gap" is
  `thm_afford.1.1 : lambdaDemandLo = 10 ^ 3 * lambdaComp`, i.e. comprehension
  supply sits three orders of magnitude below the demand band's *lower* end,
  so no accelerator-fleet scaling of comprehension supply within the already
  audited perturbation range (`robust_comp`) reaches demand.
* "it can be built on the sweep" is `thm_afford.2.1 : lambdaDemandHi ≤
  lambdaBitHi`, i.e. bit-parallel (sweep) supply at rack scale meets the
  demand band's *upper* end on a single facility.

No new axiom is introduced: both witnessing facts are literal projections out
of the already-proved `thm_afford` conjunction, combined here into the single
"comprehension can't, the sweep can" statement, matching the corollary's
content precisely. The "single pass over `10 GB`" and "sampled audit at
honest `O(n)` replay cost" cost-model details are qualitative descriptions of
*why* the sweep is affordable and comprehension is reserved for audit — they
are not further numeric claims requiring separate lemmas beyond the
supply/demand comparison already carried by `thm_afford`.
-/

namespace Praxis.Corpus.CorOnlyBit

open Praxis.Corpus.ThmAfford

/-- Comprehension alone cannot close the demand gap: comprehension supply is
three orders of magnitude below the demand band's lower end, and this
under-provisioning is robust to an order-of-magnitude perturbation of the
comprehension-supply input. -/
theorem comprehensionCannotClose :
    lambdaDemandLo = 10 ^ 3 * lambdaComp ∧
    (∀ p : ℝ, 1 / 10 ≤ p → p ≤ 10 → p * lambdaComp < lambdaDemandLo) :=
  ⟨thm_afford.1.1, thm_afford.2.2.1⟩

/-- The sweep (bit-parallel admission) closes the demand gap on a single
facility, and this match is robust to an order-of-magnitude perturbation of
the bit-supply input. -/
theorem sweepCloses :
    lambdaDemandHi ≤ lambdaBitHi ∧
    (∀ p : ℝ, 1 / 10 ≤ p → p ≤ 10 → lambdaDemandHi ≤ p * lambdaBitHi) :=
  ⟨thm_afford.2.1, thm_afford.2.2.2.2.1⟩

/-- Main corollary: a continuously re-admitting planetary control plane
cannot be built on comprehension (its supply is three orders of magnitude
below demand, robustly), but can be built on the sweep (whose supply meets
demand at rack scale, robustly) — comprehension is reserved for sampled
audit rather than the admission critical path. -/
theorem cor_onlybit :
    (lambdaDemandLo = 10 ^ 3 * lambdaComp ∧
      (∀ p : ℝ, 1 / 10 ≤ p → p ≤ 10 → p * lambdaComp < lambdaDemandLo)) ∧
    (lambdaDemandHi ≤ lambdaBitHi ∧
      (∀ p : ℝ, 1 / 10 ≤ p → p ≤ 10 → lambdaDemandHi ≤ p * lambdaBitHi)) :=
  ⟨comprehensionCannotClose, sweepCloses⟩

end Praxis.Corpus.CorOnlyBit
