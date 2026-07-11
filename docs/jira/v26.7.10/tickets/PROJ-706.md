# PROJ-706 — CONSTRUCT manufacture of helper/main subproblems

Status: ALIVE — evidenced this session (uncommitted; HEAD `1f3f9bc`, Phase 6 commit not run)

Track: P (planning/decomposition).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

Helper and main problem graphs are manufactured as RDF via SPARQL CONSTRUCT, with
decomposition provenance triples linking every manufactured atom back to the admitted facts
it derives from. No English subgoal, no per-scenario hand-authored problem text. Gate: G5.

## Evidence (this session)

`crates/cng/src/bench/decomp/manufacture.rs`,
`crates/cng/queries/decomp/construct-helper-problem.rq`,
`crates/cng/queries/decomp/construct-main-problem.rq`,
`crates/cng/queries/decomp/construct-provenance.rq` on disk; exercised by every `decompose()`
call in the green suite, e.g. `potato_decomposition_is_typed_receipted_and_replayable`
(`cng_decomp.rs:78`), part of the 107-test `cargo test -p cng --features bench` run.
