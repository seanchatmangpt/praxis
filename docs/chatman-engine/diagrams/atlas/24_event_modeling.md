# 24. Event Modeling Diagram Family

This file contains the Event Modeling diagram family for the Chatman Engine, structured across the 8 projection lenses.

Fallback rendering for Mermaid compatibility.

---

## Lens 1: Semantic Authority

Diagram ID: EVENT_MODELING-L1
Diagram family: Event Modeling
Projection lens: Semantic Authority
Architectural invariant preserved: RDF/Oxigraph is the single semantic source of truth. Shadow copies of RDF data are strictly prohibited.
Information-loss risk if omitted: Mismatch between Command assertion, ingested Event, and the resulting query state.
TPS visual-control purpose: Ensures the event-to-read model flow has zero shadow copy buffers.
DfLSS CTQ protected: Zero semantic shadow copies.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Visualizes command-event-read model mapping for semantic writes.

```mermaid
flowchart LR
    subgraph Commands ["Commands (Inputs)"]
        C1["Command: Assert Semantic Triples"]
    end
    subgraph Events ["Events (State Changes)"]
        E1["Event: Triples Ingested into Oxigraph"]
        E2["Event: BLAKE3 Graph Hash Calculated"]
    end
    subgraph ReadModels ["Read Models (Outputs)"]
        R1["Read Model: Oxigraph Canonical Graph State"]
    end
    C1 --> E1
    E1 --> E2
    E2 --> R1
```

---

## Lens 2: Routing Constitution

Diagram ID: EVENT_MODELING-L2
Diagram family: Event Modeling
Projection lens: Routing Constitution
Architectural invariant preserved: Least-expressive-power routing; hot/warm/cold path isolation. N3 is disabled by default.
Information-loss risk if omitted: Warm-path execution events spilling over into hot-path execution environments.
TPS visual-control purpose: Isolates event streams by path type to prevent routing bottlenecks.
DfLSS CTQ protected: Safe isolation of cold-path N3 execution.
CENG ticket or boundary constrained: CENG-411 (design-only, implementation blocked).
Why this diagram is non-redundant: Details routing events and read models.

```mermaid
flowchart LR
    subgraph Commands ["Commands (Inputs)"]
        C1["Command: Request Route Dispatch"]
    end
    subgraph Events ["Events (State Changes)"]
        E1["Event: Route Type Classified"]
        E2["Event: N3 Rule Detected & Quarantined (CENG-411)"]
    end
    subgraph ReadModels ["Read Models (Outputs)"]
        R1["Read Model: Hot Path Mask Registry"]
        R2["Read Model: Warm Path SPARQL Registry"]
        R3["Read Model: Cold Path Quarantine Log"]
    end
    C1 --> E1
    E1 -->|Hot| R1
    E1 -->|Warm| R2
    E1 -->|Cold| E2
    E2 --> R3
```

---

## Lens 3: Type Kernel Ownership

Diagram ID: EVENT_MODELING-L3
Diagram family: Event Modeling
Projection lens: Type Kernel Ownership
Architectural invariant preserved: Canonical type ownership across wasm4pm-compat, wasm4pm-cognition, bcinr-pddl, bcinr-powl, and praxis-graphlaw.
Information-loss risk if omitted: Redundant type register events causing state corruption during replay.
TPS visual-control purpose: Restricts type registration commands to canonical system boundaries.
DfLSS CTQ protected: Zero duplicate type classes.
CENG ticket or boundary constrained: CENG-412 (design-only, implementation blocked).
Why this diagram is non-redundant: Models the registration and verification timeline of type models.

```mermaid
flowchart LR
    subgraph Commands ["Commands (Inputs)"]
        C1["Command: Register New Domain Type"]
    end
    subgraph Events ["Events (State Changes)"]
        E1["Event: Crate Dependency Verified"]
        E2["Event: Type Registered in wasm4pm-compat"]
        E3["Event: Blocked (CENG-412): Resolve Powl choice types"]
    end
    subgraph ReadModels ["Read Models (Outputs)"]
        R1["Read Model: Canonical Type Registry"]
    end
    C1 --> E1
    E1 --> E2
    E2 --> E3
    E3 --> R1
```

---

## Lens 4: Transition Lifecycle

Diagram ID: EVENT_MODELING-L4
Diagram family: Event Modeling
Projection lens: Transition Lifecycle
Architectural invariant preserved: Every transition must pass through candidate invocation, validation, planning, execution, receipting, and replay.
Information-loss risk if omitted: Lifecycle event sequence bypasses validation before receipting.
TPS visual-control purpose: Restricts WIP in transition phases by mapping events sequentially.
DfLSS CTQ protected: Replayable state transitions under fixed seed.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Details lifecycle command-to-event sequence.

```mermaid
flowchart LR
    subgraph Commands ["Commands (Inputs)"]
        C1["Command: Initiate State Transition"]
    end
    subgraph Events ["Events (State Changes)"]
        E1["Event: Candidate Invoked & Audited"]
        E2["Event: Graph Rules Validated (SHACL)"]
        E3["Event: Executed & BLAKE3 Receipt Signed"]
    end
    subgraph ReadModels ["Read Models (Outputs)"]
        R1["Read Model: Replay Store Journal"]
    end
    C1 --> E1
    E1 --> E2
    E2 --> E3
    E3 --> R1
```

---

## Lens 5: Event / Hook / Actuation

Diagram ID: EVENT_MODELING-L5
Diagram family: Event Modeling
Projection lens: Event / Hook / Actuation
Architectural invariant preserved: Hooks cannot actuate without receipts; no unreceipted actuation.
Information-loss risk if omitted: Ingested events triggering side-effects without receipt logs.
TPS visual-control purpose: Error-proofs hook actuation via interlocked receipt verification.
DfLSS CTQ protected: Zero unreceipted actuation events.
CENG ticket or boundary constrained: CENG-416A-F (design-only, implementation blocked).
Why this diagram is non-redundant: Visualizes Hook-actuation events and read models.

```mermaid
flowchart LR
    subgraph Commands ["Commands (Inputs)"]
        C1["Command: Trigger OCEL Event"]
    end
    subgraph Events ["Events (State Changes)"]
        E1["Event: OCEL Event Stream Ingested"]
        E2["Event: Knowledge Hook Matched"]
        E3["Event: BLAKE3 Receipt Generated"]
        E4["Event: Boundary Actuation Executed (CENG-416)"]
    end
    subgraph ReadModels ["Read Models (Outputs)"]
        R1["Read Model: Actuation History Ledger"]
    end
    C1 --> E1
    E1 --> E2
    E2 --> E3
    E3 --> E4
    E4 --> R1
```

---

## Lens 6: Performance / 8-Constraint Hot Path

Diagram ID: EVENT_MODELING-L6
Diagram family: Event Modeling
Projection lens: Performance / 8-Constraint Hot Path
Architectural invariant preserved: RDFTriple8, ConditionCell<BITS> byte masks, and 256-state tables.
Information-loss risk if omitted: High hot-path constraint counts leading to latency spikes.
TPS visual-control purpose: Tracks the lowering events of hot-path constraints to maintain speed.
DfLSS CTQ protected: Latency bound of hot path operations.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Models constraint byte mask lowering timeline events.

```mermaid
flowchart LR
    subgraph Commands ["Commands (Inputs)"]
        C1["Command: Check Hot Path Invariant"]
    end
    subgraph Events ["Events (State Changes)"]
        E1["Event: RDFTriple8 Projected"]
        E2["Event: Vector Lowered to Byte Mask"]
        E3["Event: ConditionCell Checked against 256-State Table"]
    end
    subgraph ReadModels ["Read Models (Outputs)"]
        R1["Read Model: Latency Benchmarks telemetry"]
    end
    C1 --> E1
    E1 --> E2
    E2 --> E3
    E3 --> R1
```

---

## Lens 7: Refusal / Risk / Governance

Diagram ID: EVENT_MODELING-L7
Diagram family: Event Modeling
Projection lens: Refusal / Risk / Governance
Architectural invariant preserved: Typed Refusal hierarchy; N3 quarantine rules.
Information-loss risk if omitted: Silent failures during transaction execution.
TPS visual-control purpose: Separates normal events from exception refusal events.
DfLSS CTQ protected: Zero untyped exceptions or panics.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Visualizes refusal events and audit log models.

```mermaid
flowchart LR
    subgraph Commands ["Commands (Inputs)"]
        C1["Command: Submit Invalid Signature Candidate"]
    end
    subgraph Events ["Events (State Changes)"]
        E1["Event: Signature Validation Failed"]
        E2["Event: Typed Refusal Exception Emitted"]
        E3["Event: Candidate Refusal Logged"]
    end
    subgraph ReadModels ["Read Models (Outputs)"]
        R1["Read Model: Refusal Audit Journal"]
    end
    C1 --> E1
    E1 --> E2
    E2 --> E3
    E3 --> R1
```

---

## Lens 8: TPS / DfLSS / Continuous Improvement

Diagram ID: EVENT_MODELING-L8
Diagram family: Event Modeling
Projection lens: TPS / DfLSS / Continuous Improvement
Architectural invariant preserved: Continuous Kaizen optimization loops, visual gauges, waste reduction.
Information-loss risk if omitted: Untracked process bottleneck events and drift.
TPS visual-control purpose: Shows telemetry events mapped to continuous improvement read models.
DfLSS CTQ protected: Throughput and defect-free execution rate.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Maps continuous improvement loops in event modeling format.

```mermaid
flowchart LR
    subgraph Commands ["Commands (Inputs)"]
        C1["Command: Adjust WIP Limit Settings"]
    end
    subgraph Events ["Events (State Changes)"]
        E1["Event: WIP Violations Measured"]
        E2["Event: Kaizen Optimization Suggestion Logged"]
        E3["Event: Benchmark Feedback Applied"]
    end
    subgraph ReadModels ["Read Models (Outputs)"]
        R1["Read Model: Continuous Quality Dashboard"]
    end
    C1 --> E1
    E1 --> E2
    E2 --> E3
    E3 --> R1
```
