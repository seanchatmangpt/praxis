# Ticket: Document the "Cognitive Breed" Catalog as a Mapping onto Existing Modules

## Title
Write `docs/v26.7.3/COGNITIVE_BREED_MAPPING.md` mapping named reasoning roles to existing code (PROJ-206)

## Description
The "cognitive breeds" / "periodic table of cognition" material proposes named bounded-reasoning
roles (Guardian, Detector, Tracker, Retriever, Planner, Herding, Recorder, Verifier, Translator,
Cartographer, Broker, Dachshund, Service, Meta). Rather than building a new "breed" abstraction
in code — which would be pure vocabulary with no new behavior — this ticket produces a doc that
cites, for each named role, the praxis-synthesis module that already performs that function.
This keeps the vocabulary traceable to code (falsifiable) instead of aspirational.

Expected mapping (to be confirmed against actual code during the ticket, not assumed):
- **Guardian** -> `quarantine.rs` (`RiceQuarantine::inspect`, `Admission::admit`) +
  `kernel.rs::enforce_surrender_boundary`
- **Detector** -> `hooks.rs` (delta/threshold/count/window condition evaluation)
- **Tracker** -> `firing.rs` outer chain fold order (event -> admission -> ... -> outcome)
- **Retriever** -> `life.rs` query helpers (`open_resentments`, `open_debts`, etc.)
- **Planner** -> `ground.rs` + `solver8.rs`/`sequence.rs`
- **Herding** -> `graph.rs`'s workflow execution / `dag.rs` supervised execution
- **Recorder** -> `firing.rs` receipt folding, `envelope.rs`
- **Verifier** -> `scripts/foreign_verify_graph.py` + `firing::replay_firing`
- **Translator** -> N/A or REFUSED (no public-ontology-binding layer exists yet — name this
  gap honestly rather than force a mapping)
- **Cartographer/Broker/Dachshund/Service/Meta** -> audit each; if no code home exists, mark
  REFUSED/NOT IMPLEMENTED rather than inventing a placeholder module.

Any role with **no existing code home** is marked plainly as not implemented in this doc — it
does NOT automatically become a new ticket unless a concrete use case demands the capability.

## Acceptance Criteria
- `docs/v26.7.3/COGNITIVE_BREED_MAPPING.md` exists with one entry per named breed: either a
  cited file + function/type, or an explicit "NOT IMPLEMENTED" marker.
- Zero new Rust types, traits, or modules named after "breeds" are introduced — this is a
  documentation-only ticket.
- The doc does not use the word "physics" as a claim of scientific grounding — it is careful,
  named engineering-role vocabulary only.

## Dependencies
PROJ-203, PROJ-204 (their audits determine whether the Guardian/Verifier mappings in this doc
are accurate — do not write this doc before those land).

## Verification Mechanism
1. Manual review: every cited file/function in the doc actually exists (spot-check via Read
   tool, do not cite from memory).
2. `cargo test -p praxis-synthesis` unaffected (docs-only change) — green.
3. No new source files added under `crates/praxis-synthesis/src/` as a result of this ticket.
