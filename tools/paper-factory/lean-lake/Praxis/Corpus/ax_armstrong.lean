/-!
Label: ax:armstrong

The four opaque axioms underlying `lineage:armstrong`
(`Praxis/Corpus/lineage_armstrong.lean`): claims credited to Erlang/OTP by
the corpus statement --

"processes are isolated so failure cannot spread by memory; supervisors --
not the failing code -- own the recovery decision; and restart strategies
plus a restart intensity turn `let it crash' into a bounded, structured
discipline. What OTP does not do is derive the tree: a human writes the
supervision hierarchy, and the crash space lives in the programmer's head."

Reclassified here (out of `lineage_armstrong.lean`) to attach a per-axiom
verification note: each `axiom _ : Prop` is a real-world claim about a
running system's behavior, not a mathematical object, so none of the four
is a theorem waiting to be derived -- `Prop` alone carries no domain
structure to induct on, regardless of what real code exists. What *does*
differ per axiom is whether this repository contains code the claim can be
pinned to. Two do; two do not, for different reasons documented below.
This file exists so that distinction is visible instead of four
identically-opaque names sitting together with one blanket justification.
-/

/-- Claims that Erlang/OTP process isolation prevents a crash's memory
    effects from spreading to other processes. This is a property of the
    BEAM virtual machine itself (per-process heaps, no shared mutable
    memory, message-passing-only inter-process communication) -- it is
    external to this repository: no code under `apps/` implements,
    configures, or tests BEAM's memory isolation, and no such test exists
    to cite. Kept as a pure-aspiration axiom: it credits a guarantee of the
    Erlang runtime that praxis inherits by running on BEAM, not a
    guarantee praxis's own code establishes or verifies. -/
axiom ProcessIsolationPreventsMemorySpread : Prop

/-- Claims that the supervisor, not the failing worker, owns the recovery
    decision after a crash. Real corresponding code exists in this repo:
    `apps/arazzo_runner/src/arazzo_runner_sup.erl` (`init/1`, lines
    16-51) returns `{SupFlags, ChildSpecs}` to OTP's `supervisor`
    behaviour, which alone decides whether/how to restart a child: the
    child never restarts itself. On the worker side,
    `apps/arazzo_runner/src/arazzo_runner_workflow.erl` deliberately does
    not attempt local recovery on failure -- it calls `exit/1` and hands
    the decision upward (see `handle_reaction/3` at lines 453-458 for
    `{admission_result,{refused,Reason}}` and line 483 for
    `transition_crashed`). Still kept as an axiom, not a theorem: the
    *general* claim is about OTP's `supervisor` behaviour semantics (an
    external runtime contract), and `arazzo_runner_sup.erl` only
    *exercises* that contract via stock OTP, it does not reimplement or
    formally specify it inside this repo, so there is nothing in the Lean
    corpus to derive the claim from. -/
axiom SupervisorOwnsRecoveryDecision : Prop

/-- Claims that a restart strategy plus a restart intensity together bound
    crash recovery to a structured discipline rather than unbounded
    thrashing. Real corresponding code exists:
    `apps/arazzo_runner/src/arazzo_runner_sup.erl` lines 19-21 declares
    `SupFlags = #{strategy => simple_one_for_one, intensity => 10,
    period => 1}` -- a concrete strategy (`simple_one_for_one`) and
    intensity bound (10 restarts per 1s), with `restart => transient`
    child-spec semantics documented in the surrounding comment block
    (lines 26-48). Kept as an axiom, not a theorem: this is one example
    configuration exercising OTP's restart-intensity accounting, not a
    formal model of restart-intensity bounding inside this Lean corpus
    that a proof could unfold. -/
axiom RestartStrategyAndIntensityBoundCrashes : Prop

/-- The statement's closing, negative claim: OTP does not derive the
    supervision tree -- a human authors the hierarchy, and the crash space
    is not formalized by the runtime itself. `arazzo_runner_sup.erl` is
    itself one small, hand-written tree (a single `simple_one_for_one`
    child spec, lines 16-51) -- consistent with, but not a proof of, the
    general claim: no code anywhere in this repo encodes or tests the
    *negative* half of the statement (that OTP itself does not, could not,
    or does not attempt to derive such trees). The only place this
    negative claim is engaged with substantively is prose contrast in
    `docs/thesis/synthesis_thesis.tex` (around lines 590-609), which
    argues a *later* praxis capability against this Armstrong-era
    baseline -- a documentation argument, not Erlang or Lean code. Kept as
    a pure-aspiration axiom for the negative claim specifically: there is
    no code in this repository that could ground a proof either way. -/
axiom SupervisionTreeIsHumanAuthoredNotDerived : Prop
