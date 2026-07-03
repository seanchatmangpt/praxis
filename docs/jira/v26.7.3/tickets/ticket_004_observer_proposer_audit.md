# Ticket: Confirm Rice Quarantine Already Implements the PHDITC Observer/Proposer Protocol

## Title
Audit for any LLM-output path that bypasses `Admission::admit` (PROJ-204)

## Description
The PHDITC material claims: live user ontology outranks stale model priors, and model output
must be treated as quarantined observation, never executable truth, until admitted. This is
already the architecture of `crates/praxis-synthesis/src/quarantine.rs`:
`Origin::{Operator, Proposer, Bridge}` marks provenance only (every origin passes through
identical decidable checks); `RiceQuarantine::inspect` performs parser/cap/vocab checks only,
never semantic judgment; `Admission::admit` computes the post-state hash from applying the
delta — nothing is ever asserted. `AdmittedEvent` has private fields and no `Deserialize`
specifically so a hand-built (e.g. LLM-fabricated) post-state cannot reach
`ground::ground_fired_action`.

This ticket is a confirmation audit, not new implementation: verify that no code path in
`crates/praxis-synthesis/src/` constructs an `AdmittedEvent`, mutates `Reference`, or reaches
`execute_from_triples`/`ground_fired_action` without going through `Admission::admit` first.
`tests/no_llm_runtime.rs` already asserts no LLM dependency/symbol exists in the crate at all
— this ticket additionally confirms the STRUCTURAL guarantee (quarantine-before-admission)
holds even for non-LLM proposers (any external/automated input source), since that is the
general form of the PHDITC claim.

## Acceptance Criteria
- A grep-based audit across `crates/praxis-synthesis/src/*.rs` for every call site that
  constructs a `Reference` or `AdmittedEvent`, confirming each goes through
  `Reference::genesis` or `Admission::admit` (the only two constructors, per the existing
  adversarial-hardening doctrine of private fields + no `Deserialize`).
- If a bypass is found, it is a genuine regression: fix it, add a regression test, and file it
  as its own gate finding (do not silently patch without a test).
- If no bypass is found (expected outcome, given `AdmittedEvent`'s already-private fields),
  this ticket closes with a one-line confirmation added to
  `docs/v26.7.3/RICE_QUARANTINE.md` citing the audit.

## Acceptance Criteria (continued)
- No new code required unless a bypass is found.

## Dependencies
None — can run independently of PROJ-201..203.

## Verification Mechanism
1. `cargo test -p praxis-synthesis --test no_llm_runtime` — green.
2. `grep -rn "AdmittedEvent {" crates/praxis-synthesis/src/` — confirm the only construction
   site is inside `Admission::admit`'s implementation (the struct literal), not any external
   caller (which would fail to compile anyway due to private fields — this grep is a
   belt-and-suspenders confirmation, not the primary guarantee).
3. `cargo test -p praxis-synthesis` full suite green.
