# PROJ-728 — Multi-process harness + isolation falsifiers

Status: ALIVE, scoped to the CARGO_BIN_EXE test harness — evidenced this session (uncommitted;
HEAD `40f6020`, Phase 6 commit not run)

Track: E (multi-engine execution).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

`tests/cng_multi_engine.rs` (CARGO_BIN_EXE pattern): C+H+M as separate OS processes on one
host. Isolation falsifiers per DoD §14, including the bypass-injection negative. Determinism
pinning: sorted inbox scans, admit-in-dispatch-id order, zero-padded ids, no PIDs or absolute
paths in digests. Concurrency-vs-determinism fallback (plan risk 2): coordinator gates one
engine at a time, labeled as such if needed.

## Evidence (this session)

`cargo test -p cng --features bench --test cng_multi_engine -- --test-threads=1`: 6/6 passed
this session (was blocked by the G13 watch-loop race before PROJ-734's fix; single-threaded
because each test spawns its own engine processes). Tests:
`multi_engine_concurrent_dispatch_execute_readmit`,
`isolation_falsifier_hostile_graph_is_refuted_by_markers`,
`double_admit_falsifier_replayed_collect_refuses_cng_r25` (`cng_multi_engine.rs:200,279,303`)
— real CARGO_BIN_EXE-spawned separate OS processes, not in-process simulation. Scoped: this is
a test-harness-orchestrated multi-process run, not yet a standalone `cng engine serve`/
`resume` production deployment outside `cargo test`.

## Evidence (follow-up round)

The same `cng_multi_engine.rs` harness (`serialized_run` helper, unmodified) was exercised at
7x its prior heaviest per-run load by PROJ-729's follow-up `recursion_crosses_engines_full_8x2_
fanout` test (fan_out=8/depth=2, 146 total dispatches across two roots) — 2/2 runs green,
37.19s and 32.50s, no orphaned processes or flakes observed. This is corroborating evidence for
this ticket's own harness-robustness claim, not a new capability of PROJ-728 itself; the
primary evidence for the fan-out demonstration lives under PROJ-729.
