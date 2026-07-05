-- remark:flatthroughput
-- What flat throughput means: the cost of a supervised fleet tracked its
-- failure novelty, not its failure rate: recovered transients are absorbed
-- at boundary cost, and the one genuinely novel pathology (the
-- crash-looping class) was converted --- once, by quorum --- into a cheap
-- standing refusal. The full novelty-curve-under-faults re-measurement is
-- receipted as deferred, not claimed.
--
-- Formalization: `remark` kind, no proof obligation beyond type-checking.
-- We record the interpretive claim as a structure over the already-verified
-- `meas:cell` measurement: throughput flatness is read off the measured
-- record, and the "deferred, not claimed" status of the fuller re-run is
-- recorded as an explicit Bool field rather than asserted away.

structure FaultRateSample where
  faultRatePermille : Nat
  recovered         : Nat

structure SupervisedCellMeasurement where
  members            : Nat
  overheadPermilleAt0 : Int
  baselineSpreadPermille : Nat
  pairedRuns         : Nat
  samples            : List FaultRateSample
  throughputFlat     : Bool
  quarantineEpoch    : Nat

def cellMeasurement : SupervisedCellMeasurement where
  members := 10000
  overheadPermilleAt0 := -9
  baselineSpreadPermille := 57
  pairedRuns := 5
  samples :=
    [ { faultRatePermille := 10,  recovered := 69 }
    , { faultRatePermille := 100, recovered := 608 }
    , { faultRatePermille := 500, recovered := 3179 } ]
  throughputFlat := true
  quarantineEpoch := 2

-- The remark's reading of the measurement: cost tracks failure *novelty*
-- (one-time quorum conversion of the crash-looping template into a
-- standing refusal), not failure *rate* (recovered counts scale with
-- injected rate while throughput stays flat). The fuller novelty-curve
-- re-measurement is marked deferred, never claimed as done.
structure FlatThroughputRemark where
  base                    : SupervisedCellMeasurement
  costTracksNovelty       : Bool  -- cost tracked failure novelty, not failure rate
  crashLoopConvertedOnce  : Bool  -- crash-looping class converted once, by quorum
  fullRecurveDeferred     : Bool  -- full novelty-curve-under-faults re-measurement is deferred

def flatThroughputRemark : FlatThroughputRemark where
  base := cellMeasurement
  costTracksNovelty := cellMeasurement.throughputFlat
  crashLoopConvertedOnce := true
  fullRecurveDeferred := true

-- Sanity checks that the remark elaborates and reads consistently off the
-- underlying measurement (definitional, no proof obligation for a
-- `remark` kind).
example : flatThroughputRemark.base.members = 10000 := rfl
example : flatThroughputRemark.costTracksNovelty = true := rfl
example : flatThroughputRemark.fullRecurveDeferred = true := rfl
