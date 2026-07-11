# PROJ-704 — Decomposition Datalog rules (decomp.dl, decomp-resources.dl)

Status: ALIVE — evidenced this session (uncommitted; HEAD `40f6020`, Phase 6 commit not run)

Track: P (planning/decomposition).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

`rules/decomp.dl` derives `achieves/threatens/mutex/dependsOn/mustPrecede/custodyConflict/
releasesResource` from precondition ∩ effect structure (stratified, NAF), plus EDB queries.
Resource predicates are admitted facts in `decomp-resources.dl`, never Rust constants
(anti-hardcoding doctrine, DoD §17). Gate: G3.

## Evidence (this session)

`crates/cng/rules/decomp.dl`, `crates/cng/rules/decomp-resources.dl`,
`crates/cng/src/bench/decomp/rules.rs` on disk; exercised by every `decompose()` call in the
green 107-test suite. This is also the exact join PROJ-733 fixed this session for
performance — `decomp/rules.rs:126`'s documented `O(|rules|·|facts|²)` mutex/custody join was
the dominant cost of the pre-fix 60-120s hangs (untyped high-arity domains blew up ground-atom
degree); after the `pddl-index` grounder swap, `cargo test -p cng --features bench --test
cng_decomp`: 3/3 passed, 0.18s (was 60s+), confirming the rule-derivation path is real,
exercised, and now fast.
