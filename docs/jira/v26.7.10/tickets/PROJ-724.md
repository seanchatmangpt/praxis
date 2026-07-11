# PROJ-724 — cng engine resume + partial prefix replay (G13 machinery)

Status: ALIVE — evidenced this session (uncommitted; HEAD `1f3f9bc`, Phase 6 commit not run)

Track: E (multi-engine execution).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

`cng engine resume` reloads the ledger tail and verifies the receipt-chain prefix; a torn
ledger tail refuses lawfully. `--partial` prefix-replay mode in `workday_verify`. This is the
machinery PROJ-729 exercises as the G13 crash-restart-resume falsifier.

## Evidence (this session)

`#[verb("resume", "engine")]` (`crates/cng/src/main.rs:844`). Tests:
`resume_verifies_ledger_prefix_and_skips_processed_contracts`,
`torn_ledger_tail_refuses_cng_r11_on_resume` (`engine_test.rs:135,164`) — part of the `67 lib`
tests in the green 107-test `cargo test -p cng --features bench` run this session. G13
end-to-end exercise (kill/restart/resume across real processes) is evidenced under PROJ-729,
not re-cited here.
