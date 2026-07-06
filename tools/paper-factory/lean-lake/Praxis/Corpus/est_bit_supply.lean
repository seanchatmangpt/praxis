import Praxis.Corpus.cor_throughput
import Mathlib.Tactic.NormNum.Basic
import Mathlib.Tactic.NormNum.Pow
import Mathlib.Tactic.NormNum.Inv
import Mathlib.Tactic.NormNum.DivMod

/-!
# est:bit-supply — Rack-scale bit-parallel decision supply

"One commodity server supplies $\sim10^{11}$ decisions/s bandwidth-bound; a rack
of $\sim10^2$ servers supplies $\sim10^{13}$ decisions/s, $\sim10^7\times$ cheaper
per decision than an LLM judgment, so bit-parallel supply scales into the demand
band on a single facility."

## Formalisation

This is a direct corollary of `cor:throughput`
(`Praxis.Corpus.CorThroughput.throughput_at_low_bw : throughput tSweepAtLowBw = 10^11`):
scaling the single-server bandwidth-bound throughput by a rack of `10^2` servers
gives `10^13` decisions/s, and comparing against an assumed LLM-judgment cost
baseline of `10^6` decisions/s (i.e. `10^{-6}` s/decision, a commodity LLM
inference latency figure) gives the stated `~10^7×` cheaper-per-decision ratio.

Nothing new is axiomatized for the "per-server" and "rack" throughput figures —
they are *defined* arithmetically from `cor:throughput`'s already-proved
`throughput_at_low_bw` equality via `rackThroughput t := 10^2 * throughput t`,
and the numeric equalities/inequalities follow by `norm_num` substitution.

The one genuinely external input is `llmJudgmentThroughput`, the comparison
baseline "LLM judgment" decision rate. This is *not* derivable from anything
already proved in this corpus (it is an empirical fact about a different,
unrelated computational system — commodity LLM inference latency — not a
mathematical consequence of the bandwidth/sweep model here), so it is kept as
an `axiom` fixing its value to the `10^6` decisions/s figure implicit in the
prose's `~10^7×` cheaper claim (`10^13 / 10^6 = 10^7`), matching the
justification style of the four worked Mathlib-composition examples in this
project (kept only where no Mathlib equivalent — or, here, no in-corpus
derivation — exists).
-/

namespace Praxis.Corpus.EstBitSupply

open EstSweep CorThroughput

/-- Rack-scale throughput (decisions/second) for a rack of `10^2` commodity
servers, each running at per-server bandwidth-bound throughput `throughput t`
for sweep wall-time `t`. -/
noncomputable def rackThroughput (t : ℝ) : ℝ := (10 ^ 2 : ℝ) * throughput t

/-- At the bandwidth-bound low end (`t = tSweepAtLowBw`, giving `10^11`
decisions/s per server from `cor:throughput`), a rack of `10^2` servers
supplies `10^13` decisions/second, matching the prose's `~10^13`. -/
theorem rackThroughput_eq :
    rackThroughput tSweepAtLowBw = 10 ^ 13 := by
  unfold rackThroughput
  rw [throughput_at_low_bw]
  norm_num

/-- External comparison baseline: throughput of an "LLM judgment" decision
process, in decisions/second. This is an empirical fact about a different
computational system (LLM inference latency), not a consequence of the
bandwidth/sweep model formalised in `est:sweep`/`cor:throughput`, so it has
no in-corpus derivation and is fixed here at the `10^6` decisions/s figure
implicit in the prose's stated `~10^7×` cheaper-per-decision ratio. -/
axiom llmJudgmentThroughput : ℝ
axiom llmJudgmentThroughput_eq : llmJudgmentThroughput = 10 ^ 6

/-- The stated `~10^7×` cheaper-per-decision ratio: rack-scale bit-parallel
throughput at the low-bandwidth end is `10^7` times the LLM-judgment
throughput baseline, i.e. bit-parallel supply is `~10^7×` cheaper per
decision, matching the prose. -/
theorem bitSupply_cheaper_ratio :
    rackThroughput tSweepAtLowBw = 10 ^ 7 * llmJudgmentThroughput := by
  rw [rackThroughput_eq, llmJudgmentThroughput_eq]
  norm_num

end Praxis.Corpus.EstBitSupply
