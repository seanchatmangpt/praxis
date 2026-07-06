# Cognitive Breed Mapping

Source ticket: `docs/jira/v26.7.3/tickets/ticket_006_cognitive_breed_catalog.md` (PROJ-206).

## Purpose

The "cognitive breed" / "periodic table of cognition" material names bounded-reasoning
roles (Guardian, Detector, Tracker, Retriever, Planner, Herding, Recorder, Verifier,
Translator, Cartographer, Broker, Dachshund, Service, Meta). This doc does not introduce a
new "breed" abstraction in code — that would be pure vocabulary with no new behavior. It
instead cites, for each named role, the `praxis-synthesis` module that already performs
that function, or states plainly that no such module exists.

This is documentation only. No new Rust types, traits, or modules are introduced by this
ticket, and none should be inferred from the naming below.

## Mapping

| Breed | Status | Citation |
|---|---|---|
| Guardian | IMPLEMENTED | `crates/praxis-synthesis/src/quarantine.rs` (`RiceQuarantine::inspect`, line 56; `Admission::admit`, line 193) + `crates/praxis-synthesis/src/kernel.rs` (`enforce_surrender_boundary`, line 411) |
| Detector | IMPLEMENTED | `crates/praxis-synthesis/src/hooks.rs` (delta/threshold/count/window condition evaluation) |
| Tracker | IMPLEMENTED | `crates/praxis-synthesis/src/firing.rs` (outer chain fold order: event -> admission -> ... -> outcome) |
| Retriever | IMPLEMENTED | `crates/praxis-synthesis/src/life.rs` (`open_resentments`, line 93; `open_debts`, line 99) |
| Planner | IMPLEMENTED | `crates/praxis-synthesis/src/ground.rs` + `crates/praxis-synthesis/src/solver8.rs` + `crates/praxis-synthesis/src/sequence.rs` |
| Herding | IMPLEMENTED | `crates/praxis-synthesis/src/graph.rs` (workflow execution) / `crates/praxis-synthesis/src/dag.rs` (`execute_supervised`, line 521; `SupervisionTopology`) |
| Recorder | IMPLEMENTED | `crates/praxis-synthesis/src/firing.rs` (receipt folding) + `crates/praxis-synthesis/src/envelope.rs` |
| Verifier | IMPLEMENTED | `scripts/foreign_verify_graph.py` + `crates/praxis-synthesis/src/firing.rs` (`replay_firing`, line 395) |
| Translator | NOT IMPLEMENTED | No public-ontology-binding layer exists in `praxis-synthesis`; this gap is named honestly rather than mapped to unrelated code. |
| Cartographer | NOT IMPLEMENTED | No code home found; `crates/praxis-synthesis/src/` was searched for this name with zero hits. |
| Broker | NOT IMPLEMENTED | No code home found; `crates/praxis-synthesis/src/` was searched for this name with zero hits. |
| Dachshund | NOT IMPLEMENTED | No code home found; `crates/praxis-synthesis/src/` was searched for this name with zero hits. |
| Service | NOT IMPLEMENTED | No code home found; `crates/praxis-synthesis/src/` was searched for this name with zero hits. |
| Meta | NOT IMPLEMENTED | No code home found; `crates/praxis-synthesis/src/` was searched for this name with zero hits. |

An unimplemented role is not automatically a new ticket. It becomes work only when a
concrete use case demands the capability.

## Downstream use

This doc exists so that PROJ-305 can promote the mapping above into a compile-checked
const table in `praxis-synthesis`, rather than PROJ-305 re-deriving the mapping from
scratch. Until that promotion happens, this file is the single source of truth for the
breed-to-code mapping and is not itself enforced by the compiler or by tests.
