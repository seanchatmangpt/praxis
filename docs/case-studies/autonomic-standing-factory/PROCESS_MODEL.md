# Process Model

## N3 and Datalog are the same praxis-graphlaw engine

`case-study/rules/judgment.n3` (N3 readiness rules) and
`case-study/rules/readiness.dl.n3` (Datalog closure rules) are loaded into
the SAME `TripleStore` via the SAME `load_rules`/`add_rules` API
(`src/bin/case_study_judge.rs`, step 2), which parses both files into
identical `Rule`/`BodyLiteral` structs
(`praxis_graphlaw::TripleStore::load_rules`) and RE-STRATIFIES the COMBINED
rule set on every load (`crates/praxis-graphlaw/src/lib.rs::add_rules` →
`datalog::validate_rules`). There is exactly one forward-chaining reasoner
(`crates/praxis-graphlaw/src/reasoner/mod.rs::Reasoner::materialize`) and
one stratifier for the whole rule set — "N3" and "Datalog" here name two
FILES distinguished by convention (judgment.n3 = readiness-fact derivation
and mutually-exclusive verdict classification; readiness.dl.n3 =
transitive-closure + stratified-negation dependency accounting), not two
engines. `readiness.dl.n3`'s own header states this explicitly and Lane 2's
report cites the exact source lines.

The one place text-syntax rules cannot express something
(`unsatisfiedDependencyCount`, a COUNT aggregate) is handled by a Rust-only
API, `TripleStore::add_rule_with_aggregate` — no N3/Datalog TEXT syntax for
aggregates exists in this crate (`readiness.dl.n3`'s header documents this
too).

## The case-study POWL model (Lane 3)

See `POWL_EXECUTION_MODEL.md` for the full table. Summary: 16 children
(event types), 114 order pairs, built via a new `--model case-study` flag
on the existing `ocel_process_validate` binary — additive, the v26.7.6
release model's `CHILD_SPECS`/`ORDER_LABEL_PAIRS`/`release_loop_model()`
are byte-for-byte unchanged (`build_loop_model(specs, pairs)` is the shared
helper both models now call).

## Partial-order deviation

Lane 3 flagged, before any real log existed, that the case-study's order
pairs were ASSERTED from the ticket specification rather than mined from an
observed trace, and instructed Lane 6 to resolve explicitly if Lane 4's
real capture showed a different execution order (e.g. `ocel_log_written`
occurring earlier than modeled).

**Resolution (Lane 6): no deviation found.** Lane 4's real OCEL capture
(`case-study/ocel_case_study.json`, 20 events) was checked against the
asserted model via `ocel_process_validate --model case-study
ocel_case_study.json`, which reports `is_conforming: true, fitness: 1.0,
violations: []` — the real execution order the driver produced conforms to
the asserted partial order without any `CASE_STUDY_ORDER_LABEL_PAIRS` edit.
Lane 6 independently re-ran this exact check (see
`WASM4PM_VALIDATION_REPORT.md`) and confirmed the same result.

## GraphLaw's own process (Lane 2 + Lane 6)

`case_study_judge`'s `run()` executes in three `materialize()` stages
(structural, post-gate-injection, post-aggregate) — see
`GRAPHLAW_JUDGMENT_MODEL.md` for the real defect Lane 6 found in this
staging (a query bug in verdict selection, fixed; plus a disclosed,
not-yet-fixed graph-consistency artifact of the staged materialize() design
itself).
