# PROJ-705 — Bounded canonical candidate enumeration

Status: ALIVE — evidenced this session (uncommitted; HEAD `1f3f9bc`, Phase 6 commit not run)

Track: P (planning/decomposition).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

`src/bench/decomp/search.rs`: union-find partitioning of goal atoms over derived edges;
canonical lexicographic enumeration of 2-way splits (max 8 components / 32 candidates);
single-actor is always candidate #0. Bounds are declared and receipted; exceeding a bound is
a typed result, never a panic or silent truncation. Gate: G4.

## Evidence (this session)

`crates/cng/src/bench/decomp/search.rs` on disk. `single_actor_is_always_candidate_zero`
(`decomp_test.rs:315`), part of the green 107-test `cargo test -p cng --features bench` run
this session.
