# 25. Treemap Diagram Family

This file contains the Treemap diagram family for the Chatman Engine, structured across the 8 projection lenses.

Fallback rendering for Mermaid compatibility.

---

## Lens 1: Semantic Authority

Diagram ID: TREEMAP-L1
Diagram family: Treemap
Projection lens: Semantic Authority
Architectural invariant preserved: RDF/Oxigraph is the single semantic source of truth. Shadow copies of RDF data are strictly prohibited.
Information-loss risk if omitted: Ambiguity about the hierarchy of semantic data structures inside Oxigraph.
TPS visual-control purpose: Eliminates waste by structuring semantic scopes.
DfLSS CTQ protected: Zero semantic shadow copies.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Details RDF data hierarchies inside the database authority domain.

```mermaid
flowchart TD
    subgraph OxigraphStore ["Oxigraph RDF Store"]
        subgraph GraphAlpha ["Graph Authority Zone 1"]
            subgraph TriplesA ["Triples Domain"]
                A1["Subject: wf:transition"]
                A2["Predicate: wf:hasReceipt"]
                A3["Object: BLAKE3 Hash"]
            end
        end
        subgraph GraphBeta ["Graph Authority Zone 2"]
            subgraph TriplesB ["Triples Domain"]
                B1["Subject: hook:boundary"]
                B2["Predicate: hook:actuator"]
            end
        end
    end
```

---

## Lens 2: Routing Constitution

Diagram ID: TREEMAP-L2
Diagram family: Treemap
Projection lens: Routing Constitution
Architectural invariant preserved: Least-expressive-power routing; hot/warm/cold path isolation. N3 is disabled by default.
Information-loss risk if omitted: Incorrect nesting of path constraints within routing boundaries.
TPS visual-control purpose: Isolates the warm, hot, and cold paths visually to detect complexity drift.
DfLSS CTQ protected: Safe isolation of cold-path N3 execution.
CENG ticket or boundary constrained: CENG-411 (design-only, implementation blocked).
Why this diagram is non-redundant: Visualizes nested routing paths and quarantine regions.

```mermaid
flowchart TD
    subgraph RoutingEngine ["Routing Constitution"]
        subgraph HotPath ["Hot Path (Least Expressive)"]
            H1["ConditionCell Byte Mask"]
            H2["RDFTriple8 Local Projections"]
        end
        subgraph WarmPath ["Warm Path (Medium Expressive)"]
            W1["SHACL Engine"]
            W2["SPARQL Queries"]
            W3["PDDL / POWL Plans"]
        end
        subgraph ColdPath ["Cold Path (N3 Quarantine - Blocked CENG-411)"]
            C1["Quarantined N3 Engines"]
        end
    end
```

---

## Lens 3: Type Kernel Ownership

Diagram ID: TREEMAP-L3
Diagram family: Treemap
Projection lens: Type Kernel Ownership
Architectural invariant preserved: Canonical type ownership across wasm4pm-compat, wasm4pm-cognition, bcinr-pddl, bcinr-powl, and praxis-graphlaw.
Information-loss risk if omitted: Duplicate types created inside wrong crates.
TPS visual-control purpose: Groups types by crate boundaries to prevent redundant declarations.
DfLSS CTQ protected: Zero duplicate type classes.
CENG ticket or boundary constrained: CENG-412 (design-only, implementation blocked).
Why this diagram is non-redundant: Details type module namespaces and boundaries.

```mermaid
flowchart TD
    subgraph SystemTypes ["System Canonical Types"]
        subgraph CompatCrate ["wasm4pm-compat"]
            T1["Core WASM Types"]
            T2["WASM Buffer Envelopes"]
        end
        subgraph CognitionCrate ["wasm4pm-cognition"]
            T3["Cognitive Breed Types"]
        end
        subgraph BCINRCrates ["bcinr-pddl / bcinr-powl (CENG-412)"]
            T4["PDDL Solver Domains"]
            T5["POWL Workflows"]
        end
        subgraph GraphlawCrate ["praxis-graphlaw"]
            T6["RDF triple mapping types"]
        end
    end
```

---

## Lens 4: Transition Lifecycle

Diagram ID: TREEMAP-L4
Diagram family: Treemap
Projection lens: Transition Lifecycle
Architectural invariant preserved: Every transition must pass through candidate invocation, validation, planning, execution, receipting, and replay.
Information-loss risk if omitted: Skipping lifecycle phases or mixing execution states.
TPS visual-control purpose: Visually tracks checkpoint allocation to maintain flow efficiency.
DfLSS CTQ protected: Replayable state transitions under fixed seed.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Details nesting of lifecycle checkpoints.

```mermaid
flowchart TD
    subgraph LifecycleRunner ["Transition Lifecycle"]
        subgraph VerificationPhase ["Verification Phase"]
            P1["Candidate Invocation Auditing"]
            P2["SHACL Graph Rules Validation"]
        end
        subgraph ExecutionPhase ["Execution Phase"]
            P3["bcinr Planning Validation"]
            P4["Boundary Hook execution"]
            P5["BLAKE3 Receipt generation"]
        end
        subgraph StoragePhase ["Storage Phase"]
            P6["Replay Store validation"]
        end
    end
```

---

## Lens 5: Event / Hook / Actuation

Diagram ID: TREEMAP-L5
Diagram family: Treemap
Projection lens: Event / Hook / Actuation
Architectural invariant preserved: Hooks cannot actuate without receipts; no unreceipted actuation.
Information-loss risk if omitted: Shadow actuation processes escaping event loops.
TPS visual-control purpose: Eliminates unreceipted actuation waste using strict boundary nested scopes.
DfLSS CTQ protected: Zero unreceipted actuation events.
CENG ticket or boundary constrained: CENG-416A-F (design-only, implementation blocked).
Why this diagram is non-redundant: Details Hook execution hierarchies.

```mermaid
flowchart TD
    subgraph IngestionActuation ["Engine Event Model"]
        subgraph Ingestion ["Event Ingestion"]
            I1["OCEL Event Stream Reader"]
            I2["Raw Log parser"]
        end
        subgraph Matching ["Hook Processing"]
            M1["Knowledge Hook Matcher"]
            M2["Profile Gate filter"]
        end
        subgraph Actuation ["Boundary Actuation (CENG-416)"]
            A1["BLAKE3 Receipt Verification"]
            A2["External Actuator Executor"]
        end
    end
```

---

## Lens 6: Performance / 8-Constraint Hot Path

Diagram ID: TREEMAP-L6
Diagram family: Treemap
Projection lens: Performance / 8-Constraint Hot Path
Architectural invariant preserved: RDFTriple8, ConditionCell<BITS> byte masks, and 256-state tables.
Information-loss risk if omitted: Overflow of hot-path constraint bounds.
TPS visual-control purpose: Visual control of constraint limits.
DfLSS CTQ protected: Latency bound of hot path operations.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Maps performance constraint domains and mask boundaries.

```mermaid
flowchart TD
    subgraph HotPathEngine ["Hot Path Engine"]
        subgraph ConstraintSet ["Constraint Set (Max 8)"]
            C1["RDFTriple8 Projections"]
        end
        subgraph ByteMaskRegisters ["Byte Mask Registers"]
            R1["ConditionCell<BITS> Memory Allocations"]
        end
        subgraph AdmissionTables ["Admission Tables"]
            A1["256-State Admission Table"]
        end
    end
```

---

## Lens 7: Refusal / Risk / Governance

Diagram ID: TREEMAP-L7
Diagram family: Treemap
Projection lens: Refusal / Risk / Governance
Architectural invariant preserved: Typed Refusal hierarchy; N3 quarantine rules.
Information-loss risk if omitted: Lack of containment classification for exceptions.
TPS visual-control purpose: Isolates exceptions by category to prevent panic propagation.
DfLSS CTQ protected: Zero untyped exceptions or panics.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Details risk and refusal containment hierarchies.

```mermaid
flowchart TD
    subgraph ExceptionBoundary ["Exception Boundary"]
        subgraph RefusalTypes ["Typed Refusal System"]
            R1["InvalidSignature Refusal"]
            R2["WorkflowRefusal Exception"]
        end
        subgraph QuarantineZone ["Quarantine Zone"]
            Q1["Untrusted N3 Sandbox"]
        end
        subgraph GovernanceZone ["Governance Audits"]
            G1["CENG Board Exception Log"]
        end
    end
```

---

## Lens 8: TPS / DfLSS / Continuous Improvement

Diagram ID: TREEMAP-L8
Diagram family: Treemap
Projection lens: TPS / DfLSS / Continuous Improvement
Architectural invariant preserved: Continuous Kaizen optimization loops, visual gauges, waste reduction.
Information-loss risk if omitted: Blind spots in Lean quality matrices and metrics.
TPS visual-control purpose: Groups Six Sigma categories to manage process improvement.
DfLSS CTQ protected: Throughput and defect-free execution rate.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Details continuous improvement containment classes.

```mermaid
flowchart TD
    subgraph SixSigma ["Lean Six Sigma Metrics"]
        subgraph WasteClasses ["Waste Categories"]
            W1["WIP Limit Overflows"]
            W2["Routing Delays"]
            W3["Shadow Copy Defects"]
        end
        subgraph CTQParameters ["CTQ Parameters"]
            C1["Graph Signature Mismatches"]
            C2["Replay Mismatch Rates"]
        end
        subgraph ImprovementLoops ["Kaizen Loops"]
            I1["Benchmark Feedback loop"]
        end
    end
```
