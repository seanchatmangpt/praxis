-- meas:cell
-- The supervised cell, 10,000 members, release: overhead of supervision at
-- fault rate 0 is -0.9% of medians over five paired runs, inside the
-- baseline's own +/-5.7% run spread -- statistical noise. Recovery counts
-- track injected transient rates exactly: 69 / 608 / 3,179 recovered at
-- 1% / 10% / 50%. Throughput is flat across all fault rates. The
-- crash-looping template is detected at quorum and quarantined by epoch 2
-- in every configuration.
--
-- Formalization: `measurement` kind, no proof obligation beyond
-- type-checking. We record the reported quantities as a structure of
-- rational-valued fields (percentages scaled by 1000 to stay exact, counts
-- as naturals) and give the concrete measured instance from the release.

structure FaultRateSample where
  faultRatePermille : Nat   -- fault rate, in parts-per-thousand (0, 10, 100, 500)
  recovered         : Nat   -- recovered count at this injected transient rate

structure SupervisedCellMeasurement where
  members            : Nat
  overheadPermilleAt0 : Int   -- overhead vs. baseline at fault rate 0, in per-mille (-9 = -0.9%)
  baselineSpreadPermille : Nat -- baseline's own run-to-run spread, in per-mille (57 = 5.7%)
  pairedRuns         : Nat
  samples            : List FaultRateSample
  throughputFlat     : Bool   -- throughput observed flat across all fault rates
  quarantineEpoch    : Nat    -- epoch by which the crash-looping template is quarantined

-- The concrete measurement reported for the release.
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

-- Sanity check that the measurement record elaborates and its fields are
-- accessible (definitional, no proof obligation for a `measurement` kind).
example : cellMeasurement.members = 10000 := rfl
example : cellMeasurement.samples.length = 3 := rfl
