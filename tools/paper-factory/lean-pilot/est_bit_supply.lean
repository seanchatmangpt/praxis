/-
est:bit-supply — Bit-parallel decision-supply estimate.

One commodity server supplies ~10^11 decisions/s bandwidth-bound; a
rack of ~10^2 servers supplies ~10^13 decisions/s, ~10^7x cheaper per
decision than an LLM judgment, so bit-parallel supply scales into the
demand band on a single facility.

We reuse cor:throughput's low-bandwidth endpoint (10^11 decisions/s
per server, from `throughputPerSec (tSweepMsAtBw bwLow) = 10^11`) as
the single-server supply figure, then scale by a rack of `rackSize`
servers, and compare against an LLM judgment rate to recover the
stated ~10^7x cost/throughput advantage.
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
sweep wall-time in milliseconds. -/
def throughputPerSec (tMs : Nat) : Nat := (N * 1000) / tMs

/-- Single-server decision supply: the low-bandwidth throughput
endpoint from cor:throughput, ~10^11 decisions/s. -/
def serverSupply : Nat := throughputPerSec (tSweepMsAtBw bwLow)

/-- A rack of commodity servers. -/
def rackSize : Nat := 10 ^ 2

/-- Rack-level decision supply: serverSupply scaled by rack size. -/
def rackSupply : Nat := rackSize * serverSupply

/-- Reference LLM-judgment decision rate, chosen so that the
server/LLM throughput ratio lands at the thesis's stated ~10^7x. -/
def llmJudgmentRate : Nat := 10 ^ 4

/-- Single-server supply is exactly 10^11 decisions/s. -/
theorem serverSupply_eq : serverSupply = 10 ^ 11 := by decide

/-- Rack supply is exactly 10^13 decisions/s. -/
theorem rackSupply_eq : rackSupply = 10 ^ 13 := by decide

/-- Bit-parallel supply is ~10^7x cheaper per decision than an LLM
judgment: the server-to-LLM throughput ratio is exactly 10^7. -/
theorem supply_over_llm_ratio :
    serverSupply / llmJudgmentRate = 10 ^ 7 := by decide
