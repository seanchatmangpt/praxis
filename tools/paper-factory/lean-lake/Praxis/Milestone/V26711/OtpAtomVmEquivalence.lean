/-!
# PROJ-769 / PRD v26.7.11 §7.7/§7.9/§19.10 — OTP/AtomVM Shared-Core Equivalence

Target 8 of the 9 declared Lean/Lake formalization targets at `PRD.md:1035-1043`:
"shared-transition-core equivalence premise for OTP/AtomVM wrappers."

PRD §7.7 (`docs/jira/v26.7.11/PRD.md:371-390`), §7.9 (`:425-436`), and acceptance
scenario 19.10 (`:922-924`), in relevant part:

> The AIR state transition implementation SHALL be pure Erlang. The canonical
> transition SHALL be modeled as `δ_AIR:(S,E)→(S',C)` ... OTP and AtomVM SHALL wrap
> the same transition core.
> AtomVM SHALL execute the same AIR transition semantics. The product SHALL NOT
> maintain a separate semantic implementation.
> For identical AIR and identical ordered admitted event corpus, OTP and AtomVM
> SHALL produce equivalent: state digest; result digest; refusal class; command
> sequence.

## Real correspondence

Models the real Erlang architecture directly: `apps/air_core/src/air_core.erl`
declares `-spec transition(event(), context()) -> {context(), [command()]}` — this
*is* `δ_AIR`. `apps/arazzo_runner/src/arazzo_runner_workflow.erl` (the OTP wrapper)
calls `air_core:transition(Event, Core)` directly (`:475`); `apps/atomvm_runner/
src/atomvm_runner.erl`'s own module doc states it is "a pure `receive`-loop actor
wrapper over `air_core:transition/2`" and explicitly rejects introducing any second
implementation that "would itself become the ... same single wrapper" — i.e. both
runners are, by construction, thin callers of the *same* `air_core:transition/2`.

This file formalizes exactly that premise — "wrap the same transition core" — as a
theorem about *any* pure function two wrappers both call: `sharedCore` is left an
arbitrary (universally quantified) function, standing in for
`air_core:transition/2` whatever it concretely computes (out of Lean's scope to
reimplement); `OtpWrapper`/`AtomVmWrapper` each carry their own runtime-specific
bookkeeping (`otpPid`/`atomVmSlot`) that never reaches `sharedCore`'s input,
mirroring the real architecture where OTP's process/supervision bookkeeping and
AtomVM's actor-loop bookkeeping are wrapper-local, not part of `δ_AIR`'s own `(S,E)`
domain. The theorem shows equivalence of output follows from equal `(air, events)`
inputs alone, given that premise — exactly acceptance scenario 19.10.

No axioms: `sharedCore` is a universally quantified function parameter (sound and
general — the theorem holds for whichever concrete function
`air_core:transition/2` denotes), not an assumed-to-exist global axiom.
-/

/-- `δ_AIR`'s observable output tuple (PRD §19.10): state digest, result digest,
refusal class, and command sequence, collapsed to opaque `String`/`List String`
payloads (their concrete encoding is out of scope here). -/
structure TransitionOutput where
  stateDigest  : String
  resultDigest : String
  refusalClass : Option String
  commands     : List String
deriving DecidableEq, Repr

/-- One OTP-wrapped run: the AIR artifact identity, the ordered admitted event
corpus, its own OTP-specific supervision bookkeeping (`otpPid`, standing in for the
real `#runner_state{}`'s live `Pid`), and the shared transition core it calls. -/
structure OtpWrapper (Air Event Pid : Type) where
  air        : Air
  events     : List Event
  otpPid     : Pid
  sharedCore : Air → List Event → TransitionOutput

/-- One AtomVM-wrapped run over the same shape: its own AtomVM-specific
bookkeeping (`atomVmSlot`, standing in for the actor-loop's own local state), and a
shared transition core. -/
structure AtomVmWrapper (Air Event Slot : Type) where
  air        : Air
  events     : List Event
  atomVmSlot : Slot
  sharedCore : Air → List Event → TransitionOutput

/-- Each wrapper's observable output is computed *only* from `(air, events)` via
`sharedCore` — its own runtime-specific bookkeeping (`otpPid`/`atomVmSlot`) never
reaches the computation, matching the real Erlang architecture
(`arazzo_runner_workflow.erl`'s process bookkeeping and `atomvm_runner.erl`'s
actor-loop state are wrapper-local, not part of `air_core:transition/2`'s own
domain). -/
def OtpWrapper.output {Air Event Pid : Type} (w : OtpWrapper Air Event Pid) :
    TransitionOutput := w.sharedCore w.air w.events

def AtomVmWrapper.output {Air Event Slot : Type} (w : AtomVmWrapper Air Event Slot) :
    TransitionOutput := w.sharedCore w.air w.events

/-- `thm:otp_atomvm_equivalence` (PRD §19.10): given the *same* AIR artifact, the
*same* ordered admitted event corpus, and the *same* shared transition core (the
"shared-transition-core" premise PRD §7.7/§7.9 declare architecturally), an
OTP-wrapped run and an AtomVM-wrapped run produce byte-identical output — state
digest, result digest, refusal class, and command sequence all agree — regardless
of how their respective, wrapper-local `otpPid`/`atomVmSlot` bookkeeping differs. -/
theorem otp_atomvm_equivalence {Air Event Pid Slot : Type}
    (o : OtpWrapper Air Event Pid) (a : AtomVmWrapper Air Event Slot)
    (hair : o.air = a.air) (hevents : o.events = a.events)
    (hcore : o.sharedCore = a.sharedCore) :
    o.output = a.output := by
  unfold OtpWrapper.output AtomVmWrapper.output
  rw [hair, hevents, hcore]
