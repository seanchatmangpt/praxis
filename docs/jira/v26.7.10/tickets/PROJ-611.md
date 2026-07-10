# PROJ-611 — Bounded admission loop (refusal → AdmissionRequest → resume)

Status: ALIVE (session-verified via `just cng-test-bench`; RELEASE_CONTROL.md Sec. 8 flips on
the final gate — PROJ-617)

## Summary

Workday refusals manufacture `ex:AdmissionRequest` artifacts naming the minimal missing
admission; the loop deterministically synthesizes the grant (labeled MOCKED-HUMAN for the
consequence, ALIVE for the mechanism), admits it, and resumes at tick+1. The Fortune-5 path is
unchanged. OCEL constructs gain admission/refusal events, with `OCEL_CONSTRUCT_STEMS` updated
in the same commit. Code landed this session in `crates/cng/src/bench/workday.rs` and the
`queries/ocel-*.construct.rq` set.

## Acceptance criteria

1. A refusal mid-day produces an `ex:AdmissionRequest` naming the minimal missing admission;
   the loop resumes at tick+1 after admission — never aborts the day, never skips silently.
2. Synthesized grants are labeled MOCKED-HUMAN in the evidence; the mechanism itself is real.
3. Admission and refusal events are OCEL-queryable; `OCEL_CONSTRUCT_STEMS` and the new
   construct queries land in the same commit.
4. Fortune-5 `benchmark run` output byte-identical to pre-change for a fixed seed.

## Verification

`just cng-test-bench` — admission-resume tests green this session (orchestrator-verified).
Shared Sec. 8 verdict unchanged pending PROJ-617 sign-off.

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` Sec. 3, 8
- `docs/releases/v26.7.10/RELEASE_CONTROL.md` Sec. 8
