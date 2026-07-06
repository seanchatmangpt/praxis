/-!
refusal:simpleoneforone

No SimpleOneForOne; intensity > 8 is refused, not clamped: derived plans
have static step sets, and `RestartPolicy::new(9, _)` returns a Refusal --
the byte governor applies to recovery itself, and silently clamping a
stated policy would be the executor editing the operator's law.

Composition over fresh axioms:
* the restart intensity is exactly a `Nat` (a count of retries) -- no new
  numeric type needed.
* "returns a Refusal, not a clamped value" is exactly what Lean/Mathlib's
  own `Except` sum type already models: `Except RefusalReason RestartPolicy`
  is either `.error reason` (refused) or `.ok policy` (accepted), reusing
  core's built-in disjoint-outcome type instead of inventing a bespoke
  two-constructor inductive for the same shape.
* `RefusalReason := String`, matching the same composition already used
  in `Praxis/Mathlib/DefReceipt.lean` for the identical concept.

What remains genuinely axiomatized is `RestartPolicy` itself (the opaque
step-plan/backoff configuration the executor constructs) and the
constructor `RestartPolicy.new`, together with the one behavioral axiom
tying them together. `RestartPolicy` is deliberately abstract here: the
thesis statement is about the refusal boundary at intensity 9, not about
the internal structure of a restart plan, and no Mathlib type captures
"an executor-internal restart configuration" -- there is nothing
pre-built to compose it from.
-/

/-- An opaque restart-policy configuration (the executor-internal
    structure of a derived recovery plan). Deliberately abstract: this
    statement is about the refusal boundary, not the plan's internals. -/
axiom RestartPolicy : Type

/-- A human-readable refusal reason, reusing the same composition as
    `RefusalReason := String` in `Praxis/Mathlib/DefReceipt.lean`. -/
abbrev RefusalReason : Type := String

/-- The restart-policy constructor: given an intensity and auxiliary
    parameters, either produces a policy or refuses (via `Except`,
    Lean core's own disjoint-outcome type -- not a bespoke sum type). -/
axiom RestartPolicy.new (intensity : Nat) (aux : Type) (a : aux) :
    Except RefusalReason RestartPolicy

/-- The byte governor applies to recovery itself: any intensity strictly
    greater than 8 is refused outright (`.error _`), never silently
    clamped down into an accepted policy (`.ok _`). This is the "no
    SimpleOneForOne" refusal: the executor does not edit the operator's
    stated law. -/
axiom no_simple_one_for_one {aux : Type} {a : aux} {intensity : Nat}
    (h : intensity > 8) :
    ∃ reason : RefusalReason, RestartPolicy.new intensity aux a = Except.error reason
