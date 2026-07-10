# PROJ-619 — Broker dispatch surface + lawful re-admission (loopback)

Status: DONE (session-verified via this session's green `just cng-test-bench` — 31 lib tests
+ integration suites passing, recorded in session logs; loopback-real only, live network
endpoints UNVERIFIED per `RELEASE_CONTROL.md` Sec. 8.2)

## Summary

Three execution classes — `LOCAL_ACTUATION`, `EXTERNAL_MACHINE_DISPATCH`,
`EXTERNAL_HUMAN_DISPATCH` — routed exclusively through the broker (dialect → manufactured
action artifact → broker → hook or dispatch adapter). Loopback adapter: outbound contract
serialized to `dispatch/outbox/`; consequence produced into `dispatch/inbox/`, deterministic
and seed-derived; human-dispatch consequences labeled MOCKED-HUMAN. Return path enforced in
order: provenance verification → identity/correlation check → authority verification →
structural validation (SHACL) → semantic conformance → admission or
`CngRefusal::ExternalConsequenceRefused` (`CNG_R17`). Both outbound dispatch and inbound
consequence are receipted into the BLAKE3 chain. Bounded polling only, as a registered
workflow activity; unbounded polling is structurally impossible. Loopback-real, not
network-real: live third-party endpoints are out of scope (mechanism ALIVE when gated;
network UNVERIFIED by design).

## Acceptance criteria

1. No dispatch bypasses the broker; dialects/LLM/scripts cannot dispatch directly.
2. Return path runs the six checks in the stated order; any failure ⇒ `CNG_R17` typed refusal
   (negative test: forged inbox consequence with wrong correlation id refuses).
3. Outbound and inbound receipts fold into the BLAKE3 chain; same-seed byte-identity holds.
4. Polling activities carry bounded frequency (ticks), timeout, termination condition, and
   receipt.

## Verification

`just cng-test-bench` after the wave lands: broker-exclusivity, forged-consequence, and
polling-bound tests green; `EXTERNAL_WORKFLOW_DISPATCH_PROVEN` /
`EXTERNAL_RESULT_READMISSION_PROVEN` markers derive true via SPARQL (PROJ-622).

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` Sec. 7, 8, 9
- `docs/releases/v26.7.10/RELEASE_CONTROL.md` Sec. 8
