# 27. Ishikawa Diagram Family

This file contains the Ishikawa diagram family for the Chatman Engine, structured across the 8 projection lenses.

Fallback rendering for Mermaid compatibility.

---

## Lens 1: Semantic Authority

Diagram ID: ISHIKAWA-L1
Diagram family: Ishikawa
Projection lens: Semantic Authority
Architectural invariant preserved: RDF/Oxigraph is the single semantic source of truth. Shadow copies of RDF data are strictly prohibited.
Information-loss risk if omitted: Inability to determine the root cause of semantic database drift or shadow copies.
TPS visual-control purpose: Identifies root causes of information waste in the semantic layers.
DfLSS CTQ protected: Zero semantic shadow copies.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Visualizes root causes of semantic shadow copies.

```mermaid
flowchart LR
    CentralLine((Backbone)) --> Effect["Effect: Semantic Shadow Copy Defect"]
    
    Methods["Methods (Bypasses)"] --> CentralLine
    M1["Direct Writes to Local Cache"] --> Methods
    M2["Shadow Cache Synchronization Delay"] --> Methods
    
    Machine["Machine (Db Sync)"] --> CentralLine
    Ma1["Oxigraph store read latency"] --> Machine
    Ma2["Unvalidated Graph Commits"] --> Machine
    
    Manpower["Manpower (Witnesses)"] --> CentralLine
    Mp1["LLM writes without authority checks"] --> Manpower
    Mp2["Operator manual SQL updates"] --> Manpower
    
    Measurement["Measurement (Validation)"] --> CentralLine
    Me1["Missing Graph Hash signature verification"] --> Measurement
```

---

## Lens 2: Routing Constitution

Diagram ID: ISHIKAWA-L2
Diagram family: Ishikawa
Projection lens: Routing Constitution
Architectural invariant preserved: Least-expressive-power routing; hot/warm/cold path isolation. N3 is disabled by default.
Information-loss risk if omitted: Failure to pinpoint root causes of route leakage between hot/warm/cold paths.
TPS visual-control purpose: Isolates causes of routing waste and gate bypass.
DfLSS CTQ protected: Safe isolation of cold-path N3 execution.
CENG ticket or boundary constrained: CENG-411 (design-only, implementation blocked).
Why this diagram is non-redundant: Maps root causes of routing errors.

```mermaid
flowchart LR
    CentralLine((Backbone)) --> Effect["Effect: N3 Route Leak / Path Pollution"]
    
    Methods["Methods (Default Paths)"] --> CentralLine
    M1["N3 enabled without profile approval"] --> Methods
    M2["Undefined route fallback strategy"] --> Methods
    
    Machine["Machine (Gates)"] --> CentralLine
    Ma1["Profile gate component failure"] --> Machine
    Ma2["N3 quarantine sandbox bypass"] --> Machine
    
    Manpower["Manpower (Operators)"] --> CentralLine
    Mp1["Developer overrides profile constraints"] --> Manpower
    
    Measurement["Measurement (Profiling)"] --> CentralLine
    Me1["No routing path latency checks"] --> Measurement
```

---

## Lens 3: Type Kernel Ownership

Diagram ID: ISHIKAWA-L3
Diagram family: Ishikawa
Projection lens: Type Kernel Ownership
Architectural invariant preserved: Canonical type ownership across wasm4pm-compat, wasm4pm-cognition, bcinr-pddl, bcinr-powl, and praxis-graphlaw.
Information-loss risk if omitted: Inability to trace root causes of duplicate type definition crossovers.
TPS visual-control purpose: Root cause tracking for type sprawl and serialization errors.
DfLSS CTQ protected: Zero duplicate type classes.
CENG ticket or boundary constrained: CENG-412 (design-only, implementation blocked).
Why this diagram is non-redundant: Details root causes of type definition conflicts.

```mermaid
flowchart LR
    CentralLine((Backbone)) --> Effect["Effect: Duplicate Type Definition Crossover"]
    
    Methods["Methods (Imports)"] --> CentralLine
    M1["Importing incorrect library crates"] --> Methods
    M2["Shadow class creations"] --> Methods
    
    Machine["Machine (Crates)"] --> CentralLine
    Ma1["Lacking compile-time dependency gate"] --> Machine
    Ma2["Blocked (CENG-412): choice compiler mismatch"] --> Machine
    
    Manpower["Manpower (Developers)"] --> CentralLine
    Mp1["Copy-pasting structs across modules"] --> Manpower
    
    Measurement["Measurement (Lints)"] --> CentralLine
    Me1["Missing CI namespace validation checks"] --> Measurement
```

---

## Lens 4: Transition Lifecycle

Diagram ID: ISHIKAWA-L4
Diagram family: Ishikawa
Projection lens: Transition Lifecycle
Architectural invariant preserved: Every transition must pass through candidate invocation, validation, planning, execution, receipting, and replay.
Information-loss risk if omitted: Failure to identify why replay state drift happens.
TPS visual-control purpose: Isolates causes of queue delays and validation bypass.
DfLSS CTQ protected: Replayable state transitions under fixed seed.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Root cause analysis of state transition replay failure.

```mermaid
flowchart LR
    CentralLine((Backbone)) --> Effect["Effect: Transition Replay Failure"]
    
    Methods["Methods (Execution)"] --> CentralLine
    M1["Nondeterministic algorithm execution"] --> Methods
    M2["Skip verification under high load"] --> Methods
    
    Machine["Machine (Time/Seed)"] --> CentralLine
    Ma1["System clock time leak in hash calculations"] --> Machine
    Ma2["System Random seed initialization used"] --> Machine
    
    Manpower["Manpower (Manual state)"] --> CentralLine
    Mp1["Manual database edit bypassing logs"] --> Manpower
    
    Measurement["Measurement (Audits)"] --> CentralLine
    Me1["Missing block replay test validation checks"] --> Measurement
```

---

## Lens 5: Event / Hook / Actuation

Diagram ID: ISHIKAWA-L5
Diagram family: Ishikawa
Projection lens: Event / Hook / Actuation
Architectural invariant preserved: Hooks cannot actuate without receipts; no unreceipted actuation.
Information-loss risk if omitted: Uncontrolled actuation without trace tracking root causes.
TPS visual-control purpose: Locates reasons for side-effect failures.
DfLSS CTQ protected: Zero unreceipted actuation events.
CENG ticket or boundary constrained: CENG-416A-F (design-only, implementation blocked).
Why this diagram is non-redundant: Diagnoses unreceipted boundary execution root causes.

```mermaid
flowchart LR
    CentralLine((Backbone)) --> Effect["Effect: Unreceipted Actuation Leaked"]
    
    Methods["Methods (Boundary Call)"] --> CentralLine
    M1["Calling external API bypassing receipt gate"] --> Methods
    M2["Missing BLAKE3 signature check"] --> Methods
    
    Machine["Machine (Matcher)"] --> CentralLine
    Ma1["Knowledge hook matcher memory leaks"] --> Machine
    Ma2["Actuator thread runner panic"] --> Machine
    
    Manpower["Manpower (Witnesses)"] --> CentralLine
    Mp1["Agent direct API activation"] --> Manpower
    
    Measurement["Measurement (Telemetry)"] --> CentralLine
    Me1["No audit event log mismatch trackers"] --> Measurement
```

---

## Lens 6: Performance / 8-Constraint Hot Path

Diagram ID: ISHIKAWA-L6
Diagram family: Ishikawa
Projection lens: Performance / 8-Constraint Hot Path
Architectural invariant preserved: RDFTriple8, ConditionCell<BITS> byte masks, and 256-state tables.
Information-loss risk if omitted: Slowness in transaction checks due to unoptimized constraint bottlenecks.
TPS visual-control purpose: Resolves causes of latency waste in hot-path checking.
DfLSS CTQ protected: Latency bound of hot path operations.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Diagnoses latency overflows on the hot-path checker.

```mermaid
flowchart LR
    CentralLine((Backbone)) --> Effect["Effect: Hot Path Latency Spike"]
    
    Methods["Methods (Constraints)"] --> CentralLine
    M1["More than 8 constraints evaluated"] --> Methods
    M2["Fallback to warm-path SHACL engine"] --> Methods
    
    Machine["Machine (Registers)"] --> CentralLine
    Ma1["ConditionCell memory lookup delay"] --> Machine
    Ma2["Vector-to-mask lowering execution time"] --> Machine
    
    Manpower["Manpower (Developers)"] --> CentralLine
    Mp1["Developer added complex rules to hot path"] --> Manpower
    
    Measurement["Measurement (Benchmarking)"] --> CentralLine
    Me1["Missing continuous latency checks"] --> Measurement
```

---

## Lens 7: Refusal / Risk / Governance

Diagram ID: ISHIKAWA-L7
Diagram family: Ishikawa
Projection lens: Refusal / Risk / Governance
Architectural invariant preserved: Typed Refusal hierarchy; N3 quarantine rules.
Information-loss risk if omitted: Silent failures or crashes from unhandled panic root causes.
TPS visual-control purpose: Diagnoses causes of exceptions escaping visual alarms.
DfLSS CTQ protected: Zero untyped exceptions or panics.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Visualizes root causes of untyped panic statements.

```mermaid
flowchart LR
    CentralLine((Backbone)) --> Effect["Effect: Unhandled System Panic"]
    
    Methods["Methods (Refusals)"] --> CentralLine
    M1["Generic error type thrown instead of Refusal"] --> Methods
    M2["Silent error swallow (unwrap/expect)"] --> Methods
    
    Machine["Machine (Quarantine)"] --> CentralLine
    Ma1["N3 quarantine sandbox memory leaks"] --> Machine
    Ma2["CENG board exception handler crashes"] --> Machine
    
    Manpower["Manpower (Developers)"] --> CentralLine
    Mp1["Developer used unwrap outside assertions"] --> Manpower
    
    Measurement["Measurement (Lints)"] --> CentralLine
    Me1["No unit tests checking error path refusals"] --> Measurement
```

---

## Lens 8: TPS / DfLSS / Continuous Improvement

Diagram ID: ISHIKAWA-L8
Diagram family: Ishikawa
Projection lens: TPS / DfLSS / Continuous Improvement
Architectural invariant preserved: Continuous Kaizen optimization loops, visual gauges, waste reduction.
Information-loss risk if omitted: Chronic process bottlenecks causing continuous quality targets to fail.
TPS visual-control purpose: Resolves root causes of process inefficiencies.
DfLSS CTQ protected: Throughput and defect-free execution rate.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Diagnoses root causes of Kanban WIP limit violations.

```mermaid
flowchart LR
    CentralLine((Backbone)) --> Effect["Effect: Kanban WIP Limit Violation"]
    
    Methods["Methods (Kaizen loops)"] --> CentralLine
    M1["Unmeasured hand-off latency delays"] --> Methods
    M2["Lack of automated process limit warnings"] --> Methods
    
    Machine["Machine (Telemetry)"] --> CentralLine
    Ma1["WIP tracking system crashes"] --> Machine
    Ma2["Performance data storage overflow"] --> Machine
    
    Manpower["Manpower (Teams)"] --> CentralLine
    Mp1["Engineers ignore Kanban limits"] --> Manpower
    
    Measurement["Measurement (CTQs)"] --> CentralLine
    Me1["Missing continuous improvement loops"] --> Measurement
```
