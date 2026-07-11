# PROJ-723 — cng engine serve verb (remote engine loop)

Status: ALIVE — evidenced this session (uncommitted; HEAD `40f6020`, Phase 6 commit not run)

Track: E (multi-engine execution).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

`cng engine serve --root <dir> --engine-id H|M`: bounded receipted poll loop over
`engines/<id>/inbox/`, executes admitted contracts via existing machinery, writes consequence
Turtle to outbox, exits on a SHACL-validated quiescence file. `SynthesisMode::RemoteEngine
{ inbox_dir }` at the documented `synthesize_consequence` seam. The one real-time element
(inter-poll sleep) sits behind a `RealTimeWait` seam and never enters digests; logical poll
counts do. Transport is filesystem (DoD §20 item 1); HTTP is rejected this increment.
Gate: G12.

## Evidence (this session)

`#[verb("serve", "engine")]` (`crates/cng/src/main.rs:819`). Tests:
`serve_executes_inbox_contract_and_writes_consequence`,
`shacl_validated_quiescence_file_ends_the_loop` (`engine_test.rs:70,110`) — part of the `67
lib` tests in the green 107-test `cargo test -p cng --features bench` run this session.
