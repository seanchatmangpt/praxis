# PROJ-725 — Arazzo 1.1 vocab/shape delta + REMOTE_* projection

Status: ALIVE — evidenced this session (uncommitted; HEAD `40f6020`, Phase 6 commit not run)

Track: E (multi-engine execution).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

Admit `asyncapi` sourceType plus only the Arazzo 1.1 step fields the projection needs; shapes
stay closed; unsupported 1.1 features are refused by name. Project dispatch onto the new
REMOTE_* states of the 16-state machine (PROJ-720). RDF stays authoritative; YAML is a
projection artifact (PROJ-726). Gate: G12.

## Evidence (this session)

`crates/cng/shapes/arazzo-shapes.ttl:10-11,85-86` (asyncapi sourceType admitted, other 1.1
features refused by name), `CNG_R18 ArazzoProfileRefused` (`powl.rs:138,256`).
`xpath_criterion_fixture_refuses_cng_r18_naming_the_feature` (`arazzo_test.rs:34`) — part of
the `67 lib` tests in the green 107-test `cargo test -p cng --features bench` run this
session.
