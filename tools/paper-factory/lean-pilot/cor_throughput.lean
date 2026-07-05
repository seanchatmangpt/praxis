/-
cor:throughput — Admission-decision throughput per server.

A single commodity server admits N = 10^10 agents in ~25--100 ms
(est:sweep), i.e. at ~10^11--4×10^11 agent-admission decisions per
second per server, bandwidth-bound.

This corollary is derived directly from est:sweep's `tSweepMsAtBw`
figures: throughput (decisions/sec) = N * 1000 / t_ms. We reuse `N`,
`bwLow`, `bwHigh`, `tSweepMsAtBw` from est:sweep and prove the
resulting rate lands exactly on the thesis's stated endpoints
10^11 and 4×10^11 per second.
-/

/-- Number of admissible-agent slots swept (10^10, as in the thesis). -/
def N : Nat := 10 ^ 10

/-- Bytes read per plane sweep: 8 planes, giving 10 GB total. -/
def bytesRead : Nat := 10 * 1000 * 1000 * 1000

/-- Streaming bandwidth range, in GB/s. -/
def bwLow : Nat := 100
def bwHigh : Nat := 400

/-- Sweep wall-time bound, in milliseconds, at a given bandwidth. -/
def tSweepMsAtBw (bw : Nat) : Nat := (bytesRead * 1000) / (bw * 1000000000)

/-- Admission-decision throughput, in decisions per second, given a
sweep wall-time in milliseconds: N agents admitted per t_ms
milliseconds, converted to a per-second rate. -/
def throughputPerSec (tMs : Nat) : Nat := (N * 1000) / tMs

/-- At the high-bandwidth endpoint (400 GB/s, t = 25 ms), throughput
is 4×10^11 decisions/sec; at the low-bandwidth endpoint (100 GB/s,
t = 100 ms), throughput is 10^11 decisions/sec — exactly the range
the thesis states. -/
theorem throughput_range :
    throughputPerSec (tSweepMsAtBw bwHigh) = 4 * 10 ^ 11 ∧
    throughputPerSec (tSweepMsAtBw bwLow) = 10 ^ 11 := by
  decide
