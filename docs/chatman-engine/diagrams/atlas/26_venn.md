# 26. Venn Diagram Family

This file contains the Venn diagram family for the Chatman Engine, structured across the 8 projection lenses.

Fallback rendering for Mermaid compatibility.

---

## Lens 1: Semantic Authority

Diagram ID: VENN-L1
Diagram family: Venn
Projection lens: Semantic Authority
Architectural invariant preserved: RDF/Oxigraph is the single semantic source of truth. Shadow copies of RDF data are strictly prohibited.
Information-loss risk if omitted: Overlapping data models from different system layers resulting in duplicate conflicting databases.
TPS visual-control purpose: Eliminates information duplicate waste at system intersections.
DfLSS CTQ protected: Zero semantic shadow copies.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Visualizes semantic overlaps and boundaries across database modules.

```mermaid
flowchart TD
    subgraph Oxigraph ["Oxigraph RDF Authority"]
        S1["Source of Truth triples"]
    end
    subgraph WarmPath ["Warm Path Projections"]
        S2["SHACL Graph States"]
    end
    subgraph Witness ["Witness Agents"]
        S3["LLM Context Objects"]
    end
    S1 --- Inter1["(Intersection: Read-Only SPARQL API)"]
    S2 --- Inter1
    S3 --- Inter1
    Inter1 -->|Never write directly| S1
```

---

## Lens 2: Routing Constitution

Diagram ID: VENN-L2
Diagram family: Venn
Projection lens: Routing Constitution
Architectural invariant preserved: Least-expressive-power routing; hot/warm/cold path isolation. N3 is disabled by default.
Information-loss risk if omitted: Route overlap letting slow-path rules pollute fast-path byte masks.
TPS visual-control purpose: Ensures routing rules do not overlap in invalid path zones.
DfLSS CTQ protected: Safe isolation of cold-path N3 execution.
CENG ticket or boundary constrained: CENG-411 (design-only, implementation blocked).
Why this diagram is non-redundant: Details path execution boundaries.

```mermaid
flowchart TD
    subgraph Hot ["Hot Path (Byte Mask)"]
        A["ConditionCell<BITS> Checks"]
    end
    subgraph Warm ["Warm Path (SHACL/PDDL)"]
        B["Graph Rule Validations"]
    end
    subgraph Cold ["Cold Path (N3 Rules)"]
        C["Quarantined Engines (CENG-411)"]
    end
    A --- Gate1["(Profile Gate Check)"]
    B --- Gate1
    C --- Gate1
    Gate1 -->|N3 Segregated| Cold
```

---

## Lens 3: Type Kernel Ownership

Diagram ID: VENN-L3
Diagram family: Venn
Projection lens: Type Kernel Ownership
Architectural invariant preserved: Canonical type ownership across wasm4pm-compat, wasm4pm-cognition, bcinr-pddl, bcinr-powl, and praxis-graphlaw.
Information-loss risk if omitted: Overlapping type namespaces causing binary serialization conflicts.
TPS visual-control purpose: Identifies shared type boundaries to eliminate duplicate coding.
DfLSS CTQ protected: Zero duplicate type classes.
CENG ticket or boundary constrained: CENG-412 (design-only, implementation blocked).
Why this diagram is non-redundant: Visualizes overlapping type domain responsibilities.

```mermaid
flowchart TD
    subgraph Compat ["wasm4pm-compat"]
        A["WASM System Structs"]
    end
    subgraph Cognition ["wasm4pm-cognition"]
        B["Breed Structs"]
    end
    subgraph Planning ["bcinr-pddl / bcinr-powl (CENG-412)"]
        C["PDDL Actions / Choice compiler"]
    end
    A --- Law["(praxis-graphlaw Intersection)"]
    B --- Law
    C --- Law
```

---

## Lens 4: Transition Lifecycle

Diagram ID: VENN-L4
Diagram family: Venn
Projection lens: Transition Lifecycle
Architectural invariant preserved: Every transition must pass through candidate invocation, validation, planning, execution, receipting, and replay.
Information-loss risk if omitted: Executing state changes without complete validation overlays.
TPS visual-control purpose: Ensures audit overlap covering all lifecycle phases.
DfLSS CTQ protected: Replayable state transitions under fixed seed.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Focuses on lifecycle phase intersections.

```mermaid
flowchart TD
    subgraph Intake ["Intake Validation"]
        A["Candidate Audits"]
    end
    subgraph Plan ["Planning Verification"]
        B["PDDL Action Rules"]
    end
    subgraph Actuate ["Boundary Execution"]
        C["Knowledge Hook Actuators"]
    end
    A --- Receipt["(BLAKE3 Cryptographic Receipt Core)"]
    B --- Receipt
    C --- Receipt
```

---

## Lens 5: Event / Hook / Actuation

Diagram ID: VENN-L5
Diagram family: Venn
Projection lens: Event / Hook / Actuation
Architectural invariant preserved: Hooks cannot actuate without receipts; no unreceipted actuation.
Information-loss risk if omitted: Mismatch in Hook execution overlaps leading to unverified actuation actions.
TPS visual-control purpose: Eliminates unreceipted action leaks.
DfLSS CTQ protected: Zero unreceipted actuation events.
CENG ticket or boundary constrained: CENG-416A-F (design-only, implementation blocked).
Why this diagram is non-redundant: Highlights the receipt intersection in hook-based systems.

```mermaid
flowchart TD
    subgraph OCEL ["OCEL Event Streams"]
        A["Event Log Ingestion"]
    end
    subgraph Matcher ["Knowledge Hook Matchers"]
        B["Knowledge Rule Matches"]
    end
    subgraph Boundary ["Boundary Actuators (CENG-416)"]
        C["External Process Activator"]
    end
    A --- Core["(Receipted Actuation Intersection)"]
    B --- Core
    C --- Core
```

---

## Lens 6: Performance / 8-Constraint Hot Path

Diagram ID: VENN-L6
Diagram family: Venn
Projection lens: Performance / 8-Constraint Hot Path
Architectural invariant preserved: RDFTriple8, ConditionCell<BITS> byte masks, and 256-state tables.
Information-loss risk if omitted: Inability to execute parallel checks on constraint boundaries.
TPS visual-control purpose: Ensures hot-path criteria matches execution bounds.
DfLSS CTQ protected: Latency bound of hot path operations.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Focuses on constraint optimization loops.

```mermaid
flowchart TD
    subgraph Projection ["RDFTriple8 Projections"]
        A["RDF Local Projection"]
    end
    subgraph ByteMask ["ConditionCell Byte Mask"]
        B["Register State Allocations"]
    end
    subgraph Admission ["256-State Table Checks"]
        C["Table Lookups"]
    end
    A --- Admit["(Hot Path Admission Intersection)"]
    B --- Admit
    C --- Admit
```

---

## Lens 7: Refusal / Risk / Governance

Diagram ID: VENN-L7
Diagram family: Venn
Projection lens: Refusal / Risk / Governance
Architectural invariant preserved: Typed Refusal hierarchy; N3 quarantine rules.
Information-loss risk if omitted: Silent failures bypassing governance and board verification overlays.
TPS visual-control purpose: Clearly flags quarantine intersections to protect core safety.
DfLSS CTQ protected: Zero untyped exceptions or panics.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Visualizes governance and exception handling boundary overlaps.

```mermaid
flowchart TD
    subgraph Refusals ["Typed Refusal Exceptions"]
        A["Hierarchy of Refusals"]
    end
    subgraph Quarantine ["N3 Quarantine Rules"]
        B["Untrusted Sandboxing"]
    end
    subgraph Board ["CENG Board Rules"]
        C["Board Compliance Exception policies"]
    end
    A --- Secure["(Audit-Cleared Secure Area)"]
    B --- Secure
    C --- Secure
```

---

## Lens 8: TPS / DfLSS / Continuous Improvement

Diagram ID: VENN-L8
Diagram family: Venn
Projection lens: TPS / DfLSS / Continuous Improvement
Architectural invariant preserved: Continuous Kaizen optimization loops, visual gauges, waste reduction.
Information-loss risk if omitted: Isolation of Lean quality targets from runtime feedback loops.
TPS visual-control purpose: Visual control of continuous quality intersection boundaries.
DfLSS CTQ protected: Throughput and defect-free execution rate.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Shows intersections of Six Sigma domains.

```mermaid
flowchart TD
    subgraph TPS ["TPS Flow Constraints"]
        A["WIP Controls"]
    end
    subgraph DfLSS ["DfLSS CTQ Targets"]
        B["Zero-Shadow copy validation"]
    end
    subgraph Kaizen ["Kaizen Performance Loops"]
        C["Continuous Benchmark telemetry"]
    end
    A --- Target["(Zero-Defect Operations Intersection)"]
    B --- Target
    C --- Target
```
