import Praxis.Mathlib.DefReceipt

/-!
Label: ax:restartpolicy

The executor-internal restart-policy configuration `RestartPolicy`, its
constructor `RestartPolicy.new`, and the behavioral law tying them together:
"the byte governor applies to recovery itself -- any intensity strictly
greater than 8 is refused outright, never silently clamped down into an
accepted policy."

Reclassified here (out of `refusal_simpleoneforone.lean`) after a genuine
attempt to derive `no_simple_one_for_one` failed. The reason it cannot be
derived is structural, not a gap in proof effort: `RestartPolicy` and
`RestartPolicy.new` are declared as fully opaque axioms with zero defining
equations anywhere in this corpus (`grep -rl RestartPolicy` finds no other
file). Lean's logic gives no information about what an axiomatized black-box
*function* computes beyond what is separately asserted about it -- there is
no internal structure, recursion, or Mathlib composition to induct on or
unfold. `no_simple_one_for_one` is a claim about the *behavior* of that
black box (it always returns `.error _` above intensity 8), which is an
empirical/design fact about the real executor's implementation, not a
theorem derivable from the shape of `Except`/`Nat`/`Type` alone. Composing
`RestartPolicy.new`'s return type from Mathlib's `Except` (done in the
importing file) captures the *shape* of the refusal/accept split; it cannot
capture *which* intensities land in which branch -- that information has to
be asserted, exactly as `ax:obs`'s `Obs` and `ax:refusal`'s `Adm`/
`ReasonSpace` assert structure the source statement leaves abstract.

This is the same class of axiom as `ax_obs.lean` and `ax_refusal.lean`: an
opaque type/behavior the thesis statement declares abstract by design, kept
as an axiom rather than forced into a disprovable Mathlib encoding that
would smuggle in detail the source does not supply.
-/

/-- An opaque restart-policy configuration (the executor-internal
    structure of a derived recovery plan). Deliberately abstract: this
    statement is about the refusal boundary, not the plan's internals. -/
axiom RestartPolicy : Type

/-- The restart-policy constructor: given an intensity and auxiliary
    parameters, either produces a policy or refuses (via `Except`,
    Lean core's own disjoint-outcome type -- not a bespoke sum type). -/
axiom RestartPolicy.new (intensity : Nat) (aux : Type) (a : aux) :
    Except RefusalReason RestartPolicy

/-- The byte governor applies to recovery itself: any intensity strictly
    greater than 8 is refused outright (`.error _`), never silently
    clamped down into an accepted policy (`.ok _`). This is the "no
    SimpleOneForOne" refusal: the executor does not edit the operator's
    stated law. Kept as an axiom (not derived): it is a behavioral fact
    about the opaque `RestartPolicy.new` black box, which has no internal
    structure in this model for a proof to unfold -- see the module
    docstring for why this is structural, not a missing proof effort. -/
axiom no_simple_one_for_one {aux : Type} {a : aux} {intensity : Nat}
    (h : intensity > 8) :
    ∃ reason : RefusalReason, RestartPolicy.new intensity aux a = Except.error reason
