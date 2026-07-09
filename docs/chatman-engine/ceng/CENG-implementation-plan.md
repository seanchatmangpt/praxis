# CENG Implementation Plan — Chatman Engine Manufacturing Run

This document converts the 240-diagram design atlas located at `/Users/sac/praxis/docs/chatman-engine/diagrams/atlas/` into concrete implementation rails. It maps the visual controls and invariants of the atlas to structured development phases and verification gates.

---

## 1. Architectural Mapping (TPS & Design for Lean Six Sigma)

The Chatman Engine is built using the **Toyota Production System (TPS)** visual factory floor methodology combined with **Design for Lean Six Sigma (DfLSS)** quality principles. The 240 diagrams in the atlas serve as Visual Gauges protecting flow, minimizing Work in Progress (WIP), and preventing representation drift across 8 projection lenses.

### The 8 Projection Lenses

1. **Semantic Authority (Lens 1)**: Ensures RDF/Oxigraph is the sole semantic source of truth. No shadow copies of triple state are permitted.
2. **Routing Constitution (Lens 2)**: Enforces least-expressive-power query routing. Isolates hot, warm, and cold paths, quarantining N3.
3. **Type Kernel Ownership (Lens 3)**: Preserves canonical types across packages (`wasm4pm-compat`, `bcinr-powl`, `bcinr-pddl`, `wasm4pm-cognition`, and `praxis-graphlaw`).
4. **Transition Lifecycle (Lens 4)**: Maps linear progression of transition candidates (Candidate -> Validation -> Planning -> Legality -> Receipt).
5. **Event / Hook / Actuation (Lens 5)**: Manages OCEL logging and SPARQL CONSTRUCT Knowledge Hooks delta projections.
6. **Performance / 8-Constraint Hot Path (Lens 6)**: Implements ConditionCell structures (BITS ≤ 8) and vector-to-mask lowering.
7. **Refusal / Risk / Governance (Lens 7)**: Provides a uniform taxonomy of typed Refusals, eliminating raw unwraps or panics.
8. **TPS / DfLSS / Continuous Improvement (Lens 8)**: Drives continuous benchmark tuning, replay validation, and process improvement loops.

---

## 2. Phased Implementation Roadmap

To maintain compilation integrity and enforce the Rust Core Team discipline, implementation progresses in five sequential phases:

### Phase 1: Core ABI & Bridging
- **Scope**: CENG-411 (CE-ABI) and CENG-412 (Tape Bridge).
- **Goal**: Define request envelopes, base Refusal variants, and plan tape mapping traits.
- **Visual Control**: Governed by Lens 1 (Semantic Authority) and Lens 3 (Type Kernel Ownership) diagrams.
- **Gate Criteria**: 100% compilation check. Zero type duplicates across crates.

### Phase 2: Orchestration & Gating
- **Scope**: CENG-413 (CE-ISA Orchestrator), CENG-414 (Profile Gates), and CENG-415 (Refusal Taxonomy).
- **Goal**: Establish the stateful `ChatmanEngine` pipeline, LER constraint solver, and error taxonomy mapping.
- **Visual Control**: Governed by Lens 4 (Transition Lifecycle), Lens 2 (Routing Constitution), and Lens 7 (Refusal/Risk) diagrams.
- **Gate Criteria**: Integration tests executing mock plans. Verify N3 quarantine blocks.

### Phase 3: Hot Path & Dialect Routing
- **Scope**: CENG-416A-F (8-Constraint Hot Path) and CENG-420+ (Dialect Routing).
- **Goal**: Implement ConditionCell const-generics and vector-to-mask compiler optimizations.
- **Visual Control**: Governed by Lens 6 (Performance) and Lens 2 (Routing) diagrams.
- **Gate Criteria**: `trybuild` tests confirming `ConditionCell<9>` compilation failure. Benchmark suite execution.

### Phase 4: Event Actuation & Hooking
- **Scope**: CENG-460+ (Events/Hooks) and CENG-480+ (Breeds/Agents).
- **Goal**: Connect OCEL causally chained logs, SPARQL hooks, and WASM4PM cognitive breed registry.
- **Visual Control**: Governed by Lens 5 (Events/Hooks) and Lens 3 (Type Ownership) diagrams.
- **Gate Criteria**: Verify BLAKE3 causal chaining hashes. Breed execution sandbox isolation.

### Phase 5: Replay & Verification
- **Scope**: CENG-500+ (Receipts/Replay).
- **Goal**: Execute deterministic transaction replay and state hash audits.
- **Visual Control**: Governed by Lens 8 (TPS/Continuous Improvement) diagrams.
- **Gate Criteria**: Deterministic replay verification suite passing (5x run comparison producing byte-identical receipts).

---

## 3. Invariant Verification Matrix

| Classification | Invariant Name | Primary CENG Ticket | Verification Tool / Command |
|---|---|---|---|
| **Compile-time** | *Need9 means split* | CENG-416A-F | `cargo test --package wasm4pm-compat --test law_trybuild` |
| | *Rational bounded metric* | CENG-411 | `cargo test --package wasm4pm-compat --lib law` |
| | *Crate isolation* | CENG-412 | `cargo check --workspace` |
| **Runtime** | *Acyclic Workflow* | CENG-413 | `cargo test --package bcinr-powl --lib compiler` |
| | *Transition Firing* | CENG-416A-F | `cargo test --package bcinr-powl --lib scheduler_wired` |
| | *PDDL Grounding Match* | CENG-440+ | `cargo test --package bcinr-pddl --lib ground` |
| | *Hook Permission* | CENG-460+ | `cargo test --package praxis-graphlaw --lib hooks` |
| **Receipt-based** | *BLAKE3 Chaining* | CENG-460+ | `cargo test --package bcinr-powl-receipt --lib causal_receipt` |
| | *Contiguity* | CENG-500+ | `cargo test --package praxis-graphlaw --lib replay` |
| | *Deterministic Replay* | CENG-500+ | `cargo test --package praxis-graphlaw --lib replay` |

---

## 4. Comprehensive Diagram Atlas Index

Below is the complete inventory of the 240 diagrams from the design atlas, showing how each visual control maps to the projection lenses and boundary constraints.

| Diagram ID | Family | Projection Lens | Constrained CENG Boundary | File Path |
|---|---|---|---|---|
| **ARCHITECTURE-L1** | Architecture | Semantic Authority | CENG-410-FINAL (in progress). | [22_architecture.md](22_architecture.md) |
| **ARCHITECTURE-L2** | Architecture | Routing Constitution | CENG-411 (design-only, implementation blocked). | [22_architecture.md](22_architecture.md) |
| **ARCHITECTURE-L3** | Architecture | Type Kernel Ownership | CENG-412 (design-only, implementation blocked). | [22_architecture.md](22_architecture.md) |
| **ARCHITECTURE-L4** | Architecture | Transition Lifecycle | CENG-410-FINAL (in progress). | [22_architecture.md](22_architecture.md) |
| **ARCHITECTURE-L5** | Architecture | Event / Hook / Actuation | CENG-416A-F (design-only, implementation blocked). | [22_architecture.md](22_architecture.md) |
| **ARCHITECTURE-L6** | Architecture | Performance / 8-Constraint Hot Path | CENG-410-FINAL (in progress). | [22_architecture.md](22_architecture.md) |
| **ARCHITECTURE-L7** | Architecture | Refusal / Risk / Governance | CENG-410-FINAL (in progress). | [22_architecture.md](22_architecture.md) |
| **ARCHITECTURE-L8** | Architecture | TPS / DfLSS / Continuous Improvement | CENG-410-FINAL (in progress). | [22_architecture.md](22_architecture.md) |
| **BLOCK-L1** | Block | Semantic Authority | CENG-410-FINAL (in progress). | [19_block.md](19_block.md) |
| **BLOCK-L2** | Block | Routing Constitution | CENG-411 (design-only, implementation blocked). | [19_block.md](19_block.md) |
| **BLOCK-L3** | Block | Type Kernel Ownership | CENG-412 (design-only, implementation blocked). | [19_block.md](19_block.md) |
| **BLOCK-L4** | Block | Transition Lifecycle | CENG-410-FINAL (in progress). | [19_block.md](19_block.md) |
| **BLOCK-L5** | Block | Event / Hook / Actuation | CENG-416A-F (design-only, implementation blocked). | [19_block.md](19_block.md) |
| **BLOCK-L6** | Block | Performance / 8-Constraint Hot Path | CENG-410-FINAL (in progress). | [19_block.md](19_block.md) |
| **BLOCK-L7** | Block | Refusal / Risk / Governance | CENG-410-FINAL (in progress). | [19_block.md](19_block.md) |
| **BLOCK-L8** | Block | TPS / DfLSS / Continuous Improvement | CENG-410-FINAL (in progress). | [19_block.md](19_block.md) |
| **C4-L1** | C4 | Semantic Authority | CENG-410-FINAL (in progress). | [13_c4.md](13_c4.md) |
| **C4-L2** | C4 | Routing Constitution | CENG-411 (design-only, implementation blocked). | [13_c4.md](13_c4.md) |
| **C4-L3** | C4 | Type Kernel Ownership | CENG-412 (design-only, implementation blocked). | [13_c4.md](13_c4.md) |
| **C4-L4** | C4 | Transition Lifecycle | CENG-410-FINAL (in progress). | [13_c4.md](13_c4.md) |
| **C4-L5** | C4 | Event / Hook / Actuation | CENG-416A-F (design-only, implementation blocked). | [13_c4.md](13_c4.md) |
| **C4-L6** | C4 | Performance / 8-Constraint Hot Path | CENG-410-FINAL (in progress). | [13_c4.md](13_c4.md) |
| **C4-L7** | C4 | Refusal / Risk / Governance | CENG-410-FINAL (in progress). | [13_c4.md](13_c4.md) |
| **C4-L8** | C4 | TPS / DfLSS / Continuous Improvement | CENG-410-FINAL (in progress). | [13_c4.md](13_c4.md) |
| **CLASS-L1** | Class | Semantic Authority | Bound by CENG-410-FINAL. | [04_class.md](04_class.md) |
| **CLASS-L2** | Class | Routing Constitution | CENG-410-M1. | [04_class.md](04_class.md) |
| **CLASS-L3** | Class | Type Kernel Ownership | CENG-411 (design-only). | [04_class.md](04_class.md) |
| **CLASS-L4** | Class | Transition Lifecycle | CENG-410-FINAL. | [04_class.md](04_class.md) |
| **CLASS-L5** | Class | Event / Hook / Actuation | CENG-412 (design-only). | [04_class.md](04_class.md) |
| **CLASS-L6** | Class | Performance / 8-Constraint Hot Path | CENG-410-M1. | [04_class.md](04_class.md) |
| **CLASS-L7** | Class | Refusal / Risk / Governance | CENG-410-FINAL. | [04_class.md](04_class.md) |
| **CLASS-L8** | Class | TPS / DfLSS / Continuous Improvement | CENG-416A-F (design-only). | [04_class.md](04_class.md) |
| **CYNEFIN-L1** | Cynefin | Semantic Authority | CENG-410-FINAL (in progress). | [29_cynefin.md](29_cynefin.md) |
| **CYNEFIN-L2** | Cynefin | Routing Constitution | CENG-411 (design-only, implementation blocked). | [29_cynefin.md](29_cynefin.md) |
| **CYNEFIN-L3** | Cynefin | Type Kernel Ownership | CENG-412 (design-only, implementation blocked). | [29_cynefin.md](29_cynefin.md) |
| **CYNEFIN-L4** | Cynefin | Transition Lifecycle | CENG-410-FINAL (in progress). | [29_cynefin.md](29_cynefin.md) |
| **CYNEFIN-L5** | Cynefin | Event / Hook / Actuation | CENG-416A-F (design-only, implementation blocked). | [29_cynefin.md](29_cynefin.md) |
| **CYNEFIN-L6** | Cynefin | Performance / 8-Constraint Hot Path | CENG-410-FINAL (in progress). | [29_cynefin.md](29_cynefin.md) |
| **CYNEFIN-L7** | Cynefin | Refusal / Risk / Governance | CENG-410-FINAL (in progress). | [29_cynefin.md](29_cynefin.md) |
| **CYNEFIN-L8** | Cynefin | TPS / DfLSS / Continuous Improvement | CENG-410-FINAL (in progress). | [29_cynefin.md](29_cynefin.md) |
| **ENTITY_RELATIONSHIP-L1** | Entity Relationship | Semantic Authority | Bound by CENG-410-FINAL. | [06_entity_relationship.md](06_entity_relationship.md) |
| **ENTITY_RELATIONSHIP-L2** | Entity Relationship | Routing Constitution | CENG-410-M1. | [06_entity_relationship.md](06_entity_relationship.md) |
| **ENTITY_RELATIONSHIP-L3** | Entity Relationship | Type Kernel Ownership | CENG-411 (design-only). | [06_entity_relationship.md](06_entity_relationship.md) |
| **ENTITY_RELATIONSHIP-L4** | Entity Relationship | Transition Lifecycle | CENG-410-FINAL. | [06_entity_relationship.md](06_entity_relationship.md) |
| **ENTITY_RELATIONSHIP-L5** | Entity Relationship | Event / Hook / Actuation | CENG-412 (design-only). | [06_entity_relationship.md](06_entity_relationship.md) |
| **ENTITY_RELATIONSHIP-L6** | Entity Relationship | Performance / 8-Constraint Hot Path | CENG-410-M1. | [06_entity_relationship.md](06_entity_relationship.md) |
| **ENTITY_RELATIONSHIP-L7** | Entity Relationship | Refusal / Risk / Governance | CENG-410-FINAL. | [06_entity_relationship.md](06_entity_relationship.md) |
| **ENTITY_RELATIONSHIP-L8** | Entity Relationship | TPS / DfLSS / Continuous Improvement | CENG-416A-F (design-only). | [06_entity_relationship.md](06_entity_relationship.md) |
| **EVENT_MODELING-L1** | Event Modeling | Semantic Authority | CENG-410-FINAL (in progress). | [24_event_modeling.md](24_event_modeling.md) |
| **EVENT_MODELING-L2** | Event Modeling | Routing Constitution | CENG-411 (design-only, implementation blocked). | [24_event_modeling.md](24_event_modeling.md) |
| **EVENT_MODELING-L3** | Event Modeling | Type Kernel Ownership | CENG-412 (design-only, implementation blocked). | [24_event_modeling.md](24_event_modeling.md) |
| **EVENT_MODELING-L4** | Event Modeling | Transition Lifecycle | CENG-410-FINAL (in progress). | [24_event_modeling.md](24_event_modeling.md) |
| **EVENT_MODELING-L5** | Event Modeling | Event / Hook / Actuation | CENG-416A-F (design-only, implementation blocked). | [24_event_modeling.md](24_event_modeling.md) |
| **EVENT_MODELING-L6** | Event Modeling | Performance / 8-Constraint Hot Path | CENG-410-FINAL (in progress). | [24_event_modeling.md](24_event_modeling.md) |
| **EVENT_MODELING-L7** | Event Modeling | Refusal / Risk / Governance | CENG-410-FINAL (in progress). | [24_event_modeling.md](24_event_modeling.md) |
| **EVENT_MODELING-L8** | Event Modeling | TPS / DfLSS / Continuous Improvement | CENG-410-FINAL (in progress). | [24_event_modeling.md](24_event_modeling.md) |
| **FLOWCHART-L1** | Flowchart | Semantic Authority | Bound by CENG-410-FINAL (securing the final state boundaries). | [01_flowchart.md](01_flowchart.md) |
| **FLOWCHART-L2** | Flowchart | Routing Constitution | CENG-410-M1 (routing gating accepted). | [01_flowchart.md](01_flowchart.md) |
| **FLOWCHART-L3** | Flowchart | Type Kernel Ownership | Bound by CENG-411 (design-only, implementation blocked). | [01_flowchart.md](01_flowchart.md) |
| **FLOWCHART-L4** | Flowchart | Transition Lifecycle | CENG-410-FINAL (boundary checks). | [01_flowchart.md](01_flowchart.md) |
| **FLOWCHART-L5** | Flowchart | Event / Hook / Actuation | CENG-412 (design-only, auditing). | [01_flowchart.md](01_flowchart.md) |
| **FLOWCHART-L6** | Flowchart | Performance / 8-Constraint Hot Path | CENG-410-M1 (accepted). | [01_flowchart.md](01_flowchart.md) |
| **FLOWCHART-L7** | Flowchart | Refusal / Risk / Governance | CENG-410-FINAL. | [01_flowchart.md](01_flowchart.md) |
| **FLOWCHART-L8** | Flowchart | TPS / DfLSS / Continuous Improvement | CENG-416A-F (design-only). | [01_flowchart.md](01_flowchart.md) |
| **GANTT-L1** | Gantt | Semantic Authority | Bound by CENG-410-FINAL. | [08_gantt.md](08_gantt.md) |
| **GANTT-L2** | Gantt | Routing Constitution | CENG-410-M1. | [08_gantt.md](08_gantt.md) |
| **GANTT-L3** | Gantt | Type Kernel Ownership | CENG-411 (design-only). | [08_gantt.md](08_gantt.md) |
| **GANTT-L4** | Gantt | Transition Lifecycle | CENG-410-FINAL. | [08_gantt.md](08_gantt.md) |
| **GANTT-L5** | Gantt | Event / Hook / Actuation | CENG-412 (design-only). | [08_gantt.md](08_gantt.md) |
| **GANTT-L6** | Gantt | Performance / 8-Constraint Hot Path | CENG-410-M1. | [08_gantt.md](08_gantt.md) |
| **GANTT-L7** | Gantt | Refusal / Risk / Governance | CENG-410-FINAL. | [08_gantt.md](08_gantt.md) |
| **GANTT-L8** | Gantt | TPS / DfLSS / Continuous Improvement | CENG-416A-F (design-only). | [08_gantt.md](08_gantt.md) |
| **GITGRAPH-L1** | GitGraph | Semantic Authority | CENG-410-FINAL (in progress). | [12_gitgraph.md](12_gitgraph.md) |
| **GITGRAPH-L2** | GitGraph | Routing Constitution | CENG-411 (design-only, implementation blocked). | [12_gitgraph.md](12_gitgraph.md) |
| **GITGRAPH-L3** | GitGraph | Type Kernel Ownership | CENG-412 (design-only, implementation blocked). | [12_gitgraph.md](12_gitgraph.md) |
| **GITGRAPH-L4** | GitGraph | Transition Lifecycle | CENG-410-FINAL (in progress). | [12_gitgraph.md](12_gitgraph.md) |
| **GITGRAPH-L5** | GitGraph | Event / Hook / Actuation | CENG-416A-F (design-only, implementation blocked). | [12_gitgraph.md](12_gitgraph.md) |
| **GITGRAPH-L6** | GitGraph | Performance / 8-Constraint Hot Path | CENG-410-FINAL (in progress). | [12_gitgraph.md](12_gitgraph.md) |
| **GITGRAPH-L7** | GitGraph | Refusal / Risk / Governance | CENG-410-FINAL (in progress). | [12_gitgraph.md](12_gitgraph.md) |
| **GITGRAPH-L8** | GitGraph | TPS / DfLSS / Continuous Improvement | CENG-410-FINAL (in progress). | [12_gitgraph.md](12_gitgraph.md) |
| **ISHIKAWA-L1** | Ishikawa | Semantic Authority | CENG-410-FINAL (in progress). | [27_ishikawa.md](27_ishikawa.md) |
| **ISHIKAWA-L2** | Ishikawa | Routing Constitution | CENG-411 (design-only, implementation blocked). | [27_ishikawa.md](27_ishikawa.md) |
| **ISHIKAWA-L3** | Ishikawa | Type Kernel Ownership | CENG-412 (design-only, implementation blocked). | [27_ishikawa.md](27_ishikawa.md) |
| **ISHIKAWA-L4** | Ishikawa | Transition Lifecycle | CENG-410-FINAL (in progress). | [27_ishikawa.md](27_ishikawa.md) |
| **ISHIKAWA-L5** | Ishikawa | Event / Hook / Actuation | CENG-416A-F (design-only, implementation blocked). | [27_ishikawa.md](27_ishikawa.md) |
| **ISHIKAWA-L6** | Ishikawa | Performance / 8-Constraint Hot Path | CENG-410-FINAL (in progress). | [27_ishikawa.md](27_ishikawa.md) |
| **ISHIKAWA-L7** | Ishikawa | Refusal / Risk / Governance | CENG-410-FINAL (in progress). | [27_ishikawa.md](27_ishikawa.md) |
| **ISHIKAWA-L8** | Ishikawa | TPS / DfLSS / Continuous Improvement | CENG-410-FINAL (in progress). | [27_ishikawa.md](27_ishikawa.md) |
| **KANBAN-L1** | Kanban | Semantic Authority | CENG-410-FINAL (in progress). | [21_kanban.md](21_kanban.md) |
| **KANBAN-L2** | Kanban | Routing Constitution | CENG-411 (design-only, implementation blocked). | [21_kanban.md](21_kanban.md) |
| **KANBAN-L3** | Kanban | Type Kernel Ownership | CENG-412 (design-only, implementation blocked). | [21_kanban.md](21_kanban.md) |
| **KANBAN-L4** | Kanban | Transition Lifecycle | CENG-410-FINAL (in progress). | [21_kanban.md](21_kanban.md) |
| **KANBAN-L5** | Kanban | Event / Hook / Actuation | CENG-416A-F (design-only, implementation blocked). | [21_kanban.md](21_kanban.md) |
| **KANBAN-L6** | Kanban | Performance / 8-Constraint Hot Path | CENG-410-FINAL (in progress). | [21_kanban.md](21_kanban.md) |
| **KANBAN-L7** | Kanban | Refusal / Risk / Governance | CENG-410-FINAL (in progress). | [21_kanban.md](21_kanban.md) |
| **KANBAN-L8** | Kanban | TPS / DfLSS / Continuous Improvement | CENG-410-FINAL (in progress). | [21_kanban.md](21_kanban.md) |
| **MINDMAP-L1** | Mindmap | Semantic Authority | CENG-410-FINAL (in progress). | [14_mindmap.md](14_mindmap.md) |
| **MINDMAP-L2** | Mindmap | Routing Constitution | CENG-411 (design-only, implementation blocked). | [14_mindmap.md](14_mindmap.md) |
| **MINDMAP-L3** | Mindmap | Type Kernel Ownership | CENG-412 (design-only, implementation blocked). | [14_mindmap.md](14_mindmap.md) |
| **MINDMAP-L4** | Mindmap | Transition Lifecycle | CENG-410-FINAL (in progress). | [14_mindmap.md](14_mindmap.md) |
| **MINDMAP-L5** | Mindmap | Event / Hook / Actuation | CENG-416A-F (design-only, implementation blocked). | [14_mindmap.md](14_mindmap.md) |
| **MINDMAP-L6** | Mindmap | Performance / 8-Constraint Hot Path | CENG-410-FINAL (in progress). | [14_mindmap.md](14_mindmap.md) |
| **MINDMAP-L7** | Mindmap | Refusal / Risk / Governance | CENG-410-FINAL (in progress). | [14_mindmap.md](14_mindmap.md) |
| **MINDMAP-L8** | Mindmap | TPS / DfLSS / Continuous Improvement | CENG-410-FINAL (in progress). | [14_mindmap.md](14_mindmap.md) |
| **PACKET-L1** | Packet | Semantic Authority | CENG-410-FINAL (in progress). | [20_packet.md](20_packet.md) |
| **PACKET-L2** | Packet | Routing Constitution | CENG-411 (design-only, implementation blocked). | [20_packet.md](20_packet.md) |
| **PACKET-L3** | Packet | Type Kernel Ownership | CENG-412 (design-only, implementation blocked). | [20_packet.md](20_packet.md) |
| **PACKET-L4** | Packet | Transition Lifecycle | CENG-410-FINAL (in progress). | [20_packet.md](20_packet.md) |
| **PACKET-L5** | Packet | Event / Hook / Actuation | CENG-416A-F (design-only, implementation blocked). | [20_packet.md](20_packet.md) |
| **PACKET-L6** | Packet | Performance / 8-Constraint Hot Path | CENG-410-FINAL (in progress). | [20_packet.md](20_packet.md) |
| **PACKET-L7** | Packet | Refusal / Risk / Governance | CENG-410-FINAL (in progress). | [20_packet.md](20_packet.md) |
| **PACKET-L8** | Packet | TPS / DfLSS / Continuous Improvement | CENG-410-FINAL (in progress). | [20_packet.md](20_packet.md) |
| **PIE-L1** | Pie | Semantic Authority | Bound by CENG-410-FINAL. | [09_pie.md](09_pie.md) |
| **PIE-L2** | Pie | Routing Constitution | CENG-410-M1. | [09_pie.md](09_pie.md) |
| **PIE-L3** | Pie | Type Kernel Ownership | CENG-411 (design-only). | [09_pie.md](09_pie.md) |
| **PIE-L4** | Pie | Transition Lifecycle | CENG-410-FINAL. | [09_pie.md](09_pie.md) |
| **PIE-L5** | Pie | Event / Hook / Actuation | CENG-412 (design-only). | [09_pie.md](09_pie.md) |
| **PIE-L6** | Pie | Performance / 8-Constraint Hot Path | CENG-410-M1. | [09_pie.md](09_pie.md) |
| **PIE-L7** | Pie | Refusal / Risk / Governance | CENG-410-FINAL. | [09_pie.md](09_pie.md) |
| **PIE-L8** | Pie | TPS / DfLSS / Continuous Improvement | CENG-416A-F (design-only). | [09_pie.md](09_pie.md) |
| **QUADRANT-L1** | Quadrant | Semantic Authority | Bound by CENG-410-FINAL. | [10_quadrant.md](10_quadrant.md) |
| **QUADRANT-L2** | Quadrant | Routing Constitution | CENG-410-M1. | [10_quadrant.md](10_quadrant.md) |
| **QUADRANT-L3** | Quadrant | Type Kernel Ownership | CENG-411 (design-only). | [10_quadrant.md](10_quadrant.md) |
| **QUADRANT-L4** | Quadrant | Transition Lifecycle | CENG-410-FINAL. | [10_quadrant.md](10_quadrant.md) |
| **QUADRANT-L5** | Quadrant | Event / Hook / Actuation | CENG-412 (design-only). | [10_quadrant.md](10_quadrant.md) |
| **QUADRANT-L6** | Quadrant | Performance / 8-Constraint Hot Path | CENG-410-M1. | [10_quadrant.md](10_quadrant.md) |
| **QUADRANT-L7** | Quadrant | Refusal / Risk / Governance | CENG-410-FINAL. | [10_quadrant.md](10_quadrant.md) |
| **QUADRANT-L8** | Quadrant | TPS / DfLSS / Continuous Improvement | CENG-416A-F (design-only). | [10_quadrant.md](10_quadrant.md) |
| **RADAR-L1** | Radar | Semantic Authority | CENG-410-FINAL (in progress). | [23_radar.md](23_radar.md) |
| **RADAR-L2** | Radar | Routing Constitution | CENG-411 (design-only, implementation blocked). | [23_radar.md](23_radar.md) |
| **RADAR-L3** | Radar | Type Kernel Ownership | CENG-412 (design-only, implementation blocked). | [23_radar.md](23_radar.md) |
| **RADAR-L4** | Radar | Transition Lifecycle | CENG-410-FINAL (in progress). | [23_radar.md](23_radar.md) |
| **RADAR-L5** | Radar | Event / Hook / Actuation | CENG-416A-F (design-only, implementation blocked). | [23_radar.md](23_radar.md) |
| **RADAR-L6** | Radar | Performance / 8-Constraint Hot Path | CENG-410-FINAL (in progress). | [23_radar.md](23_radar.md) |
| **RADAR-L7** | Radar | Refusal / Risk / Governance | CENG-410-FINAL (in progress). | [23_radar.md](23_radar.md) |
| **RADAR-L8** | Radar | TPS / DfLSS / Continuous Improvement | CENG-410-FINAL (in progress). | [23_radar.md](23_radar.md) |
| **REQUIREMENT-L1** | Requirement | Semantic Authority | CENG-410-FINAL (in progress). | [11_requirement.md](11_requirement.md) |
| **REQUIREMENT-L2** | Requirement | Routing Constitution | CENG-411 (design-only, implementation blocked). | [11_requirement.md](11_requirement.md) |
| **REQUIREMENT-L3** | Requirement | Type Kernel Ownership | CENG-412 (design-only, implementation blocked). | [11_requirement.md](11_requirement.md) |
| **REQUIREMENT-L4** | Requirement | Transition Lifecycle | CENG-410-FINAL (in progress). | [11_requirement.md](11_requirement.md) |
| **REQUIREMENT-L5** | Requirement | Event / Hook / Actuation | CENG-416A-F (design-only, implementation blocked). | [11_requirement.md](11_requirement.md) |
| **REQUIREMENT-L6** | Requirement | Performance / 8-Constraint Hot Path | CENG-410-FINAL (in progress). | [11_requirement.md](11_requirement.md) |
| **REQUIREMENT-L7** | Requirement | Refusal / Risk / Governance | CENG-410-FINAL (in progress). | [11_requirement.md](11_requirement.md) |
| **REQUIREMENT-L8** | Requirement | TPS / DfLSS / Continuous Improvement | CENG-410-FINAL (in progress). | [11_requirement.md](11_requirement.md) |
| **SANKEY-L1** | Sankey | Semantic Authority | CENG-410-FINAL (in progress). | [17_sankey.md](17_sankey.md) |
| **SANKEY-L2** | Sankey | Routing Constitution | CENG-411 (design-only, implementation blocked). | [17_sankey.md](17_sankey.md) |
| **SANKEY-L3** | Sankey | Type Kernel Ownership | CENG-412 (design-only, implementation blocked). | [17_sankey.md](17_sankey.md) |
| **SANKEY-L4** | Sankey | Transition Lifecycle | CENG-410-FINAL (in progress). | [17_sankey.md](17_sankey.md) |
| **SANKEY-L5** | Sankey | Event / Hook / Actuation | CENG-416A-F (design-only, implementation blocked). | [17_sankey.md](17_sankey.md) |
| **SANKEY-L6** | Sankey | Performance / 8-Constraint Hot Path | CENG-410-FINAL (in progress). | [17_sankey.md](17_sankey.md) |
| **SANKEY-L7** | Sankey | Refusal / Risk / Governance | CENG-410-FINAL (in progress). | [17_sankey.md](17_sankey.md) |
| **SANKEY-L8** | Sankey | TPS / DfLSS / Continuous Improvement | CENG-410-FINAL (in progress). | [17_sankey.md](17_sankey.md) |
| **SEQUENCE-L1** | Sequence | Semantic Authority | Bound by CENG-410-FINAL. | [03_sequence.md](03_sequence.md) |
| **SEQUENCE-L2** | Sequence | Routing Constitution | CENG-410-M1. | [03_sequence.md](03_sequence.md) |
| **SEQUENCE-L3** | Sequence | Type Kernel Ownership | CENG-411 (design-only). | [03_sequence.md](03_sequence.md) |
| **SEQUENCE-L4** | Sequence | Transition Lifecycle | CENG-410-FINAL. | [03_sequence.md](03_sequence.md) |
| **SEQUENCE-L5** | Sequence | Event / Hook / Actuation | CENG-412 (design-only). | [03_sequence.md](03_sequence.md) |
| **SEQUENCE-L6** | Sequence | Performance / 8-Constraint Hot Path | CENG-410-M1. | [03_sequence.md](03_sequence.md) |
| **SEQUENCE-L7** | Sequence | Refusal / Risk / Governance | CENG-410-FINAL. | [03_sequence.md](03_sequence.md) |
| **SEQUENCE-L8** | Sequence | TPS / DfLSS / Continuous Improvement | CENG-416A-F (design-only). | [03_sequence.md](03_sequence.md) |
| **STATE-L1** | State | Semantic Authority | Bound by CENG-410-FINAL. | [05_state.md](05_state.md) |
| **STATE-L2** | State | Routing Constitution | CENG-410-M1. | [05_state.md](05_state.md) |
| **STATE-L3** | State | Type Kernel Ownership | CENG-411 (design-only). | [05_state.md](05_state.md) |
| **STATE-L4** | State | Transition Lifecycle | CENG-410-FINAL. | [05_state.md](05_state.md) |
| **STATE-L5** | State | Event / Hook / Actuation | CENG-412 (design-only). | [05_state.md](05_state.md) |
| **STATE-L6** | State | Performance / 8-Constraint Hot Path | CENG-410-M1. | [05_state.md](05_state.md) |
| **STATE-L7** | State | Refusal / Risk / Governance | CENG-410-FINAL. | [05_state.md](05_state.md) |
| **STATE-L8** | State | TPS / DfLSS / Continuous Improvement | CENG-416A-F (design-only). | [05_state.md](05_state.md) |
| **SWIMLANES-L1** | Swimlanes | Semantic Authority | Bound by CENG-410-FINAL. | [02_swimlanes.md](02_swimlanes.md) |
| **SWIMLANES-L2** | Swimlanes | Routing Constitution | CENG-410-M1. | [02_swimlanes.md](02_swimlanes.md) |
| **SWIMLANES-L3** | Swimlanes | Type Kernel Ownership | CENG-411 (design-only). | [02_swimlanes.md](02_swimlanes.md) |
| **SWIMLANES-L4** | Swimlanes | Transition Lifecycle | CENG-410-FINAL. | [02_swimlanes.md](02_swimlanes.md) |
| **SWIMLANES-L5** | Swimlanes | Event / Hook / Actuation | CENG-412 (design-only). | [02_swimlanes.md](02_swimlanes.md) |
| **SWIMLANES-L6** | Swimlanes | Performance / 8-Constraint Hot Path | CENG-410-M1. | [02_swimlanes.md](02_swimlanes.md) |
| **SWIMLANES-L7** | Swimlanes | Refusal / Risk / Governance | CENG-410-FINAL. | [02_swimlanes.md](02_swimlanes.md) |
| **SWIMLANES-L8** | Swimlanes | TPS / DfLSS / Continuous Improvement | CENG-416A-F (design-only). | [02_swimlanes.md](02_swimlanes.md) |
| **TIMELINE-L1** | Timeline | Semantic Authority | CENG-410-FINAL (in progress). | [15_timeline.md](15_timeline.md) |
| **TIMELINE-L2** | Timeline | Routing Constitution | CENG-411 (design-only, implementation blocked). | [15_timeline.md](15_timeline.md) |
| **TIMELINE-L3** | Timeline | Type Kernel Ownership | CENG-412 (design-only, implementation blocked). | [15_timeline.md](15_timeline.md) |
| **TIMELINE-L4** | Timeline | Transition Lifecycle | CENG-410-FINAL (in progress). | [15_timeline.md](15_timeline.md) |
| **TIMELINE-L5** | Timeline | Event / Hook / Actuation | CENG-416A-F (design-only, implementation blocked). | [15_timeline.md](15_timeline.md) |
| **TIMELINE-L6** | Timeline | Performance / 8-Constraint Hot Path | CENG-410-FINAL (in progress). | [15_timeline.md](15_timeline.md) |
| **TIMELINE-L7** | Timeline | Refusal / Risk / Governance | CENG-410-FINAL (in progress). | [15_timeline.md](15_timeline.md) |
| **TIMELINE-L8** | Timeline | TPS / DfLSS / Continuous Improvement | CENG-410-FINAL (in progress). | [15_timeline.md](15_timeline.md) |
| **TREEMAP-L1** | Treemap | Semantic Authority | CENG-410-FINAL (in progress). | [25_treemap.md](25_treemap.md) |
| **TREEMAP-L2** | Treemap | Routing Constitution | CENG-411 (design-only, implementation blocked). | [25_treemap.md](25_treemap.md) |
| **TREEMAP-L3** | Treemap | Type Kernel Ownership | CENG-412 (design-only, implementation blocked). | [25_treemap.md](25_treemap.md) |
| **TREEMAP-L4** | Treemap | Transition Lifecycle | CENG-410-FINAL (in progress). | [25_treemap.md](25_treemap.md) |
| **TREEMAP-L5** | Treemap | Event / Hook / Actuation | CENG-416A-F (design-only, implementation blocked). | [25_treemap.md](25_treemap.md) |
| **TREEMAP-L6** | Treemap | Performance / 8-Constraint Hot Path | CENG-410-FINAL (in progress). | [25_treemap.md](25_treemap.md) |
| **TREEMAP-L7** | Treemap | Refusal / Risk / Governance | CENG-410-FINAL (in progress). | [25_treemap.md](25_treemap.md) |
| **TREEMAP-L8** | Treemap | TPS / DfLSS / Continuous Improvement | CENG-410-FINAL (in progress). | [25_treemap.md](25_treemap.md) |
| **TREEVIEW-L1** | Treeview | Semantic Authority | CENG-410-FINAL (in progress). | [30_treeview.md](30_treeview.md) |
| **TREEVIEW-L2** | Treeview | Routing Constitution | CENG-411 (design-only, implementation blocked). | [30_treeview.md](30_treeview.md) |
| **TREEVIEW-L3** | Treeview | Type Kernel Ownership | CENG-412 (design-only, implementation blocked). | [30_treeview.md](30_treeview.md) |
| **TREEVIEW-L4** | Treeview | Transition Lifecycle | CENG-410-FINAL (in progress). | [30_treeview.md](30_treeview.md) |
| **TREEVIEW-L5** | Treeview | Event / Hook / Actuation | CENG-416A-F (design-only, implementation blocked). | [30_treeview.md](30_treeview.md) |
| **TREEVIEW-L6** | Treeview | Performance / 8-Constraint Hot Path | CENG-410-FINAL (in progress). | [30_treeview.md](30_treeview.md) |
| **TREEVIEW-L7** | Treeview | Refusal / Risk / Governance | CENG-410-FINAL (in progress). | [30_treeview.md](30_treeview.md) |
| **TREEVIEW-L8** | Treeview | TPS / DfLSS / Continuous Improvement | CENG-410-FINAL (in progress). | [30_treeview.md](30_treeview.md) |
| **USER_JOURNEY-L1** | User Journey | Semantic Authority | Bound by CENG-410-FINAL. | [07_user_journey.md](07_user_journey.md) |
| **USER_JOURNEY-L2** | User Journey | Routing Constitution | CENG-410-M1. | [07_user_journey.md](07_user_journey.md) |
| **USER_JOURNEY-L3** | User Journey | Type Kernel Ownership | CENG-411 (design-only). | [07_user_journey.md](07_user_journey.md) |
| **USER_JOURNEY-L4** | User Journey | Transition Lifecycle | CENG-410-FINAL. | [07_user_journey.md](07_user_journey.md) |
| **USER_JOURNEY-L5** | User Journey | Event / Hook / Actuation | CENG-412 (design-only). | [07_user_journey.md](07_user_journey.md) |
| **USER_JOURNEY-L6** | User Journey | Performance / 8-Constraint Hot Path | CENG-410-M1. | [07_user_journey.md](07_user_journey.md) |
| **USER_JOURNEY-L7** | User Journey | Refusal / Risk / Governance | CENG-410-FINAL. | [07_user_journey.md](07_user_journey.md) |
| **USER_JOURNEY-L8** | User Journey | TPS / DfLSS / Continuous Improvement | CENG-416A-F (design-only). | [07_user_journey.md](07_user_journey.md) |
| **VENN-L1** | Venn | Semantic Authority | CENG-410-FINAL (in progress). | [26_venn.md](26_venn.md) |
| **VENN-L2** | Venn | Routing Constitution | CENG-411 (design-only, implementation blocked). | [26_venn.md](26_venn.md) |
| **VENN-L3** | Venn | Type Kernel Ownership | CENG-412 (design-only, implementation blocked). | [26_venn.md](26_venn.md) |
| **VENN-L4** | Venn | Transition Lifecycle | CENG-410-FINAL (in progress). | [26_venn.md](26_venn.md) |
| **VENN-L5** | Venn | Event / Hook / Actuation | CENG-416A-F (design-only, implementation blocked). | [26_venn.md](26_venn.md) |
| **VENN-L6** | Venn | Performance / 8-Constraint Hot Path | CENG-410-FINAL (in progress). | [26_venn.md](26_venn.md) |
| **VENN-L7** | Venn | Refusal / Risk / Governance | CENG-410-FINAL (in progress). | [26_venn.md](26_venn.md) |
| **VENN-L8** | Venn | TPS / DfLSS / Continuous Improvement | CENG-410-FINAL (in progress). | [26_venn.md](26_venn.md) |
| **WARDLEY-L1** | Wardley Map | Semantic Authority | CENG-410-FINAL (in progress). | [28_wardley.md](28_wardley.md) |
| **WARDLEY-L2** | Wardley Map | Routing Constitution | CENG-411 (design-only, implementation blocked). | [28_wardley.md](28_wardley.md) |
| **WARDLEY-L3** | Wardley Map | Type Kernel Ownership | CENG-412 (design-only, implementation blocked). | [28_wardley.md](28_wardley.md) |
| **WARDLEY-L4** | Wardley Map | Transition Lifecycle | CENG-410-FINAL (in progress). | [28_wardley.md](28_wardley.md) |
| **WARDLEY-L5** | Wardley Map | Event / Hook / Actuation | CENG-416A-F (design-only, implementation blocked). | [28_wardley.md](28_wardley.md) |
| **WARDLEY-L6** | Wardley Map | Performance / 8-Constraint Hot Path | CENG-410-FINAL (in progress). | [28_wardley.md](28_wardley.md) |
| **WARDLEY-L7** | Wardley Map | Refusal / Risk / Governance | CENG-410-FINAL (in progress). | [28_wardley.md](28_wardley.md) |
| **WARDLEY-L8** | Wardley Map | TPS / DfLSS / Continuous Improvement | CENG-410-FINAL (in progress). | [28_wardley.md](28_wardley.md) |
| **XY_CHART-L1** | XY Chart | Semantic Authority | CENG-410-FINAL (in progress). | [18_xy_chart.md](18_xy_chart.md) |
| **XY_CHART-L2** | XY Chart | Routing Constitution | CENG-411 (design-only, implementation blocked). | [18_xy_chart.md](18_xy_chart.md) |
| **XY_CHART-L3** | XY Chart | Type Kernel Ownership | CENG-412 (design-only, implementation blocked). | [18_xy_chart.md](18_xy_chart.md) |
| **XY_CHART-L4** | XY Chart | Transition Lifecycle | CENG-410-FINAL (in progress). | [18_xy_chart.md](18_xy_chart.md) |
| **XY_CHART-L5** | XY Chart | Event / Hook / Actuation | CENG-416A-F (design-only, implementation blocked). | [18_xy_chart.md](18_xy_chart.md) |
| **XY_CHART-L6** | XY Chart | Performance / 8-Constraint Hot Path | CENG-410-FINAL (in progress). | [18_xy_chart.md](18_xy_chart.md) |
| **XY_CHART-L7** | XY Chart | Refusal / Risk / Governance | CENG-410-FINAL (in progress). | [18_xy_chart.md](18_xy_chart.md) |
| **XY_CHART-L8** | XY Chart | TPS / DfLSS / Continuous Improvement | CENG-410-FINAL (in progress). | [18_xy_chart.md](18_xy_chart.md) |
| **ZENUML-L1** | ZenUML | Semantic Authority | CENG-410-FINAL (in progress). | [16_zenuml.md](16_zenuml.md) |
| **ZENUML-L2** | ZenUML | Routing Constitution | CENG-411 (design-only, implementation blocked). | [16_zenuml.md](16_zenuml.md) |
| **ZENUML-L3** | ZenUML | Type Kernel Ownership | CENG-412 (design-only, implementation blocked). | [16_zenuml.md](16_zenuml.md) |
| **ZENUML-L4** | ZenUML | Transition Lifecycle | CENG-410-FINAL (in progress). | [16_zenuml.md](16_zenuml.md) |
| **ZENUML-L5** | ZenUML | Event / Hook / Actuation | CENG-416A-F (design-only, implementation blocked). | [16_zenuml.md](16_zenuml.md) |
| **ZENUML-L6** | ZenUML | Performance / 8-Constraint Hot Path | CENG-410-FINAL (in progress). | [16_zenuml.md](16_zenuml.md) |
| **ZENUML-L7** | ZenUML | Refusal / Risk / Governance | CENG-410-FINAL (in progress). | [16_zenuml.md](16_zenuml.md) |
| **ZENUML-L8** | ZenUML | TPS / DfLSS / Continuous Improvement | CENG-410-FINAL (in progress). | [16_zenuml.md](16_zenuml.md) |

