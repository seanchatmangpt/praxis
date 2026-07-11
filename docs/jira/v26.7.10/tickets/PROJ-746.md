# PROJ-746 — Flip ticket file statuses (PROJ-701..729) to evidenced state

Status: DONE (doc) — evidenced this session (uncommitted; HEAD `40f6020`, Phase 6 commit not
run)

Track: D (doc closure wave, Phase 5 of the closure plan; agent: release-doc).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised closure plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

## Summary

Flipped `docs/jira/v26.7.10/tickets/PROJ-701.md` .. `PROJ-729.md` from `PLANNED` to their
evidenced state (ALIVE for 25 of the 26 tickets — PROJ-711 additionally carries a named
PARTIAL scope gap on the full 5x20 IPC corpus scale; PROJ-714 stays PLANNED/cut per its own
declared cut-line status, unchanged by this ticket), each citing the exact command+output from
the verified-evidence list this session. Created ticket files for PROJ-733/734 (the two fixes
this session made beyond the original plan's ticket range) and PROJ-739..745, matching the
existing ticket-file format. Updated `RELEASE_CONTROL.md` §9's table (was all-PLANNED with a
header claiming "no code exists" — now false) and fixed the header text. Synced `index.md`.

## Verification

Every status flip in this ticket traces to a file:line citation or a `cargo test` command +
result already given as verified evidence this session (see each individual ticket's own
"Evidence (this session)" section for its specific citation) — no new commands were run to
produce this ticket itself.

## Links

- `docs/releases/v26.7.10/RELEASE_CONTROL.md` §9
- `docs/jira/v26.7.10/tickets/index.md`
- `docs/jira/v26.7.10/tickets/PROJ-701.md` .. `PROJ-729.md`, `PROJ-733.md`, `PROJ-734.md`,
  `PROJ-739.md` .. `PROJ-745.md`
