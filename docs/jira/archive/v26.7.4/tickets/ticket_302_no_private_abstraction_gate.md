# Ticket: No-Private-Abstraction Gate — CLOSED

## Title
Tie private vocabulary growth to a checked public-ontology mapping doc (PROJ-302) — **STATUS: CLOSED**

## Description
The vision doc's "no private abstraction by default" doctrine currently lives only as prose
(this session's ticket discipline, `docs/claims/WITHHELD_CLAIMS.md`). This ticket makes it
structural: every closed-world predicate table in the crate (`graph.rs`'s `WF_PREDICATES`,
`hooks.rs`'s `HOOK_PREDICATES`, `kernel.rs`'s kernel vocabulary, `agent_registry.rs`'s `agent:`
predicates) is enumerated by a new test that cross-checks each predicate against a new doc,
`docs/v26.7.4/PUBLIC_ONTOLOGY_MAPPING.md`, which must state for every PRIVATE predicate either
(a) the nearest public-ontology predicate it substitutes for and why no public one sufficed, or
(b) that it is pure operational machinery (e.g. `wf:handler`, which has no public-ontology
analog because it names a praxis-internal registry key, not a real-world referent).

This reuses the existing closed-vocabulary-table pattern already present in every module —
no new mechanism, just a doc + a test that fails loudly if a new private predicate is added
without a corresponding mapping-doc entry (a real regression gate, not aspirational prose).

## Acceptance Criteria
- `docs/v26.7.4/PUBLIC_ONTOLOGY_MAPPING.md` exists with one entry per existing private
  predicate across `WF_PREDICATES`, `HOOK_PREDICATES`, the kernel vocabulary, and the `agent:`
  vocabulary (read each via the Read tool — do not enumerate from memory).
- A new test (e.g. `tests/no_private_abstraction.rs`) parses the doc's predicate list and
  asserts it is a superset of every predicate name appearing in the four closed-world tables.
- The test fails (by design) if a future PR adds a new private predicate without updating the
  doc — this is the actual "gate," not just documentation.
- No existing predicate is renamed or removed; this ticket is additive only.

## Dependencies
PROJ-301 (establishes the reality-address precedent this ticket generalizes).

## Verification Mechanism
1. `cargo test -p praxis-synthesis --test no_private_abstraction` — new test green.
2. `cargo test -p praxis-synthesis` — full suite green, no existing test touched.
3. Manually add a throwaway private predicate to a vocab table without a doc entry, confirm
   the new test fails, then revert — proves the gate is real, not decorative.
