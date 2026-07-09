# 12. GitGraph Diagram Family

This file contains the GitGraph diagram family for the Chatman Engine, structured across the 8 projection lenses.

---

## Lens 1: Semantic Authority

Diagram ID: GITGRAPH-L1
Diagram family: GitGraph
Projection lens: Semantic Authority
Architectural invariant preserved: RDF/Oxigraph is the single semantic source of truth. Shadow copies of RDF data are strictly prohibited.
Information-loss risk if omitted: Merging commits that introduce duplicate semantic types or memory shadow caches.
TPS visual-control purpose: Andon check on merge requests to prevent semantic waste.
DfLSS CTQ protected: Zero semantic shadow copies.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Visualizes the sequence of changes to ensure semantic integrity in the repository.

```mermaid
gitGraph
    commit id: "Oxigraph-Init"
    commit id: "Closed-Vocab-Enforcement"
    branch feat-rdf-authority
    checkout feat-rdf-authority
    commit id: "Prevent-Shadow-Copies"
    commit id: "Assert-BLAKE3-Hash"
    checkout main
    merge feat-rdf-authority
    commit id: "CENG-410-M1-Tag" tag: "CENG-410-M1"
```

---

## Lens 2: Routing Constitution

Diagram ID: GITGRAPH-L2
Diagram family: GitGraph
Projection lens: Routing Constitution
Architectural invariant preserved: Least-expressive-power routing; hot/warm/cold path isolation. N3 is disabled by default.
Information-loss risk if omitted: High-expressivity rules (N3) bypassing gates and merging directly into hot-path runtime.
TPS visual-control purpose: Visual control over path isolation branches before merging.
DfLSS CTQ protected: Safe isolation of cold-path N3 execution.
CENG ticket or boundary constrained: CENG-411 (design-only, implementation blocked).
Why this diagram is non-redundant: Traces the path-isolation design branches in git history.

```mermaid
gitGraph
    commit id: "Core-Routing-Init"
    branch feat-ceng-411-design
    checkout feat-ceng-411-design
    commit id: "Design-N3-Quarantine"
    commit id: "Design-Profile-Gates"
    checkout main
    commit id: "Hot-Path-Byte-Mask"
    branch feat-ceng-411-blocked
    checkout feat-ceng-411-blocked
    commit id: "Blocked-N3-Implementation"
```

---

## Lens 3: Type Kernel Ownership

Diagram ID: GITGRAPH-L3
Diagram family: GitGraph
Projection lens: Type Kernel Ownership
Architectural invariant preserved: Canonical type ownership across wasm4pm-compat, wasm4pm-cognition, bcinr-pddl, bcinr-powl, and praxis-graphlaw.
Information-loss risk if omitted: Concurrent branching creating duplicate type classes in different packages.
TPS visual-control purpose: Prevents rework and duplicate types across parallel team branches.
DfLSS CTQ protected: Zero duplicate type classes.
CENG ticket or boundary constrained: CENG-412 (design-only, implementation blocked).
Why this diagram is non-redundant: Shows ownership branch separations for key type kernels.

```mermaid
gitGraph
    commit id: "Types-Init"
    branch wasm4pm-compat
    checkout wasm4pm-compat
    commit id: "Define-Compat-Types"
    checkout main
    branch bcinr-pddl-powl
    checkout bcinr-pddl-powl
    commit id: "Define-PDDL-Types"
    checkout main
    merge wasm4pm-compat
    merge bcinr-pddl-powl
    commit id: "Block-CENG-412"
```

---

## Lens 4: Transition Lifecycle

Diagram ID: GITGRAPH-L4
Diagram family: GitGraph
Projection lens: Transition Lifecycle
Architectural invariant preserved: Every transition must pass through candidate invocation, validation, planning, execution, receipting, and replay.
Information-loss risk if omitted: Untracked and unvalidated lifecycle transitions.
TPS visual-control purpose: visualizes workflow completion gates before releases.
DfLSS CTQ protected: Guaranteed transaction replay validation under fixed seed.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Maps lifecycle milestone commits in the version control history.

```mermaid
gitGraph
    commit id: "Base-System"
    branch feat-lifecycle
    checkout feat-lifecycle
    commit id: "1-Invocation-Register"
    commit id: "2-OWL-SHACL-Validate"
    commit id: "3-BLAKE3-Receipt-Gen"
    commit id: "4-OWL-Time-Replay"
    checkout main
    merge feat-lifecycle
    commit id: "Release-v1.0"
```

---

## Lens 5: Event / Hook / Actuation

Diagram ID: GITGRAPH-L5
Diagram family: GitGraph
Projection lens: Event / Hook / Actuation
Architectural invariant preserved: Hooks cannot actuate without receipts; no unreceipted actuation.
Information-loss risk if omitted: Code allowing unreceipted events or direct hook invocation.
TPS visual-control purpose: Andon check verifying receipt tests prior to hook integration.
DfLSS CTQ protected: Zero unreceipted execution events.
CENG ticket or boundary constrained: CENG-416A-F (design-only, implementation blocked).
Why this diagram is non-redundant: Outlines git status of event-handling and hook features.

```mermaid
gitGraph
    commit id: "Event-Bus-Init"
    branch feat-ceng-416-design
    checkout feat-ceng-416-design
    commit id: "Design-OCEL-Ingest"
    commit id: "Design-Knowledge-Hooks"
    checkout main
    commit id: "Enforce-No-Unreceipted-Actuation"
    branch feat-ceng-416-blocked
    checkout feat-ceng-416-blocked
    commit id: "Blocked-Boundary-Actuator"
```

---

## Lens 6: Performance / 8-Constraint Hot Path

Diagram ID: GITGRAPH-L6
Diagram family: GitGraph
Projection lens: Performance / 8-Constraint Hot Path
Architectural invariant preserved: Maximum of 8 constraints checked in parallel via RDFTriple8 and ConditionCell<BITS>.
Information-loss risk if omitted: Uncontrolled growth of the constraint set size in mainline.
TPS visual-control purpose: Continuous integration performance gate monitoring.
DfLSS CTQ protected: Latency bound of hot path operations.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Visualizes optimization and hot-path integration commits.

```mermaid
gitGraph
    commit id: "Hotpath-Base"
    branch feat-perf-opt
    checkout feat-perf-opt
    commit id: "Lower-RDFTriple8-To-Mask"
    commit id: "256-State-Admission-Tables"
    commit id: "Verify-Latency-SLA"
    checkout main
    merge feat-perf-opt
```

---

## Lens 7: Refusal / Risk / Governance

Diagram ID: GITGRAPH-L7
Diagram family: GitGraph
Projection lens: Refusal / Risk / Governance
Architectural invariant preserved: Every failure is a typed Refusal; N3 quarantine rules are strictly enforced.
Information-loss risk if omitted: Committing code with generic panics or unmapped exceptions.
TPS visual-control purpose: Ensures audit gates prevent unhandled exceptions.
DfLSS CTQ protected: No panic or silent fallbacks.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Tracks safety-related code changes and refusal logic.

```mermaid
gitGraph
    commit id: "Err-Handling-Base"
    branch feat-safety-refusals
    checkout feat-safety-refusals
    commit id: "Typed-Refusal-Hierarchy"
    commit id: "Remove-Panics-Unwraps"
    commit id: "N3-Quarantine-Enforce"
    checkout main
    merge feat-safety-refusals
```

---

## Lens 8: TPS / DfLSS / Continuous Improvement

Diagram ID: GITGRAPH-L8
Diagram family: GitGraph
Projection lens: TPS / DfLSS / Continuous Improvement
Architectural invariant preserved: WIP reduction, continuous process improvement loops, and visual waste elimination.
Information-loss risk if omitted: Loss of tracking on optimization feedback loops.
TPS visual-control purpose: visualizes the main Kaizen improvement loop commits.
DfLSS CTQ protected: Flow efficiency and defect rate minimization.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Outlines Kaizen-driven git workflow.

```mermaid
gitGraph
    commit id: "Kaizen-Loop-Base"
    branch continuous-improvement
    checkout continuous-improvement
    commit id: "Reduce-Build-WIP"
    commit id: "Automate-Metrics-Collection"
    commit id: "Refine-Kaizen-Loops"
    checkout main
    merge continuous-improvement
```
