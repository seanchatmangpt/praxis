/-!
lineage:armstrong

"Erlang/OTP made three moves that survive every re-examination: processes
are isolated so failure cannot spread by memory; supervisors -- not the
failing code -- own the recovery decision; and restart strategies plus a
restart intensity turn `let it crash' into a bounded, structured
discipline. What OTP does not do is derive the tree: a human writes the
supervision hierarchy, and the crash space lives in the programmer's
head."

This is a historical/design-lineage claim about the Erlang/OTP runtime,
not a mathematical statement, so there is no computation or theorem to
compose from Mathlib. Per `kind = lineage` there is no proof obligation
beyond type-checking: the claim is captured as a `structure` whose fields
are the three properties the statement credits to OTP, each stated as a
`Prop` field (so a concrete instance witnesses that the three properties
hold together), plus one closing field for the negative claim ("OTP does
not derive the tree").

Every field is `axiom`atized rather than composed from a pre-built
Mathlib notion because each names a property of a real, external runtime
system (the Erlang VM's process isolation, OTP's supervisor behaviour,
and its restart-intensity accounting) -- these are facts about a piece of
software, not mathematical objects with a Mathlib-native encoding. No
search of `Mathlib/` turns up a model of "OTP supervision trees"; the
closest generic notions (e.g. `Mathlib.Order` well-founded trees, process
algebras) would have to be filled with invented semantics to force-fit
this claim, which is exactly the fabrication the migration discipline
prohibits. The four axioms below are the smallest faithful restatement.
-/

/-- The three properties the corpus statement credits to Erlang/OTP,
    stated as opaque propositions (axiomatized: they describe a real
    external runtime, not a formalizable mathematical object). -/
axiom ProcessIsolationPreventsMemorySpread : Prop

axiom SupervisorOwnsRecoveryDecision : Prop

axiom RestartStrategyAndIntensityBoundCrashes : Prop

/-- The statement's closing, negative claim: OTP does not derive the
    supervision tree -- a human authors the hierarchy, and the crash
    space is not formalized by the runtime itself. -/
axiom SupervisionTreeIsHumanAuthoredNotDerived : Prop

/-- Bundled record of the full corpus claim: OTP achieves the three
    positive properties, and it explicitly does not derive the tree. -/
structure OTPLineageClaim where
  isolation : ProcessIsolationPreventsMemorySpread
  supervisorOwnsRecovery : SupervisorOwnsRecoveryDecision
  boundedRestart : RestartStrategyAndIntensityBoundCrashes
  treeNotDerived : SupervisionTreeIsHumanAuthoredNotDerived
