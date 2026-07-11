# PROJ-701 — pddl-strips ontology and closed shapes

Status: ALIVE — evidenced this session (uncommitted; HEAD `1f3f9bc`, Phase 6 commit not run)

Track: P (planning/decomposition).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

New clean-room `crates/cng/ontologies/pddl-strips.ttl` (name-compatible with pddlv3.1 for a
later import), with closed SHACL shapes for Action/precondition/AddEffect/DelEffect/Problem/
initAtom/goal/object. Unknown predicates refused by name; shapes stay closed. Gate: G1.

## Evidence (this session)

`crates/cng/ontologies/pddl-strips.ttl` and `crates/cng/shapes/pddl-strips-shapes.ttl` exist
on disk and are loaded/exercised by every `decompose()` call in the green suite: `cargo test
-p cng --features bench` — 107 tests total, 0 failures (`RELEASE_CONTROL.md` §9 ladder). No
standalone closed-shape-violation negative test for `pddl-strips-shapes.ttl` specifically was
identified or run this session — PARTIAL on that narrower claim; the decomposition-level
proof-obligation refusals (`CNG_R21`-`CNG_R24`) are separately evidenced under PROJ-707/708/
710/712 below.
