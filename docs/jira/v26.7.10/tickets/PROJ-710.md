# PROJ-710 — Selection law, candidate receipts, typed DecompositionOutcome

Status: ALIVE — evidenced this session (uncommitted; HEAD `40f6020`, Phase 6 commit not run)

Track: P (planning/decomposition).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

Lexicographic selection (Makespan via Kahn longest path; DispatchCost = actions + k·overhead
so single-actor can win; Risk = cross-workflow edges), canonical-id tie-break. Per-candidate
receipts for accepted AND rejected candidates. `NO_ADMISSIBLE_DECOMPOSITION` /
`NO_BENEFICIAL_DECOMPOSITION` are typed success results (`DecompositionOutcome`), receipted,
never refusals or silent fallbacks; `CNG_R21 DecompositionInadmissible` covers the refusal
cases. `audit_replay` recomputes the argmin. Integration point for Track E: this ticket's
`DecompositionResult` graph is what PROJ-723/725 dispatch across engines. Gate: G7.

## Evidence (this session)

`crates/cng/src/bench/decomp/select.rs`, `CNG_R21 DecompositionInadmissible`
(`powl.rs:170-185,259`). Tests: `forcing_an_unknown_candidate_refuses_cng_r21`
(`cng_decomp.rs:145`), `forced_inadmissible_candidate_refuses_cng_r21`
(`decomp_test.rs:276`), `single_atom_goal_yields_no_admissible_decomposition`
(`decomp_test.rs:422`), `decompose_is_deterministic_across_runs` (`decomp_test.rs:443`,
byte-identical result graphs across two same-input runs) — all green in the 107-test `cargo
test -p cng --features bench` run this session.
