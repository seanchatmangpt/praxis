# Milestone Overview: v26.7.10 — Recursive Workflow Full Autonomic Loop

This milestone closes the gap from the Fortune-5 Recursive Workflow benchmark (OCEL authority,
receipts, replay, Datalog roles — ALIVE per `docs/releases/v26.7.10/RELEASE_CONTROL.md` §1/§7)
to the behavioral Definition of Done in `docs/releases/v26.7.10/DEFINITION_OF_DONE.md`: one
operator executing an entire production workday through standing-preserving recursive workflow
manufacture, hook-receipted actuation, loopback-real external dispatch with lawful
re-admission, and SPARQL-derived success markers. `RELEASE_CONTROL.md` is the single control
surface; if this index and its Sec. 8 table disagree, Sec. 8 wins.

Statuses below use the no-overclaiming vocabulary and reflect this session (2026-07-10).
ALIVE rows cite `just cng-test-bench` runs made this session; the Sec. 8 table flips only at
the PROJ-617 final gate.

## Ticket status

| Ticket | Scope | Status |
|---|---|---|
| [PROJ-601](PROJ-601.md) | `digests.json` path portability in `verify` | CLOSED (`40f6020`) |
| [PROJ-602](PROJ-602.md) | `cng evidence replay` verb for auditors | CLOSED (`40f6020`) |
| [PROJ-603](PROJ-603.md) | Bundle manifest schema (every digest named) | CLOSED (`40f6020`) |
| [PROJ-604](PROJ-604.md) | Close inline-SPARQL sites + guard test | CLOSED (`40f6020`) |
| [PROJ-605](PROJ-605.md) | `CNG_R11 AuditMismatch` refusal variant | CLOSED (`40f6020`) |
| [PROJ-606](PROJ-606.md) | DEFINITION_OF_DONE.md doctrine document | CLOSED (this session) |
| [PROJ-607](PROJ-607.md) | Doc reconciliation pass | CLOSED (this session) |
| [PROJ-608](PROJ-608.md) | `benchmark workday` verb | ALIVE (session-verified) |
| [PROJ-609](PROJ-609.md) | Interruption + planning categories (14) | ALIVE (session-verified) |
| [PROJ-610](PROJ-610.md) | `standing-next-action.rq` + `CNG_R12` | ALIVE (session-verified) |
| [PROJ-611](PROJ-611.md) | Bounded admission resume loop | ALIVE (session-verified) |
| [PROJ-612](PROJ-612.md) | graphlaw hook pack actuation, `CNG_R13` | ALIVE (session-verified) |
| [PROJ-613](PROJ-613.md) | Dialect registry + HookStanding, `CNG_R14` | ALIVE (session-verified) |
| [PROJ-618](PROJ-618.md) | Dispatch contract + 13-state machine | DONE (session-verified) |
| [PROJ-619](PROJ-619.md) | Broker dispatch + re-admission (loopback) | DONE (loopback-real) |
| [PROJ-620](PROJ-620.md) | Recursive closure / timeout / compensation | DONE (session-verified) |
| [PROJ-621](PROJ-621.md) | Arazzo dialect | DONE (session-verified) |
| [PROJ-614](PROJ-614.md) | Graph-authoritative metrics closure | DONE (final build green) |
| [PROJ-615](PROJ-615.md) | Optional ed25519 signatures | CUT (`RELEASE_CONTROL.md` §8.1) |
| [PROJ-616](PROJ-616.md) | Verification harness + tamper negatives | DONE (final build green) |
| [PROJ-622](PROJ-622.md) | SPARQL-derived success markers | DONE (all 11 markers TRUE) |
| [PROJ-617](PROJ-617.md) | Release closure + DoD sign-off | DONE |

## Execution sequence

```text
606, 607 (parallel) → 608 → 609 → 610 → 611 → 612 → 613
  → 618 → 619 → 620 → 621 → 614 → (615) → 616 → 622 → 617
```

Docs first; hooks before dispatch (the broker needs hooks); dispatch before Arazzo (Arazzo
projects onto dispatch contracts); metrics after all evidence producers. Cut line after
PROJ-619 still proves external dispatch + readmission.

## Standing boundaries (honesty notes)

- External dispatch is loopback-real (deterministic local outbox/inbox); live network
  endpoints are out of scope for v26.7.10 — UNVERIFIED by design, never claimed.
- Synthesized human consequences are MOCKED-HUMAN wherever they appear.
- ChatmanEngine adoption is DEFERRED; this release uses the `TripleStore` hook surface.

## See Also

- `docs/releases/v26.7.10/RELEASE_CONTROL.md` — single control surface (Sec. 8 = this table's
  authoritative counterpart)
- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` — behavioral doctrine (PROJ-606)
- `docs/releases/v26.7.10/PRD.md`, `docs/releases/v26.7.10/ARD.md`
- `docs/jira/archive/v26.7.4/tickets/index.md` — prior-art index format
