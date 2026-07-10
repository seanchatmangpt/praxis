# PROJ-618 — Dispatch contract + 13-state machine vocabulary

Status: DONE (session-verified via this session's green `just cng-test-bench` — 31 lib tests
+ integration suites passing, recorded in session logs; `RELEASE_CONTROL.md` Sec. 8)

## Summary

`ex:DispatchContract` TTL template with all 20 required fields (dispatch id, workflow/parent
ids, recursive depth, target actor/system, required role, declared authority, input and
expected-output artifact sets, activity identity, deadline in logical ticks — never wall
clock, idempotency key, correlation id, callback/collection surface, retry/escalation/
compensation law, refusal conditions, receipt and replay requirements); a SHACL shape making
every field mandatory; `ex:dispatchState` over the 13-state machine (MANUFACTURED …
UNKNOWN). A contract missing any field refuses with `CngRefusal::DispatchContractIncomplete`
(`CNG_R15`) before leaving the broker. Rust types plus on-disk templates only.

## Acceptance criteria

1. Contract template + closed SHACL shape on disk; all 20 fields mandatory.
2. Missing field ⇒ `CNG_R15` `DispatchContractIncomplete` pre-dispatch (negative test).
3. State transitions restricted to the lawful 13-state machine; unlawful transition ⇒
   `CNG_R16` typed refusal.
4. Deadlines expressed in logical ticks; no wall clock in any digest path.

## Verification

`just cng-test-bench` after the wave lands: contract-completeness and state-machine negative
tests green; then two same-seed `just cng-workday` runs byte-identical (PROJ-616 gate).

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` Sec. 7, 8
- `docs/releases/v26.7.10/RELEASE_CONTROL.md` Sec. 8
