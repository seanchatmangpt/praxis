# Gantt Diagram Family

This document contains exactly 8 Gantt diagrams mapping the Chatman Engine across its 8 projection lenses, preserving key architectural invariants under Design for Combinatorial Maximalism.

## Diagrams

### GANTT-L1: Semantic Authority

Diagram ID: GANTT-L1
Diagram family: Gantt
Projection lens: Semantic Authority
Architectural invariant preserved: Transactional integrity of the Oxigraph store.
Information-loss risk if omitted: Scheduling receipt generation and database updates in parallel rather than sequentially, leading to out-of-order execution states.
TPS visual-control purpose: Tracking and optimizing time spent in database commit phases to reduce transaction cycle time.
DfLSS CTQ protected: 100% of receipts generated post-commit.
CENG ticket or boundary constrained: Bound by CENG-410-FINAL.
Why this diagram is non-redundant: Visualizes the temporal scheduling and execution phases of database writing, which other views cannot show.

```mermaid
gantt
    title Oxigraph Transaction Write Timeline
    dateFormat  YYYY-MM-DD
    section Transaction Ingress
    Ingest & Parse Request      :active, t1, 2026-07-08, 1d
    Verify Syntax & Schemas     :t2, after t1, 1d
    section Database Commit
    Write Triples to Oxigraph   :crit, t3, after t2, 2d
    section Receipt Generation
    Compute BLAKE3 Hash         :t4, after t3, 1d
    Sign & Return Receipt       :t5, after t4, 1d
```

---

### GANTT-L2: Routing Constitution

Diagram ID: GANTT-L2
Diagram family: Gantt
Projection lens: Routing Constitution
Architectural invariant preserved: Least-expressive-power path routing; Cold path N3 is quarantined unless explicitly enabled.
Information-loss risk if omitted: Overlapping development schedules of warm and cold path evaluators, allowing N3 to bypass gater checks.
TPS visual-control purpose: Visualizing scheduling dependencies of routing subsystems to prevent structural defects.
DfLSS CTQ protected: Least-expressive routing model assignment.
CENG ticket or boundary constrained: CENG-410-M1.
Why this diagram is non-redundant: Tracks development and release phases of the query routing engine.

```mermaid
gantt
    title Query Routing Engine Development
    dateFormat  YYYY-MM-DD
    section Parser Setup
    Complexity Classifier       :done, p1, 2026-06-01, 15d
    section Evaluators
    Hot Path Evaluator          :done, e1, after p1, 10d
    Warm Path Evaluator         :done, e2, after e1, 12d
    Cold Path Evaluator         :active, e3, after e2, 10d
    section Quarantine
    N3 Quarantine Gating        :crit, q1, after e3, 7d
```

---

### GANTT-L3: Type Kernel Ownership

Diagram ID: GANTT-L3
Diagram family: Gantt
Projection lens: Type Kernel Ownership
Architectural invariant preserved: Strict crate-level type kernel mapping to enforce modular domain boundaries.
Information-loss risk if omitted: Parallel development of types in separate crates, causing circular compilation dependencies.
TPS visual-control purpose: Eliminating development rework waste caused by circular crate dependencies.
DfLSS CTQ protected: Crate-level type boundary isolation.
CENG ticket or boundary constrained: CENG-411 (design-only).
Why this diagram is non-redundant: Tracks the implementation sequence of crate-level type registries.

```mermaid
gantt
    title Kernel Type Registration Schedule
    dateFormat  YYYY-MM-DD
    section Core Registry
    wasm4pm-compat Types        :done, k1, 2026-05-01, 10d
    wasm4pm-cognition Profiles  :done, k2, after k1, 12d
    section Planning & Gating
    bcinr-pddl Schemas          :done, k3, after k2, 14d
    praxis-graphlaw Hooks       :active, k4, after k3, 10d
```

---

### GANTT-L4: Transition Lifecycle

Diagram ID: GANTT-L4
Diagram family: Gantt
Projection lens: Transition Lifecycle
Architectural invariant preserved: Linear state progression of transition candidates through all validation gates.
Information-loss risk if omitted: Running validations in parallel without proper cascading dependencies, causing unvalidated admissions.
TPS visual-control purpose: Visualizing gate sequence scheduling for quality control.
DfLSS CTQ protected: Complete verification of candidate transitions.
CENG ticket or boundary constrained: CENG-410-FINAL.
Why this diagram is non-redundant: Details the temporal execution timeline of a single transition candidate.

```mermaid
gantt
    title Single State Transition Process Timeline
    dateFormat  HH:mm:ss
    axisFormat %H:%M:%S
    section Validation Gating
    Propose Candidate           :active, a1, 00:00:00, 5s
    SHACL Shape Verification    :a2, after a1, 10s
    PDDL Plan Legality Check    :a3, after a2, 15s
    POWL Legality Check         :a4, after a3, 5s
    section Finalization
    Compute BLAKE3 Receipt      :crit, a5, after a4, 2s
    Ledger Write Commit         :a6, after a5, 8s
```

---

### GANTT-L5: Event / Hook / Actuation

Diagram ID: GANTT-L5
Diagram family: Gantt
Projection lens: Event / Hook / Actuation
Architectural invariant preserved: OCEL event ingestion to pure hook action mapping.
Information-loss risk if omitted: Scheduling delta projection and receipt generation after actuation, risking side effect leaks.
TPS visual-control purpose: Preventing side-effect pollution (waste reduction).
DfLSS CTQ protected: 100% pure delta projections.
CENG ticket or boundary constrained: CENG-412 (design-only).
Why this diagram is non-redundant: Focuses on the event matching and delta projection sequence timeline.

```mermaid
gantt
    title Event Ingestion to Actuation Lifecycle
    dateFormat  YYYY-MM-DD
    section Event Ingest
    Ingest OCEL Event           :done, ev1, 2026-07-01, 1d
    Match Registered Hooks      :done, ev2, after ev1, 1d
    section Delta Project
    Project SPARQL Delta        :active, ev3, after ev2, 2d
    section Receipt Sign
    Generate BLAKE3 Receipt     :crit, ev4, after ev3, 1d
    Commit Delta to Oxigraph    :ev5, after ev4, 1d
```

---

### GANTT-L6: Performance / 8-Constraint Hot Path

Diagram ID: GANTT-L6
Diagram family: Gantt
Projection lens: Performance / 8-Constraint Hot Path
Architectural invariant preserved: Sub-microsecond hot-path query execution via binary lowering.
Information-loss risk if omitted: Allocating development time to dynamic hash maps rather than binary lowering, causing project delays.
TPS visual-control purpose: Eliminating processing waste (latency overhead of warm-path fallbacks).
DfLSS CTQ protected: Hot path execution time boundaries.
CENG ticket or boundary constrained: CENG-410-M1.
Why this diagram is non-redundant: Details the development phases of the hot-path optimizer.

```mermaid
gantt
    title Hot Path Performance Gating Development
    dateFormat  YYYY-MM-DD
    section Core Structs
    Binary RDFTriple8 Struct    :done, h1, 2026-06-10, 8d
    ConditionCell BITS Struct   :done, h2, after h1, 7d
    section Optimizer
    256-state Admission Table   :done, h3, after h2, 10d
    Micro-benchmarking Setup    :active, h4, after h3, 5d
    Lowering Compiler Release   :crit, h5, after h4, 6d
```

---

### GANTT-L7: Refusal / Risk / Governance

Diagram ID: GANTT-L7
Diagram family: Gantt
Projection lens: Refusal / Risk / Governance
Architectural invariant preserved: Typed refusal response delivery and audit logging.
Information-loss risk if omitted: Developing error handling systems in isolation without a centralized refusal roadmap, leading to untyped panics.
TPS visual-control purpose: Error containment logging (Poka-Yoke).
DfLSS CTQ protected: Zero undocumented refusals.
CENG ticket or boundary constrained: CENG-410-FINAL.
Why this diagram is non-redundant: Visualizes the timeline for setting up risk management and quarantine subsystems.

```mermaid
gantt
    title Governance & Refusal Implementation Roadmap
    dateFormat  YYYY-MM-DD
    section Refusal Core
    Refusal Taxonomy Definition :done, r1, 2026-06-15, 6d
    Replay Failure Handler      :done, r2, after r1, 8d
    section Quarantine
    N3 Quarantine Gating        :active, r3, after r2, 10d
    section Audit Gates
    CENG Board Governance Audit :crit, r4, after r3, 8d
```

---

### GANTT-L8: TPS / DfLSS / Continuous Improvement

Diagram ID: GANTT-L8
Diagram family: Gantt
Projection lens: TPS / DfLSS / Continuous Improvement
Architectural invariant preserved: Continuous performance improvement loop via metric analysis.
Information-loss risk if omitted: Running benchmark optimization cycles without strict deadlines, causing performance drift.
TPS visual-control purpose: Exposing metrics schema dependencies for optimization feedback loops.
DfLSS CTQ protected: Accurate telemetry tracking schema bounds.
CENG ticket or boundary constrained: CENG-416A-F (design-only).
Why this diagram is non-redundant: Details the schedule of a continuous performance improvement Kaizen sprint.

```mermaid
gantt
    title Performance Tuning Kaizen Sprint
    dateFormat  YYYY-MM-DD
    section Telemetry
    Deploy Base Configurations  :done, k1, 2026-07-01, 2d
    Gather Telemetry Batch      :done, k2, after k1, 3d
    section Analysis
    Analyze Variance Metrics    :active, k3, after k2, 2d
    section Optimization
    Configure 256-state Tables  :k4, after k3, 2d
    Deploy & Verify             :crit, k5, after k4, 3d
```
