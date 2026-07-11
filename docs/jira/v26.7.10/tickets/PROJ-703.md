# PROJ-703 — Deterministic PDDL renderer + round-trip property test

Status: ALIVE — evidenced this session (uncommitted; HEAD `40f6020`, Phase 6 commit not run)

Track: P (planning/decomposition).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

Renderer turns a Problem graph back into deterministic PDDL text
(`templates/decomp-problem.template.pddl`) for the unchanged
`bcinr_pddl::GroundProblem::find_plan` path — the proven planner is not forked. Round-trip
property test: lift ∘ render = id, byte-stable. Gate: G2.

## Evidence (this session)

`crates/cng/src/bench/decomp/render.rs` + `templates/decomp-problem.template.pddl`,
`templates/decomp-domain.template.pddl` on disk. `lift_render_round_trip_preserves_atom_sets`
(`decomp_test.rs:336`), part of the green 107-test `cargo test -p cng --features bench` run
this session. Note (PROJ-733, unrelated to this ticket's scope): the grounding stage
downstream of `find_plan` now uses `pddl_index::IndexedGroundProblem`, not
`bcinr_pddl::GroundProblem`, for performance; the renderer's output format and this ticket's
round-trip property are unaffected — the swap is at the grounder, not the PDDL text renderer.
