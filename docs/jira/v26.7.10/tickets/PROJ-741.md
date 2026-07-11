# PROJ-741 — `cng plan decompose` verb (real, non-test entrypoint)

Status: ALIVE — evidenced this session (uncommitted; HEAD `40f6020`, Phase 6 commit not run)

Track: D (doctrine — marker/evidence reconciliation, Phase 3 of the closure plan).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised closure plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

## Summary

New verb `cng plan decompose --domain <path> --problem <path> --out <dir> [--base-iri <str>]`
(`crates/cng/src/main.rs:258-324`, `#[verb("decompose", "plan")]`, `#[cfg(feature =
"bench")]`) that runs `decompose()` and writes the evidence bundle (`decomposition-result.ttl`
+ OCEL construct), so the PROJ-739/740 planning markers are provable from an actual run, not
only from `cargo test` in-process state. Deliberately no `--seed` flag — `decompose()` is
fully deterministic, no randomness anywhere in the module.

## Evidence (this session)

`crates/cng/src/main.rs:258-324` (`plan_decompose` function, `DecomposeReport` struct). Two
dead-code warnings in the non-bench build (`DEFAULT_DECOMP_BASE_IRI` const, `DecomposeReport`
struct) were fixed this session by gating them `#[cfg(feature = "bench")]`, matching the
already-gated verb function itself — re-verified: `cargo check -p cng` (non-bench) is
0-warning clean; `cargo test -p cng --features bench --lib` re-confirmed 67/67 passing after
the gate fix. `planning_markers_prove_true_on_a_healthy_decompose_run` exercises the
equivalent `decompose()` call this verb wraps (part of the green 107-test full-suite run).

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` §16 (two-run composition, PROJ-743)
- `docs/jira/v26.7.10/tickets/PROJ-742.md`
