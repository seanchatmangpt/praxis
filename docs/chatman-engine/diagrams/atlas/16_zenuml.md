# 16. ZenUML Diagram Family

This file contains the ZenUML diagram family for the Chatman Engine, structured across the 8 projection lenses.

Fallback rendering for Mermaid compatibility.

---

## Lens 1: Semantic Authority

Diagram ID: ZENUML-L1
Diagram family: ZenUML
Projection lens: Semantic Authority
Architectural invariant preserved: RDF/Oxigraph is the single semantic source of truth. Shadow copies of RDF data are strictly prohibited.
Information-loss risk if omitted: Loss of transaction sequence visibility in semantic updates.
TPS visual-control purpose: Prevents messaging waste and redundant query calls.
DfLSS CTQ protected: Zero semantic shadow copies.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Details sequence flow for updating and validating RDF data.

```mermaid
sequenceDiagram
    autonumber
    actor Client as Operator
    participant API as Graph API
    participant Auth as Auth Engine
    participant DB as Oxigraph Store

    Client->>API: query(rdf_triples)
    API->>Auth: validate_authority(rdf_triples)
    activate Auth
    Auth-->>API: authorized
    deactivate Auth
    API->>DB: commit(rdf_triples)
    activate DB
    DB-->>API: commit_receipt(hash)
    deactivate DB
    API-->>Client: success(receipt)
```

---

## Lens 2: Routing Constitution

Diagram ID: ZENUML-L2
Diagram family: ZenUML
Projection lens: Routing Constitution
Architectural invariant preserved: Least-expressive-power routing; hot/warm/cold path isolation. N3 is disabled by default.
Information-loss risk if omitted: Executing high-expressivity queries on hot paths due to lack of routing protocol sequence visibility.
TPS visual-control purpose: Prevents routing logic overhead waste.
DfLSS CTQ protected: Safe isolation of cold-path N3 execution.
CENG ticket or boundary constrained: CENG-411 (design-only, implementation blocked).
Why this diagram is non-redundant: Outlines the message-passing sequences for query route classification.

```mermaid
sequenceDiagram
    autonumber
    actor Client as Client API
    participant Router as Route Classifier
    participant Hot as Hot Path
    participant Warm as Warm Path
    participant Cold as Cold Path (Blocked)

    Client->>Router: route_query(query_data)
    Router->>Router: count_constraints()
    alt constraints <= 8
        Router->>Hot: execute_hot(query_data)
        Hot-->>Client: hot_result
    else constraints > 8
        Router->>Warm: execute_warm(query_data)
        Warm-->>Client: warm_result
    else N3 Rule Detected
        Router->>Router: quarantine_n3()
        Router->>Cold: execute_cold(query_data) [Blocked CENG-411]
        Cold-->>Client: refusal("CENG-411 blocked")
    end
```

---

## Lens 3: Type Kernel Ownership

Diagram ID: ZENUML-L3
Diagram family: ZenUML
Projection lens: Type Kernel Ownership
Architectural invariant preserved: Canonical type ownership across wasm4pm-compat, wasm4pm-cognition, bcinr-pddl, bcinr-powl, and praxis-graphlaw.
Information-loss risk if omitted: Duplicate types created at runtime during inter-module calls.
TPS visual-control purpose: Prevents redundant type mapping work.
DfLSS CTQ protected: Zero duplicate type classes.
CENG ticket or boundary constrained: CENG-412 (design-only, implementation blocked).
Why this diagram is non-redundant: Maps sequence flow of type kernel validation.

```mermaid
sequenceDiagram
    autonumber
    participant Compiler as Build Linker
    participant Compat as wasm4pm-compat
    participant PDDL as bcinr-pddl [Blocked]
    participant Law as praxis-graphlaw

    Compiler->>Compat: load_canonical_types()
    activate Compat
    Compat-->>Compiler: type_registry
    deactivate Compat
    Compiler->>PDDL: check_pddl_types(type_registry) [Blocked CENG-412]
    PDDL-->>Compiler: refusal("CENG-412 blocked")
    Compiler->>Law: map_graph_types(type_registry)
    Law-->>Compiler: success
```

---

## Lens 4: Transition Lifecycle

Diagram ID: ZENUML-L4
Diagram family: ZenUML
Projection lens: Transition Lifecycle
Architectural invariant preserved: Every transition must pass through candidate invocation, validation, planning, execution, receipting, and replay.
Information-loss risk if omitted: Bypassing validation or receipting sequences during state transitions.
TPS visual-control purpose: Restricts lifecycle WIP to eliminate timing bottlenecks.
DfLSS CTQ protected: Guaranteed transaction replay validation under fixed seed.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Models transition lifecycle step sequencing.

```mermaid
sequenceDiagram
    autonumber
    participant Invoker as Invocator
    participant Val as Validator
    participant Exec as Executor
    participant Rec as Receipt Builder
    participant Rep as Replayer

    Invoker->>Val: validate_transition(candidate)
    activate Val
    Val->>Val: check_owl_shacl()
    Val-->>Exec: validated_candidate
    deactivate Val
    activate Exec
    Exec->>Exec: execute_hook()
    Exec->>Rec: generate_receipt()
    deactivate Exec
    activate Rec
    Rec-->>Rep: signed_receipt(blake3_hash)
    deactivate Rec
    activate Rep
    Rep->>Rep: verify_replay(fixed_seed)
    Rep-->>Invoker: transition_committed
    deactivate Rep
```

---

## Lens 5: Event / Hook / Actuation

Diagram ID: ZENUML-L5
Diagram family: ZenUML
Projection lens: Event / Hook / Actuation
Architectural invariant preserved: Hooks cannot actuate without receipts; no unreceipted actuation.
Information-loss risk if omitted: Actuators triggering side effects before receiving signed event receipts.
TPS visual-control purpose: Poka-Yoke gating of actuation commands.
DfLSS CTQ protected: Zero unreceipted execution events.
CENG ticket or boundary constrained: CENG-416A-F (design-only, implementation blocked).
Why this diagram is non-redundant: Traces messaging between event router, hook manager, and actuator.

```mermaid
sequenceDiagram
    autonumber
    participant Bus as OCEL Event Bus
    participant Matcher as Hook Matcher
    participant Gate as Receipt Gate
    participant Act as Actuator (Blocked)

    Bus->>Matcher: publish_event(event)
    activate Matcher
    Matcher->>Matcher: match_knowledge_hook(event)
    Matcher->>Gate: check_receipt_signature(event)
    deactivate Matcher
    activate Gate
    alt receipt_valid
        Gate->>Act: actuate_boundary(event) [Blocked CENG-416A-F]
        Act-->>Gate: refusal("CENG-416A-F blocked")
    else receipt_invalid
        Gate-->>Bus: refusal("Invalid receipt")
    end
    deactivate Gate
```

---

## Lens 6: Performance / 8-Constraint Hot Path

Diagram ID: ZENUML-L6
Diagram family: ZenUML
Projection lens: Performance / 8-Constraint Hot Path
Architectural invariant preserved: Maximum of 8 constraints checked in parallel via RDFTriple8 and ConditionCell<BITS>.
Information-loss risk if omitted: CPU execution budget exceeded if lowering paths are suboptimal.
TPS visual-control purpose: Andon check monitoring constraint compiler pipeline.
DfLSS CTQ protected: Latency bound of hot path operations.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Resolves hot-path vector compilation and execution sequencing.

```mermaid
sequenceDiagram
    autonumber
    participant Engine as Query Engine
    participant Comp as Vector Compiler
    participant Cell as ConditionCell Gate
    participant Table as Admission Table

    Engine->>Comp: compile_hotpath(rdf_triples)
    activate Comp
    Comp->>Comp: compile_to_mask(rdf_triples)
    Comp-->>Cell: bitmask(ConditionCell<BITS>)
    deactivate Comp
    activate Cell
    Cell->>Table: evaluate_state(bitmask)
    activate Table
    Table-->>Engine: allowed / denied
    deactivate Table
    deactivate Cell
```

---

## Lens 7: Refusal / Risk / Governance

Diagram ID: ZENUML-L7
Diagram family: ZenUML
Projection lens: Refusal / Risk / Governance
Architectural invariant preserved: Every failure is a typed Refusal; N3 quarantine rules are strictly enforced.
Information-loss risk if omitted: Silent failures or untyped exceptions escaping container boundaries.
TPS visual-control purpose: Standardizes error signaling to reduce processing defects.
DfLSS CTQ protected: No panic or silent fallbacks.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Displays exception-handling and refusal typing message flows.

```mermaid
sequenceDiagram
    autonumber
    participant App as Application Logic
    participant Handler as Refusal Handler
    participant Guard as N3 Quarantine Guard
    participant Board as CENG Governance Board

    App->>Handler: trigger_error(exception)
    activate Handler
    Handler->>Handler: map_to_typed_refusal(exception)
    Handler->>Guard: check_n3_quarantine(refusal)
    deactivate Handler
    activate Guard
    alt quarantined
        Guard->>Board: request_exception_review(refusal)
        Board-->>App: refusal_logged
    else normal_refusal
        Guard-->>App: typed_refusal_response
    end
    deactivate Guard
```

---

## Lens 8: TPS / DfLSS / Continuous Improvement

Diagram ID: ZENUML-L8
Diagram family: ZenUML
Projection lens: TPS / DfLSS / Continuous Improvement
Architectural invariant preserved: WIP reduction, continuous process improvement loops, and visual waste elimination.
Information-loss risk if omitted: Operational drift and degradation of continuous validation metrics over time.
TPS visual-control purpose: Kaizen feedback loop sequencing.
DfLSS CTQ protected: Flow efficiency and defect rate minimization.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Outlines sequence of measurements in continuous improvement loops.

```mermaid
sequenceDiagram
    autonumber
    participant Kaizen as Kaizen Monitor
    participant Metric as CTQ Metrics Collector
    participant Bench as Benchmarking Engine
    participant WIP as WIP Controller

    Kaizen->>Metric: retrieve_metrics()
    activate Metric
    Metric-->>Kaizen: metrics_data
    deactivate Metric
    Kaizen->>Bench: execute_latency_benchmarks()
    activate Bench
    Bench-->>Kaizen: latency_report
    deactivate Bench
    Kaizen->>WIP: adjust_wip_limits(latency_report)
    activate WIP
    WIP-->>Kaizen: wip_limits_updated
    deactivate WIP
```
