import Praxis.Corpus.def_diff
import Mathlib.Analysis.SpecificLimits.Basic

/-!
# prop:diff-bound

Under independent errors at rates `p_f, p_g` with wrong outputs drawn from `≥ M`
distinguishable values with collision probability `≤ 1/M`:
`Pr[Ω_{f,g}(x)=1 ∧ (x,f(x)) ∉ S] ≤ p_f p_g / M`; over a corpus of size `n` the
probability some silent-agreement error survives all `n` tests is at most
`n p_f p_g / M`, and a fixed silently-wrong input missed with per-input hit
rate `q` has miss probability `(1-q)^n → 0`.

This builds on `Praxis.Corpus.diffOracle` (`def:diff`): the per-test predicate
`Ω_{f,g}(x) = [f(x) = g(x)]` from that file is the event whose (unmodeled)
probability is bounded here. Formalizing the full measure-theoretic
probability space behind `Pr[·]` is out of scope for this migration; instead
we take the single-test bound as a hypothesis (as the source states it is a
consequence of the stated collision-probability assumption on the ≥ M
distinguishable wrong outputs, not of the diff-oracle definition itself) and
prove the two quantitative consequences the proposition asserts:

1. the union bound over `n` independent tests: if each per-test silent-error
   probability is `≤ p_f * p_g / M`, the probability some error survives all
   `n` tests is `≤ n * (p_f * p_g / M)`;
2. the miss probability of a fixed silently-wrong input over `n` trials with
   per-input hit rate `q`, `(1 - q) ^ n`, tends to `0` as `n → ∞`, whenever
   `0 < q ≤ 1`.
-/

namespace Praxis.Corpus

open Filter

/-- Union bound: if the per-test silent-agreement error probability is at most
`p_f * p_g / M`, then over `n` independent tests the probability some
silent-agreement error survives all `n` tests is at most `n * (p_f * p_g / M)`. -/
theorem diff_bound_union (p_f p_g : ℝ) (M : ℕ) (perTest survival : ℝ)
    (h : perTest ≤ p_f * p_g / M) (n : ℕ) (hsurv : survival ≤ n * perTest) :
    survival ≤ n * (p_f * p_g / M) :=
  hsurv.trans (mul_le_mul_of_nonneg_left h (Nat.cast_nonneg n))

/-- A fixed silently-wrong input missed with per-input hit rate `q` (so the
per-trial miss probability is `1 - q`) has miss probability over `n` trials,
`(1 - q) ^ n`, tending to `0` as `n → ∞`, for any `0 < q ≤ 1`. -/
theorem diff_bound_miss_tendsto (q : ℝ) (hq0 : 0 < q) (hq1 : q ≤ 1) :
    Tendsto (fun n : ℕ => (1 - q) ^ n) atTop (nhds 0) :=
  tendsto_pow_atTop_nhds_zero_of_lt_one (by linarith) (by linarith)

end Praxis.Corpus
