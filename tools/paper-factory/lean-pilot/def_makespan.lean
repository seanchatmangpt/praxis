/-
def:makespan — Given the precedence DAG with per-op durations `d_i` and
release times, the forward pass computes
`es_i = max(r_i, max_{j ≺ i} ef_j)`, `ef_i = es_i + d_i`; the makespan is
`T = max_i ef_i`; the backward pass computes `ls_i = lf_i - d_i`; the slack
of op `i` is `ls_i - es_i`, and the critical path is the set of zero-slack
ops.

Formalized in bare Lean 4 core (no mathlib), reusing the tape shape from
`con:tape`. An op is modelled as a release time, a duration, and a
predecessor mask (`List Bool`, one bit per earlier tape position, exactly
as `con_tape.lean`'s `TemporalStep.finishesBefore`). Ops are processed in
tape order (index order), so an op's predecessors are always among the
earlier-computed `es`/`ef` values — this mirrors the DAG's topological
order assumption in the source text.
-/

structure Op where
  release : Nat
  duration : Nat
  -- `predMask[j] = true` iff the op at earlier tape position `j` must
  -- finish before this op starts (same convention as con:tape).
  predMask : List Bool

/-- Look up a boolean mask bit, `false` outside its range. -/
def hasBit (m : List Bool) (j : Nat) : Bool :=
  m.getD j false

/-- `maxOver n f` folds `Nat.max` over `f 0, f 1, …, f (n-1)`, starting
from `0` (the identity for `max` on `Nat`, matching an empty predecessor
set contributing nothing to the `max` in `es_i = max(r_i, max_{j≺i} ef_j)`). -/
def maxOver (n : Nat) (f : Nat → Nat) : Nat :=
  (List.range n).foldl (fun acc j => Nat.max acc (f j)) 0

/-- Forward pass: given the already-computed `ef` values for earlier ops
(indexed `0, …, i-1`) and the op at position `i`, compute its earliest
start `es_i = max(r_i, max_{j ≺ i} ef_j)`. -/
def earliestStart (efSoFar : List Nat) (op : Op) : Nat :=
  Nat.max op.release (maxOver efSoFar.length (fun j =>
    if hasBit op.predMask j then efSoFar.getD j 0 else 0))

/-- Earliest finish `ef_i = es_i + d_i`. -/
def earliestFinish (efSoFar : List Nat) (op : Op) : Nat :=
  earliestStart efSoFar op + op.duration

/-- Forward pass over a whole tape of ops (index order = topological
order): accumulate the list of `ef` values, one per op, each computed from
the `ef`s of all strictly earlier ops. -/
def forwardPass (ops : List Op) : List Nat :=
  ops.foldl (fun efSoFar op => efSoFar ++ [earliestFinish efSoFar op]) []

/-- The makespan `T = max_i ef_i`: the largest earliest-finish time over
the whole forward pass, `0` for an empty plan. -/
def makespan (ops : List Op) : Nat :=
  (forwardPass ops).foldl Nat.max 0
