# Swimlanes Diagram Family

This document contains exactly 8 swimlane diagrams (implemented via Mermaid flowchart subgraphs representing lanes) mapping the Chatman Engine across its 8 projection lenses, preserving key architectural invariants under Design for Combinatorial Maximalism.

## Diagrams

### SWIMLANES-L1: Semantic Authority

Diagram ID: SWIMLANES-L1
Diagram family: Swimlanes
Projection lens: Semantic Authority
Architectural invariant preserved: RDF/Oxigraph remains the sole semantic source of truth; no client or ingress gate may store shadow copies of the triple data.
Information-loss risk if omitted: Ingress gates caching read requests or keeping local states, resulting in dirty reads or inconsistent semantic states between the gateway and Oxigraph.
TPS visual-control purpose: Visualizing structural boundaries between the client, ingress validation, and core semantic storage to prevent data replication waste.
DfLSS CTQ protected: Zero semantic shadow copies and absolute transactional isolation.
CENG ticket or boundary constrained: Bound by CENG-410-FINAL.
Why this diagram is non-redundant: Visualizes runtime process boundaries across ownership lanes, which is not captured by simple flowcharts.

```mermaid
flowchart TB
    subgraph ClientLane [External Client]
        A[Query Request] --> B[Write Request]
    end
    subgraph BoundaryLane [Semantic Ingress Boundary]
        C[Validate Request Format]
        D[Compute SHA256 of Ingress Payload]
    end
    subgraph OxigraphLane [Oxigraph Store]
        E[Query Execution Engine]
        F[Direct Write Execution]
    end

    A --> C
    B --> D
    C --> E
    D --> F
```

---

### SWIMLANES-L2: Routing Constitution

Diagram ID: SWIMLANES-L2
Diagram family: Swimlanes
Projection lens: Routing Constitution
Architectural invariant preserved: Safe routing isolation; cold path (N3) is quarantined unless explicitly enabled.
Information-loss risk if omitted: Dynamic execution bypasses the request gater, running unverified N3 rules on warm or hot paths and causing system security breaches.
TPS visual-control purpose: Visualizing segregation of execution environments to eliminate safety defects.
DfLSS CTQ protected: Under-execution/over-execution routing path mapping.
CENG ticket or boundary constrained: CENG-410-M1.
Why this diagram is non-redundant: Represents routing decisions specifically partitioned by execution engines.

```mermaid
flowchart TB
    subgraph GaterLane [Request Gater]
        A[Analyze Complexity]
        B{Select Path}
    end
    subgraph HotLane [Hot Path Evaluator]
        C[Evaluate RDFTriple8 Masks]
    end
    subgraph WarmLane [Warm Path Evaluator]
        D[Execute SPARQL Query]
    end
    subgraph ColdLane [Cold Path Evaluator]
        E[Execute Permissioned N3 Engine]
    end

    A --> B
    B -->|Hot| C
    B -->|Warm| D
    B -->|Cold| E
```

---

### SWIMLANES-L3: Type Kernel Ownership

Diagram ID: SWIMLANES-L3
Diagram family: Swimlanes
Projection lens: Type Kernel Ownership
Architectural invariant preserved: Separation of concerns and kernel definition boundaries; no module may cross-compile another module's types.
Information-loss risk if omitted: Circular dependencies between `praxis-graphlaw` and `wasm4pm-cognition`, breaking compilation.
TPS visual-control purpose: Exposing dependency and duplication waste across compilation boundaries.
DfLSS CTQ protected: Crate-level type isolation and zero-copy kernel mappings.
CENG ticket or boundary constrained: CENG-411 (design-only).
Why this diagram is non-redundant: Shows ownership lanes of data structures at the compile/crate level.

```mermaid
flowchart TB
    subgraph CompatLane [wasm4pm-compat]
        A[Base RDF types & context]
    end
    subgraph CognitionLane [wasm4pm-cognition]
        B[Cognitive Breed profiles]
    end
    subgraph PlannerLane [bcinr-pddl / bcinr-powl]
        C[Planning domain representations]
    end
    subgraph GraphlawLane [praxis-graphlaw]
        D[Knowledge Hook engine core]
    end

    A --> B
    A --> C
    C --> D
    B --> D
```

---

### SWIMLANES-L4: Transition Lifecycle

Diagram ID: SWIMLANES-L4
Diagram family: Swimlanes
Projection lens: Transition Lifecycle
Architectural invariant preserved: Multi-stage admission gate sequence (Validation -> Planning -> Legality -> Receipting).
Information-loss risk if omitted: Execution of state transitions without validation, corrupting history or workflow integrity.
TPS visual-control purpose: Visualizing quality gates in the processing line to prevent defective transitions.
DfLSS CTQ protected: 100% verification rate for all transition candidates.
CENG ticket or boundary constrained: CENG-410-FINAL.
Why this diagram is non-redundant: Maps stages of transition processing to dedicated engine actors.

```mermaid
flowchart TB
    subgraph InvokerLane [Transition Invoker]
        A[Submit Transition Candidate]
    end
    subgraph ValidatorLane [SHACL/ShEx Validator]
        B[Validate Shape Constraints]
    end
    subgraph PlannerLane [PDDL/POWL Planner]
        C[Verify Temporal Legality]
    end
    subgraph LedgerLane [Receipt Ledger]
        D[Issue BLAKE3 Transition Receipt]
    end

    A --> B
    B --> C
    C --> D
```

---

### SWIMLANES-L5: Event / Hook / Actuation

Diagram ID: SWIMLANES-L5
Diagram family: Swimlanes
Projection lens: Event / Hook / Actuation
Architectural invariant preserved: Pure SPARQL CONSTRUCT delta projection; zero side-effects outside of graph delta receipts.
Information-loss risk if omitted: Hooks executing side-effects directly during matching phase, violating transactional rollback constraints.
TPS visual-control purpose: Jidoka (stopping flow on unreceipted actuation attempt).
DfLSS CTQ protected: 100% receipted and verified hook actuations.
CENG ticket or boundary constrained: CENG-412 (design-only).
Why this diagram is non-redundant: Details the event-to-actuation lifecycle across distinct architectural subsystems.

```mermaid
flowchart TB
    subgraph IngressLane [Event Ingress]
        A[Receive OCEL Event]
    end
    subgraph RegistryLane [Hook Registry]
        B[Match Event to Registered Hooks]
    end
    subgraph ProjectorLane [Delta Projector]
        C[Project kh:addQuad / kh:deleteQuad]
    end
    subgraph HashLane [BLAKE3 Hash Core]
        D[Generate Canonical Receipt]
    end
    subgraph StoreLane [Oxigraph Store]
        E[Apply Verified Deltas]
    end

    A --> B
    B --> C
    C --> D
    D --> E
```

---

### SWIMLANES-L6: Performance / 8-Constraint Hot Path

Diagram ID: SWIMLANES-L6
Diagram family: Swimlanes
Projection lens: Performance / 8-Constraint Hot Path
Architectural invariant preserved: RDFTriple8 binary lowering and execution on the 8-constraint hot path.
Information-loss risk if omitted: Compiling hot path queries into generalized warm-path queries, causing CPU cache misses and high execution latency.
TPS visual-control purpose: Minimizing execution path length (visualizing waste removal).
DfLSS CTQ protected: Hot-path processing latency ≤ threshold.
CENG ticket or boundary constrained: CENG-410-M1.
Why this diagram is non-redundant: Isolates low-level data structures (RDFTriple8, ConditionCell) from high-level services.

```mermaid
flowchart TB
    subgraph IngressLane [Tuple Ingress]
        A[Receive Raw Triple]
    end
    subgraph LoweringLane [Lowering Compiler]
        B[Convert Triple to RDFTriple8]
        C[Extract 8-bit Attribute Mask]
    end
    subgraph CellLane [ConditionCell Lane]
        D[Evaluate Bitwise Match against ConditionCell BITS]
    end
    subgraph TableLane [256-state Admission Table]
        E[Consult Pre-computed State Table]
        F[Admit or Forward to Warm Path]
    end

    A --> B
    B --> C
    C --> D
    D --> E
    E --> F
```

---

### SWIMLANES-L7: Refusal / Risk / Governance

Diagram ID: SWIMLANES-L7
Diagram family: Swimlanes
Projection lens: Refusal / Risk / Governance
Architectural invariant preserved: Isolated quarantine of invalid state candidates; all failures are typed Refusals.
Information-loss risk if omitted: Untrusted N3 rules leaking or modifying core system states without containment.
TPS visual-control purpose: Visualizing safety gates and containment areas (Poka-Yoke).
DfLSS CTQ protected: Zero untyped failures or unlogged refusals.
CENG ticket or boundary constrained: CENG-410-FINAL.
Why this diagram is non-redundant: Visualizes quarantine and governance loops which are strictly excluded in standard operational views.

```mermaid
flowchart TB
    subgraph ExecLane [Execution Core]
        A[Run Transition Rule]
    end
    subgraph ClassifierLane [Refusal Classifier]
        B[Type the Failure]
    end
    subgraph QuarantineLane [N3 Quarantine Gate]
        C[Quarantine Triples and Rules]
    end
    subgraph BoardLane [CENG Governance Board]
        D[Log Refusal to Ledger]
        E[Trigger Diagnostic Audit]
    end

    A -->|Failure| B
    B -->|N3 Violation| C
    B -->|Other Refusal| D
    C --> E
    D --> E
```

---

### SWIMLANES-L8: TPS / DfLSS / Continuous Improvement

Diagram ID: SWIMLANES-L8
Diagram family: Swimlanes
Projection lens: TPS / DfLSS / Continuous Improvement
Architectural invariant preserved: Continuous monitoring and reconfiguration loops based on benchmark outputs.
Information-loss risk if omitted: Failure to detect performance drift, leading to slow accumulation of latency regressions.
TPS visual-control purpose: Continuous improvement cycle (Kaizen) for performance optimization.
DfLSS CTQ protected: Zero-drift execution latency.
CENG ticket or boundary constrained: CENG-416A-F (design-only).
Why this diagram is non-redundant: Captures the optimization cycle feedback loop across system actors.

```mermaid
flowchart TB
    subgraph MonitorLane [Benchmark Monitor]
        A[Collect Execution Metrics]
    end
    subgraph AnalysisLane [Engine Analyzer]
        B[Detect Path Violations]
        C[Calculate Standard Deviation of Latency]
    end
    subgraph OptimizerLane [Optimizer Core]
        D[Optimize 256-state Admission Table]
    end
    subgraph ActiveLane [Active Engine]
        E[Apply Optimized Admission Tables]
    end

    A --> B
    B --> C
    C --> D
    D --> E
```
