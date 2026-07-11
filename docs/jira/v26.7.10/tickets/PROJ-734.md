# PROJ-734 — Fix G13 watch-loop race in `cng_multi_engine.rs`

Status: ALIVE — evidenced this session (uncommitted; HEAD `40f6020`, Phase 6 commit not run)

Track: closure (beyond the original v26.7.10-revised plan's PROJ-701..731 ticket range; filed
this session per the approved closure plan's Phase 1).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised closure plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

## Summary

The G13 crash-resume test's watch loop counted *any* dir entry in `ledger/`, including
transient `.tmp` files from the atomic write-then-rename — it could fire `child.kill()` before
a committed `.ttl` file existed, making the torn-tail branch's `.expect("a ledger file
exists")` flaky. Fix: filter the watch loop to `.ttl` extension only, matching the torn-tail
branch's own filter (`cng_multi_engine.rs:345,367,411,419,500`).

## Evidence (this session)

`cargo test -p cng --features bench --test cng_multi_engine -- --test-threads=1`: 6/6 passed,
including `g13_crash_resume_verifies_chain_and_completes` (confirms the fix holds). All
spawned processes remain `--max-polls`-bounded (no infinite hangs); orphan leaks on
assertion-panic paths are real but self-terminate at their poll ceiling (60s/500s) — lower
priority, not fixed this session.

## Links

- `docs/jira/v26.7.10/tickets/PROJ-728.md`, `PROJ-729.md` (downstream evidence)
- `crates/cng/tests/cng_multi_engine.rs`
