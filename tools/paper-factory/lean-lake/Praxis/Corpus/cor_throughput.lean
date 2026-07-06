import Praxis.Corpus.est_sweep
import Mathlib.Tactic.NormNum.Basic
import Mathlib.Tactic.NormNum.Pow
import Mathlib.Tactic.NormNum.Inv
import Mathlib.Tactic.NormNum.DivMod

/-!
# cor:throughput — Per-server agent-admission throughput

"A single commodity server admits $N=10^{10}$ agents in $\sim25$--$100\,\mathrm{ms}$,
i.e. at $\sim10^{11}$--$4\times10^{11}$ agent-admission decisions per second per
server, bandwidth-bound."

## Formalisation

This is a direct corollary of `est:sweep`
(`Praxis.Corpus.EstSweep.tSweepAtHighBw_eq`, `tSweepAtLowBw_eq`): dividing the
fixed agent count `N = 10^10` by the two sweep wall-times already proved there
(`25 ms = 1/40 s` and `100 ms = 1/10 s`) gives the throughput range `10^11`–
`4×10^11` agents/second stated in the prose. Nothing new is axiomatized —
`throughput` is *defined* as `N / t`, and both endpoint values follow from
`est:sweep`'s equalities by `norm_num` substitution.
-/

namespace Praxis.Corpus.CorThroughput

open EstSweep

/-- Agent-admission throughput (agents/second) for a sweep taking wall-time
`t` seconds over the fixed `N = 10^10` agent population. -/
noncomputable def throughput (t : ℝ) : ℝ := (10 ^ 10 : ℝ) / t

/-- At the high-bandwidth end (`t = 25 ms = 1/40 s`), throughput is
`4 × 10^11` agent-admission decisions per second. -/
theorem throughput_at_high_bw :
    throughput tSweepAtHighBw = 4 * 10 ^ 11 := by
  unfold throughput
  rw [tSweepAtHighBw_eq]
  norm_num

/-- At the low-bandwidth end (`t = 100 ms = 1/10 s`), throughput is
`10^11` agent-admission decisions per second. -/
theorem throughput_at_low_bw :
    throughput tSweepAtLowBw = 10 ^ 11 := by
  unfold throughput
  rw [tSweepAtLowBw_eq]
  norm_num

/-- The stated throughput range: at the sweep-time extremes proved in
`est:sweep`, per-server agent-admission throughput lies in
`[10^11, 4×10^11]` decisions/second, matching the prose's
`~10^11`–`4×10^11`. -/
theorem throughput_range :
    throughput tSweepAtLowBw = 10 ^ 11 ∧
    throughput tSweepAtHighBw = 4 * 10 ^ 11 ∧
    throughput tSweepAtLowBw ≤ throughput tSweepAtHighBw := by
  refine ⟨throughput_at_low_bw, throughput_at_high_bw, ?_⟩
  rw [throughput_at_low_bw, throughput_at_high_bw]
  norm_num

end Praxis.Corpus.CorThroughput
