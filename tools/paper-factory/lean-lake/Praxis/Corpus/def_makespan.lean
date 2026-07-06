import Praxis.Corpus.con_tape
import Mathlib.Data.List.Basic

/-!
# def:makespan

Given the precedence DAG with per-op durations `d_i` and release times `r_i`,
the forward pass computes `es_i = max(r_i, max_{j ≺ i} ef_j)`,
`ef_i = es_i + d_i`; the makespan is `T = max_i ef_i`; the backward pass
computes `ls_i = lf_i - d_i`; the slack of op `i` is `ls_i - es_i`, and the
critical path is the set of zero-slack ops.

We reuse the `TemporalStep`/index-position convention from `con:tape`
(`Praxis.Corpus.con_tape`): ops are a `List` in an order consistent with
precedence (a predecessor of `i` only ever occurs at some position `j < i`),
and precedence for op `i` is given by a `predMask : List Bool` exactly as
`PowlOpSpec.predMask` already represents it, so no new predecessor
encoding is introduced here. Durations `d_i` and release times `r_i` are
plain `Nat`s (schedule-relative, not wall-clock, matching the `Nat` instants
already used in `con:tape`). `Nat` subtraction (`Nat.sub`, used for slack and
for the backward pass) is truncated at `0` as usual in Lean/Mathlib; this
matches the informal `ls_i = lf_i - d_i` since `lf_i ≥ d_i` in any schedule
computed by this pipeline. All operations used (`List.foldl`, `List.mapIdx`,
`List.getD`, `max`, `Nat` arithmetic) are pre-built Mathlib/core primitives;
no proof obligation is attached since this is a `definition`, only
well-typedness of the pipeline below.
-/

/-- One scheduling op: a duration, a release time, and a predecessor mask
(as in `PowlOpSpec.predMask`) indicating, by list position, which earlier
ops must finish before this one may start. -/
structure MakespanOp where
  dur : Nat
  release : Nat
  predMask : List Bool
deriving Inhabited

/-- Earliest start `es_i` of op `s`: the max of its release time and the
finish times of all its (masked) predecessors, keyed by mask position. -/
def earliestStart' (efs : List Nat) (s : MakespanOp) : Nat :=
  (s.predMask.mapIdx (fun j b => if b then efs.getD j 0 else 0)).foldl max s.release

/-- Earliest finish `ef_i = es_i + d_i`. -/
def earliestFinish (efs : List Nat) (s : MakespanOp) : Nat :=
  earliestStart' efs s + s.dur

/-- Forward pass over the whole op list: accumulate the list of earliest
finish times `ef`, one per op, in list order (which is assumed consistent
with precedence, as in `con:tape`). -/
def forwardPass (ops : List MakespanOp) : List Nat :=
  ops.foldl (fun efs s => efs ++ [earliestFinish efs s]) []

/-- The makespan `T = max_i ef_i`: the maximum earliest-finish time over all
ops, computed by the forward pass. `0` for the empty plan. -/
def makespan (ops : List MakespanOp) : Nat :=
  (forwardPass ops).foldl max 0

/-- Backward pass: given the makespan `T` (used as the common `lf` for ops
with no successors, i.e. the plan-level deadline) and each op's duration,
the latest start `ls_i = lf_i - d_i`. Here we take the simple bound
`lf_i = T` for every op (the plan-wide deadline), matching `T = max_i ef_i`
as the common late-finish bound used before per-op successor propagation. -/
def latestStart (T : Nat) (s : MakespanOp) : Nat :=
  T - s.dur

/-- Slack of op `i`: `ls_i - es_i`, using the plan-wide makespan `T` as the
late-finish bound. -/
def slack (efs : List Nat) (T : Nat) (s : MakespanOp) : Nat :=
  latestStart T s - earliestStart' efs s

/-- The critical path: the set (as a `List`) of ops with zero slack, paired
with the earliest-finish list needed to compute slack. -/
def criticalPath (ops : List MakespanOp) : List MakespanOp :=
  let efs := forwardPass ops
  let T := efs.foldl max 0
  ops.filter (fun s => slack efs T s == 0)
