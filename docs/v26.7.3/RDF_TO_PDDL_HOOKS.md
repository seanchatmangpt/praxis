# RDF -> PDDL: knowledge hooks and action grounding

Sources: `crates/praxis-synthesis/src/hooks.rs` (the hook registry) and
`src/ground.rs` (the grounding connector). Hooks are declared IN the
admitted graph as `hook:` nodes
(`http://seanchatmangpt.github.io/praxis/hook#`), so hook declarations are
content-addressed for free — the same law-hash that covers the graph covers
its hooks.

## The hook tuple

h = (trigger, check, act, receipt). One `hook:Hook` node declares:

| Field | Predicate | Values |
|-------|-----------|--------|
| trigger | `hook:on` | `assert` \| `retract` \| `any` (default `any`) |
| check | `hook:kind` + kind fields | see condition kinds below |
| act | `hook:effect` | `emit-delta` \| `ground-action` \| `refuse` |
| act detail | `hook:action` (IRI, required iff ground-action) / `hook:reason` (required iff refuse) | |
| identity | `hook:name` (unique), `hook:priority` 0..=7 | |

Registry bound: at most `MAX_HOOKS = 8` hooks. Every registered hook
produces a verdict record on every event — `Fired`, `NotFired`, or `Gated`
— so silence is provable. Closed-world vocabulary: an unknown `hook:`
predicate or class is `Refusal::HookIllFormed`.

## Condition kinds (supported)

- `datalog` — fires iff `hook:goal` is derivable: post-state triples become
  `t(s, p, o)` EDB facts; rules come from the bounded micro-syntax in
  `hook:program` (e.g. `head(?0) :- t(?0, <iri>, <iri>), !neg(?0).`, every
  position canonically rendered as `<iri>`, `"str"`, or int); evaluated by
  the crate's own stratified semi-naive engine. Programs are parse-validated
  at REGISTRATION, capped at `MAX_PROGRAM_BYTES = 4096`.
- `delta` — fires iff this delta touches predicate `hook:var`.
- `threshold` — count of POST-state triples with predicate `var` OP `k`.
- `count` — count of occurrences in THIS delta with predicate `var` OP `k`.
- `window` — count over this delta plus the last `window - 1` deltas,
  window 1..=8.

Operators: `=` `!=` `<` `<=` `>` `>=`.

## Condition kinds (refused BY NAME, with honest analogs)

Kinds praxis has no bounded engine for are refused at registration with
`Refusal::ConditionUnsupported`, naming the supported analog — never faked
(`hooks.rs :: REFUSED_KINDS`):

| Refused kind | Stated analog |
|--------------|---------------|
| `sparql-ask` | datalog (goal reachability) |
| `sparql-select` | datalog (goal reachability) |
| `shacl` | datalog (integrity rules) |
| `n3` | datalog (no bounded N3 engine in-tree; cold-only N3 is not implemented) |
| `semantic-inference` | (none — refused everywhere; unrdf itself throws unimplemented) |

## ground_fired_action

`src/ground.rs :: ground_fired_action` is the RDF-event -> action connector
the archaeology found missing. For a FIRED verdict with effect
`ground-action`:

1. `restrict_to_fragment` — restrict the admitted post-graph to the
   `hook:action` node's `wf:Workflow` fragment: closure under `wf:`
   object-IRI references, plus all `wf:Capability`/`wf:Constraint` nodes,
   minus every OTHER `wf:Workflow` typing (so the exactly-one-workflow law
   holds).
2. Run the fragment through the EXISTING chain: extract IR -> lower ->
   Solver8 -> derived topology/geometry -> supervised execution.

No new solver, no synthesized actions at firing time: actions are declared
before deviations, never invented during them. The result is the standard
inner v1 `WorkflowReceipt` — grounding adds no new receipt shape.

Evidence: `tests/prayer_kernel.rs ::
provision_anxiety_grounds_the_daily_prayer_workflow` (the solver derives
the clause order from declared preconditions);
`tests/prayer_kernel.rs :: v1_chain_golden_pin_direct_execution_unchanged_by_the_hook_layer`
(the inner chain is byte-identical with and without the hook layer).

## CapabilityTaskSpec bridge

`ground.rs :: capability_task_spec` projects a fired hook into a plain-data
`CapabilityTaskSpec { hook_iri, action_iri, desired_effects }` — the action
fragment's `wf:goal` atoms as sorted (predicate, first-arg) pairs. This is
the bridge TOWARD `bcinr_pddl::route_capability_plan`; the mapping onto
`bcinr_pddl::CapabilityTask` is consumed by the praxis root crate
(praxis-synthesis takes no bcinr dependency).

## Deviation routes

Every deviation is: event -> RDF delta -> hook -> pre-declared grounded
action (or declared refusal) -> receipt. No LLM replanning anywhere. The
implemented routes and their tests (`tests/deviation_routes.rs`,
`tests/prayer_kernel.rs`):

| Deviation | Hook kind | Effect | Evidence |
|-----------|-----------|--------|----------|
| Provision anxiety (daily bread) | delta | ground-action | `prayer_kernel.rs :: provision_anxiety_grounds_the_daily_prayer_workflow` |
| Open resentment (forgive-debtors) | datalog | ground-action | `prayer_kernel.rs :: resentment_open_loop_fires_by_datalog_rule_and_release_quiets_it` |
| Unbounded threat (deliverance) | delta | refuse (surrender) | `prayer_kernel.rs :: unbounded_threat_is_surrendered_not_computed` |
| Temptation overload (guard) | threshold | refuse | `prayer_kernel.rs :: day_window_over_the_eight_bound_trips_the_temptation_guard` |
| Unrepaired debt | datalog | ground-action | `deviation_routes.rs :: open_debt_fires_by_rule_and_grounds_confess_and_repair`, `repaired_debt_quiets_the_debt_rule` |
| Missing receipt | delta | ground-action | `deviation_routes.rs :: missing_receipt_grounds_the_one_step_repair_fragment` |
| Day-window overload | count | refuse (declared reschedule reason) | `deviation_routes.rs :: five_same_day_placements_in_one_delta_refuse_with_reschedule`, `four_same_day_placements_do_not_trip_the_overload` |
| Sponsor withdrawal | delta (on `life#withdrawsCapability`) | refuse (park for the human) | `deviation_routes.rs :: sponsor_withdrawal_refuses_and_parks_for_the_human` |
