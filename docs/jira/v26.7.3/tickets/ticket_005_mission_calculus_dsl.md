# Ticket: Evaluate an ORTAC+-Style Ontology-to-PDDL Authoring Layer, Scoped

## Title
ADOPT/DEFER verdict on a higher-level authoring front-end over the existing TTL grounding pipeline (PROJ-205)

## Description
The ORTAC+ comparison observes that PDDL is too low-level for domain operators (e.g. military
mission planners), so a domain-specific language compiles down to PDDL for them.
Praxis-synthesis's `crates/praxis-synthesis/src/ground.rs` already has an equivalent shape: a
TTL `wf:Workflow` fragment (the "DSL," expressed as RDF) is grounded through
`extract_ir -> lower -> Solver8` into an executable plan — the ontology vocabulary itself is
already the authoring surface, and it is arguably higher-level than PDDL since it's declarative
graph data rather than a bespoke planning-language syntax.

This ticket asks one narrow, falsifiable question: is hand-writing Turtle fragments (as seen in
`ontology/lord_prayer.ttl`) an ergonomics problem worth solving with a thinner front-end (e.g. a
small YAML/TOML schema that emits the existing TTL vocabulary), or is directly authoring TTL
sufficient for this project's actual authors? This is explicitly NOT a request to build a
general "universal mission calculus" (that claim from the source material is unfalsifiable and
belongs on the Refuse list in `index.md`) — only a concrete, scoped authoring-ergonomics
evaluation with a yes/no verdict.

## Acceptance Criteria
- A written ADOPT/DEFER verdict, with the reasoning, is added as a new doc
  (`docs/v26.7.3/AUTHORING_ERGONOMICS.md` or a section of `docs/v26.7.3/RDF_TO_PDDL_HOOKS.md`).
- If DEFER (plausible expected outcome given the project has exactly one hand-authored
  ontology file and no evidence of authoring friction yet): no new code.
- If ADOPT: a minimal front-end is built that emits the EXISTING TTL vocabulary unchanged (no
  new grounding semantics), with a round-trip test: front-end input -> generated TTL ->
  `extract_ir`/`ground_fired_action` pipeline produces the identical result as hand-written TTL
  for an equivalent fragment.
- No new Cargo dependency without explicit justification in the ticket closure.

## Dependencies
None.

## Verification Mechanism
1. Read `crates/praxis-synthesis/src/ground.rs` and `ontology/lord_prayer.ttl` via the Read
   tool to ground the ergonomics judgment in the actual current authoring experience, not
   assumption.
2. If ADOPT: `cargo test -p praxis-synthesis` including a new round-trip test, green.
3. If DEFER: no test changes required; the doc verdict is the only deliverable.
