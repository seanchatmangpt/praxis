# PROJ-721 — Durable dispatch ledger + idempotent consume

Status: ALIVE — evidenced this session (uncommitted; HEAD `40f6020`, Phase 6 commit not run)

Track: E (multi-engine execution).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

Append-only `ledger/<dispatch_id>.ttl` StateEntry per `advance()` via a `LedgerSink` trait
(atomic tmp+rename); eager per-tick obs flush; `processed.ttl` idempotency set closing the
double-admit hole with a new typed `DoubleAdmit` refusal (+ negative test). Gate: G10.

## Evidence (this session)

`FileLedgerSink`/`LedgerSink` (`dispatch.rs:389-755`), `CNG_R25 DoubleAdmit`
(`powl.rs:219-224,263`). Tests: `ledger_records_every_advance_and_replays_chain_verified`,
`replayed_consequence_refuses_cng_r25_double_admit` (`dispatch_test.rs:538,594`) — part of the
`67 lib` tests in the green 107-test `cargo test -p cng --features bench` run this session.
