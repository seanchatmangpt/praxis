import Mathlib.Tactic
import Praxis.Corpus.prop_totalaccounting

/-!
`meas:cell`: The supervised cell, 10,000 members, release: overhead of
supervision at fault rate 0 is `-0.9%` of medians over five paired runs,
inside the baseline's own `±5.7%` run spread -- statistical noise. Recovery
counts track injected transient rates exactly: `69 / 608 / 3,179` recovered
at `1% / 10% / 50%`. Throughput is flat across all fault rates. The
crash-looping template is detected at quorum and quarantined by epoch 2 in
every configuration.

Design notes on reuse vs. axiomatization:
- This is an empirical *measurement record*, not a mathematical proposition:
  a fixed table of numbers produced by running the supervised cell (10,000
  members) at four fault rates and recording overhead, recovery counts, and
  quarantine epoch. There is no proof obligation -- the corresponding
  thesis kind is `measurement`, parallel to `definition`/`axiom`/
  `construction` entries elsewhere in this corpus.
- We reuse `prop:totalaccounting`'s `Disposition` machinery indirectly: the
  "recovered" counts here are exactly counts of nodes whose disposition
  resolved to `Disposition.Completed` after a transient fault, so this file
  imports that module rather than re-deriving a disposition notion.
- No new axioms: every field is a plain `Nat`/rational-as-`Int`×`Nat`
  (permille) literal or `Bool`, packaged in a single `structure` so the
  record type-checks and is available for later corpus entries to cite by
  name (`CellMeasurement`) rather than as free-floating numbers. Percent
  overhead `-0.9%` and spread `±5.7%` are recorded as signed permille
  integers (`-9`, `57`) to avoid pulling in `Float` (non-canonical in a
  kernel-checked artifact) for a value that is only ever compared/reported,
  never computed on.
-/

/-- One fault-rate row of the recovery-tracking table: the injected
    transient fault rate (in percent) paired with the exact number of
    members recovered, out of the fixed 10,000-member cell. -/
structure RecoveryRow : Type where
  faultRatePercent : Nat
  recovered        : Nat
  totalMembers     : Nat := 10000
  deriving DecidableEq, Repr

/-- `meas:cell`: the pinned measurement record for the supervised cell
    release benchmark (10,000 members, five paired runs at fault rate 0). -/
structure CellMeasurement : Type where
  /-- Cell size: number of supervised members. -/
  members : Nat := 10000
  /-- Number of paired runs the fault-rate-0 overhead was measured over. -/
  pairedRuns : Nat := 5
  /-- Supervision overhead at fault rate 0, in permille (‰) of medians;
      signed, so `-9` means `-0.9%`. -/
  overheadPermilleAtFaultRate0 : Int := -9
  /-- Baseline's own run-to-run spread, in permille; `57` means `±5.7%`. -/
  baselineSpreadPermille : Nat := 57
  /-- The three pinned recovery rows at 1%, 10%, 50% injected fault rate. -/
  recovery1  : RecoveryRow := ⟨1, 69, 10000⟩
  recovery10 : RecoveryRow := ⟨10, 608, 10000⟩
  recovery50 : RecoveryRow := ⟨50, 3179, 10000⟩
  /-- Throughput is flat (unchanged) across all measured fault rates. -/
  throughputFlatAcrossFaultRates : Bool := true
  /-- Epoch by which the crash-looping template is quarantined, in every
      configuration, once detected at quorum. -/
  quarantineByEpoch : Nat := 2
  deriving DecidableEq, Repr

/-- The overhead at fault rate 0 (as an absolute permille value) is smaller
    than the baseline's own run spread -- i.e. it is inside the noise band,
    matching the thesis's "statistical noise" reading of the measurement. -/
theorem CellMeasurement.overhead_within_baseline_spread
    (m : CellMeasurement) (h : m = {}) :
    m.overheadPermilleAtFaultRate0.natAbs < m.baselineSpreadPermille := by
  subst h; decide

/-- The pinned instance of `meas:cell` used by the thesis narrative. -/
def cell : CellMeasurement := {}
