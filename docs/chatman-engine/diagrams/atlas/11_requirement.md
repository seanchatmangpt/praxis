# 11. Requirement Diagram Family

This file contains the Requirement diagram family for the Chatman Engine, structured across the 8 projection lenses.

---

## Lens 1: Semantic Authority

Diagram ID: REQUIREMENT-L1
Diagram family: Requirement
Projection lens: Semantic Authority
Architectural invariant preserved: RDF/Oxigraph is the single semantic source of truth. Shadow copies of RDF data are strictly prohibited.
Information-loss risk if omitted: Loss of visibility into RDF authority requirements, leading to developers introducing unauthorized semantic shadow copies.
TPS visual-control purpose: Exposes requirement defects and semantic waste on the factory floor.
DfLSS CTQ protected: Zero semantic shadow copies.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Outlines the system requirements for semantic authority that must be verified by the code.

```mermaid
requirementDiagram
    requirement req_rdf_truth {
        id: "REQ-L1"
        text: "RDF/Oxigraph is the single semantic source of truth."
        severity: "critical"
        verifymethod: "analysis"
    }
    element oxigraph_store {
        type: "module"
    }
    oxigraph_store - satisfies -> req_rdf_truth
```

---

## Lens 2: Routing Constitution

Diagram ID: REQUIREMENT-L2
Diagram family: Requirement
Projection lens: Routing Constitution
Architectural invariant preserved: Least-expressive-power routing. Hot, warm, and cold paths must be isolated. N3 is disabled by default.
Information-loss risk if omitted: Danger of executing high-expressivity N3 rules in hot paths, leading to performance degradation or security escape.
TPS visual-control purpose: Visually isolates routing constraints to prevent logic waste.
DfLSS CTQ protected: Safe isolation of cold-path N3 execution.
CENG ticket or boundary constrained: CENG-411 (design-only, implementation blocked).
Why this diagram is non-redundant: Specifying the routing system constraints for the three distinct paths.

```mermaid
requirementDiagram
    requirement req_routing_const {
        id: "REQ-L2"
        text: "Least-expressive-power routing. Hot/warm/cold path isolation. N3 disabled by default."
        severity: "critical"
        verifymethod: "test"
    }
    element routing_engine {
        type: "module"
    }
    routing_engine - satisfies -> req_routing_const
```

---

## Lens 3: Type Kernel Ownership

Diagram ID: REQUIREMENT-L3
Diagram family: Requirement
Projection lens: Type Kernel Ownership
Architectural invariant preserved: Canonical type ownership across wasm4pm-compat, wasm4pm-cognition, bcinr-pddl, bcinr-powl, and praxis-graphlaw.
Information-loss risk if omitted: Duplication of type kernel mappings leading to interface mismatch and compilation failure.
TPS visual-control purpose: Column alignment of type definitions prevents redundant work.
DfLSS CTQ protected: Zero duplicate type classes.
CENG ticket or boundary constrained: CENG-412 (design-only, implementation blocked).
Why this diagram is non-redundant: Formally defines type kernel boundaries in the requirement space.

```mermaid
requirementDiagram
    requirement req_kernel_own {
        id: "REQ-L3"
        text: "Canonical type ownership across wasm4pm-compat, wasm4pm-cognition, bcinr-pddl, bcinr-powl, and praxis-graphlaw."
        severity: "critical"
        verifymethod: "inspection"
    }
    element type_kernel {
        type: "module"
    }
    type_kernel - satisfies -> req_kernel_own
```

---

## Lens 4: Transition Lifecycle

Diagram ID: REQUIREMENT-L4
Diagram family: Requirement
Projection lens: Transition Lifecycle
Architectural invariant preserved: Every transition must pass through candidate invocation, validation, planning, execution, receipting, and replay.
Information-loss risk if omitted: Transition execution without validation or receipting, leading to inconsistent ledger states.
TPS visual-control purpose: Regulates phase transition flow to eliminate process bottleneck waste.
DfLSS CTQ protected: Guaranteed transaction replay validation under fixed seed.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Validates lifecycle transition steps against rigorous verification requirements.

```mermaid
requirementDiagram
    requirement req_trans_life {
        id: "REQ-L4"
        text: "Transitions must pass through candidate invocation, validation, planning, execution, receipting, and replay."
        severity: "critical"
        verifymethod: "test"
    }
    element lifecycle_mgr {
        type: "module"
    }
    lifecycle_mgr - satisfies -> req_trans_life
```

---

## Lens 5: Event / Hook / Actuation

Diagram ID: REQUIREMENT-L5
Diagram family: Requirement
Projection lens: Event / Hook / Actuation
Architectural invariant preserved: Hooks cannot actuate without receipts; no unreceipted actuation.
Information-loss risk if omitted: Unverified boundary effects triggering real-world actions without audit trails.
TPS visual-control purpose: Error-proofing (Poka-Yoke) actuation paths by demanding receipt validation.
DfLSS CTQ protected: Zero unreceipted execution events.
CENG ticket or boundary constrained: CENG-416A-F (design-only, implementation blocked).
Why this diagram is non-redundant: Outlines the verification requirements for event hooks and actuations.

```mermaid
requirementDiagram
    requirement req_hook_act {
        id: "REQ-L5"
        text: "Hooks cannot actuate without receipts. No unreceipted actuation."
        severity: "critical"
        verifymethod: "test"
    }
    element hook_actuator {
        type: "module"
    }
    hook_actuator - satisfies -> req_hook_act
```

---

## Lens 6: Performance / 8-Constraint Hot Path

Diagram ID: REQUIREMENT-L6
Diagram family: Requirement
Projection lens: Performance / 8-Constraint Hot Path
Architectural invariant preserved: Maximum of 8 constraints checked in parallel via RDFTriple8 and ConditionCell<BITS>.
Information-loss risk if omitted: CPU utilization surges and latency SLAs violated if constraints exceed 8.
TPS visual-control purpose: Alerts developers when constraint sizes exceed hot-path limits (Andon).
DfLSS CTQ protected: Latency bound of hot path operations.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Tracks constraints optimization and parallel check constraints.

```mermaid
requirementDiagram
    requirement req_hot_path {
        id: "REQ-L6"
        text: "Maximum of 8 constraints checked in parallel via RDFTriple8 and ConditionCell."
        severity: "critical"
        verifymethod: "test"
    }
    element hotpath_cell {
        type: "module"
    }
    hotpath_cell - satisfies -> req_hot_path
```

---

## Lens 7: Refusal / Risk / Governance

Diagram ID: REQUIREMENT-L7
Diagram family: Requirement
Projection lens: Refusal / Risk / Governance
Architectural invariant preserved: Every failure is a typed Refusal; N3 quarantine rules are strictly enforced.
Information-loss risk if omitted: Risk of silent exceptions, untyped panics, or unmitigated security vulnerabilities.
TPS visual-control purpose: Visual isolation of refusal classification rules.
DfLSS CTQ protected: No panic or silent fallbacks.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Ensures all potential failure cases map to explicit refusal requirements.

```mermaid
requirementDiagram
    requirement req_refusal_gov {
        id: "REQ-L7"
        text: "Every failure is a typed Refusal. N3 quarantine rules strictly enforced."
        severity: "critical"
        verifymethod: "test"
    }
    element gov_verifier {
        type: "module"
    }
    gov_verifier - satisfies -> req_refusal_gov
```

---

## Lens 8: TPS / DfLSS / Continuous Improvement

Diagram ID: REQUIREMENT-L8
Diagram family: Requirement
Projection lens: TPS / DfLSS / Continuous Improvement
Architectural invariant preserved: WIP reduction, continuous process improvement loops, and visual waste elimination.
Information-loss risk if omitted: Process drift and decay of continuous validation feedback loops.
TPS visual-control purpose: Enforces continuous improvement loops on requirement validation.
DfLSS CTQ protected: Flow efficiency and defect rate minimization.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Maps continuous improvement goals directly to system requirements.

```mermaid
requirementDiagram
    requirement req_tps_dflss {
        id: "REQ-L8"
        text: "WIP reduction, continuous process improvement loops, and visual waste elimination."
        severity: "major"
        verifymethod: "analysis"
    }
    element kaizen_loop {
        type: "module"
    }
    kaizen_loop - satisfies -> req_tps_dflss
```
