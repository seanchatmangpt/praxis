# PROJ-604 — Close remaining inline-SPARQL sites and extend the guard test

Status: PLANNED

`crates/cng/tests/no_inline_ttl_guard.rs` scans all `.rs` files under `src/` and `tests/` but
its needles check for inline Turtle-prefix/PDDL markers, not inline SPARQL text, and only
`bench.rs` is actually free of inline SPARQL. `crates/cng/src/pipeline.rs:135` (1 inline
`SELECT`) and `crates/cng/src/shape.rs:75,82,122,133,146,159` (6 inline `SELECT`s) still hold
inline SPARQL strings. Move these queries to disk `.rq` files (matching the `bench.rs` pattern)
and extend the guard test to detect inline `SELECT`/`CONSTRUCT`/`ASK` text so regressions fail
loud. Links back to `docs/releases/v26.7.10/PRD.md` (Claims Reconciliation row 5) and
`RELEASE_CONTROL.md` Sec. 5.

Implementation detail: `docs/releases/v26.7.10/IMPLEMENTATION_SPEC.md` (exact edits,
anchors, tests, and acceptance commands for this ticket).
