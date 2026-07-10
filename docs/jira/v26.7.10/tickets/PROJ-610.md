# PROJ-610 — `standing-next-action.rq`: unique lawful next action per tick

Status: ALIVE (session-verified via `just cng-test-bench`; RELEASE_CONTROL.md Sec. 8 flips on
the final gate — PROJ-617)

## Summary

On-disk `queries/standing-next-action.rq` derives the unique lawful next action per tick from
standing (open steps, awaited dispatches, expired deadlines, pending admissions). The workday
loop refuses with `CngRefusal::StandingAmbiguous` (`CNG_R12`,
`crates/cng/src/powl.rs:69-75`) if the query returns ≠ 1 row while work remains. Results are
logged to the observation stream. This is the executable form of DoD Sec. 2 item 1 ("the
operator never wonders what to do next").

## Acceptance criteria

1. `queries/standing-next-action.rq` exists on disk; no inline SPARQL.
2. Loop consumes exactly one row per tick; ≠ 1 rows with work remaining ⇒ `CNG_R12`
   `StandingAmbiguous` typed refusal, never a silent pick.
3. Negative test: an ambiguous-standing fixture refuses with `CNG_R12`.
4. Query results appear in the observation stream (OCEL-queryable).

## Verification

`just cng-test-bench` — including the ambiguous-standing negative test, green this session
(orchestrator-verified). Shared Sec. 8 verdict unchanged pending PROJ-617 sign-off.

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` Sec. 2, 3
- `docs/releases/v26.7.10/RELEASE_CONTROL.md` Sec. 8
