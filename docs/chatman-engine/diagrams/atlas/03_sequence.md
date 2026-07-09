# Sequence Diagram Family

This document contains exactly 8 sequence diagrams mapping the Chatman Engine across its 8 projection lenses, preserving key architectural invariants under Design for Combinatorial Maximalism.

## Diagrams

### SEQUENCE-L1: Semantic Authority

Diagram ID: SEQUENCE-L1
Diagram family: Sequence
Projection lens: Semantic Authority
Architectural invariant preserved: Direct Oxigraph write gating; all modifications to the semantic graph must execute atomically inside the Oxigraph store before receipt generation.
Information-loss risk if omitted: Receipts generated for updates that fail to commit to the Oxigraph database, breaking the cryptographic state link.
TPS visual-control purpose: Eliminating rework and transaction rollbacks by gating receipt generation behind the physical commit.
DfLSS CTQ protected: 100% synchronization between the cryptographic ledger and Oxigraph state.
CENG ticket or boundary constrained: Bound by CENG-410-FINAL.
Why this diagram is non-redundant: Details the temporal ordering of write-validation-commit-receipt actions, which flowcharts and swimlanes cannot represent chronologically.

```mermaid
sequenceDiagram
    participant Client as External Client
    participant Ingress as Ingress Boundary
    participant Store as Oxigraph Store

    Client->>Ingress: Write Request (RDF payload)
    activate Ingress
    Ingress->>Ingress: Validate Syntax & Rules
    Ingress->>Store: Atomic Commit (RDF Triples)
    activate Store
    Store-->>Ingress: Commit Acknowledged (Success)
    deactivate Store
    Ingress->>Ingress: Compute BLAKE3 Cryptographic Receipt
    Ingress-->>Client: Success Response + Receipt
    deactivate Ingress
```

---

### SEQUENCE-L2: Routing Constitution

Diagram ID: SEQUENCE-L2
Diagram family: Sequence
Projection lens: Routing Constitution
Architectural invariant preserved: Least-expressive-power path routing and quarantine of unauthorized N3 cold-path rules.
Information-loss risk if omitted: Bypassing the routing constitution gate, executing quarantined N3 code on the warm path.
TPS visual-control purpose: Restricting WIP (limiting execution of un-constituted rules).
DfLSS CTQ protected: 100% compliance with rule expressiveness categorization.
CENG ticket or boundary constrained: CENG-410-M1.
Why this diagram is non-redundant: Visualizes the conditional routing checks and quarantine return paths for queries.

```mermaid
sequenceDiagram
    participant Client as External Client
    participant Gater as Request Gater
    participant Hot as Hot Path Evaluator
    participant N3Engine as N3 Cold Path Engine

    Client->>Gater: Submit Query/Rule
    activate Gater
    Gater->>Gater: Analyze Complexity
    alt Complexity is simple (8-constraint)
        Gater->>Hot: Forward Query
        Hot-->>Gater: Fast Evaluation Result
    else Complexity is N3 & N3 is disabled (default)
        Gater->>Gater: Quarantine N3 Query
        Gater-->>Client: Refusal (N3 Quarantined)
    else Complexity is N3 & N3 is enabled
        Gater->>N3Engine: Forward Query
        activate N3Engine
        N3Engine-->>Gater: Cold Path Evaluation Result
        deactivate N3Engine
    end
    Gater-->>Client: Return Result
    deactivate Gater
```

---

### SEQUENCE-L3: Type Kernel Ownership

Diagram ID: SEQUENCE-L3
Diagram family: Sequence
Projection lens: Type Kernel Ownership
Architectural invariant preserved: Strict hierarchical registration of types from base compat to cognition, planning, and hook execution.
Information-loss risk if omitted: Initialization of components out of order, leading to missing type definitions at runtime.
TPS visual-control purpose: Visualizing initialization flow to prevent system integration defects.
DfLSS CTQ protected: Correct initialization sequence with zero duplicate types.
CENG ticket or boundary constrained: CENG-411 (design-only).
Why this diagram is non-redundant: Focuses on the temporal boot/initialization order of modular type registry dependencies.

```mermaid
sequenceDiagram
    participant Engine as Engine Core
    participant Compat as wasm4pm-compat
    participant Cognition as wasm4pm-cognition
    participant Planning as bcinr-pddl/powl
    participant Graphlaw as praxis-graphlaw

    Engine->>Compat: Register Base RDF Types
    activate Compat
    Compat-->>Engine: Base Types Registered
    deactivate Compat
    Engine->>Cognition: Load Breed Profiles (depend on Base Types)
    activate Cognition
    Cognition-->>Engine: Breed Types Registered
    deactivate Cognition
    Engine->>Planning: Initialize PDDL/POWL Domain Schemas
    activate Planning
    Planning-->>Engine: Domain Handlers Active
    deactivate Planning
    Engine->>Graphlaw: Mount Knowledge Hook Registry (depend on all above)
    activate Graphlaw
    Graphlaw-->>Engine: Hooks Registered and Active
    deactivate Graphlaw
```

---

### SEQUENCE-L4: Transition Lifecycle

Diagram ID: SEQUENCE-L4
Diagram family: Sequence
Projection lens: Transition Lifecycle
Architectural invariant preserved: Candidate progression through multi-level validation and ledger recording.
Information-loss risk if omitted: State transition executed before validation is fully completed, leading to corrupted state history.
TPS visual-control purpose: Sequential gating to ensure quality-at-source (Jidoka).
DfLSS CTQ protected: 100% of admitted transitions are valid and receipted.
CENG ticket or boundary constrained: CENG-410-FINAL.
Why this diagram is non-redundant: Focuses on the lifecycle stages of a single candidate payload across validators.

```mermaid
sequenceDiagram
    participant Invoker as Transition Invoker
    participant Validator as SHACL/ShEx Validator
    participant Planner as PDDL/POWL Planner
    participant Ledger as Replay Ledger

    Invoker->>Validator: Validate Candidate Invariants
    activate Validator
    Validator-->>Invoker: Candidate Valid (Success)
    deactivate Validator
    Invoker->>Planner: Verify Plan Legality
    activate Planner
    Planner-->>Invoker: Plan Admitted (Success)
    deactivate Planner
    Invoker->>Ledger: Write State Transition Receipt
    activate Ledger
    Ledger->>Ledger: Compute BLAKE3 hash & write journal
    Ledger-->>Invoker: Transaction Receipt Generated
    deactivate Ledger
```

---

### SEQUENCE-L5: Event / Hook / Actuation

Diagram ID: SEQUENCE-L5
Diagram family: Sequence
Projection lens: Event / Hook / Actuation
Architectural invariant preserved: Event-hook-actuation receipt coupling; deltas project via pure SPARQL CONSTRUCT query.
Information-loss risk if omitted: Side-effect actions executing when the corresponding database update fails or receipt generation fails.
TPS visual-control purpose: Stopping downstream processing (actuation) if receipt generation fails.
DfLSS CTQ protected: Zero side-effects without corresponding cryptographic receipt.
CENG ticket or boundary constrained: CENG-412 (design-only).
Why this diagram is non-redundant: Captures the event-driven async matching, projection, and commit cycle.

```mermaid
sequenceDiagram
    participant Ingress as Event Ingress
    participant Registry as Hook Registry
    participant Projector as Delta Projector
    participant HashCore as BLAKE3 Hash Core
    participant Store as Oxigraph Store

    Ingress->>Registry: Ingest OCEL Event
    activate Registry
    Registry->>Registry: Match Event to Registered Hooks
    Registry->>Projector: Project Graph Delta (SPARQL CONSTRUCT)
    activate Projector
    Projector-->>Registry: Delta (kh:addQuad, kh:deleteQuad)
    deactivate Projector
    Registry->>HashCore: Generate Canonical Receipt
    activate HashCore
    HashCore-->>Registry: BLAKE3 Cryptographic Receipt
    deactivate HashCore
    Registry->>Store: Commit Delta & Receipt
    activate Store
    Store-->>Registry: Transaction Successful
    deactivate Store
    Registry-->>Ingress: Event Actuated + Receipt
    deactivate Registry
```

---

### SEQUENCE-L6: Performance / 8-Constraint Hot Path

Diagram ID: SEQUENCE-L6
Diagram family: Sequence
Projection lens: Performance / 8-Constraint Hot Path
Architectural invariant preserved: Low-latency fast-path evaluation using binary lowering and pre-computed state table lookup.
Information-loss risk if omitted: Performance degradation due to unnecessary traversal of high-level warm path parsers.
TPS visual-control purpose: Exposing processing time waste (latency overhead of warm-path fallbacks).
DfLSS CTQ protected: Hot-path lookup latency ≤ target limit.
CENG ticket or boundary constrained: CENG-410-M1.
Why this diagram is non-redundant: Focuses on the low-level byte-mask check and 256-state admission table.

```mermaid
sequenceDiagram
    participant Client as Ingress Client
    participant Lowerer as Lowering Compiler
    participant Cell as ConditionCell
    participant Table as Admission Table

    Client->>Lowerer: Submit Triple
    activate Lowerer
    Lowerer->>Lowerer: Lower to RDFTriple8 representation
    Lowerer->>Cell: Check Bitwise Match
    activate Cell
    Cell-->>Lowerer: Match Status & Mask
    deactivate Cell
    Lowerer->>Table: Consult 256-state Admission Table
    activate Table
    Table-->>Lowerer: Decision (Admitted / Forward)
    deactivate Table
    alt Admitted
        Lowerer-->>Client: Hot Path Success
    else Forward
        Lowerer-->>Client: Redirect to Warm Path Engine
    end
    deactivate Lowerer
```

---

### SEQUENCE-L7: Refusal / Risk / Governance

Diagram ID: SEQUENCE-L7
Diagram family: Sequence
Projection lens: Refusal / Risk / Governance
Architectural invariant preserved: Containment of failures via typed refusal translation and governance logging.
Information-loss risk if omitted: Leaking internal stack traces or database errors, or missing audit logs for failures.
TPS visual-control purpose: Poka-Yoke (fail-safe error handling to prevent defect escape).
DfLSS CTQ protected: 100% of runtime errors mapped to refusal schema and logged.
CENG ticket or boundary constrained: CENG-410-FINAL.
Why this diagram is non-redundant: Visualizes the governance logging and refusal mapping path which is separate from operational logic.

```mermaid
sequenceDiagram
    participant Exec as Execution Core
    participant Classifier as Refusal Classifier
    participant Board as Governance Board
    participant Client as External Client

    Exec->>Exec: Run Transition Candidate (Violation Detected)
    activate Exec
    Exec->>Classifier: Classify Error State
    activate Classifier
    Classifier->>Classifier: Map to typed Refusal variant
    Classifier-->>Exec: Refusal Object
    deactivate Classifier
    Exec->>Board: Log Refusal State to Audit Ledger
    activate Board
    Board-->>Exec: Log Acknowledged
    deactivate Board
    Exec-->>Client: Return Typed Refusal Response
    deactivate Exec
```

---

### SEQUENCE-L8: TPS / DfLSS / Continuous Improvement

Diagram ID: SEQUENCE-L8
Diagram family: Sequence
Projection lens: TPS / DfLSS / Continuous Improvement
Architectural invariant preserved: Continuous performance improvement loop via metric analysis and optimization feed-forward.
Information-loss risk if omitted: Undetected performance degradation due to structural code drift.
TPS visual-control purpose: Kaizen feedback loop to optimize hot-path table boundaries.
DfLSS CTQ protected: Zero variance in processing times and low latency thresholds.
CENG ticket or boundary constrained: CENG-416A-F (design-only).
Why this diagram is non-redundant: Maps the control-loop execution of telemetry feedback.

```mermaid
sequenceDiagram
    participant Executor as Execution Core
    participant Monitor as Benchmark Monitor
    participant Analyzer as Engine Analyzer
    participant Optimizer as Table Optimizer

    Executor->>Executor: Execute Query
    Executor->>Monitor: Send Latency Telemetry
    activate Monitor
    Monitor->>Analyzer: Forward Metrics Batch
    deactivate Monitor
    activate Analyzer
    Analyzer->>Analyzer: Analyze standard deviation & path hits
    Analyzer->>Optimizer: Trigger Table Optimization
    deactivate Analyzer
    activate Optimizer
    Optimizer->>Optimizer: Compute optimized 256-state configurations
    Optimizer->>Executor: Load Optimized Admission Tables
    deactivate Optimizer
```
