import Mathlib.Tactic
import Praxis.Corpus.meas_cell

/-!
`remark:flatthroughput`: What flat throughput means: the cost of a
supervised fleet tracked its failure novelty, not its failure rate:
recovered transients are absorbed at boundary cost, and the one genuinely
novel pathology (the crash-looping class) was converted -- once, by
quorum -- into a cheap standing refusal. The full novelty-curve-under-faults
re-measurement is receipted as deferred, not claimed.

Design notes on reuse vs. axiomatization:
- This is a `remark`: an interpretive gloss on the already-pinned
  `meas:cell` measurement record, not a new empirical claim or a
  mathematical proposition with fresh content. No proof obligation beyond
  type-checking, matching `axiom`/`definition`/`construction` treatment
  elsewhere in this corpus.
- No new axioms or structures. We reuse `CellMeasurement` (and its pinned
  instance `cell`) from `Praxis.Corpus.meas_cell` directly: the remark's
  two claims -- "throughput flat across fault rates" and "crash-looping
  quarantined at quorum by a fixed epoch" -- are exactly the fields
  `throughputFlatAcrossFaultRates` and `quarantineByEpoch` already recorded
  there. We state them as trivial propositions about the pinned instance
  (`decide`), so the remark is machine-checked to be reading off the
  measurement record it glosses, rather than asserting something new.
- The remark's final sentence ("the full novelty-curve re-measurement is
  receipted as deferred, not claimed") is a scope disclaimer about future
  work, not a formalizable mathematical statement -- it is recorded here
  only as a doc comment, not as a Lean declaration, since there is no
  well-typed content to check.
-/

/-- The remark's throughput claim: the pinned cell measurement records
    throughput as flat across all measured fault rates. -/
theorem remark_flatthroughput_throughput :
    cell.throughputFlatAcrossFaultRates = true := by decide

/-- The remark's quarantine claim: the pinned cell measurement records the
    crash-looping template as quarantined by epoch 2. -/
theorem remark_flatthroughput_quarantine :
    cell.quarantineByEpoch = 2 := by decide
