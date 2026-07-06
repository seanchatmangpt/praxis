/-!
`refusal:nomeasuredticks` -- No measured CPU ticks: knhk's own hot receipts
carried dummy tick values behind a hardcoded 4 GHz assumption. Declared
deterministic costs (`Ticks` as an abstract unit) are the honest model for a
planner; `rdtsc` instrumentation is a bench follow-up, documented as a
divergence, not smuggled in as a claim.

Per this repo's invariant "no wall clock in any hash/receipt path -- time
only from graph OWL-Time literals", `Ticks` is modeled here as a plain `Nat`:
a declared, deterministic cost unit with no relation asserted to any wall-clock
measurement (`rdtsc`, a hardcoded clock-frequency conversion, etc). `Nat`
already has the total order and additive structure a planner's declared cost
model needs, so no new numeric type is axiomatized.

The one thing kept as an `axiom` is `rdtscTicks`, standing for an *actual*
`rdtsc`-based wall-clock tick reading. No Lean/Mathlib term can honestly
stand in for reading a real hardware cycle counter -- that is exactly the
kind of externally-measured, non-deterministic quantity this statement says
must NOT be smuggled into the deterministic cost model. Axiomatizing its
mere existence (as an opaque `Nat`-valued function of a `Frame`), while
refusing to assert any equation relating it to `declaredTicks`, is the
correct way to formalize "this is a bench follow-up, documented as a
divergence, not a claim": the divergence itself (`Divergence`) is a first
class field, not proved or assumed away.
-/

/-- An opaque cost-accounting frame a planner assigns a declared tick cost
to. Modeled as `Nat` (an index/identity), matching how `TransitionId` is
modeled in `Praxis.Mathlib.DefReceipt`. -/
abbrev Frame := Nat

/-- The abstract, deterministic cost unit a planner declares. Plain `Nat`:
no relation to wall-clock time is built in, per the "no wall clock in any
hash/receipt path" invariant. -/
abbrev Ticks := Nat

/-- The planner's own declared, deterministic cost for a frame. This is the
honest model: computed from the plan, not measured off any clock. -/
axiom declaredTicks : Frame → Ticks

/-- An actual hardware `rdtsc` cycle-counter reading for a frame, standing in
for real wall-clock instrumentation. Kept axiomatized (not defined) because a
genuine cycle counter is an external, non-deterministic measurement -- no
Lean/Mathlib term can honestly represent hardware timing, and defining one
concretely here would smuggle a wall-clock claim into the model, which is
exactly what this statement refuses to do. -/
axiom rdtscTicks : Frame → Ticks

/-- A recorded divergence between the planner's declared cost and an actual
`rdtsc` reading for the same frame, together with the hardcoded-4GHz
conversion constant that was (wrongly) used to produce a dummy tick value.
This is deliberately a *record of disagreement*, not a proof that the two
agree -- the statement's whole point is that no such agreement is claimed. -/
structure Divergence (f : Frame) where
  declared      : Ticks
  measured      : Ticks
  declaredEq    : declared = declaredTicks f
  measuredEq    : measured = rdtscTicks f
  /-- The dummy conversion assumption that produced fabricated tick values
  in knhk's hot receipts: a hardcoded 4 GHz clock rate. Recorded as data
  (not asserted as a fact about hardware), since the statement's refusal is
  precisely that this constant does not honestly relate `declared` to
  `measured`. -/
  assumedHz     : Nat := 4000000000

/-- The refusal itself: for every frame, a planner may only ever expose the
declared, deterministic cost -- `rdtscTicks`/`Divergence` exist as data one
could construct for a bench follow-up, but no equation forcing
`declaredTicks f = rdtscTicks f` is asserted anywhere in this file. The
refusal is witnessed by the mere fact that `Ticks` (the planner-facing type)
is defined without any dependency on `rdtscTicks` at all. -/
noncomputable example (f : Frame) : Ticks := declaredTicks f
