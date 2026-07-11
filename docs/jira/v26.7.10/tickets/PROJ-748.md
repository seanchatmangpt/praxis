# PROJ-748 — Revised DOD_SIGNOFF.md + DOD_EVIDENCE_MAP.md for v26.7.10-revised

Status: DONE (doc) — evidenced this session (uncommitted; HEAD `40f6020`, Phase 6 commit not
run)

Track: D (doc closure wave, Phase 5 of the closure plan; agent: release-doc).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised closure plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

## Summary

Authored revised `DOD_SIGNOFF.md` and `DOD_EVIDENCE_MAP.md` for the v26.7.10-revised DoD's
~20 sections (the prior versions on disk were the INTERIM PROJ-617 artifacts for the
superseded prior DoD, now preserved as historical record, not overwritten). Clause-by-clause
against `DEFINITION_OF_DONE.md`'s sections 1-20, citing the evidence verified this session
(PROJ-733/734/739..745). PROJ-731 (final release closure) closes on this sign-off being
complete and honest — not on every underlying claim being unscoped ALIVE.

## Honest gaps stated explicitly in the sign-off

1. The arazzo digest-verify wiring gap (PROJ-745): `verify_arazzo_render_digest` exists and is
   unit-tested, but is not wired into `dispatch.rs`'s `DispatchState::ArazzoRendered`
   transition — PARTIAL.
2. `full_production_ready`'s real two-bundle invocation (PROJ-742): the combinator has never
   been invoked end-to-end against real `workday()` + `cng plan decompose` bundle outputs
   together in this session — only its constituent marker families were separately verified
   true, and the combinator itself only unit-tested with one fabricated half — UNVERIFIED.
3. PROJ-711's IPC corpus scale: seeds 0..3 verified, not the full 5x20 corpus — PARTIAL.
4. PROJ-714 (G14/G15 long-horizon scenarios): declared cut, never built — see PROJ-747's
   `RELEASE_CONTROL.md` cut-line record.
5. Live third-party network dispatch, real (non-synthesized) human consequences, and the
   whole-workspace `just verify-all` gate remain out of scope / not re-attempted, carried
   forward from the interim DoD's own honest-boundary language.
6. Phase 6 (commit) was not run this session — `git status` is not clean; HEAD is still
   `40f6020`. Nothing in this sign-off claims the increment is committed.

## Verification

Every ALIVE/PARTIAL/UNVERIFIED line in `DOD_SIGNOFF.md` traces to a command+output cited in
this session's verified-evidence list or a file:line citation confirmed by `Read`/`Grep` this
session — see `DOD_EVIDENCE_MAP.md` for the per-clause index.

## Links

- `docs/releases/v26.7.10/DOD_SIGNOFF.md`, `DOD_EVIDENCE_MAP.md`
- `docs/releases/v26.7.10/RELEASE_CONTROL.md`
- `docs/jira/v26.7.10/tickets/PROJ-731.md`
