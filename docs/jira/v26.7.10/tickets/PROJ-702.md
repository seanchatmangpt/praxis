# PROJ-702 — PDDL lifter — string literal to pddl-strips triples

Status: ALIVE — evidenced this session (uncommitted; HEAD `1f3f9bc`, Phase 6 commit not run)

Track: P (planning/decomposition).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

A Rust lifter parses admitted PDDL strings (existing bcinr parser) and CONSTRUCTs pddl-strips
triples into a fresh store (CONSTRUCT-into-new-store). PDDL today is opaque string literals in
RDF (`ceng:pddlDomain/pddlProblem`); after this ticket the STRIPS structure is queryable.
Gate: G2 (with PROJ-703).

## Evidence (this session)

`crates/cng/src/bench/decomp/lift.rs` on disk. `lift_render_round_trip_preserves_atom_sets`
(`crates/cng/src/bench/decomp/decomp_test.rs:336`), part of the `67 lib` tests in the green
`cargo test -p cng --features bench` run (107 tests, 0 failures, this session).
