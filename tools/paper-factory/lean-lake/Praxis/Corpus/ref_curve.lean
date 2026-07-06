/-
ref:curve (refusal)

"No novelty-curve-under-faults measurement -- yet: member-level fault injection
recovers without re-running solver work, so a work-proxy re-measurement would
overstate the cache dividend -- the flattering error. The claim is withheld
until node-level injection is wired through the fleet path; the receipt file
names this in its own deferred list."

This is not a mathematical proposition to prove or a structure to construct: it
is a project-level epistemic marker recording that a specific empirical claim
(a novelty/degradation curve for solver work under *node*-level fault
injection) is deliberately withheld pending instrumentation that does not yet
exist (node-level injection wired through the fleet path). There is no Mathlib
type or theorem this could be "composed from" -- Mathlib has no notion of
"measurement withheld pending future instrumentation," and inventing one here
would be formalizing a documentation string, not doing mathematics. We record
it as an opaque axiom (a `Prop` witness) purely so the statement has a durable,
kernel-checkable anchor in the corpus, matching the way other `Refusal`
variants in this codebase are named rather than proved.
-/

namespace Praxis.Corpus.RefCurve

/-- Marker proposition: the novelty-curve-under-faults measurement described
above is withheld pending node-level fault injection instrumentation. -/
axiom NoveltyCurveUnderFaultsWithheld : Prop

/-- The withheld-claim marker holds by construction: this file's existence
records the refusal itself (member-level injection recovery was measured;
node-level injection was not, so the stronger claim is not asserted). -/
axiom noveltyCurveUnderFaultsWithheld : NoveltyCurveUnderFaultsWithheld

end Praxis.Corpus.RefCurve
