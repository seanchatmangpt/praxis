# Entity Relationship Diagram Family

This document contains exactly 8 Entity Relationship (ER) diagrams mapping the Chatman Engine across its 8 projection lenses, preserving key architectural invariants under Design for Combinatorial Maximalism.

## Diagrams

### ENTITY_RELATIONSHIP-L1: Semantic Authority

Diagram ID: ENTITY_RELATIONSHIP-L1
Diagram family: Entity Relationship
Projection lens: Semantic Authority
Architectural invariant preserved: RDF Quad structure semantics; every fact is represented as a canonical RDF Quad consisting of Subject, Predicate, Object, and Graph.
Information-loss risk if omitted: modeling triples with custom properties or skipping graph name contexts, losing Oxigraph semantics.
TPS visual-control purpose: Visualizing fact structures to eliminate semantic representation waste.
DfLSS CTQ protected: Zero non-standard triple shapes in storage.
CENG ticket or boundary constrained: Bound by CENG-410-FINAL.
Why this diagram is non-redundant: Details the structural layout of Oxigraph storage tables and attributes.

```mermaid
erDiagram
    OXIGRAPH_STORE ||--|{ GRAPH : contains
    GRAPH ||--|{ QUAD : holds
    QUAD ||--|| TERM : subject
    QUAD ||--|| TERM : predicate
    QUAD ||--|| TERM : object
    QUAD ||--|| TERM : graph_name
    TERM {
        string value
        string datatype
        string lang
        string term_type
    }
```

---

### ENTITY_RELATIONSHIP-L2: Routing Constitution

Diagram ID: ENTITY_RELATIONSHIP-L2
Diagram family: Entity Relationship
Projection lens: Routing Constitution
Architectural invariant preserved: Query-to-path routing mapping relationships.
Information-loss risk if omitted: Unmapped query routing schemas causing errors during complexity resolution.
TPS visual-control purpose: Restricting execution paths based on complexity metadata.
DfLSS CTQ protected: Least-expressive routing model assignment.
CENG ticket or boundary constrained: CENG-410-M1.
Why this diagram is non-redundant: Models relational data used by the router for path gating and quarantine checks.

```mermaid
erDiagram
    REQUEST ||--|| COMPLEXITY_METADATA : analyzes
    REQUEST ||--|| ROUTE : allocated_to
    ROUTE ||--|| EVALUATOR : maps_to
    REQUEST ||--o| QUARANTINE_REGISTRY : isolated_in
    ROUTE {
        string path_id
        string path_type
        bool is_active
    }
    COMPLEXITY_METADATA {
        int constraint_count
        string expression_language
        bool requires_n3
    }
    QUARANTINE_REGISTRY {
        string incident_id
        timestamp quarantined_at
        string violation_reason
    }
```

---

### ENTITY_RELATIONSHIP-L3: Type Kernel Ownership

Diagram ID: ENTITY_RELATIONSHIP-L3
Diagram family: Entity Relationship
Projection lens: Type Kernel Ownership
Architectural invariant preserved: Dependency constraints and registry ownership relations of kernel types.
Information-loss risk if omitted: Creating circular references or duplicates of kernel types across separate tables.
TPS visual-control purpose: Exposing type duplication waste across compile-time schemas.
DfLSS CTQ protected: Crate-level type boundary isolation.
CENG ticket or boundary constrained: CENG-411 (design-only).
Why this diagram is non-redundant: Focuses on domain type relationships and modular dependency chains.

```mermaid
erDiagram
    CRATE ||--|{ CANONICAL_TYPE : defines
    CRATE ||--o{ DEPENDENCY : imports
    CANONICAL_TYPE ||--o| KERNEL_INTERFACE : implements
    CRATE {
        string crate_name
        string version
        string owner_team
    }
    CANONICAL_TYPE {
        string type_id
        string module_path
        string serialized_format
    }
```

---

### ENTITY_RELATIONSHIP-L4: Transition Lifecycle

Diagram ID: ENTITY_RELATIONSHIP-L4
Diagram family: Entity Relationship
Projection lens: Transition Lifecycle
Architectural invariant preserved: Verification record chaining for admitted transitions.
Information-loss risk if omitted: Orphan transition receipts that do not reference validation or planning logs.
TPS visual-control purpose: Visualizing validation data chains to ensure auditability.
DfLSS CTQ protected: 100% of receipts contain a valid planning and validation reference.
CENG ticket or boundary constrained: CENG-410-FINAL.
Why this diagram is non-redundant: Details validation audit entity relationships.

```mermaid
erDiagram
    CANDIDATE_PAYLOAD ||--|| VALIDATION_RECORD : passes
    CANDIDATE_PAYLOAD ||--|| PLANNING_RESULT : conforms_to
    CANDIDATE_PAYLOAD ||--|| TRANSITION_RECEIPT : produces
    VALIDATION_RECORD {
        string record_id
        string validator_name
        bool is_valid
        string failure_logs
    }
    PLANNING_RESULT {
        string plan_id
        string pddl_actions
        bool is_executable
    }
    TRANSITION_RECEIPT {
        bytes32 blake3_hash
        timestamp timestamp
        string signature
    }
```

---

### ENTITY_RELATIONSHIP-L5: Event / Hook / Actuation

Diagram ID: ENTITY_RELATIONSHIP-L5
Diagram family: Entity Relationship
Projection lens: Event / Hook / Actuation
Architectural invariant preserved: OCEL event ingestion to pure hook action mapping.
Information-loss risk if omitted: Untraceable delta executions because events and hook triggers are decoupled in the schema.
TPS visual-control purpose: Exposing side-effect paths to maintain pure functional graph transitions.
DfLSS CTQ protected: Complete traceability of delta actuation to source OCEL events.
CENG ticket or boundary constrained: CENG-412 (design-only).
Why this diagram is non-redundant: Details schema relationships for hooks, events, and projected quads.

```mermaid
erDiagram
    OCEL_EVENT ||--o{ KNOWLEDGE_HOOK : triggers
    KNOWLEDGE_HOOK ||--|| SPARQL_QUERY : executes
    SPARQL_QUERY ||--|| GRAPH_DELTA : projects
    GRAPH_DELTA ||--|| BLAKE3_RECEIPT : generates
    GRAPH_DELTA {
        string delta_id
        string quads_to_add
        string quads_to_delete
    }
    OCEL_EVENT {
        string event_id
        string activity
        timestamp timestamp
    }
    KNOWLEDGE_HOOK {
        string hook_id
        string name
        bool is_active
    }
```

---

### ENTITY_RELATIONSHIP-L6: Performance / 8-Constraint Hot Path

Diagram ID: ENTITY_RELATIONSHIP-L6
Diagram family: Entity Relationship
Projection lens: Performance / 8-Constraint Hot Path
Architectural invariant preserved: RDFTriple8 performance mappings; ConditionCell binary matching structures.
Information-loss risk if omitted: Heap mapping models that represent low-level byte masks, hiding performance-critical memory alignments.
TPS visual-control purpose: Direct memory mapping representation to reduce evaluation latency.
DfLSS CTQ protected: Hot path execution data constraints.
CENG ticket or boundary constrained: CENG-410-M1.
Why this diagram is non-redundant: Visualizes register-aligned entity relationships on the hot path.

```mermaid
erDiagram
    RDF_TRIPLE_8 ||--|| BYTE_MASK : lowers_to
    CONDITION_CELL ||--|| BYTE_MASK : checks
    ADMISSION_TABLE_256 ||--|{ BYTE_MASK : filters
    RDF_TRIPLE_8 {
        uint8 subject_id
        uint8 predicate_id
        uint8 object_id
        uint8 attribute_mask
    }
    CONDITION_CELL {
        uint8 target_bits
        uint8 match_mask
    }
    ADMISSION_TABLE_256 {
        binary bitmap_256
    }
```

---

### ENTITY_RELATIONSHIP-L7: Refusal / Risk / Governance

Diagram ID: ENTITY_RELATIONSHIP-L7
Diagram family: Entity Relationship
Projection lens: Refusal / Risk / Governance
Architectural invariant preserved: Schema constraints on refusal records, quarantines, and governance logs.
Information-loss risk if omitted: Loss of audit trace data for quarantined triple failures.
TPS visual-control purpose: Error containment logging (Poka-Yoke).
DfLSS CTQ protected: Zero undocumented refusals.
CENG ticket or boundary constrained: CENG-410-FINAL.
Why this diagram is non-redundant: Models security/risk metadata relationships.

```mermaid
erDiagram
    FAILURE_RECORD ||--|| REFUSAL_TYPE : categorized_as
    FAILURE_RECORD ||--|| AUDIT_LOG : written_to
    FAILURE_RECORD ||--o| QUARANTINE_ZONE : isolates
    FAILURE_RECORD {
        string failure_id
        timestamp timestamp
        string raw_payload
    }
    REFUSAL_TYPE {
        string code
        string description
    }
    QUARANTINE_ZONE {
        string zone_id
        string isolated_triples
    }
```

---

### ENTITY_RELATIONSHIP-L8: TPS / DfLSS / Continuous Improvement

Diagram ID: ENTITY_RELATIONSHIP-L8
Diagram family: Entity Relationship
Projection lens: TPS / DfLSS / Continuous Improvement
Architectural invariant preserved: Metric relational schemas for Kaizen performance tuning.
Information-loss risk if omitted: Telemetry data schema omissions that hide path latency regression parameters.
TPS visual-control purpose: Exposing metrics schema dependencies for optimization feedback loops.
DfLSS CTQ protected: Accurate telemetry tracking schema bounds.
CENG ticket or boundary constrained: CENG-416A-F (design-only).
Why this diagram is non-redundant: Models relational diagnostic tables.

```mermaid
erDiagram
    TELEMETRY_BATCH ||--|{ PERFORMANCE_METRIC : aggregates
    TELEMETRY_BATCH ||--|| OPTIMIZATION_RULE : evaluates
    OPTIMIZATION_RULE ||--|| ADMISSION_TABLE : updates
    PERFORMANCE_METRIC {
        string metric_id
        int64 execution_ns
        int64 cache_misses
    }
    TELEMETRY_BATCH {
        string batch_id
        timestamp start_time
        timestamp end_time
    }
    OPTIMIZATION_RULE {
        string rule_id
        string action_type
        float priority
    }
```
