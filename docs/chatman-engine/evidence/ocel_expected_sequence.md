# OCEL Expected Evidence Shape — post-SealGuard-wiring

Defined before reading any produced `.ocel.json`, so review is checklist-driven.

## Expected per-event fields
- suite name (matches `Scenario::input.suite()` per fixture)
- case identity (matches `scenario.case` from the fixture JSON)
- admission or refusal event (mirrors `record_admitted`/`record_refused` call sites in
  `harness/mod.rs`)
- final status (admitted / refused + `Refusal::name()` on refusal)

Note: `record_admitted`/`record_refused` are the only two event-recording call sites in the
harness (confirmed this session) — there is no separate routing/planning/workflow sub-event
currently emitted per fixture. Do not expect a richer S1→S6 sub-event sequence per case; that
would require additional instrumentation inside `dispatch_*` functions, which is out of scope
for "wire the existing mechanism," not "build a new one."

## Required checks once the suite run completes
1. `.cargo-cicd/ocel/chatman/<suite>.ocel.json` exists for every suite exercised
   (receipt, routing, triple8, admission_table, hook, agent, replay, static_gate).
2. `.cargo-cicd/ocel/chatman/<suite>.receipt.json` exists alongside each.
3. Files are non-empty and contain one event per fixture case in that suite.
4. Re-running the same suite produces a byte-identical `.receipt.json` digest (determinism —
   `seal_run` sorts by event id and BLAKE3-hashes per its own doc).
5. `grep -c "SealGuard::new" tests/chatman_engine_acceptance/harness/mod.rs` shows exactly one
   call site (the one inserted at `run_fixture`) — confirms `SealGuard` is no longer dead code
   without introducing a second, divergent construction path.
