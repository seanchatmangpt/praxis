# PROJ-743 — DoD §16 prose reconciled to exact on-disk marker names

Status: DONE (doc) — evidenced this session (uncommitted; HEAD `1f3f9bc`, Phase 6 commit not
run)

Track: D (doc closure wave, Phase 5 of the closure plan; agent: release-doc).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised closure plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

## Summary

Reconciled `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` §16 to the exact on-disk marker
names and query files (per decision 4 of the closure plan: reconcile doctrine to code, never
the reverse). Three sub-corrections beyond the planning-set cosmetic `_PROVEN`/`_ZERO` suffix
convention:

1. Planning set (PROJ-739/740) and `LLM_CALLS_ZERO` family named exactly, with their query
   files and the `PLANNING_MARKER_MAP`/`build_decomp_marker_store`/`evaluate_planning_markers`
   machinery (PROJ-742) that evaluates them over a DEDICATED `decomposition-result.ttl` store,
   never the obs∪evidence∪registry union.
2. Distributed set (PROJ-727) reconciled: prior prose named `ENGINE_INSTANCES`,
   `ARAZZO_WORKFLOWS_GENERATED`, `REMOTE_DISPATCHES_SENT`, `REMOTE_CONSEQUENCES_ADMITTED`,
   `RESUME_VERIFIED` — none exist on disk under those identifiers. Reconciled to the real
   `DISTRIBUTED_MARKER_MAP` (nine names across six query files).
3. `CRASH_RESUME_PROVEN` — named in prior "Revised final markers" prose but does not exist
   anywhere in `crates/cng`. Reconciled: G13 is proven by `REPLAY_DIVERGENCES_ZERO` (folding in
   the `resume_verified` obs-kind check) plus the `g13_crash_resume_verifies_chain_and_
   completes` integration test.

Also documented, as the load-bearing correction: `V26_7_10_PRODUCTION_READY`'s two-run
composition via `full_production_ready` (PROJ-742) — `workday()` alone never proves the full
§16 conjunction — and the honest gap that this combinator's real two-bundle invocation is
itself UNVERIFIED this session (constituent marker families verified separately; the
end-to-end composition was not exercised).

## Verification

`docs/releases/v26.7.10/DEFINITION_OF_DONE.md` §16 resolves to file paths and test names that
were read/greeped and confirmed on disk this session (see the section's own citations). No
`cargo`/`just` command was re-run for this ticket specifically — it cites the command+output
evidence already produced by PROJ-733/734/739..742's sessions in the same pass.

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` §16
- `docs/jira/v26.7.10/tickets/PROJ-739.md`, `PROJ-740.md`, `PROJ-741.md`, `PROJ-742.md`
