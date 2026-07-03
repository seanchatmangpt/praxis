# Ticket: Authority Ledger via Firing-Time Provenance Check

## Title
Refuse ground-action firings whose action node has no admitted authority anchor (PROJ-303)

## Description
The vision doc's "AuthorityLedger" claims every proxy action must bind to admitted authority,
not live human interaction. Rather than building a new ledger subsystem, this ticket reuses
`reality.rs`'s `provenance_anchor` (PROV-O `wasAttributedTo`) as the authority binding: when
`ground::ground_fired_action` grounds a `GroundAction` verdict, it must additionally attempt
`RealityAddressRecord::bind` on the action's own IRI within the restricted fragment; if the
resulting record has no `provenance_anchor`, the firing is refused
(`Refusal::DelegabilityViolation`-adjacent — reuse or extend the existing refusal, do not
invent a new "AuthorityVacuum" identifier per the PROJ-203 audit's discipline against EHDIT-
style metaphor vocabulary).

This is deliberately narrow: it does not model who the authority IS beyond the PROV-O IRI
already declared in the graph — it only enforces that one must be declared before an action
executes, closing the literal gap named in the vision ("action force exists without lawful
authority vector" — reduced here to a concrete, testable graph-shape rule).

## Acceptance Criteria
- `ground_fired_action` (or a new thin wrapper it calls) refuses a `ground-action` firing whose
  action node lacks a PROV-O `wasAttributedTo` triple, citing the existing
  `Refusal::WorkflowIllFormed` (preferred — this is a workflow-shape violation, not a new
  refusal category) with a detail naming "no authority anchor."
- A regression test: a demo TTL fragment with a `ground-action` hook whose action node has NO
  `prov:wasAttributedTo` triple is refused; the same fragment WITH the triple succeeds.
- No new `Refusal` variant unless the audit shows the existing ones are a genuinely bad fit —
  default to reuse.
- Existing `tests/prayer_kernel.rs`/`tests/deviation_routes.rs` fixtures either already declare
  authority anchors (if so, no change) or are updated to declare them (if this is a breaking
  requirement) — confirm which by running the suite before deciding.

## Dependencies
PROJ-301 (this ticket is literally a consumer of `RealityAddressRecord::bind`).

## Verification Mechanism
1. `cargo test -p praxis-synthesis --test prayer_kernel --test deviation_routes --test
   firing_chain` — green (either unchanged, or updated fixtures pass).
2. New regression test added and green, demonstrating both the refusal and success paths.
3. `cargo clippy -p praxis-synthesis --all-targets -- -D warnings` — clean.
