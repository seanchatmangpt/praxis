# Quadrant Chart Diagram Family

This document contains exactly 8 Quadrant chart diagrams mapping the Chatman Engine across its 8 projection lenses, preserving key architectural invariants under Design for Combinatorial Maximalism.

## Diagrams

### QUADRANT-L1: Semantic Authority

Diagram ID: QUADRANT-L1
Diagram family: Quadrant
Projection lens: Semantic Authority
Architectural invariant preserved: RDF/Oxigraph is the single semantic source of truth; all facts are classified by semantic authority and persistence guarantees.
Information-loss risk if omitted: Treating local transient variables with the same authority as the Oxigraph store.
TPS visual-control purpose: Identifying and eliminating volatile and unverified data storage paths.
DfLSS CTQ protected: Zero semantic shadow copies.
CENG ticket or boundary constrained: Bound by CENG-410-FINAL.
Why this diagram is non-redundant: Visualizes the classification of database components by persistence and authority, which flow or sequence charts cannot do.

```mermaid
quadrantChart
    title Data Path Semantic Authority and Persistence
    x-axis Low Semantic Authority --> High Semantic Authority
    y-axis Low Persistence Guarantees --> High Persistence Guarantees
    quadrant-1 Volatile Caches
    quadrant-2 Unverified Databases
    quadrant-3 Quarantined Stores
    quadrant-4 Canonical Oxigraph Store
    "Memory Cache": [0.2, 0.3]
    "Local Struct Cache": [0.25, 0.45]
    "Staging TripleStore": [0.6, 0.5]
    "Oxigraph Core": [0.9, 0.95]
    "BLAKE3 Receipt Index": [0.85, 0.8]
```

---

### QUADRANT-L2: Routing Constitution

Diagram ID: QUADRANT-L2
Diagram family: Quadrant
Projection lens: Routing Constitution
Architectural invariant preserved: Least-expressive-power path routing.
Information-loss risk if omitted: Over-allocating complex paths to low-complexity queries, causing runtime inefficiency.
TPS visual-control purpose: Exposing routing waste (mapping query complexity against performance latency).
DfLSS CTQ protected: Least-expressive routing model assignment.
CENG ticket or boundary constrained: CENG-410-M1.
Why this diagram is non-redundant: Classifies execution engines and paths based on complexity and latency characteristics.

```mermaid
quadrantChart
    title Query Route Optimization
    x-axis Low Routing Complexity --> High Routing Complexity
    y-axis Low Performance Latency --> High Performance Latency
    quadrant-1 Ideal Hot Path
    quadrant-2 Inefficient Simple Path
    quadrant-3 Complex Warm Path
    quadrant-4 Quarantined Cold Path
    "Hot Path (RDFTriple8)": [0.1, 0.15]
    "SPARQL ASK Query": [0.5, 0.45]
    "SHACL Shapes Gate": [0.6, 0.6]
    "N3 Engine execution": [0.85, 0.9]
    "Quarantined N3 Request": [0.9, 0.1]
```

---

### QUADRANT-L3: Type Kernel Ownership

Diagram ID: QUADRANT-L3
Diagram family: Quadrant
Projection lens: Type Kernel Ownership
Architectural invariant preserved: Strict crate-level type kernel mapping to enforce modular domain boundaries.
Information-loss risk if omitted: Monolithic type structures and loose dependency management causing architectural regression.
TPS visual-control purpose: Eliminating development rework waste caused by circular crate dependencies.
DfLSS CTQ protected: Crate-level type boundary isolation.
CENG ticket or boundary constrained: CENG-411 (design-only).
Why this diagram is non-redundant: Maps crate dependencies and cohesion levels.

```mermaid
quadrantChart
    title Crate Cohesion and Dependency Mapping
    x-axis Low Crate Cohesion --> High Crate Cohesion
    y-axis Low Dependency Coupling --> High Dependency Coupling
    quadrant-1 Orphan Implementations
    quadrant-2 Modular Helpers
    quadrant-3 Monolithic Kernels
    quadrant-4 Canonical Domain Libraries
    "wasm4pm-compat": [0.85, 0.2]
    "wasm4pm-cognition": [0.8, 0.4]
    "bcinr-pddl": [0.75, 0.5]
    "praxis-graphlaw": [0.9, 0.6]
    "Dynamic Test Mocks": [0.3, 0.3]
```

---

### QUADRANT-L4: Transition Lifecycle

Diagram ID: QUADRANT-L4
Diagram family: Quadrant
Projection lens: Transition Lifecycle
Architectural invariant preserved: Linear state progression of transition candidates.
Information-loss risk if omitted: Spending excessive resources optimizing low-priority validations.
TPS visual-control purpose: Optimizing verification gating sequences based on execution cost and priority.
DfLSS CTQ protected: Complete verification of candidate transitions.
CENG ticket or boundary constrained: CENG-410-FINAL.
Why this diagram is non-redundant: Classifies transition steps by execution priority and validation cost.

```mermaid
quadrantChart
    title Transition Step Verification Gating
    x-axis Low Execution Priority --> High Execution Priority
    y-axis Low Verification Cost --> High Verification Cost
    quadrant-1 Trivial Actions
    quadrant-2 Critical Fast Paths
    quadrant-3 Complex Operations
    quadrant-4 Expensive Audits
    "Propose Candidate": [0.8, 0.1]
    "SHACL Validation Check": [0.6, 0.7]
    "PDDL Planning Phase": [0.7, 0.85]
    "BLAKE3 Signing Event": [0.9, 0.15]
    "Ledger Replay Step": [0.5, 0.9]
```

---

### QUADRANT-L5: Event / Hook / Actuation

Diagram ID: QUADRANT-L5
Diagram family: Quadrant
Projection lens: Event / Hook / Actuation
Architectural invariant preserved: OCEL event ingestion to pure hook action mapping.
Information-loss risk if omitted: Failing to isolate high-risk side effect operations.
TPS visual-control purpose: Exposing side-effect paths to maintain pure functional graph transitions.
DfLSS CTQ protected: 100% pure delta projections.
CENG ticket or boundary constrained: CENG-412 (design-only).
Why this diagram is non-redundant: Classifies event/hook actions by ingestion frequency and actuation risk.

```mermaid
quadrantChart
    title Hook Actuation Risk and Frequency
    x-axis Low Actuation Risk --> High Actuation Risk
    y-axis Low Ingestion Frequency --> High Ingestion Frequency
    quadrant-1 Cold Audit Logs
    quadrant-2 Standard Events
    quadrant-3 High Frequency Streams
    quadrant-4 Critical Actuation Hooks
    "OCEL Log Entry": [0.2, 0.8]
    "Hook Match Rule": [0.5, 0.6]
    "SPARQL CONSTRUCT delta": [0.7, 0.5]
    "Graph Delta Commit": [0.85, 0.3]
    "Side Effect Attempt": [0.9, 0.9]
```

---

### QUADRANT-L6: Performance / 8-Constraint Hot Path

Diagram ID: QUADRANT-L6
Diagram family: Quadrant
Projection lens: Performance / 8-Constraint Hot Path
Architectural invariant preserved: Sub-microsecond hot-path query execution via binary lowering.
Information-loss risk if omitted: Maintaining un-aligned structures in CPU registers, degrading hot path speed.
TPS visual-control purpose: Monitoring performance pathways.
DfLSS CTQ protected: Hot path execution time boundaries.
CENG ticket or boundary constrained: CENG-410-M1.
Why this diagram is non-redundant: Classifies data structs by bitmask efficiency and CPU cache locality.

```mermaid
quadrantChart
    title Hot Path Structure Cache and Register Efficiency
    x-axis Low Bitmask Efficiency --> High Bitmask Efficiency
    y-axis Low CPU Cache Locality --> High CPU Cache Locality
    quadrant-1 Unoptimized Lookups
    quadrant-2 Cache Friendlies
    quadrant-3 Highly Aligned Register Hits
    quadrant-4 Inefficient Registers
    "RDFTriple8 Struct": [0.9, 0.95]
    "ConditionCell MATCH": [0.85, 0.85]
    "256-state Table check": [0.95, 0.9]
    "Dynamic Hash Map search": [0.15, 0.2]
    "SPARQL Query Parser": [0.1, 0.1]
```

---

### QUADRANT-L7: Refusal / Risk / Governance

Diagram ID: QUADRANT-L7
Diagram family: Quadrant
Projection lens: Refusal / Risk / Governance
Architectural invariant preserved: Containment of failures via typed refusal translation.
Information-loss risk if omitted: Bypassing recovery steps for critical security violations.
TPS visual-control purpose: Error containment logging (Poka-Yoke).
DfLSS CTQ protected: Zero undocumented refusals.
CENG ticket or boundary constrained: CENG-410-FINAL.
Why this diagram is non-redundant: Classifies failure modes by security risk and recovery priority.

```mermaid
quadrantChart
    title Failure Variant Security Risk and Recovery
    x-axis Low Security Risk --> High Security Risk
    y-axis Low Recovery Priority --> High Recovery Priority
    quadrant-1 Minor Glitches
    quadrant-2 System Containments
    quadrant-3 Critical Gated Failures
    quadrant-4 High Exposure Risks
    "N3 Quarantine Violation": [0.9, 0.9]
    "Profile Denial": [0.6, 0.3]
    "Replay Mismatch Error": [0.8, 0.8]
    "Audit Gate Refusal": [0.7, 0.4]
    "Syntax Warnings": [0.2, 0.2]
```

---

### QUADRANT-L8: TPS / DfLSS / Continuous Improvement

Diagram ID: QUADRANT-L8
Diagram family: Quadrant
Projection lens: TPS / DfLSS / Continuous Improvement
Architectural invariant preserved: Continuous performance improvement loop via metric analysis.
Information-loss risk if omitted: Wasting resources on low-impact optimizations.
TPS visual-control purpose: Prioritizing engineering efforts based on waste reduction impact.
DfLSS CTQ protected: Accurate telemetry tracking.
CENG ticket or boundary constrained: CENG-416A-F (design-only).
Why this diagram is non-redundant: Classifies Kaizen initiatives by complexity and waste reduction impact.

```mermaid
quadrantChart
    title Kaizen Initiative Impact and Complexity
    x-axis Low Waste Reduction Impact --> High Waste Reduction Impact
    y-axis Low Implementation Complexity --> High Implementation Complexity
    quadrant-1 Low Value Tweaks
    quadrant-2 Quick Wins
    quadrant-3 Long Term Kaizen Projects
    quadrant-4 Inefficient Initiatives
    "256-state Table Tuning": [0.9, 0.2]
    "Lowering Compiler tuning": [0.85, 0.65]
    "SPARQL Parser Refactoring": [0.4, 0.8]
    "Telemetry Aggregation build": [0.6, 0.4]
    "Manual Audit Checks": [0.2, 0.1]
```
