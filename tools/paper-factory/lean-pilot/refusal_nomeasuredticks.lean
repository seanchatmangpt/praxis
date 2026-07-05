/-
Label: refusal:nomeasuredticks
Kind: refusal

No measured CPU ticks: knhk's own hot receipts carried dummy tick values behind
a hardcoded 4 GHz assumption. Declared deterministic costs (`Ticks` as an
abstract unit) are the honest model for a planner; rdtsc instrumentation is a
bench follow-up, documented as a divergence, not smuggled in as a claim.

Formalized as a refusal construction: `Ticks` is declared as an abstract,
opaque cost unit with no built-in numeric representation and no baked-in
clock frequency. Recovering a real elapsed-time reading requires an
explicit, separately-justified `ClockModel` value (standing in for bench
rdtsc instrumentation), never a hardcoded constant multiplication. The
absence of any default/canonical `ClockModel` is the refusal: this file
supplies none.
-/

/-- An abstract, opaque unit of declared deterministic planner cost.
No numeric representation or clock frequency is assumed. -/
opaque Ticks : Type

/-- A `ClockModel` is the explicit, separately-justified bridge from abstract
`Ticks` to real elapsed time (standing in for bench rdtsc instrumentation).
It is declared opaque and is never assumed to have a canonical instance. -/
opaque ClockModel : Type

/-- The only sanctioned route from `Ticks` to wall-clock time: reading a
`ClockModel` explicitly supplied by the caller. There is no other function
in this file producing nanoseconds from `Ticks` alone. -/
opaque ClockModel.toNanos : ClockModel → Ticks → Nat

/-- Refusal, as a documented divergence rather than a claim: this
construction deliberately does not provide any term of type `ClockModel`
(e.g. no hardcoded "4 GHz" instance). Real-time conversion is left to a
bench follow-up that must supply its own `ClockModel` with justification. -/
def noHardcodedClockModel : Option ClockModel := none
