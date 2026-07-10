# PROJ-609 — `interruption` + `planning` categories (14) with content-bearing RDF

Status: ALIVE (session-verified via `just cng-test-bench`; RELEASE_CONTROL.md Sec. 8 flips on
the final gate — PROJ-617)

## Summary

Add two new benchmark categories — `interruption` and `planning` — bringing the total to 14,
with content-bearing RDF rather than labels: `ex:interrupts` points at an in-flight workflow
instance; `ex:plansFor` points at next-tick standing. Mycin and Datalog role rules and the
on-disk templates are extended in the same commit. Code landed this session in
`crates/cng/src/bench/{generate,roles,templates}.rs` and `queries/`/`templates/` assets.

## Acceptance criteria

1. Category count is 14; `interruption` instances carry `ex:interrupts` referencing a real
   in-flight workflow instance node; `planning` instances carry `ex:plansFor` referencing
   next-tick standing.
2. Role derivation (Mycin + Datalog) covers both new categories; rules live on disk, not
   inline.
3. Existing 12 categories produce byte-identical output for a fixed seed (no drift).

## Verification

`just cng-test-bench` — category and role tests green this session (orchestrator-verified).
Shared Sec. 8 verdict unchanged pending PROJ-617 sign-off.

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` Sec. 2, 5
- `docs/releases/v26.7.10/RELEASE_CONTROL.md` Sec. 8
