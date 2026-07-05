/-
con:tape — `temporal_plan_to_powl_tape` converts a `TemporalPlan` into a tape
of `PowlOpSpec`s: each step `i` becomes an activity with a `pred_mask` of
steps `j < i` finishing before `i` starts and a one-hot `succ_mask`; a second
pass performs transitive reduction, clearing bits of predecessors' own
predecessors, leaving only direct predecessors.

Formalized in bare Lean 4 core (no mathlib). A `TemporalPlan` is modelled as
a list of steps, where step `i`'s "finishes before" relation to earlier steps
is recorded as a boolean mask (`List Bool` of length `i`, read left-to-right
as steps `0, 1, …, i-1`). A `PowlOpSpec` bundles an activity label together
with a predecessor mask and a successor mask (both boolean lists, one bit per
tape position). The construction builds the initial tape from the plan's
raw precedence masks, then performs a transitive-reduction pass that clears,
for each step, the bits corresponding to predecessors of its own
predecessors — leaving only direct predecessors.
-/

variable {A : Type u}

structure TemporalStep (A : Type u) where
  activity : A
  -- `finishesBefore[j] = true` iff step `j` (j < index of this step) must
  -- finish before this step starts.
  finishesBefore : List Bool

structure TemporalPlan (A : Type u) where
  steps : List (TemporalStep A)

structure PowlOpSpec (A : Type u) where
  activity : A
  predMask : List Bool
  succMask : List Bool

/-- `oneHot n i` is a boolean list of length `n` with `true` at position `i`
and `false` elsewhere (or all `false` if `i ≥ n`). -/
def oneHot (n i : Nat) : List Bool :=
  (List.range n).map (fun j => j == i)

/-- Pad or truncate a boolean list to exactly length `n`, padding with
`false`. -/
def resize (n : Nat) (l : List Bool) : List Bool :=
  (List.range n).map (fun j => (l.getD j false))

/-- First pass: build the raw tape from a `TemporalPlan`. Step `i` gets:
  * `predMask` = its recorded `finishesBefore` mask, resized to the full
    tape length `n`;
  * `succMask` = one-hot at position `i` (a step is, trivially, its own
    unique "successor slot" placeholder prior to reduction). -/
def rawTape (plan : TemporalPlan A) : List (PowlOpSpec A) :=
  let n := plan.steps.length
  (plan.steps.zip (List.range n)).map (fun (s, i) =>
    { activity := s.activity
      predMask := resize n s.finishesBefore
      succMask := oneHot n i })

/-- Does mask `m` have a `true` bit at position `j`? -/
def hasBit (m : List Bool) (j : Nat) : Bool :=
  m.getD j false

/-- Transitive-reduction pass: for step `i`, clear predecessor bit `j` if some
other predecessor `k` of `i` (`k ≠ j`) already has `j` as one of *its*
predecessors — i.e. `j` is reachable via `k`, so the direct edge `j → i` is
redundant. `tape` supplies each step's current predecessor mask (looked up
by index) used as the "predecessors of k" witness. -/
def reduceStep (tape : List (PowlOpSpec A)) (spec : PowlOpSpec A) : PowlOpSpec A :=
  let n := spec.predMask.length
  let isRedundant (j : Nat) : Bool :=
    (List.range n).any (fun k =>
      k != j &&
      hasBit spec.predMask k &&
      (tape.getD k { activity := spec.activity, predMask := [], succMask := [] }
        |>.predMask |> fun pm => hasBit pm j))
  { spec with
      predMask := (List.range n).map (fun j => hasBit spec.predMask j && !isRedundant j) }

/-- Second pass: transitive reduction over the whole tape, using the raw
(un-reduced) predecessor masks as the lookup table for reachability. -/
def reduceTape (tape : List (PowlOpSpec A)) : List (PowlOpSpec A) :=
  tape.map (reduceStep tape)

/-- The construction: convert a `TemporalPlan` into a transitively-reduced
tape of `PowlOpSpec`s. -/
def temporal_plan_to_powl_tape (plan : TemporalPlan A) : List (PowlOpSpec A) :=
  reduceTape (rawTape plan)
