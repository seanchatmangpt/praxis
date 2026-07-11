# PROJ-722 — Deterministic EngineIdentity + per-engine bundle layout

Status: ALIVE — evidenced this session (uncommitted; HEAD `1f3f9bc`, Phase 6 commit not run)

Track: E (multi-engine execution).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

`EngineIdentity { engine_id, ENGINE_VERSION, instance_nonce = splitmix64(seed, id) }` — no
PID or wall clock. Enters the contract (`disp:targetEngine`), every observation
(`obs:producedByEngine`), OTEL Resource `service.instance.id` (non-digest), and the
per-engine bundle layout `engines/<id>/{inbox,outbox,control,ticks,admissions,receipts,
ledger}` with independently replayable receipt chains; coordinator bundle links child
digests. Gate: G11.

## Evidence (this session)

`EngineIdentity`/`ENGINE_VERSION`/`instance_nonce` (`crates/cng/src/bench/engine.rs:57-83`).
`engine_identity_is_deterministic_and_engine_distinct` (`engine_test.rs:57`) — part of the
`67 lib` tests in the green 107-test `cargo test -p cng --features bench` run this session.
