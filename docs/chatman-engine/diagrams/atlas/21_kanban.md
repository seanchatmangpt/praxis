# 21. Kanban Diagram Family

This file contains the Kanban diagram family for the Chatman Engine, structured across the 8 projection lenses.

Fallback rendering for Mermaid compatibility.

---

## Lens 1: Semantic Authority

Diagram ID: KANBAN-L1
Diagram family: Kanban
Projection lens: Semantic Authority
Architectural invariant preserved: RDF/Oxigraph is the single semantic source of truth. Shadow copies of RDF data are strictly prohibited.
Information-loss risk if omitted: Loss of visibility into RDF authority tasks, allowing development of unauthorized semantic shadow copies.
TPS visual-control purpose: Exposes waste and duplicate semantic tasks in the pipeline.
DfLSS CTQ protected: Zero semantic shadow copies.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Tracks semantic authority task transitions specifically on the Kanban board.

```mermaid
flowchart LR
    subgraph Backlog ["Semantic Backlog"]
        direction TB
        B1["Task: Refactor SPARQL Query Engine"]
        B2["Task: Enforce Closed Vocabularies (wf:, hook:)"]
    end
    subgraph Progress ["In Progress (CENG-410-FINAL)"]
        direction TB
        P1["Task: Integrate Oxigraph RDF Store"]
        P2["Task: Prevent RDF Semantic Shadow Copies"]
    end
    subgraph Verification ["Verification Gate"]
        direction TB
        V1["Task: BLAKE3 Graph Hash Conformity Check"]
    end
    subgraph Done ["Done (CENG-410-M1)"]
        direction TB
        D1["Task: Define Oxigraph Store Interface"]
    end
    Backlog --> Progress
    Progress --> Verification
    Verification --> Done
```

---

## Lens 2: Routing Constitution

Diagram ID: KANBAN-L2
Diagram family: Kanban
Projection lens: Routing Constitution
Architectural invariant preserved: Least-expressive-power routing; hot/warm/cold path isolation. N3 is disabled by default.
Information-loss risk if omitted: Uncontrolled propagation of N3 rules leading to performance degradation or escape from profile-local gates.
TPS visual-control purpose: Prevents routing logic waste by visually separating path implementation tickets.
DfLSS CTQ protected: Safe isolation of cold-path N3 execution.
CENG ticket or boundary constrained: CENG-411 (design-only, implementation blocked).
Why this diagram is non-redundant: Visualizes routing-related tasks and constraints across path states.

```mermaid
flowchart LR
    subgraph Blocked ["Design-Only Blocked (CENG-411)"]
        direction TB
        BL1["Task: Implement N3 Quarantine Gate"]
        BL2["Task: Cold Path External Routing Integration"]
    end
    subgraph Progress ["In Progress (CENG-410-FINAL)"]
        direction TB
        P1["Task: Warm Path SHACL/SPARQL Engine Routing"]
        P2["Task: Hot Path Byte Mask Routing (ConditionCell)"]
    end
    subgraph Done ["Done (CENG-410-M1)"]
        direction TB
        D1["Task: Establish Path Boundaries (Hot/Warm/Cold)"]
    end
    Blocked -.-> Progress
    Progress --> Done
```

---

## Lens 3: Type Kernel Ownership

Diagram ID: KANBAN-L3
Diagram family: Kanban
Projection lens: Type Kernel Ownership
Architectural invariant preserved: Canonical type ownership across wasm4pm-compat, wasm4pm-cognition, bcinr-pddl, bcinr-powl, and praxis-graphlaw.
Information-loss risk if omitted: Overlapping type definitions leading to duplicate semantic serialization formats.
TPS visual-control purpose: Swimlane column boundaries prevent duplicate type development.
DfLSS CTQ protected: Zero duplicate type classes.
CENG ticket or boundary constrained: CENG-412 (design-only, implementation blocked).
Why this diagram is non-redundant: Specifically tracks type ownership assignments across system boundaries.

```mermaid
flowchart LR
    subgraph Cognition ["wasm4pm-cognition Tasks"]
        direction TB
        C1["Task: Map Cognitive Breed Type Definitions"]
    end
    subgraph PDDL_POWL ["bcinr-pddl / bcinr-powl Tasks"]
        direction TB
        PP1["Task: Refactor PDDL Domain Ownership Rules"]
        PP2["Task: Blocked (CENG-412): Type Alignment"]
    end
    subgraph Graphlaw ["praxis-graphlaw Tasks"]
        direction TB
        G1["Task: Maintain Core Triple Mapping Types"]
    end
    Cognition --> PDDL_POWL
    PDDL_POWL --> Graphlaw
```

---

## Lens 4: Transition Lifecycle

Diagram ID: KANBAN-L4
Diagram family: Kanban
Projection lens: Transition Lifecycle
Architectural invariant preserved: Every transition must pass through candidate invocation, validation, planning, execution, receipting, and replay.
Information-loss risk if omitted: Bypassing of validation or receipting phases in lifecycle execution.
TPS visual-control purpose: WIP limits on lifecycle stages prevent transaction pile-up and memory leaks.
DfLSS CTQ protected: Guaranteed transaction replay validation under fixed seed.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Models lifecycle transitions as distinct Kanban columns to control WIP.

```mermaid
flowchart LR
    subgraph Invocation ["1. Invocation [WIP: 5]"]
        T1["Audit: Register Invocation Candidate"]
    end
    subgraph Validation ["2. Validation [WIP: 3]"]
        T2["Audit: Validate OWL-RL & SHACL"]
    end
    subgraph Execution ["3. Actuation [WIP: 2]"]
        T3["Audit: Execute Hook & Generate BLAKE3 Receipt"]
    end
    subgraph Replay ["4. Replay & Archive"]
        T4["Audit: Replay Graph State from OWL-Time"]
    end
    Invocation --> Validation
    Validation --> Execution
    Execution --> Replay
```

---

## Lens 5: Event / Hook / Actuation

Diagram ID: KANBAN-L5
Diagram family: Kanban
Projection lens: Event / Hook / Actuation
Architectural invariant preserved: Hooks cannot actuate without receipts; no unreceipted actuation.
Information-loss risk if omitted: Actuation without proof-of-execution receipt, breaking auditing logs.
TPS visual-control purpose: Error-proofing (Poka-Yoke) actuation by locking the task column until receipt signature is attached.
DfLSS CTQ protected: Zero unreceipted execution events.
CENG ticket or boundary constrained: CENG-416A-F (design-only, implementation blocked).
Why this diagram is non-redundant: Visualizes task dependencies in hook execution pipeline on Kanban.

```mermaid
flowchart LR
    subgraph Ingest ["Event Ingestion"]
        I1["Task: OCEL Event Stream Reader"]
    end
    subgraph Match ["Hook Matcher"]
        M1["Task: Knowledge Hook Pattern Matcher"]
    end
    subgraph BlockedActuation ["Blocked Actuation (CENG-416A-F)"]
        A1["Task: Actuate Boundary Hook"]
    end
    subgraph Receipting ["Receipt Generator"]
        R1["Task: BLAKE3 Receipt Compiler"]
    end
    Ingest --> Match
    Match --> Receipting
    Receipting --> BlockedActuation
```

---

## Lens 6: Performance / 8-Constraint Hot Path

Diagram ID: KANBAN-L6
Diagram family: Kanban
Projection lens: Performance / 8-Constraint Hot Path
Architectural invariant preserved: Maximum of 8 constraints checked in parallel via RDFTriple8 and ConditionCell<BITS>.
Information-loss risk if omitted: Performance bottlenecks or CPU overhead if hot path expands beyond 8 constraints.
TPS visual-control purpose: Kanban column limits act as visual alerts for hot-path constraint violations.
DfLSS CTQ protected: Latency bound of hot path operations.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Tracks constraints optimization and parallel check tasks.

```mermaid
flowchart LR
    subgraph Analysis ["Constraint Analysis"]
        A1["Task: Audit Constraint Set Size"]
    end
    subgraph Lowering ["Vector-to-Mask Lowering"]
        L1["Task: Compile RDFTriple8 to Mask"]
    end
    subgraph Execution ["ConditionCell<BITS> Gate"]
        E1["Task: 256-State Admission Table Execution"]
    end
    subgraph Benchmarks ["Latency Gate"]
        B1["Task: Verify 8-Constraint Hot-Path Latency"]
    end
    Analysis --> Lowering
    Lowering --> Execution
    Execution --> Benchmarks
```

---

## Lens 7: Refusal / Risk / Governance

Diagram ID: KANBAN-L7
Diagram family: Kanban
Projection lens: Refusal / Risk / Governance
Architectural invariant preserved: Every failure is a typed Refusal; N3 quarantine rules are strictly enforced.
Information-loss risk if omitted: Silent failures or untyped panic execution paths.
TPS visual-control purpose: Standardized visual separation of Refusal classification tasks.
DfLSS CTQ protected: No panic or silent fallbacks.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Tracks risk remediation and refusal system tasks.

```mermaid
flowchart LR
    subgraph Audit ["Risk Audit"]
        R1["Task: Audit Replay Failure States"]
    end
    subgraph Classification ["Refusal Typing"]
        C1["Task: Implement Typed Refusal Hierarchy"]
    end
    subgraph Quarantine ["N3 Quarantine Gate"]
        Q1["Task: Segregate Untrusted N3 Code"]
    end
    subgraph Board ["CENG Board Approval"]
        B1["Task: Review Governance Exceptions"]
    end
    Audit --> Classification
    Classification --> Quarantine
    Quarantine --> Board
```

---

## Lens 8: TPS / DfLSS / Continuous Improvement

Diagram ID: KANBAN-L8
Diagram family: Kanban
Projection lens: TPS / DfLSS / Continuous Improvement
Architectural invariant preserved: WIP reduction, continuous process improvement loops, and visual waste elimination.
Information-loss risk if omitted: Accumulation of hidden process waste and visual blind spots in the engineering cycle.
TPS visual-control purpose: Kaizen-driven visual controls to maintain throughput and minimize lead times.
DfLSS CTQ protected: Flow efficiency and defect rate minimization.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Represents the meta-governance of the Kanban board itself under TPS/DfLSS rules.

```mermaid
flowchart LR
    subgraph Metrics ["Define CTQ Metrics"]
        M1["Task: Establish Zero-Shadow Copy Auditing"]
    end
    subgraph Measurement ["Measure Performance"]
        E1["Task: Run Automated Latency Benchmarks"]
    end
    subgraph Analysis ["Kaizen Waste Analysis"]
        A1["Task: Analyze Hand-off Delay Between Lenses"]
    end
    subgraph Control ["Implement Visual Controls"]
        C1["Task: Enforce Kanban WIP Constraints"]
    end
    Metrics --> Measurement
    Measurement --> Analysis
    Analysis --> Control
```
