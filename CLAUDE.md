# Praxis — v26.7.4

Milestone: PROJ-302..306, `docs/jira/v26.7.4/tickets/index.md` (dependency graph).
PROJ-301 done. PROJ-305 blocked: `docs/v26.7.3/COGNITIVE_BREED_MAPPING.md` doesn't
exist yet — author it from ticket_006, don't fabricate inline.

## Invariants (violation = the bug)

1. No panics/silent defaults — every error is a typed `Refusal` variant (`lib.rs`).
2. Receipts are computed (BLAKE3, genesis-folded), never asserted-in.
3. No wall clock in any hash/receipt path — time only from graph OWL-Time literals.
4. Closed vocabularies (`wf:`, `hook:`, `prayer-kernel:`, `agent:`) — unknown
   predicates refused by name, paired with `docs/v26.7.4/PUBLIC_ONTOLOGY_MAPPING.md`.
5. `praxis-synthesis` deps frozen to exactly: pddl-index, chatman-common, blake3,
   serde, serde_json, thiserror (`tests/no_llm_runtime.rs` enforces).
6. Smallest diff, reuse first — no new subsystems where a const table suffices.

## Commands

`just verify-all` — DoD gate, run before claiming a ticket done.
`just test-changed` — fast inner loop.

## Reporting

State findings, not verdicts. No unearned hedges, no ranking docs by assumed
authority, no quality adjectives ("messy," "stale") the task didn't ask for.
Explore mines requirements; exploit rewrites clean-room from the invariant, never
ports the explored source.
