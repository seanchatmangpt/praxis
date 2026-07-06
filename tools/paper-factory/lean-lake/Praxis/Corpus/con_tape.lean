import Praxis.Corpus.def_powl
import Mathlib.Data.List.Basic

/-!
# con:tape

`temporal_plan_to_powl_tape` converts a `TemporalPlan` into a tape of
`PowlOpSpec`s: each step `i` becomes an activity with a `pred_mask` of steps
`j < i` finishing before `i` starts and a one-hot `succ_mask`; a second pass
performs transitive reduction, clearing bits of predecessors' own
predecessors, leaving only direct predecessors.

We model a temporal plan as a `List` of steps carrying an activity label plus
integer `start`/`finish` instants (start/finish times are ordinary `Nat`
literals from the schedule, not wall-clock time -- no Mathlib type already
bundles "activity + start + finish", so it is introduced here as a small
`structure`, the standard idiom for an ad-hoc record). Masks are represented
as `List Bool` indexed by list position, matching the `Nat`-position
convention already used for `POWL.prec`/`POWL.edges` in `def:powl`. All of
the underlying operations (`List.enum`, `List.range`, `List.getD`, `Bool`
conjunction/negation, `Nat` comparison) are pre-built Mathlib/core
primitives; no proof obligation is attached since this is a `construction`,
only well-typedness of the pipeline below.
-/

universe u

/-- One step of a temporal plan: an activity together with the `start`/
`finish` instants (schedule-relative `Nat`s, not wall-clock time) used to
compute precedence. -/
structure TemporalStep (A : Type u) where
  act : A
  start : Nat
  finish : Nat
deriving Inhabited

/-- A temporal plan is simply a sequence of `TemporalStep`s, in the order
they were scheduled. -/
abbrev TemporalPlan (A : Type u) := List (TemporalStep A)

/-- A tape entry: an activity paired with its (eventually direct-only)
predecessor mask and its one-hot successor mask, both indexed by list
position exactly as `POWL`'s `prec`/`edges` relations are. -/
structure PowlOpSpec (A : Type u) where
  act : A
  predMask : List Bool
  succMask : List Bool
deriving Inhabited

variable {A : Type u}

/-- First pass: step `i`'s raw predecessor mask marks every `j < i` whose
`finish` is at or before step `i`'s `start`. -/
def rawPredMask (plan : TemporalPlan A) (i : Nat) : List Bool :=
  plan.mapIdx (fun j s =>
    match plan[i]? with
    | some si => decide (j < i ∧ s.finish ≤ si.start)
    | none => false)

/-- A one-hot successor mask over `n` positions: only position `i + 1` is
set. -/
def oneHotSuccMask (n i : Nat) : List Bool :=
  (List.range n).map (fun j => decide (j = i + 1))

/-- Second pass: given all raw predecessor masks, clear from `mi` (step
`i`'s mask) any bit `j` that is also a predecessor of some direct
predecessor `k` of `i` -- i.e. keep only *direct* predecessors. -/
def transitiveReduceMask (masks : List (List Bool)) (mi : List Bool) : List Bool :=
  mi.mapIdx (fun j b =>
    b && !(masks.mapIdx (fun k mk => mi.getD k false && mk.getD j false)).any id)

/-- `temporal_plan_to_powl_tape` : convert a `TemporalPlan` into a tape of
`PowlOpSpec`s via the two-pass construction described above. -/
def temporal_plan_to_powl_tape (plan : TemporalPlan A) : List (PowlOpSpec A) :=
  let n := plan.length
  let rawPreds : List (List Bool) := (List.range n).map (fun i => rawPredMask plan i)
  let redPreds : List (List Bool) :=
    (List.range n).map (fun i => transitiveReduceMask rawPreds (rawPreds.getD i []))
  plan.mapIdx (fun i s =>
    { act := s.act
      predMask := redPreds.getD i []
      succMask := oneHotSuccMask n i })
