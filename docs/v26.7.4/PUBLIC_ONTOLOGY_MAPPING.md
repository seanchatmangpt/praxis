# Public Ontology Mapping — PROJ-302

Every predicate in praxis's closed-world vocabularies (`wf:`, `hook:`,
`prayer-kernel:`, `agent:`) is enumerated here. For each, either:

- **Public analog**: the nearest public-ontology predicate it would substitute
  for, and why that public predicate did not suffice as-is, or
- **Operational machinery**: it names praxis-internal solver/registry state
  with no real-world referent (a lookup key, an enum tag, a positional slot),
  so no public-ontology predicate applies.

This doc is a checked gate, not prose: `tests/no_private_abstraction.rs`
parses the predicate tokens below and asserts they are a superset of every
predicate name actually declared in `WF_PREDICATES` (`src/graph.rs`),
`HOOK_PREDICATES` (`src/hooks.rs`), `KERNEL_PREDICATES` (`src/kernel.rs`), and
`AGENT_PREDICATES` (`src/agent_registry.rs`). A new private predicate added to
any of those tables without a row here fails that test.

## `wf:` — workflow vocabulary (`src/graph.rs`)

| Predicate | Public analog | Justification |
|---|---|---|
| `wf:budget` | — | Operational: solver step horizon, an internal search-bound, not a real-world quantity. |
| `wf:init` | — | Operational: reference list into the initial-state atom set of the planning IR. |
| `wf:goal` | — | Operational: reference list into the goal atom set of the planning IR. |
| `wf:name` | `rdfs:label` | Rejected: `rdfs:label` is a human-display string; `wf:name` is an exact-match identity key the extractor uniqueness-checks, which `rdfs:label` does not guarantee. |
| `wf:params` | — | Operational: declared arity (parameter count) of a capability, 0..=8. |
| `wf:cost` | — | Operational: solver cost metric, not a real-world monetary or physical cost. |
| `wf:pre` | — | Operational: STRIPS-style precondition atom list. |
| `wf:add` | — | Operational: STRIPS-style add-effect atom list. |
| `wf:del` | — | Operational: STRIPS-style delete-effect atom list. |
| `wf:predicate` | — | Operational: names a datalog predicate symbol, an internal IR reference, not a real-world relation. |
| `wf:arg0` | — | Operational: positional datalog term slot 0 (contiguous 0..8), not a real-world property. |
| `wf:arg1` | — | Operational: positional datalog term slot 1. |
| `wf:arg2` | — | Operational: positional datalog term slot 2. |
| `wf:arg3` | — | Operational: positional datalog term slot 3. |
| `wf:arg4` | — | Operational: positional datalog term slot 4. |
| `wf:arg5` | — | Operational: positional datalog term slot 5. |
| `wf:arg6` | — | Operational: positional datalog term slot 6. |
| `wf:arg7` | — | Operational: positional datalog term slot 7. |
| `wf:kind` | — | Operational: closed enum discriminator tag shared by atoms/constraints. |
| `wf:a` | — | Operational: first capability-name slot of a binary constraint relation. |
| `wf:b` | — | Operational: second capability-name slot of a binary constraint relation. |
| `wf:k` | — | Operational: numeric bound parameter (step index, count, or budget). |
| `wf:handler` | `prov:wasAssociatedWith` | Rejected: `prov:wasAssociatedWith` names a real-world responsible agent; `wf:handler` names an internal dispatch-registry key resolving to in-process Rust code, not a real-world identity. |
| `wf:delegability` | — | Operational: lattice classification tag over the delegability lattice. |
| `wf:capability` | — | Operational: typed link from a workflow node to a declared `wf:Capability` node. |
| `wf:constraint` | — | Operational: typed link from a workflow node to a declared `wf:Constraint` node. |

## `hook:` — knowledge-hook vocabulary (`src/hooks.rs`)

| Predicate | Public analog | Justification |
|---|---|---|
| `hook:name` | `rdfs:label` | Rejected for the same reason as `wf:name`: an exact-match registry key, not a display label. |
| `hook:on` | — | Operational: names the trigger event kind (`OnCommit`/`OnDelta`/...), a closed enum tag. |
| `hook:kind` | — | Operational: condition-kind discriminator (`datalog`/`delta`/`threshold`/`count`/`window`). |
| `hook:var` | — | Operational: datalog variable name referenced by a `threshold`/`count`/`window` condition. |
| `hook:op` | — | Operational: comparison operator symbol for a `threshold` condition. |
| `hook:k` | — | Operational: numeric threshold or window bound. |
| `hook:window` | — | Operational: event-window size for a `window` condition. |
| `hook:program` | — | Operational: embedded datalog program text for a `datalog` condition. |
| `hook:goal` | — | Operational: reference to the datalog goal atom the condition checks reachability of. |
| `hook:action` | `prov:Activity` | Rejected: `prov:Activity` models a real-world activity that occurred; `hook:action` references an unfired action fragment the hook *would* execute, a plan artifact, not an occurrence. |
| `hook:effect` | — | Operational: reference to the graph-delta descriptor a firing applies. |
| `hook:reason` | `dct:description` | Rejected: `dct:description` is open-ended human prose; `hook:reason` is a short machine-audited refusal/verdict string with a bounded shape. |
| `hook:priority` | — | Operational: firing-order tiebreak integer, not a real-world priority. |

## `prayer-kernel:` — Lord's Prayer kernel vocabulary (`src/kernel.rs`)

| Predicate | Public analog | Justification |
|---|---|---|
| `prayer-kernel:clause` | — | Operational: typed link from the kernel node to one of its 11 declared clause nodes. |
| `prayer-kernel:name` | `rdfs:label` | Rejected for the same reason as `wf:name`: must exactly match one of the 11 `CANONICAL_CLAUSES` names, not a free display label. |
| `prayer-kernel:problemClass` | — | Operational: closed classification tag naming the clause's problem class. |
| `prayer-kernel:boundary` | — | Operational: one of the three lawful `BOUNDARIES` strings; a delegation-boundary marker, not a real-world property. |
| `prayer-kernel:action` | `prov:Activity` | Rejected for the same reason as `hook:action`: names a delegated action descriptor, not an activity that occurred. |

## `agent:` — agent metadata vocabulary (`src/agent_registry.rs`)

| Predicate | Public analog | Justification |
|---|---|---|
| `agent:tool` | `prov:used` | Rejected: `prov:used` binds an activity's execution-time use of an entity; `agent:tool` is a static declared capability set on an agent profile, not an execution-time fact. |
| `agent:canSpawn` | — | Operational: praxis-internal spawn-authorization edge to another agent type; no real-world referent. |
| `agent:layerDepth` | — | Operational: position (1..=5) in the five-layer agent hierarchy, an internal structural index. |
