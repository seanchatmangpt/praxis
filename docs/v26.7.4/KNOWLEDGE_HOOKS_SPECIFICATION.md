# Knowledge Hooks Specification — Praxis v26.7.4

**Status:** Technical Specification  
**Version:** 26.7.4  
**Authors:** Praxis Contributors  
**Last Updated:** 2026-07-08  

## Abstract

Knowledge Hooks are graph-declared, admission-gated trigger/condition/action/receipt units that enable deterministic, auditable, receipt-bearing graph transformations. This specification defines how hooks are represented in RDF, admitted through closed-vocabulary and SHACL-enforced gating, scheduled deterministically via dependency ordering, evaluated against nine condition dialects, and finally projected onto graph state with cryptographic receipts that enable idempotent replay.

The specification preserves two immutable principles:
1. **Hooks project, they do not silently actuate.** Every hook outcome—fired, not-fired, or gated—is recorded as a verdict. Every non-empty projection produces a receipt with a BLAKE3 delta hash and idempotency key.
2. **No silent promotion.** Unsupported features are refused at admission time. Unknown predicates, forbidden keywords, unrecognized handlers, and malformed conditions fail gating and roll back the hook pack atomically.

## 1. Scope and Normative References

This specification applies to Knowledge Hook producers (systems that declare hooks in RDF), consumers (systems that evaluate hooks), hook packs (collections of admitted hooks), hook engines (evaluation engines), and replay validators (systems that verify receipt non-ornamentality).

**Referenced Standards:**
- RDF 1.1 (W3C Recommendation) — Semantic Web standards for triple representation.
- SHACL (W3C Recommendation) — Shape validation and constraint enforcement.
- SPARQL 1.1 (W3C Recommendation) — Graph query and update syntax.
- N-Quads (W3C Recommendation) — Canonical serialization of RDF quads.
- RFC 2119 (IETF) — Keywords for normative language (MUST, SHOULD, etc.).
- BLAKE3 (specification) — Cryptographic hash function for delta integrity.

## 2. Normative Language and Definitions

### 2.1 Key Terms

**Hook:** A declarative trigger/condition/action unit represented as an RDF resource of type `kh:Hook`.

**Hook Pack:** An atomic collection of one or more hooks declared in a single Turtle or N3 document, admitted or refused as a unit.

**Admission:** The process by which a hook pack is validated against SHACL shapes, closed vocabularies, and constitutional constraints. A pack is either fully admitted (all hooks become active) or fully refused (no changes to store state).

**Condition:** A boolean predicate that determines hook eligibility, evaluated deterministically against post-state, delta, or history.

**Effect:** The outcome of a fired hook: `EmitDelta` (projects graph changes), `GroundAction` (projects changes via a SPARQL CONSTRUCT handler), or `Refuse` (rejects the entire stratum).

**Verdict:** The recorded outcome of hook evaluation for a single change transaction: `Fired` (condition held), `NotFired` (condition did not hold), or `Gated` (event gate prevented evaluation).

**Receipt:** A cryptographic record of a non-ornamental hook firing, containing the hook name, delta hash (BLAKE3), idempotency key, and canonical N-Quads representation of added/removed triples.

**Delta:** The set of triple additions and removals in a single stratum iteration.

**Idempotency Key:** A BLAKE3 hash derived from the delta hash, enabling deterministic replay and duplicate-entry detection.

**Boundary Adapter:** An explicit mechanism (currently limited to SPARQL CONSTRUCT handlers) for projecting hook outcomes across system boundaries. Direct side-effects are refused.

**Refusal:** A first-class outcome: a hook firing with `Refuse` effect terminates the current stratum, rolls back all inferred triples from that stratum, and returns an error with the hook's reason.

### 2.2 Closed Vocabulary

Knowledge Hook declarations MUST use only the following predicates in the `http://seanchatmangpt.github.io/praxis/kh#` namespace:

```
kh:name, kh:kind, kh:on, kh:var, kh:op, kh:k, kh:window, 
kh:program, kh:goal, kh:query, kh:effect, kh:action, 
kh:reason, kh:priority, kh:after, kh:handler, kh:adds_ttl
```

Any other predicate in the `kh:` namespace MUST be refused with a gating error.

## 3. Hook Declaration and Representation

### 3.1 Hook Triple Structure

A hook is declared as an RDF resource with the following properties:

```turtle
@prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
@prefix ex: <http://example.org/> .

ex:my_hook a kh:Hook ;
    kh:name "hook_name" ;
    kh:kind "condition_kind" ;
    kh:on "assert|retract|any" ;
    kh:effect "emit-delta|ground-action|refuse" ;
    kh:priority 0 ;
    kh:after ex:other_hook .
```

### 3.2 Hook Properties

| Property | Type | Cardinality | Description |
|----------|------|-------------|-------------|
| `rdf:type` | URI | Exactly 1 | MUST be `kh:Hook` |
| `kh:name` | string | Exactly 1 | Human-readable hook identifier; no whitespace or special characters |
| `kh:kind` | string | Exactly 1 | Condition dialect: one of `datalog`, `delta`, `threshold`, `count`, `window`, `shacl`, `shex`, `n3`, `sparql` |
| `kh:on` | string | 0..1 | Event gate: `assert` (additions only), `retract` (deletions only), `any` (default) |
| `kh:effect` | string | Exactly 1 | Outcome type: `emit-delta`, `ground-action`, or `refuse` |
| `kh:var` | string | 0..1 | Predicate or variable name (dialect-dependent) |
| `kh:op` | string | 0..1 | Comparison operator: `=`, `!=`, `<`, `<=`, `>`, `>=` |
| `kh:k` | integer | 0..1 | Numeric threshold value |
| `kh:window` | integer | 0..1 | History window size (for window conditions) |
| `kh:program` | string | 0..1 | Datalog/N3 rules source text |
| `kh:goal` | string | 0..1 | Datalog goal predicate or N3 rule target |
| `kh:query` | string | 0..1 | SPARQL query string |
| `kh:action` | URI | 0..1 | Reference to a `kh:Action` resource (required for `ground-action` effect) |
| `kh:reason` | string | 0..1 | Refusal reason text (required for `refuse` effect) |
| `kh:priority` | integer | 0..1 | Scheduling priority; lower values execute first (default 0) |
| `kh:after` | URI | 0..∞ | Dependency constraint(s); hook must execute after named hook(s) |

### 3.3 Action Resources

When a hook's effect is `ground-action`, it MUST reference a `kh:Action` resource:

```turtle
ex:my_action a kh:Action ;
    kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
    kh:query "CONSTRUCT { ... } WHERE { ... }" .
```

The only supported handler is `http://seanchatmangpt.github.io/praxis/handler#sparql-construct`. All other handlers MUST be refused.

## 4. Hook Pack Admission

### 4.1 Admission Gate Overview

When a hook pack (one or more triples with `rdf:type kh:Hook`) is submitted to the store, it MUST pass the following admission gates in order:

1. **SHACL Gating** — Structural validation
2. **Closed Vocabulary Gating** — Predicate restriction
3. **Forbidden Keyword Gating** — Security boundary
4. **Condition Validation** — Dialect-specific checks
5. **Effect Validation** — Effect-to-property mapping
6. **Constitutional Gating** — Handler and side-effect restrictions

If any gate fails, the entire pack is refused and rolled back; no triples are added to the store.

### 4.2 SHACL Gating

All hooks in a pack MUST conform to the SHACL Law Pack schema. The schema enforces:

**HookShape:**
- Target: all resources of type `kh:Hook`
- `sh:closed true` — no properties other than those explicitly listed
- `sh:ignoredProperties (rdf:type)` — RDF type is not counted as a shape violation
- Property constraints:
  - `kh:name`: datatype string, minCount 1, maxCount 1 (exactly one)
  - `kh:kind`: datatype string, minCount 1, maxCount 1 (exactly one)
  - `kh:on`: datatype string, minCount 0, maxCount 1 (optional)
  - `kh:effect`: datatype string, minCount 1, maxCount 1 (exactly one)
  - All other properties: minCount 0, maxCount 1 (optional, at most one value)
  - `kh:after`: nodeKind `sh:IRI`, minCount 0, maxCount ∞ (multi-valued)

**ActionShape:**
- Target: all resources of type `kh:Action`
- `sh:closed true` — no unexpected properties
- `kh:handler`: nodeKind `sh:IRI`, minCount 1, maxCount 1 (exactly one)
- `kh:query`: datatype string, minCount 0, maxCount 1 (optional)

If SHACL validation fails, return a validation error listing focus nodes, result paths, and constraint violations. The pack is refused and rolled back.

### 4.3 Closed Vocabulary Gating

Any triple where the predicate is in the `http://seanchatmangpt.github.io/praxis/kh#` namespace MUST have a local name (the part after `#`) that appears in the ALLOWED_KH_PREDICATES list:

```
name, on, kind, var, op, k, window, program, goal, query, effect, 
action, reason, priority, after, handler
```

Any other `kh:` predicate MUST be refused with an error message naming the forbidden predicate.

### 4.4 Forbidden Keyword Gating

All subject, predicate, object, and graph IRIs in the hook pack MUST be scanned for the following suspicious keywords in case-insensitive form:

```
shell, exec, curl, socket, fetch
```

If a keyword is found, it MUST be allowed ONLY if the string begins with one of the following prefixes:

```
http://seanchatmangpt.github.io/praxis/
http://www.w3.org/
```

Otherwise, the hook pack is refused with an error naming the forbidden keyword and the term where it appears.

### 4.5 Condition Validation

Depending on the value of `kh:kind`, the following properties MUST be present and validated:

| Kind | Required Properties | Validation |
|------|-------------------|------------|
| `datalog` | `kh:program`, `kh:goal` | Program MUST be parseable Datalog syntax; ≤ 8 rules; goal MUST be a predicate name; no reserved head predicate "t" |
| `delta` | `kh:var` | Var MUST be a valid IRI or predicate reference |
| `threshold` | `kh:var`, `kh:op`, `kh:k` | Op MUST be one of `=`, `!=`, `<`, `<=`, `>`, `>=`; k MUST be a non-negative integer |
| `count` | `kh:var`, `kh:op`, `kh:k` | Same as threshold |
| `window` | `kh:var`, `kh:op`, `kh:k`, `kh:window` | Same as threshold; window MUST be a positive integer ≤ 255 |
| `shacl` | `kh:program` | Program MUST be parseable SHACL shapes in Turtle syntax |
| `shex` | `kh:program`, `kh:goal` | Program MUST be parseable ShEx schema; goal MUST be a valid shape map syntax |
| `n3` | `kh:program` | Program MUST be parseable N3 rules |
| `sparql` | `kh:query` | Query MUST be parseable SPARQL (SELECT, ASK, or CONSTRUCT) |

If validation fails, return an error describing the syntax or semantic failure. The pack is refused and rolled back.

### 4.6 Effect Validation

If `kh:effect` is `emit-delta`:
- Hook MUST NOT have a `kh:action` property (though having one does not cause refusal; it is ignored).
- Projection comes from evaluating the condition's output directly.

If `kh:effect` is `ground-action`:
- Hook MUST have a `kh:action` property pointing to a `kh:Action` resource.
- If missing, refuse with error "effect 'ground-action' requires kh:action".

If `kh:effect` is `refuse`:
- Hook MUST have a `kh:reason` property containing a string explanation.
- If missing, refuse with error "effect 'refuse' requires kh:reason".

Any other value for `kh:effect` MUST be refused with an error naming the unknown effect.

### 4.7 Constitutional Gating

**Handler Restriction:**
If a hook has a `kh:action` property, that action MUST have exactly one `kh:handler` property. The handler value MUST be the IRI `http://seanchatmangpt.github.io/praxis/handler#sparql-construct`. All other handlers MUST be refused with an error "forbidden or unrecognized handler: [value]".

**Direct Side-Effect Refusal:**
If a hook's action contains a SPARQL CONSTRUCT query, the template section MUST NOT modify triples in the `http://seanchatmangpt.github.io/praxis/kh#` namespace (the hook registry itself). Any CONSTRUCT query that would write to `kh:` predicates MUST be refused with error "CONSTRUCT template attempts to modify hook registry namespace".

**Forbidden System Behaviors:**
Direct side-effects beyond graph projections are unsupported and refused. This includes:
- Network calls (no FETCH, HTTP, CURL handlers)
- File system operations (no FS handlers)
- Process execution (no EXEC, SHELL handlers)
- Socket operations (no raw socket handlers)

Such handlers are refused at admission time, or keywords in their resource IRIs/names trigger keyword gating failure.

### 4.8 Pack Size Limit

A hook pack MUST declare 12 or fewer hooks. If more than 12 `rdf:type kh:Hook` triples are present, refuse with error "too many hooks declared: [count]; max 12".

### 4.9 Atomic Admission Guarantee

If any gate fails, the engine MUST:
1. Not add any triples from the pack to the store
2. Return an error describing the first gate that failed
3. Preserve all prior store state unchanged (transaction rollback)

Success of all gates is the prerequisite for hook activation.

## 5. Condition Evaluation

### 5.1 Condition Kinds and Semantics

A hook's condition (specified by `kh:kind`) defines a deterministic predicate evaluated during materialization. Each kind maps to a different set of input parameters.

#### 5.1.1 Delta Condition

**Syntax:** `kh:kind "delta" ; kh:var "<iri>" ; kh:on "assert|retract|any"`.

**Semantics:** Evaluates to true if the predicate identified by `kh:var` is present in the current stratum's delta (additions or deletions, depending on `kh:on`).

**Evaluation:**
- Let `P` = the IRI from `kh:var` (after term cleaning).
- Let `delta.additions` = the set of triples added in the current round.
- Let `delta.removals` = the set of triples removed in the current round.
- If `kh:on` is `assert`: return true iff any triple in `delta.additions` has predicate P.
- If `kh:on` is `retract`: return true iff any triple in `delta.removals` has predicate P.
- If `kh:on` is `any` (or absent): return true iff P appears in `delta.additions` OR `delta.removals`.

**Determinism:** Fully deterministic; depends only on immutable delta and post-state.

**Diagnostics:** Upon firing, include the predicate in result_path. Upon not firing, no diagnostic detail.

#### 5.1.2 Threshold Condition

**Syntax:** `kh:kind "threshold" ; kh:var "<iri>" ; kh:op "<op>" ; kh:k <integer>`.

**Semantics:** Evaluates to true if the count of triples with predicate P in the current post-state satisfies the comparison `count(P) <op> k`.

**Evaluation:**
- Let `P` = the IRI from `kh:var`.
- Let `count` = the number of triples in the post-state (union of all facts and inferred triples in the current stratum) where the predicate is P.
- Let `<op>` be one of `=`, `!=`, `<`, `<=`, `>`, `>=`.
- Return true iff `cmp_holds(op, count, k)` where cmp_holds compares count to k using the operator.

**Determinism:** Fully deterministic; depends only on immutable post-state and condition parameters.

**Diagnostics:** Report count value and whether the comparison held.

#### 5.1.3 Count Condition

**Syntax:** `kh:kind "count" ; kh:var "<iri>" ; kh:op "<op>" ; kh:k <integer>`.

**Semantics:** Evaluates to true if the count of triples with predicate P in the current delta (additions + removals) satisfies the comparison.

**Evaluation:**
- Let `P` = the IRI from `kh:var`.
- Let `count` = |{t ∈ delta.additions | predicate(t) = P}| + |{t ∈ delta.removals | predicate(t) = P}|.
- Return true iff `cmp_holds(op, count, k)`.

**Determinism:** Fully deterministic; depends only on immutable delta.

**Diagnostics:** Report count value.

#### 5.1.4 Window Condition

**Syntax:** `kh:kind "window" ; kh:var "<iri>" ; kh:op "<op>" ; kh:k <integer> ; kh:window <integer>`.

**Semantics:** Evaluates to true if the count of triples with predicate P in the current delta plus the previous (window - 1) deltas satisfies the comparison.

**Evaluation:**
- Let `P` = the IRI from `kh:var`.
- Let `window_deltas` = [current_delta] + [previous deltas, up to (window - 1) prior rounds].
- Let `count` = sum of counts of P across all deltas in window_deltas.
- Return true iff `cmp_holds(op, count, k)`.

**Determinism:** Fully deterministic; depends only on immutable delta history and window parameter.

**Constraints:** `window` MUST be in the range 1..255. If outside this range, the hook pack is refused.

**Diagnostics:** Report windowed count.

#### 5.1.5 Datalog Condition

**Syntax:** `kh:kind "datalog" ; kh:program "<datalog_rules>" ; kh:goal "<predicate_name>"`.

**Semantics:** Translates Datalog rules to N3, materializes against the post-state, and evaluates to true if the goal predicate is derived.

**Evaluation:**
1. Parse `kh:program` as Datalog rules (see Section 5.2 for syntax).
2. Validate: ≤ 8 rules, no head predicate "t" (reserved for EDB).
3. Translate to N3 rules via datalog-to-n3 translation (see Section 5.3).
4. Create a temporary store containing all post-state triples.
5. Load N3 rules into the temporary store.
6. Materialize the store (fixpoint iteration).
7. Search the materialized store for any triple with:
   - Predicate = `http://www.w3.org/1999/02/22-rdf-syntax-ns#type` (or `a`), and
   - Object = the goal (after term cleaning).
   - OR predicate = goal (case-insensitive match after term cleaning).
8. Return true iff at least one matching triple is found; false otherwise.

**Determinism:** Fully deterministic given fixed post-state and rule text.

**Constraints:** Program MUST be well-stratified. No negation without proper stratification.

**Diagnostics:** Report whether the goal was derived.

#### 5.1.6 SHACL Condition

**Syntax:** `kh:kind "shacl" ; kh:program "<shapes_ttl>"`.

**Semantics:** Evaluates to true if the post-state does NOT conform to the SHACL shapes (i.e., violations exist).

**Evaluation:**
1. Parse `kh:program` as Turtle SHACL shapes.
2. Create a temporary store with the post-state triples.
3. Run SHACL validation against the shapes and post-state.
4. Return true iff `not report.conforms` (i.e., there are violations).

**Determinism:** Fully deterministic given fixed post-state and shapes.

**Diagnostics:** Return SHACL validation report with focus nodes, result paths, violation messages.

#### 5.1.7 ShEx Condition

**Syntax:** `kh:kind "shex" ; kh:program "<schema>" ; kh:goal "<shape_map>"`.

**Semantics:** Evaluates to true if the post-state does NOT conform to the ShEx schema and shape map.

**Evaluation:**
1. Parse `kh:program` as ShEx schema (either ShExJ JSON or ShExC compact syntax).
2. Parse `kh:goal` as a shape map (node@shape pairs, comma-separated).
3. Create a temporary store with post-state triples.
4. Run ShEx validation against the schema, shape map, and post-state.
5. Return true iff there are validation failures.

**Determinism:** Fully deterministic given fixed post-state, schema, and shape map.

**Diagnostics:** Return ShEx validation report with node, shape, reason for each failure.

#### 5.1.8 N3 Condition

**Syntax:** `kh:kind "n3" ; kh:program "<n3_rules>"`.

**Semantics:** Evaluates to true if the post-state does NOT satisfy the N3 rules (i.e., denials are violated or implications are unsatisfied).

**Evaluation:**
1. Parse `kh:program` as N3 rules.
2. Create a temporary store containing:
   - The N3 rules themselves (as quasi-triples).
   - All post-state triples.
3. Materialize the store (apply N3 implications).
4. Check for denial violations (e.g., `=> false.` implications that fire).
5. Return true iff violations exist; false if rules are satisfied.

**Determinism:** Fully deterministic given fixed post-state and rules.

**Diagnostics:** Return denial violation messages.

#### 5.1.9 SPARQL Condition

**Syntax:** `kh:kind "sparql" ; kh:query "<sparql_query>"`.

**Semantics:** Evaluates to true if the SPARQL query returns non-empty results.

**Evaluation:**
1. Parse `kh:query` as a SPARQL query (SELECT, ASK, or CONSTRUCT).
2. Create a temporary store with post-state triples.
3. Execute the query against the store.
4. If query is ASK: return the boolean result directly.
5. If query is SELECT: return true iff the result set is non-empty.
6. If query is CONSTRUCT: return true iff the constructed graph is non-empty.

**Determinism:** Fully deterministic given fixed post-state and query.

**Diagnostics:** Report the number of result rows.

### 5.2 Datalog Syntax

Datalog rules in Knowledge Hooks use the following simplified syntax:

```
atom(arg1, arg2, ...) :- atom1(arg), atom2(arg), !negated_atom(arg).
```

**Atoms:**
- Predicate name followed by `(args)`.
- Arity must be 1 or 2 for user-defined predicates.
- Special atom `t(s, p, o)` represents a triple (arity 3).

**Arguments:**
- Variables: begin with `?` (e.g., `?x`, `?0`).
- Constants: IRIs enclosed in `<>` or quoted literals enclosed in `"`.
- Bare names treated as IRIs.

**Negation:** Prefix a literal with `!` to negate it.

**Goal:** The head predicate to search for after materialization. Must be a valid predicate name (not `t`).

**Example:**

```turtle
kh:program "
  ancestor(?x, ?z) :- parent(?x, ?y), ancestor(?y, ?z) .
  ancestor(?x, ?y) :- parent(?x, ?y) .
  isolated_person(?x) :- person(?x), !ancestor(?x, ?0), !ancestor(?0, ?x) .
" ;
kh:goal "isolated_person" .
```

### 5.3 Datalog-to-N3 Translation

Datalog rules are translated to N3 implications (forward chaining rules) as follows:

1. **Atoms with arity 1:** Predicate becomes rdf:type.
   - Datalog: `p(x)` → N3: `{ x a <http://...#p> }`

2. **Atoms with arity 2:** First argument is subject, predicate is atom name, second argument is object.
   - Datalog: `p(x, y)` → N3: `{ x <http://...#p> y }`

3. **Triple atom t(s, p, o):** Direct triple pattern.
   - Datalog: `t(x, y, z)` → N3: `{ x y z }`

4. **Negation:** Wrap in `not { ... }`.
   - Datalog: `!p(x)` → N3: `not { x a <http://...#p> }`

5. **Rule body:** Join literals with `.` to form the antecedent.

6. **Rule head:** Construct the consequent from the head atom.

7. **Output:** `{ antecedent_triples } => { consequent_triples } .`

**Constraints:**
- Maximum 8 rules per program (enforced at admission).
- Head predicate must not be `t`.

### 5.4 Event Gating

The `kh:on` property controls which types of deltas trigger condition evaluation:

| Value | Semantics |
|-------|-----------|
| `assert` | Hook is gated (verdict = Gated) if `delta.additions` is empty. |
| `retract` | Hook is gated (verdict = Gated) if `delta.removals` is empty. |
| `any` (default) | Hook is never gated by event type; condition evaluation proceeds. |

If a hook is gated, the verdict is recorded as `Gated`, and the condition is not evaluated. No diagnostics are collected for gated verdicts.

## 6. Hook Scheduling

### 6.1 Deterministic Ordering

Hooks are scheduled deterministically using topological sort over the dependency graph defined by `kh:after` constraints.

**Algorithm:**
1. Build a directed graph G = (V, E) where:
   - V = {all hook IRIs}
   - E = {(dep, hook) | hook has `kh:after dep`}
2. Compute in-degree for each vertex.
3. Initialize queue Q with all vertices of in-degree 0.
4. While Q is not empty:
   - Extract vertex with lowest (priority, IRI) tuple (priority first, then lexicographic IRI).
   - Add to scheduled list.
   - For each neighbor n: decrement in-degree[n]; if 0, enqueue n.
5. If scheduled list size < |V|, a cycle exists; refuse the pack with "dependency cycle detected".

**Guarantees:**
- Hooks with no dependencies execute first.
- Hooks are ordered by `kh:priority` (lower first) and IRI (lexicographic) as tiebreaker.
- Dependencies are respected: a hook never executes before its `kh:after` hooks.

### 6.2 Materialization Phase Integration

Hooks are evaluated once per stratum iteration, after all rules in that stratum have fired and reached a fixpoint. Within a single stratum iteration:

1. Rules fire until no new triples are derived (fixpoint).
2. Hooks are evaluated against the post-state and current delta in dependency order.
3. Hooks with `emit-delta` or `ground-action` effects may add new triples (hook_changed flag).
4. If hook_changed is true, another stratum iteration begins.

## 7. Effect Application

### 7.1 EmitDelta Effect

When a hook fires with `kh:effect "emit-delta"` and no action is specified:

1. The hook's condition evaluation produces a set of bindings (for SPARQL queries) or a boolean true (for non-SPARQL conditions).
2. If the condition requires projection (e.g., SPARQL bindings), those bindings are used to instantiate a template.
3. The resulting triples are added to the post-state.
4. A receipt is generated (Section 8).

**Note:** For conditions like delta, threshold, count, and window, which do not produce bindings, the emit-delta effect without an action does not project new triples; it only generates a verdict and (if triples were already added by other means) a receipt.

### 7.2 GroundAction Effect

When a hook fires with `kh:effect "ground-action"` and a `kh:action` is specified:

1. Look up the action resource referenced by `kh:action`.
2. Verify it has exactly one `kh:handler` with value `http://seanchatmangpt.github.io/praxis/handler#sparql-construct`.
3. Retrieve the `kh:query` property from the action.
4. Execute the SPARQL CONSTRUCT query against the post-state.
   - If the hook's condition generated bindings (e.g., from SPARQL SELECT), substitute them into the CONSTRUCT query template.
5. Collect the constructed triples and deletions (if the CONSTRUCT has an empty template, interpret as DELETE).
6. Apply additions to the post-state; apply removals by removing triples.
7. Generate a receipt (Section 8) if any triples were added.

**Binding Projection:**
If the hook's condition is SPARQL SELECT and returns multiple rows of bindings, iterate over each row and project the CONSTRUCT query for that row. Collect the union of all projections.

### 7.3 Refuse Effect

When a hook fires with `kh:effect "refuse"`:

1. Retrieve the `kh:reason` property (required).
2. Immediately halt the current stratum iteration.
3. Roll back all triples inferred in the current stratum (restore to `stratum_rollback_point`).
4. Record a verdict with `Fired` and the reason in diagnostics.
5. Return an error with message "refused by hook '[name]': [reason]".
6. Stop materialization; no further strata are evaluated.

The refusal is atomic and transactional: all inferences from the current stratum are undone.

## 8. Receipt Construction and Idempotency

### 8.1 Receipt Generation

When a hook fires with `emit-delta` or `ground-action` effect and produces non-empty graph changes, a receipt MUST be generated.

**Receipt Structure:**

```rust
pub struct HookReceipt {
    pub hook_name: String,           // From kh:name
    pub delta_hash: String,          // BLAKE3 hash of canonical N-Quads
    pub idempotency_key: String,     // BLAKE3 hash of "praxis:idempotency-key:v1" || delta_hash
    pub delta_quads: String,         // Canonical N-Quads representation
}
```

### 8.2 Canonical N-Quads Serialization

The set of triples added by the hook MUST be serialized as N-Quads using canonical form:

1. **Blank node normalization:** Blank nodes are renamed to canonical forms `_:c14n0`, `_:c14n1`, etc., ordered by first appearance in document order.

2. **Triple sorting:** All N-Quads statements are sorted lexicographically as strings before hashing.

3. **Literal escaping:** Escape sequences in literals (`\`, `"`, newline, carriage return, tab) are preserved using N-Quads escape rules.

4. **Graph awareness:** If a hook projects quads to a named graph, the graph IRI is included as the fourth element of each quad.

**Example:**

```n-quads
<http://example.org/Alice> <http://example.org/status> "VIP" .
<http://example.org/Alice> <http://example.org/spent> "1500"^^<http://www.w3.org/2001/XMLSchema#integer> .
```

### 8.3 Delta Hash Computation

The delta hash is the BLAKE3 hash of the canonical N-Quads string (UTF-8 encoded), represented as a hexadecimal string:

```
delta_hash = BLAKE3(canonical_quads.encode('utf-8')).hex()
```

**Properties:**
- Deterministic: same quads → same hash, regardless of input order.
- Collision-resistant: infeasible to find two different quad sets with the same hash.
- Immutable: once a receipt is generated, the hash is immutable proof of the delta.

### 8.4 Idempotency Key Derivation

The idempotency key is derived from the delta hash to enable deterministic replay and deduplication:

```
idempotency_key = BLAKE3(
  format!("praxis:idempotency-key:v1{}", delta_hash)
).hex()
```

**Properties:**
- Monotonic: depends on delta_hash, which is deterministic.
- Version-aware: "v1" allows future key derivation changes (v2, v3, etc.).
- Idempotent: same delta → same idempotency key → can detect replays.

### 8.5 Empty Effects and Receipt Suppression

If a hook fires but produces no graph changes (no triples added or removed), no receipt is generated. Only non-empty effects produce receipts.

## 9. Boundary Projection and Refusal

### 9.1 Boundary Adapters

Knowledge Hooks are designed to operate entirely within the graph, with explicit, auditable projections to external systems. The only supported boundary projection mechanism is:

**SPARQL CONSTRUCT Handler:** A hook's action may specify `kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct>` with a SPARQL CONSTRUCT query. This query projects the hook's outcome onto the graph (or a named graph), creating an audit trail.

### 9.2 Refused Boundary Projections

The following boundary operations are unsupported and MUST be refused at admission time:

- **Network calls:** No HTTP, FETCH, CURL handlers.
- **File system operations:** No FS, FILE, or WRITE handlers.
- **Process execution:** No EXEC, SHELL, or RUN handlers.
- **Socket operations:** No raw socket, UDP, or TCP handlers.
- **Custom or unknown handlers:** Any handler IRI not in the allowed list.

If a hook's action references an unsupported handler, the entire pack is refused with error "forbidden or unrecognized handler: [handler_iri]".

### 9.3 Graph Boundary Gating

Hooks are free to project changes to named graphs (quads). The only constraint is that they MUST NOT write to the `http://seanchatmangpt.github.io/praxis/kh#` namespace (the hook registry itself). Any attempt to modify the hook registry via a CONSTRUCT query is refused.

## 10. Verdicts and Diagnostics

### 10.1 Verdict Types

Each hook evaluation produces exactly one verdict:

| Verdict | Meaning |
|---------|---------|
| `Fired` | Condition evaluated to true; effect was applied (or refusal occurred). |
| `NotFired` | Condition evaluated to false; no effect applied. |
| `Gated` | Event gate (`kh:on`) prevented evaluation; condition was not checked. |

### 10.2 Verdict Recording

Verdicts are recorded in a `HookVerdictRecord`:

```rust
pub struct HookVerdictRecord {
    pub hook_iri: String,                   // Hook resource IRI
    pub hook_name: String,                  // From kh:name
    pub condition_kind: String,             // From kh:kind
    pub condition_hash: String,             // SHA256 of condition JSON
    pub verdict: HookVerdict,               // Fired | NotFired | Gated
    pub effect: EffectKind,                 // emit-delta | ground-action | refuse
    pub action_iri: Option<String>,         // From kh:action if present
    pub diagnostics: Option<TriggerDiagnostic>,  // Condition-specific details
    pub delta_hash: Option<String>,         // BLAKE3 if receipt was generated
    pub idempotency_key: Option<String>,    // Idempotency key if receipt was generated
}
```

### 10.3 Diagnostics

Condition-specific diagnostics are collected and returned as `TriggerDiagnostic`:

```rust
pub struct TriggerDiagnostic {
    pub hook_iri: String,
    pub conforms: bool,                     // True if condition did NOT fire
    pub details: Vec<DiagnosticDetail>,
}

pub struct DiagnosticDetail {
    pub focus_node: Option<String>,
    pub result_path: Option<String>,
    pub value: Option<String>,
    pub severity: Option<String>,           // "Fired", "Violation", "Denial"
    pub message: String,
}
```

**Diagnostic Collection by Condition Kind:**

| Kind | Diagnostics |
|------|-----------|
| Delta | result_path = var, value = empty, severity = "Fired" if condition holds |
| Threshold | result_path = var, value = count, message = comparison result |
| Count | result_path = var, value = count, message = comparison result |
| Window | result_path = var, value = windowed_count, message = comparison result |
| Datalog | message = "Datalog goal '[goal]' was derived in post-state" if fired |
| SHACL | SHACL validation report details (focus_node, result_path, severity) |
| ShEx | ShEx validation failures (focus_node, shape, reason) |
| N3 | Denial violations (message = denial text) |
| SPARQL | value = result_count, message = "SPARQL query returned N results" |

No diagnostics are collected for `Gated` verdicts.

## 11. Replay and Non-Ornamentality

### 11.1 Replay Protocol

Knowledge Hooks support replay verification to prove that receipts are not ornamental (i.e., that the recorded deltas actually correspond to the hook's real outcome).

**Replay Procedure:**
1. Retrieve a hook receipt (hook_name, delta_hash, idempotency_key, delta_quads).
2. Rerun materialization with the same hook pack and initial graph state.
3. Evaluate the hook at the same point in the materialization process.
4. Collect the delta hash of the new run.
5. Compare: `new_delta_hash == receipt.delta_hash`.
6. If equal, the receipt is authentic and non-ornamental.

### 11.2 Idempotency and Duplicate Detection

The idempotency key enables systems to detect and suppress duplicate hook firings:

1. Before applying a hook's projection, compute its idempotency key.
2. Check a deduplication ledger for a prior entry with the same idempotency_key.
3. If found and replay verification confirms the prior receipt, skip the projection (idempotent).
4. If not found, apply the projection and record the idempotency_key.

This ensures that accidental re-runs of the same hook pack do not double-apply its effects.

### 11.3 Replay Evidence

A system MUST be able to prove non-ornamentality by:
1. Collecting all hook verdicts and receipts from a run.
2. Re-running materialization with identical initial state and rule set.
3. Replaying hook evaluation at the same stratum/iteration.
4. Comparing verdict records and delta hashes.
5. Verifying that all fired verdicts produced matching receipts.
6. Returning a replay evidence document: `{ hook_iri: verdict, receipt_delta_hash, replay_delta_hash, match: bool }` for each hook.

If any verdict produces a receipt, the delta_hash must match on replay. If a verdict is not fired on replay but was fired on the original run (or vice versa), the replay is non-deterministic and the run is invalid.

## 12. Conformance and Testing

### 12.1 Producer Conformance

A Knowledge Hook producer (a system that declares hooks in RDF) MUST:

1. **Closed Vocabulary:** Use only predicates from the allowed `kh:` vocabulary.
2. **SHACL Compliance:** Ensure all hooks conform to the SHACL Law Pack schema.
3. **Security:** Avoid forbidden keywords; use explicit, named actions for boundary projections.
4. **Idempotency:** Design conditions and effects to be idempotent (same inputs → same outputs).
5. **Documentation:** Provide semantics for each condition, expected outcomes, and dependencies.

### 12.2 Consumer Conformance

A Knowledge Hook consumer (a graph engine that evaluates hooks) MUST:

1. **Admission Gating:** Implement all gates (SHACL, closed vocabulary, keyword, condition, effect, constitutional). Refuse packs atomically on any gate failure.
2. **Deterministic Evaluation:** Evaluate conditions deterministically. Use the same semantics for each dialect across runs.
3. **Scheduling:** Implement topological sort for dependency ordering. Detect and refuse cycles.
4. **Receipt Generation:** Produce canonical N-Quads and BLAKE3 hashes for all non-empty effects.
5. **Audit Trail:** Record all verdicts. Make verdicts queryable for replay and forensics.

### 12.3 Engine Conformance

A Knowledge Hook engine (the materialization + hook evaluation subsystem) MUST:

1. **Stratification:** Evaluate hooks within the materialization loop, respecting stratum boundaries.
2. **Fixpoint Semantics:** Reach a fixpoint before evaluating hooks; allow hooks to trigger further derivations.
3. **Atomicity:** Enforce atomic admission; if any gate fails, no triples from the pack are added.
4. **Idempotency:** Ensure that hook effects are idempotent; replay produces identical deltas.
5. **Silence Proof:** For any hook that does not fire, record a `NotFired` verdict with no diagnostics (silence is provable).

### 12.4 Adapter Conformance

A boundary adapter (e.g., SPARQL CONSTRUCT handler) MUST:

1. **Query Execution:** Execute SPARQL queries correctly against the materialized graph.
2. **Template Instantiation:** Correctly substitute bindings into CONSTRUCT templates.
3. **Graph Projection:** Apply constructed quads to the store, respecting named graphs.
4. **Namespace Protection:** Refuse any query that would write to the `kh:` namespace.
5. **Determinism:** Produce identical output for identical bindings and queries.

### 12.5 Conformance Testing

Conformance test suites MUST verify:

1. **Admission gates** — Both passing and failing packs.
2. **All nine condition dialects** — Datalog, Delta, Threshold, Count, Window, SHACL, ShEx, N3, SPARQL.
3. **All three effects** — EmitDelta, GroundAction (via SPARQL CONSTRUCT), Refuse.
4. **Event gating** — Assert-only, retract-only, any.
5. **Scheduling** — Dependencies, priorities, cycles.
6. **Verdicts** — Fired, NotFired, Gated.
7. **Receipts** — Canonical serialization, BLAKE3 hashing, idempotency.
8. **Refusal** — Stratum rollback, error reporting.
9. **Replay** — Idempotent re-runs, duplicate detection.
10. **Boundary refusal** — Forbidden handlers and keywords are rejected.
11. **Negative fixtures** — Malformed packs, missing required properties, cycle detection.

## 13. Unsupported Features and Future Work

### 13.1 Explicitly Unsupported

The following capabilities are intentionally unsupported and MUST be refused:

1. **Direct side-effects:** No file writes, network calls, process execution, or arbitrary I/O.
2. **Non-SPARQL-CONSTRUCT actions:** Only `http://seanchatmangpt.github.io/praxis/handler#sparql-construct` is supported.
3. **Conditional composition:** Hooks cannot have OR/AND combinations of conditions; only a single condition per hook.
4. **Stateful conditions:** Conditions cannot reference external state or clock time.
5. **Recursive hook invocation:** Hooks cannot call other hooks; they can only trigger new derivations.
6. **User-defined handlers:** Custom handler implementations are not supported; only built-in handlers.
7. **Hook introspection at runtime:** Hooks cannot query or modify the hook registry during materialization.

### 13.2 Future Extensibility

The specification is designed for controlled future extension:

- **New condition dialects** may be added by extending the `kh:kind` enumeration, provided they remain fully deterministic and testable.
- **New boundary adapters** (handlers) may be added if they are query-based (SELECT/CONSTRUCT) and explicitly audited.
- **New serialization formats** for receipts may be introduced via version markers in idempotency key derivation.

Any extension MUST preserve:
- Determinism (identical inputs → identical outputs across runs).
- Auditability (all outcomes are recorded and queryable).
- Security (no silent side-effects; all boundaries are explicit).

## 14. Examples

### Example 1: Delta Trigger with EmitDelta Effect

```turtle
@prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
@prefix ex: <http://example.org/> .

ex:notify_price_change a kh:Hook ;
    kh:name "price_monitor" ;
    kh:kind "delta" ;
    kh:var "<http://example.org/price>" ;
    kh:on "assert" ;
    kh:effect "emit-delta" ;
    kh:priority 1 .
```

**Semantics:** When a price is asserted (added), fire the hook and record a receipt.

### Example 2: Threshold Trigger with GroundAction Effect

```turtle
@prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
@prefix ex: <http://example.org/> .

ex:escalate_alerts a kh:Hook ;
    kh:name "high_risk_detector" ;
    kh:kind "threshold" ;
    kh:var "<http://example.org/risk_level>" ;
    kh:op ">" ;
    kh:k 80 ;
    kh:effect "ground-action" ;
    kh:action ex:escalate_action .

ex:escalate_action a kh:Action ;
    kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
    kh:query """
        CONSTRUCT {
            ?entity <http://example.org/status> "escalated" .
        }
        WHERE {
            ?entity <http://example.org/risk_level> ?level .
            FILTER(?level > 80)
        }
    """ .
```

**Semantics:** If more than 80 risk-level triples exist in the store, add "escalated" status via SPARQL CONSTRUCT.

### Example 3: Datalog Trigger with Refuse Effect

```turtle
@prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
@prefix ex: <http://example.org/> .

ex:prevent_cycles a kh:Hook ;
    kh:name "cycle_detector" ;
    kh:kind "datalog" ;
    kh:program """
        cycle(?x) :- t(?x, <http://example.org/next>, ?y), 
                     t(?y, <http://example.org/next>, ?x) .
    """ ;
    kh:goal "cycle" ;
    kh:effect "refuse" ;
    kh:reason "Graph contains a cycle, which violates invariant: acyclic order required." ;
    kh:priority 10 .
```

**Semantics:** If the Datalog program derives a cycle, refuse the current stratum and roll back.

### Example 4: SPARQL Trigger with Dependencies

```turtle
@prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
@prefix ex: <http://example.org/> .

ex:detect_orphans a kh:Hook ;
    kh:name "orphan_detector" ;
    kh:kind "sparql" ;
    kh:query """
        SELECT ?node WHERE {
            ?node <http://example.org/exists> true .
            FILTER NOT EXISTS { ?parent <http://example.org/child> ?node }
            FILTER NOT EXISTS { ?node <http://example.org/parent> ?ancestor }
        }
    """ ;
    kh:effect "emit-delta" ;
    kh:priority 5 .

ex:tag_orphans a kh:Hook ;
    kh:name "orphan_tagger" ;
    kh:kind "sparql" ;
    kh:query """
        ASK { ?node <http://example.org/exists> true }
    """ ;
    kh:effect "ground-action" ;
    kh:action ex:tag_action ;
    kh:after ex:detect_orphans ;
    kh:priority 6 .

ex:tag_action a kh:Action ;
    kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
    kh:query """
        CONSTRUCT {
            ?node <http://example.org/tag> "orphan" .
        }
        WHERE {
            ?node <http://example.org/exists> true .
            FILTER NOT EXISTS { ?parent <http://example.org/child> ?node }
        }
    """ .
```

**Semantics:** The orphan_detector hook fires first (priority 5, no dependencies), then orphan_tagger fires after (kh:after ex:detect_orphans), ensuring correct order.

---

## Implementation Conformance Checklist

This checklist identifies what MUST be implemented, tested, and evidenced for each normative requirement.

### A. Hook Declaration and Representation (Section 3)

| Requirement | Implementation Evidence |
|-------------|------------------------|
| **3.1** Hook declared with `rdf:type kh:Hook` | Test: `test_f1_load_valid_single`, `test_f1_load_valid_multiple` |
| **3.2** All properties have correct cardinality (1 or 0..1) | Test: `test_f2_gating_malformed_shacl` (missing mandatory kh:kind) |
| **3.2** kh:on must be "assert", "retract", or "any" | Code: hooks.rs line 428-432 validation |
| **3.2** kh:effect must be "emit-delta", "ground-action", or "refuse" | Code: hooks.rs line 489-495 |
| **3.3** ground-action effect requires kh:action | Code: hooks.rs line 500-501 |
| **3.3** refuse effect requires kh:reason | Code: hooks.rs line 503-504 |

### B. Hook Pack Admission (Section 4)

| Requirement | Implementation Evidence |
|-------------|------------------------|
| **4.1** Atomic admission: all-or-nothing | Test: `test_f2_gating_rollback_state` (store length unchanged after failed pack) |
| **4.2** SHACL validation enforced | Code: hooks.rs line 330, validate_shacl call; Test: `test_f2_gating_malformed_shacl` |
| **4.3** Closed vocabulary: only ALLOWED_KH_PREDICATES | Code: hooks.rs line 393-399, predicate whitelist check |
| **4.4** Forbidden keywords: shell, exec, curl, socket, fetch | Code: hooks.rs line 260-270, keyword scanning with whitelist exceptions |
| **4.5** Condition-specific validation by kind | Code: hooks.rs line 435-487 (per-kind validation) |
| **4.6** Effect validation: ground-action requires action, refuse requires reason | Code: hooks.rs line 499-507 |
| **4.7** Handler restriction: only sparql-construct allowed | Code: hooks.rs line 365-369; Test: `test_f2_refuse_command`, `test_f2_refuse_shell`, `test_f2_refuse_unrecognized_action` |
| **4.7** CONSTRUCT queries cannot modify kh: namespace | Code: hooks.rs line 370-391, template validation |
| **4.8** Max 12 hooks per pack | Code: hooks.rs line 416-421 |
| **4.9** Atomic rollback on gate failure | Code: hooks.rs line 330-340 (SHACL failure returns Err without state change) |

### C. Condition Evaluation (Section 5)

| Requirement | Implementation Evidence |
|-------------|------------------------|
| **5.1.1** Delta: detect predicate in delta.additions/removals | Code: hooks.rs line 1008-1019, delta_touches; Test: `test_f3_delta_trigger` |
| **5.1.2** Threshold: count(P) in post-state vs k | Code: hooks.rs line 1021-1032, count_pred_in_store; Test: `test_f3_threshold_trigger` |
| **5.1.3** Count: count(P) in delta vs k | Code: hooks.rs line 1034-1053, count_pred_in_delta; Test: `test_f3_count_trigger` |
| **5.1.4** Window: windowed count over history | Code: reasoner/mod.rs line 343-351, history window summation; Test: `test_trigger_dialects` (window condition) |
| **5.1.5** Datalog: translate to N3, materialize, search for goal | Code: hooks.rs line 1142-1242, translate_datalog_to_n3; Code: hooks.rs line 1252-1293, evaluate_condition Datalog branch |
| **5.1.6** SHACL: validate shapes, return !conforms | Code: hooks.rs line 1386-1410, evaluate_condition Shacl branch |
| **5.1.7** ShEx: validate schema, return !conforms | Code: hooks.rs line 1412-1464, evaluate_condition Shex branch |
| **5.1.8** N3: load rules, check denials | Code: hooks.rs line 1466-1491, evaluate_condition N3 branch |
| **5.1.9** SPARQL: execute query, return !empty | Code: hooks.rs line 1493-1514, evaluate_condition Sparql branch; Test: `test_f3_sparql_ask_trigger`, `test_f3_sparql_select_trigger` |
| **5.2** Datalog syntax parsing: arity 1/2 atoms, t(s,p,o), negation | Code: hooks.rs line 1101-1120, parse_datalog_atom |
| **5.3** Datalog-to-N3 translation | Code: hooks.rs line 1142-1242, translate_datalog_to_n3 |
| **5.4** Event gating: kh:on "assert", "retract", "any" | Code: reasoner/mod.rs line 307-326, gated verdict for assert-only/retract-only; Code: hooks.rs line 1526-1542, gating logic in evaluate_hooks |

### D. Hook Scheduling (Section 6)

| Requirement | Implementation Evidence |
|-------------|------------------------|
| **6.1** Topological sort with priority tiebreaker | Code: hooks.rs line 533-586, schedule_hooks function |
| **6.1** Cycle detection and refusal | Code: hooks.rs line 581-583, cycle check: "if scheduled.len() < hooks.len()" → Err("cycle detected") |
| **6.2** Hooks evaluated after stratum fixpoint | Code: reasoner/mod.rs line 306, hooks evaluated within while changed loop, after rule firing |

### E. Effect Application (Section 7)

| Requirement | Implementation Evidence |
|-------------|------------------------|
| **7.1** EmitDelta: project triples | Code: reasoner/mod.rs line 510-560, EmitDelta handling via action/SPARQL CONSTRUCT |
| **7.2** GroundAction: execute CONSTRUCT, apply triples | Code: reasoner/mod.rs line 510-560, query handler execution |
| **7.3** Refuse: rollback stratum, return error | Code: reasoner/mod.rs line 477-508, Refuse effect implementation (triple removal, inferred truncate, error return) |

### F. Receipt Construction (Section 8)

| Requirement | Implementation Evidence |
|-------------|------------------------|
| **8.1** Receipt generated for non-empty effects | Code: reasoner/mod.rs line 564-584, receipt creation on hook_additions not empty |
| **8.2** Canonical N-Quads serialization | Code: hooks.rs line 784-838, canonicalize_quads function (blank node normalization, sorting, escaping) |
| **8.3** BLAKE3 delta hash | Code: reasoner/mod.rs line 567-568, blake3::hash(delta_quads).hex() |
| **8.4** Idempotency key derivation | Code: reasoner/mod.rs line 569-573, blake3::hash(format!("praxis:idempotency-key:v1{}", d_hash)) |
| **8.5** No receipt for empty effects | Code: reasoner/mod.rs line 564, conditional: "if !hook_additions.is_empty()" |

### G. Verdicts and Diagnostics (Section 10)

| Requirement | Implementation Evidence |
|-------------|------------------------|
| **10.1** Verdicts: Fired, NotFired, Gated | Code: hooks.rs line 908-913, HookVerdict enum |
| **10.2** VerdictRecord structure | Code: hooks.rs line 936-958, HookVerdictRecord struct |
| **10.2** Delta_hash and idempotency_key in verdicts | Code: reasoner/mod.rs line 586-597, verdict creation with delta_hash and idempotency_key |
| **10.3** Diagnostics by condition kind | Code: hooks.rs line 1278-1513, per-dialect diagnostic collection |
| **10.3** No diagnostics for Gated | Code: reasoner/mod.rs line 312-325, diagnostics=None for Gated verdict |

### H. Refusal and Boundary Protection (Section 9)

| Requirement | Implementation Evidence |
|-------------|------------------------|
| **9.1** SPARQL CONSTRUCT handler only | Code: hooks.rs line 365-369, handler comparison |
| **9.2** Refuse forbidden handlers | Test: `test_f2_refuse_command`, `test_f2_refuse_shell`, `test_f2_refuse_unrecognized_action` |
| **9.3** Refuse kh: namespace modifications | Code: hooks.rs line 370-391, CONSTRUCT template validation |

### I. Conformance and Testing (Section 12)

| Requirement | Implementation Evidence |
|-------------|------------------------|
| **12.1-12.5** All nine condition dialects implemented | Tests: `test_trigger_dialects` (6 dialects), `test_f3_*` tests (all 9 dialects) |
| **12.1-12.5** All three effects implemented | Tests: `test_f4_project_add_quad`, `test_f4_project_delete_quad` (emit-delta via action), `test_f4_refuse_side_effects` |
| **12.1-12.5** Event gating (assert, retract, any) | Tests: hooks declared with `kh:on "assert"`, `kh:on "retract"` |
| **12.1-12.5** Deterministic receipt generation | Test: `test_f5_receipt_sort_determinism` (same quads, different insertion order → same hash) |
| **12.1-12.5** Receipt API | Test: `test_f5_get_hook_receipts_api` (store.get_hook_receipts() returns receipts) |
| **12.1-12.5** Negative test: forbidden keywords | Code: hooks.rs line 260-270, keyword scan; implicit via failed packs |
| **12.1-12.5** Negative test: cycle detection | Code: hooks.rs line 581-583; Test: dependency cycle check in schedule_hooks |
| **12.1-12.5** Negative test: missing required properties | Test: `test_f2_gating_malformed_shacl` |

### J. Idempotency and Replay (Section 11)

| Requirement | Implementation Evidence |
|-------------|------------------------|
| **11.1** Receipt delta_hash matches canonical quads | Code: reasoner/mod.rs line 565-573, canonical quads construction and hashing |
| **11.2** Idempotency key for duplicate detection | Code: reasoner/mod.rs line 569-573, idempotency_key derivation |
| **11.3** Replay verification via verdict/receipt matching | Architecture: verdicts + receipts + delta_quads enable external replay tools to re-run and verify |

---

## Normative Summary

Knowledge Hooks are a deterministic, auditable mechanism for graph transformation. Their design enforces:

1. **No silent actuation:** Every outcome is recorded (Fired, NotFired, or Gated).
2. **No silent promotion of unsupported features:** Refusal is a first-class outcome.
3. **Atomic admission:** Hook packs are validated completely before any triples are added.
4. **Deterministic evaluation:** Conditions are evaluated against immutable post-state and delta.
5. **Explicit boundaries:** Only SPARQL CONSTRUCT projections cross system boundaries.
6. **Auditable effects:** All projections produce cryptographic receipts and canonical serializations.

Conformance requires implementing all nine condition dialects, all three effects, event gating, deterministic scheduling, receipt generation, and comprehensive testing of both positive and negative cases.

