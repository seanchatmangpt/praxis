# 17. Sankey Diagram Family

This file contains the Sankey diagram family for the Chatman Engine, structured across the 8 projection lenses.

Fallback rendering for Mermaid compatibility.

---

## Lens 1: Semantic Authority

Diagram ID: SANKEY-L1
Diagram family: Sankey
Projection lens: Semantic Authority
Architectural invariant preserved: RDF/Oxigraph is the single semantic source of truth. Shadow copies of RDF data are strictly prohibited.
Information-loss risk if omitted: Lack of visibility into the flow and volume of RDF semantic updates, leading to untracked shadow data structures.
TPS visual-control purpose: Maps flow volumes of RDF data to expose processing waste.
DfLSS CTQ protected: Zero semantic shadow copies.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Outlines RDF graph write flow rates and authorization volumes.

```mermaid
flowchart LR
    IncomingRDF["Incoming RDF Updates [100%]"]
    VerifyGate["Verification Gate [100%]"]
    AuthCheck["Auth Check [100%]"]
    OxigraphStore["Oxigraph Store [90%]"]
    Refusals["Typed Refusals [10%]"]

    IncomingRDF -- "100%" --> VerifyGate
    VerifyGate -- "100%" --> AuthCheck
    AuthCheck -- "90% (Pass)" --> OxigraphStore
    AuthCheck -- "10% (Deny)" --> Refusals
```

---

## Lens 2: Routing Constitution

Diagram ID: SANKEY-L2
Diagram family: Sankey
Projection lens: Routing Constitution
Architectural invariant preserved: Least-expressive-power routing; hot/warm/cold path isolation. N3 is disabled by default.
Information-loss risk if omitted: Incorrect allocation of query workloads across execution paths.
TPS visual-control purpose: Identifies load imbalances across hot, warm, and cold paths.
DfLSS CTQ protected: Safe isolation of cold-path N3 execution.
CENG ticket or boundary constrained: CENG-411 (design-only, implementation blocked).
Why this diagram is non-redundant: Tracks flow distribution across the routing paths.

```mermaid
flowchart LR
    TotalQueries["Total Queries [100%]"]
    Router["Route Classifier [100%]"]
    HotPath["Hot Path (<= 8 constraints) [70%]"]
    WarmPath["Warm Path (> 8 constraints) [25%]"]
    ColdPath["Cold Path (N3 / Quarantine) [5%]"]

    TotalQueries -- "100%" --> Router
    Router -- "70% (Hot)" --> HotPath
    Router -- "25% (Warm)" --> WarmPath
    Router -- "5% (Cold)" --> ColdPath
```

---

## Lens 3: Type Kernel Ownership

Diagram ID: SANKEY-L3
Diagram family: Sankey
Projection lens: Type Kernel Ownership
Architectural invariant preserved: Canonical type ownership across wasm4pm-compat, wasm4pm-cognition, bcinr-pddl, bcinr-powl, and praxis-graphlaw.
Information-loss risk if omitted: Untracked type definitions leaking into external packages.
TPS visual-control purpose: Maps compilation dependencies to detect redundant type libraries.
DfLSS CTQ protected: Zero duplicate type classes.
CENG ticket or boundary constrained: CENG-412 (design-only, implementation blocked).
Why this diagram is non-redundant: Outlines type distribution from the canonical kernel to system dependencies.

```mermaid
flowchart LR
    Kernel["wasm4pm-compat (Canonical Type Kernel) [100%]"]
    Cognition["wasm4pm-cognition [30%]"]
    PDDL_POWL["bcinr-pddl / bcinr-powl [40%]"]
    Graphlaw["praxis-graphlaw [30%]"]

    Kernel -- "30% (Breed Types)" --> Cognition
    Kernel -- "40% (Planning Types)" --> PDDL_POWL
    Kernel -- "30% (Mapping Types)" --> Graphlaw
```

---

## Lens 4: Transition Lifecycle

Diagram ID: SANKEY-L4
Diagram family: Sankey
Projection lens: Transition Lifecycle
Architectural invariant preserved: Every transition must pass through candidate invocation, validation, planning, execution, receipting, and replay.
Information-loss risk if omitted: Drop-offs and loss of transactions between lifecycle phases.
TPS visual-control purpose: Identifies pipeline blockages and scrap rates across transition steps.
DfLSS CTQ protected: Guaranteed transaction replay validation under fixed seed.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Models transition throughput and drop-off flow rates.

```mermaid
flowchart LR
    Invocation["Invocation Candidate [100%]"]
    Validation["Validation (OWL-RL/SHACL) [95%]"]
    Execution["Execution (Hook Action) [90%]"]
    Receipting["Receipting (BLAKE3) [90%]"]
    Replay["Replay & Commit [90%]"]
    Refusals["Typed Refusals [10%]"]

    Invocation -- "95% (Valid)" --> Validation
    Invocation -- "5% (Invalid)" --> Refusals
    Validation -- "90% (Pass)" --> Execution
    Validation -- "5% (Refuse)" --> Refusals
    Execution -- "100%" --> Receipting
    Receipting -- "100%" --> Replay
```

---

## Lens 5: Event / Hook / Actuation

Diagram ID: SANKEY-L5
Diagram family: Sankey
Projection lens: Event / Hook / Actuation
Architectural invariant preserved: Hooks cannot actuate without receipts; no unreceipted actuation.
Information-loss risk if omitted: Event processing leak where hooks execute without audit trails.
TPS visual-control purpose: Tracks event processing yield and gating efficiency.
DfLSS CTQ protected: Zero unreceipted execution events.
CENG ticket or boundary constrained: CENG-416A-F (design-only, implementation blocked).
Why this diagram is non-redundant: Traces event-to-actuation processing flow.

```mermaid
flowchart LR
    OCELStream["OCEL Event Stream [100%]"]
    Matcher["Hook Matcher [100%]"]
    MatchedHooks["Matched Hooks [80%]"]
    ReceiptGate["Receipt Verification Gate [80%]"]
    Actuator["Boundary Actuator (Blocked) [80%]"]
    Unmatched["Unmatched Events [20%]"]

    OCELStream -- "100%" --> Matcher
    Matcher -- "80% (Match)" --> MatchedHooks
    Matcher -- "20% (Ignore)" --> Unmatched
    MatchedHooks -- "100%" --> ReceiptGate
    ReceiptGate -- "100% (Signed)" --> Actuator
```

---

## Lens 6: Performance / 8-Constraint Hot Path

Diagram ID: SANKEY-L6
Diagram family: Sankey
Projection lens: Performance / 8-Constraint Hot Path
Architectural invariant preserved: Maximum of 8 constraints checked in parallel via RDFTriple8 and ConditionCell<BITS>.
Information-loss risk if omitted: Overhead and CPU cycles wasted on slow warm-path routing.
TPS visual-control purpose: Andon check of hot-path query execution efficiency.
DfLSS CTQ protected: Latency bound of hot path operations.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: visualizes query flow throughput through the hot-path optimizer.

```mermaid
flowchart LR
    HotQueries["Hot Queries [100%]"]
    VectorComp["Vector Compiler [100%]"]
    CellGate["ConditionCell Gate [100%]"]
    AdmissionTable["Admission Table [95%]"]
    WarmFallback["Warm Path Fallback [5%]"]

    HotQueries -- "100%" --> VectorComp
    VectorComp -- "100%" --> CellGate
    CellGate -- "95% (Pass)" --> AdmissionTable
    CellGate -- "5% (Overflow)" --> WarmFallback
```

---

## Lens 7: Refusal / Risk / Governance

Diagram ID: SANKEY-L7
Diagram family: Sankey
Projection lens: Refusal / Risk / Governance
Architectural invariant preserved: Every failure is a typed Refusal; N3 quarantine rules are strictly enforced.
Information-loss risk if omitted: Untyped errors or quarantined code escaping security containment.
TPS visual-control purpose: Tracks scrap flow and routing of risk events.
DfLSS CTQ protected: No panic or silent fallbacks.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Outlines error routing and quarantine volumes.

```mermaid
flowchart LR
    Errors["Total System Errors [100%]"]
    Typifier["Refusal Typifier [100%]"]
    StdRefusal["Standard Refusal Response [90%]"]
    Quarantine["N3 Quarantine Gate [10%]"]
    BoardReview["CENG Governance Board [10%]"]

    Errors -- "100%" --> Typifier
    Typifier -- "90%" --> StdRefusal
    Typifier -- "10%" --> Quarantine
    Quarantine -- "100%" --> BoardReview
```

---

## Lens 8: TPS / DfLSS / Continuous Improvement

Diagram ID: SANKEY-L8
Diagram family: Sankey
Projection lens: TPS / DfLSS / Continuous Improvement
Architectural invariant preserved: WIP reduction, continuous process improvement loops, and visual waste elimination.
Information-loss risk if omitted: Hidden build inventory (WIP) and delay accumulation in the deployment pipeline.
TPS visual-control purpose: visualizes Kaizen improvement resource allocation.
DfLSS CTQ protected: Flow efficiency and defect rate minimization.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Details resource and task flows through the Kaizen continuous improvement loop.

```mermaid
flowchart LR
    KaizenFocus["Kaizen Optimization Focus [100%]"]
    WIPReduction["WIP Reduction [40%]"]
    DefectPrevention["Defect Prevention [30%]"]
    LatencyOpt["Latency Optimization [30%]"]
    ProcessFlow["Improved Core Process [100%]"]

    KaizenFocus -- "40%" --> WIPReduction
    KaizenFocus -- "30%" --> DefectPrevention
    KaizenFocus -- "30%" --> LatencyOpt
    WIPReduction -- "100%" --> ProcessFlow
    DefectPrevention -- "100%" --> ProcessFlow
    LatencyOpt -- "100%" --> ProcessFlow
```
