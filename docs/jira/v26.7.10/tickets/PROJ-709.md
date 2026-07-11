# PROJ-709 — POWL composition — nested PartialOrder, powl2 emission

Status: ALIVE — evidenced this session (uncommitted; HEAD `40f6020`, Phase 6 commit not run)

Track: P (planning/decomposition).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

Each subworkflow is its own total-order `PartialOrder`; the top level is one nested
`PartialOrder` whose order contains only cross-workflow `mustPrecede` edges (absent pair =
parallel). No `Powl` enum change. Also emitted as powl2 RDF so dispatch consumes the graph.
Spike downstream acceptance of nested PartialOrder first (plan risk 4). Gate: G7.

## Evidence (this session)

`crates/cng/src/bench/decomp/compose.rs` on disk. `cyclic_composed_order_refuses_cng_r21`
(`decomp_test.rs:304`) exercises composition's own refusal path; the emitted
`decomposition-result.ttl` powl2 graph is parsed and queried by every planning-marker test
(`build_decomp_marker_store`/`evaluate_planning_markers`,
`planning_markers_prove_true_on_a_healthy_decompose_run`,
`workday_test.rs:332-383`) — all green in the 107-test `cargo test -p cng --features bench`
run this session.
