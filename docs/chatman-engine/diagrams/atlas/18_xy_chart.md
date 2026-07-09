# 18. XY Chart Diagram Family

This file contains the XY Chart diagram family for the Chatman Engine, structured across the 8 projection lenses.

Fallback rendering for Mermaid compatibility.

---

## Lens 1: Semantic Authority

Diagram ID: XY_CHART-L1
Diagram family: XY Chart
Projection lens: Semantic Authority
Architectural invariant preserved: RDF/Oxigraph is the single semantic source of truth. Shadow copies of RDF data are strictly prohibited.
Information-loss risk if omitted: Wasted CPU overhead on shadow copy checks not being tracked against graph scale.
TPS visual-control purpose: Andon alert for non-linear verification time scaling.
DfLSS CTQ protected: Zero semantic shadow copies.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Plots the verification latency against RDF graph size.

```mermaid
flowchart TD
    subgraph Grid ["Graph Scale (Triples) vs Validation Latency (ms)"]
        direction LR
        P1["(X: 100, Y: 1.2ms)"] --> P2["(X: 500, Y: 2.5ms)"]
        P2 --> P3["(X: 1000, Y: 4.8ms)"]
        P3 --> P4["(X: 5000, Y: 15.5ms)"]
    end
```

---

## Lens 2: Routing Constitution

Diagram ID: XY_CHART-L2
Diagram family: XY Chart
Projection lens: Routing Constitution
Architectural invariant preserved: Least-expressive-power routing; hot/warm/cold path isolation. N3 is disabled by default.
Information-loss risk if omitted: Inability to visualize routing overhead scaling with constraints.
TPS visual-control purpose: Prevents processing waste by identifying path selection thresholds.
DfLSS CTQ protected: Safe isolation of cold-path N3 execution.
CENG ticket or boundary constrained: CENG-411 (design-only, implementation blocked).
Why this diagram is non-redundant: Plots query complexity against routing overhead.

```mermaid
flowchart TD
    subgraph Grid ["Query Complexity (Constraints Count) vs Path Latency (ms)"]
        direction LR
        P1["(X: 4 Constraints, Y: 0.05ms - Hot Path)"] --> P2["(X: 8 Constraints, Y: 0.08ms - Hot Path)"]
        P2 --> P3["(X: 12 Constraints, Y: 2.50ms - Warm Path)"]
        P3 --> P4["(X: N3 Rules, Y: 25.0ms - Cold Path)"]
    end
```

---

## Lens 3: Type Kernel Ownership

Diagram ID: XY_CHART-L3
Diagram family: XY Chart
Projection lens: Type Kernel Ownership
Architectural invariant preserved: Canonical type ownership across wasm4pm-compat, wasm4pm-cognition, bcinr-pddl, bcinr-powl, and praxis-graphlaw.
Information-loss risk if omitted: Undetected increases in type registry compile-time overhead.
TPS visual-control purpose: Prevents build time waste by tracking compilation bloat.
DfLSS CTQ protected: Zero duplicate type classes.
CENG ticket or boundary constrained: CENG-412 (design-only, implementation blocked).
Why this diagram is non-redundant: Maps number of type definitions to linkage and compilation time.

```mermaid
flowchart TD
    subgraph Grid ["Type Count vs Linkage Latency (ms)"]
        direction LR
        P1["(X: 10 Types, Y: 10ms)"] --> P2["(X: 50 Types, Y: 15ms)"]
        P2 --> P3["(X: 100 Types, Y: 25ms - CENG-412 Blocked)"]
        P3 -.-> P4["(X: 200 Types, Y: 60ms - Projected)"]
    end
```

---

## Lens 4: Transition Lifecycle

Diagram ID: XY_CHART-L4
Diagram family: XY Chart
Projection lens: Transition Lifecycle
Architectural invariant preserved: Every transition must pass through candidate invocation, validation, planning, execution, receipting, and replay.
Information-loss risk if omitted: Undetected lag spikes at specific lifecycle processing steps.
TPS visual-control purpose: WIP tracking across lifecycle phases to identify timing queues.
DfLSS CTQ protected: Guaranteed transaction replay validation under fixed seed.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Details incremental latency contributions across lifecycle stages.

```mermaid
flowchart TD
    subgraph Grid ["Transition Stage Index vs Cumulative Latency (ms)"]
        direction LR
        P1["(X: 1-Invocation, Y: 0.1ms)"] --> P2["(X: 2-Validation, Y: 1.6ms)"]
        P2 --> P3["(X: 3-Execution, Y: 2.6ms)"]
        P3 --> P4["(X: 4-Receipting, Y: 3.1ms)"]
        P4 --> P5["(X: 5-Replay, Y: 3.6ms)"]
    end
```

---

## Lens 5: Event / Hook / Actuation

Diagram ID: XY_CHART-L5
Diagram family: XY Chart
Projection lens: Event / Hook / Actuation
Architectural invariant preserved: Hooks cannot actuate without receipts; no unreceipted actuation.
Information-loss risk if omitted: Loss of control over hook matcher queue sizes under peak event loads.
TPS visual-control purpose: Poka-Yoke check on hook processing capacity.
DfLSS CTQ protected: Zero unreceipted execution events.
CENG ticket or boundary constrained: CENG-416A-F (design-only, implementation blocked).
Why this diagram is non-redundant: Plots incoming OCEL event rates against hook matching queue sizes.

```mermaid
flowchart TD
    subgraph Grid ["OCEL Ingestion Rate (events/sec) vs Hook Queue Size"]
        direction LR
        P1["(X: 100 events/sec, Y: 2 items)"] --> P2["(X: 500 events/sec, Y: 8 items)"]
        P2 --> P3["(X: 1000 events/sec, Y: 25 items)"]
        P3 --> P4["(X: 2000 events/sec, Y: Blocked Boundary - CENG-416)"]
    end
```

---

## Lens 6: Performance / 8-Constraint Hot Path

Diagram ID: XY_CHART-L6
Diagram family: XY Chart
Projection lens: Performance / 8-Constraint Hot Path
Architectural invariant preserved: Maximum of 8 constraints checked in parallel via RDFTriple8 and ConditionCell<BITS>.
Information-loss risk if omitted: CPU utilization surges undetected when constraint size boundary is crossed.
TPS visual-control purpose: Andon indicator showing constraint limit threshold compliance.
DfLSS CTQ protected: Latency bound of hot path operations.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Formally plots constraints size against execution latency, highlighting the 8-constraint hot-path boundary.

```mermaid
flowchart TD
    subgraph Grid ["Constraint Count vs CPU Execution Latency (microseconds)"]
        direction LR
        P1["(X: 1 constraint, Y: 2us)"] --> P2["(X: 4 constraints, Y: 3us)"]
        P2 --> P3["(X: 8 constraints, Y: 5us - Hot Path limit)"]
        P3 -- "Warm Path Jump!" --> P4["(X: 9 constraints, Y: 2500us - Warm Path)"]
    end
```

---

## Lens 7: Refusal / Risk / Governance

Diagram ID: XY_CHART-L7
Diagram family: XY Chart
Projection lens: Refusal / Risk / Governance
Architectural invariant preserved: Every failure is a typed Refusal; N3 quarantine rules are strictly enforced.
Information-loss risk if omitted: System vulnerability exposure if refusal rates scale with risk without triggering quarantine.
TPS visual-control purpose: Exposes waste (scrap rates) in refusal generation.
DfLSS CTQ protected: No panic or silent fallbacks.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Plots threat/risk levels against refusal and quarantine actions.

```mermaid
flowchart TD
    subgraph Grid ["Risk Vector Score vs Quarantined Rules count"]
        direction LR
        P1["(X: Score 10, Y: 0 quarantined)"] --> P2["(X: Score 40, Y: 2 quarantined)"]
        P2 --> P3["(X: Score 70, Y: 15 quarantined)"]
        P3 --> P4["(X: Score 90, Y: 45 quarantined - CENG Review required)"]
    end
```

---

## Lens 8: TPS / DfLSS / Continuous Improvement

Diagram ID: XY_CHART-L8
Diagram family: XY Chart
Projection lens: TPS / DfLSS / Continuous Improvement
Architectural invariant preserved: WIP reduction, continuous process improvement loops, and visual waste elimination.
Information-loss risk if omitted: Inability to track Kaizen improvements against overall engine throughput (TPS).
TPS visual-control purpose: Kaizen chart showing process improvement trends.
DfLSS CTQ protected: Flow efficiency and defect rate minimization.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Plots system Kaizen iterations against overall throughput.

```mermaid
flowchart TD
    subgraph Grid ["Kaizen Iteration Index vs Core Engine Throughput (TPS)"]
        direction LR
        P1["(X: Iteration 1, Y: 1000 TPS)"] --> P2["(X: Iteration 2, Y: 1800 TPS)"]
        P2 --> P3["(X: Iteration 3, Y: 3200 TPS)"]
        P3 --> P4["(X: Iteration 4, Y: 4500 TPS - Target SLA met)"]
    end
```
