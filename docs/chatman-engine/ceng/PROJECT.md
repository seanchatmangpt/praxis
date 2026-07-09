# Chatman Engine (CENG) Project Scope Document

This document establishes the architecture, code layout, milestones, and interface contracts for the Chatman Engine (CENG) Manufacturing Run. The design and implementation are guided by the **Toyota Production System (TPS)** visual factory floor methodology and **Design for Lean Six Sigma (DfLSS)** quality principles, translating the 240-diagram design atlas into concrete, verifiable Rust implementations.

---

## 1. Architecture

The Chatman Engine utilizes a multi-layered, stateful orchestration structure. Execution transitions from initial boundary requests down to low-latency hot path checks, query routing constitution, workflow alignment, and final deterministic cryptographic receipting.

```
       +--------------------------------------------+
       |                  CE-ABI                    |
       |  (InvocationEnvelope -> ProcessReceipt)   |
       +--------------------+-----------------------+
                            |
                            v
       +--------------------+-----------------------+
       |           Warm Path Dialect Router         |
       |      (Least Expressive Route Solver)       |
       +--------------------+-----------------------+
                            |
                            | [Hot Path]
                            v
       +--------------------+-----------------------+
       |             8-Constraint Hot Path          |
       |  (ConditionCell<BITS <= 8>, PowlTape v2)   |
       +--------------------+-----------------------+
                            |
                            v
       +--------------------+-----------------------+
       |          Tape Planning/Workflow Bridge     |
       |     (Pddl8Tape + PowlTape -> Plan)         |
       +--------------------+-----------------------+
                            |
                            v
       +--------------------+-----------------------+
       |             Events / Hooks / Actuation     |
       |       (SPARQL CONSTRUCT -> Event Log)      |
       +--------------------+-----------------------+
                            |
                            v
       +--------------------+-----------------------+
       |              Agent / Breed Safety          |
       |         (DFA Breed Lifecycle Sandbox)       |
       +--------------------+-----------------------+
                            |
                            v
       +--------------------+-----------------------+
       |               Receipts & Replay            |
       |          (Deterministic BLAKE3 Ledgers)    |
       +--------------------------------------------+
```

### A. CE-ABI (Application Binary Interface)
The standard membrane for all incoming execution requests. It encapsulates call metadata, payload serialization, and authorization gates.
- **Request Formats**: Accepts `InvocationEnvelope` structs containing call identity (`InvocationId`), snapshot hashes (`GraphSnapshotId`), execution profiles (`ProfileId`), target operators (`OperatorId`), and handles (`InputHandles`).
- **Refusal Taxonomy**: Replaces raw unwraps and panic conditions. All boundary execution failures compile to a strongly-typed `Refusal` enum (such as `ValidationFailed`, `PlanInfeasible`, `TraceUnlawful`, `HookUnpermitted`, `SnapshotNotFound`, etc.) mapping to standard machine-readable JSON formats.

### B. Hot Path (RDFTriple8 & ConditionCell)
Optimized execution loop for low-latency workflow checks.
- **"Need9 means split" Law**: Enforces the compile-time capacity bounds from the Blue River Dam covenant. The hot-path conditions are bound to a maximum of 8 bits. Attempts to compile `ConditionCell<9>` trigger static compiler assertions.
- **Branchless Logic**: Executes Petri net transitions using SWAR (SIMD Within A Register) priority check masks.
- **Cache Alignment**: Aligns the 64-slot `PowlTape` (v2) structures to CPU cache lines, using flat `LabelSlab` arrays to achieve zero-heap allocations.

### C. Warm Path Dialect Router (LER Solver)
Enforces the least-expressive-power query routing constitution.
- **Complexity Assessment**: The Least Expressive Route (LER) constraint solver parses query complexity and triple count to categorize execution into `Hot`, `Warm`, or `Cold` path layers.
- **Quarantine Zone**: Quarantines high-expressivity Notation3 (N3) query execution by default. The cold-path N3 engine is accessible only under explicit, permission-gated profiles.

### D. Tape Planning / Workflow Bridge
Bridges linear planning actions with stateful Petri Net workflows.
- **Type Segregation**: Integrates planning and execution states without duplicating types across packages. Maps grounded PDDL sequences (`Pddl8Tape`) and POWL process graphs (`PowlTape` / `PowlTapeLarge`) to a unified `OrchestratedPlan`.
- **Stack Protection**: Large plan tapes (e.g. 512-slot `PowlTapeLarge` arrays) are boxed to prevent stack overflows during dispatching.

### E. Events / Hooks / Actuation
Manages dynamic knowledge triggers and graph delta projections.
- **Pure Projections**: Evaluates SPARQL CONSTRUCT queries, translating them to pure graph mutations (`kh:addQuad` and `kh:deleteQuad`). Prevents side-effects at the host level.
- **Causal Logs**: Emits causal logs from delta mutations, generating a BLAKE3-linked event chain (`OcelCausalReceipt`) where each frame is byte-aligned (128-byte `OcelCausalFrame`).

### F. Agent / Breed Safety
Enforces sandbox execution bounds for cognitive breed systems (Robinson Prolog, STRIPS, Hearsay Blackboards, and Minsky Frames).
- **Compliance Gating**: Leverages DFA-based sequence check models (`BreedLifecycleModel`) to execute 8 false-pass validation checks on candidate agents.
- **Cognition Receipt**: Successful runs emit a signed `CognitionReceipt` to audit sandbox integrity.

### G. Receipts & Replay
Validates historical execution trails to prevent state drift.
- **Unified Receipt**: Integrates multiple subsystem outputs into a single `ProcessReceipt` containing `run_id`, `pddl_receipt`, `powl_receipt`, `hook_receipts`, and `state_hash`.
- **Deterministic Replay**: Guarantees byte-identical receipt and state hash reconstruction across multiple executions under a fixed seed.

---

## 2. Code Layout

CENG source files and validation suites reside primarily in `crates/praxis-graphlaw/` (for engine orchestration and reasoning) and `crates/chatman-common/` (for shared tools and telemetry).

```
praxis/
├── crates/
│   ├── praxis-graphlaw/                 # Core Graph Reasoner & Orchestrator
│   │   ├── src/
│   │   │   ├── lib.rs                   # Crate entrypoint & TripleStore exports
│   │   │   ├── abi.rs                   # CE-ABI Request Envelope & Refusals
│   │   │   ├── engine.rs                # ChatmanEngine stateful pipeline gater
│   │   │   ├── profile_gates.rs         # LER Solver & query complexity router
│   │   │   ├── bridge/
│   │   │   │   └── mod.rs               # PDDL-to-POWL tape bridge mappings
│   │   │   ├── hooks/
│   │   │   │   ├── mod.rs               # Knowledge hook registry & execution
│   │   │   │   └── construct.rs         # CONSTRUCT delta projections & receipts
│   │   │   └── replay.rs                # Deterministic transaction replay
│   │   └── benches/
│   │       └── dialects.rs              # Warm path routing benchmarks
│   │
│   ├── chatman-common/                  # Shared Services & Telemetry Infrastructure
│   │   ├── src/
│   │   │   ├── lib.rs                   # Common exports
│   │   │   ├── error.rs                 # Engine-wide structured errors
│   │   │   ├── chain.rs                 # RollingChain BLAKE3 audit trail logs
│   │   │   ├── signed_receipt.rs        # SignedReceipt & KeyPair generation
│   │   │   ├── telemetry.rs             # TPS factory metrics and telemetry
│   │   │   └── cli.rs                   # CLI commands & management utilities
│   │
│   └── ocel/                            # OCEL Event schema & validation definitions
│
├── docs/
│   └── chatman-engine/
│       └── ceng/                        # CENG board and engineering documentation
│           ├── PROJECT.md               # [THIS FILE] Global scope
│           ├── CENG-410-FINAL.md        # Audited types & invariants
│           └── CENG-board.md            # Active ticket index & design mappings
```

### External Declared Dependencies

To prevent type compilation collisions and preserve strict ownership, the following external packages are designated as canonical domain owners:
- **`wasm4pm-compat`** (`/Users/sac/wasm4pm-compat/`): Owns `ConditionCell<BITS>`, `Between01`, `Evidence<T>`, and base typestate progress invariants.
- **`bcinr-powl`** (`../bcinr/crates/bcinr-powl/`): Owns `PowlTape` (v1/v2), `PowlTapeLarge`, `PowlPetriState`, and hot branchless Petri scheduling.
- **`bcinr-pddl`** (`../bcinr/crates/bcinr-pddl/`): Owns PDDL8 grounding, STRIPS parsing, and durative STRIPS plan compilation.
- **`bcinr-powl-receipt`** (`../bcinr/crates/bcinr-powl-receipt/`): Owns `OcelCausalReceipt` and `OcelCausalFrame` allocators.
- **`wasm4pm-cognition`** (`../wasm4pm/crates/wasm4pm-cognition/`): Owns old-AI breeds, `BreedLifecycleModel`, and sandbox validators.

---

## 3. Milestones Table

| Milestone | Target Tickets / Scope | Status | Description |
|---|---|---|---|
| **Milestone 1: Design Audit** | CENG-410-FINAL | **PARTIAL** | Design artifacts only: certification of type ownership, invariant classification, and design validation across kernel boundaries. |
| **Milestone 2: Architecture & Board** | CENG-board | **PARTIAL** | Design artifacts only: visual atlas mapping sheets, Gantt control paths, and CENG implementation ticket definitions. |
| **Milestone 3: Hot Path & ABI** | CENG-411, CENG-416A-F | **PLANNED** | Implement `InvocationEnvelope` request gates, `ConditionCell` compile-time checks, and SWAR branchless Petri engines. |
| **Milestone 4: Dialect Router** | CENG-420+ | **PLANNED** | Build the Least Expressive Route (LER) complexity classifier and enforce N3 quarantine checks. |
| **Milestone 5: Tape Bridge** | CENG-412 | **PLANNED** | Implement PDDL planning action mapping onto POWL process execution graphs without type duplication. |
| **Milestone 6: Events & Hooks** | CENG-460+ | **PLANNED** | Integrate SPARQL CONSTRUCT knowledge hooks, delta projections, and rolling BLAKE3 causal event receipt chaining. |
| **Milestone 7: Breed Safety** | CENG-480+ | **PLANNED** | Establish cognitive breed sandboxing, Prolog8 verification, and Minsky frame lifecycle validators. |
| **Milestone 8: Receipts & Replay** | CENG-500+ | **PLANNED** | Build unified `ProcessReceipt` ledgers and implement deterministic transaction replay auditing. |
| **Milestone 9: E2E Verification** | E2E Audit | **PLANNED** | Execute complete multi-perspective plan replay under fixed seeds; perform independent forensic audits. |

Authoritative implementation track: PROJ-411..414 plus the v26.7.9 workflow (wf_255e0807) are
the sole authoritative implementation track for the Chatman Engine; the external CENG-4xx
Gemini workflow is dead.

---

## 4. Interface Contracts

Below are the formal Rust struct definitions, enum variants, and function signatures governing the boundaries of the Chatman Engine.

### A. CE-ABI Ingress Envelopes & Refusals
Located in `crates/praxis-graphlaw/src/abi.rs`.

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InvocationId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GraphSnapshotId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProfileId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperatorId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InputHandles {
    pub nodes: Vec<String>,
    pub events: Vec<String>,
    pub plan_steps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvocationEnvelope {
    pub invocation_id: InvocationId,
    pub snapshot_id: GraphSnapshotId,
    pub profile_id: ProfileId,
    pub operator_id: OperatorId,
    pub input_handles: InputHandles,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Refusal {
    /// SHACL/ShEx or graph structural validation failed
    ValidationFailed(String),
    /// PDDL planner could not find a feasible plan or action is invalid
    PlanInfeasible(String),
    /// POWL trace admission rejected the workflow step
    TraceUnlawful(String),
    /// Knowledge hook conditions were not met
    HookUnpermitted(String),
    /// The final receipt could not be proven or generated
    MissingReceipt(String),
    /// The required graph snapshot was missing
    SnapshotNotFound(String),
    /// An error occurred fetching derived facts
    FactNotDerived(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Receipt {
    pub blake3_hash: String,
    pub canon_nquads: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdmittedTransition {
    pub receipt: Receipt,
}
```

### B. Core Stateful Orchestrator
Located in `crates/praxis-graphlaw/src/engine.rs`.

```rust
use oxigraph::store::Store;
use oxigraph::model::NamedNode;

pub struct ChatmanEngine {
    store: Store,
}

impl ChatmanEngine {
    /// Initializes the orchestrator with an Oxigraph semantic triple store.
    pub fn new(store: Store) -> Self;

    /// Admitts an incoming transition. Executes the complete validation pipeline:
    /// Ingress -> OWL Closure -> PDDL Planning -> POWL Conformance -> Hook Actuation -> Receipting.
    pub fn admit_transition(
        &mut self,
        invocation: InvocationEnvelope,
    ) -> Result<AdmittedTransition, Refusal>;

    /// Internal validation phases
    fn fetch_snapshot(&self, invocation: &InvocationEnvelope) -> Result<NamedNode, Refusal>;
    fn apply_owl_closure(&self, graph_name: &NamedNode) -> Result<(), Refusal>;
    fn generate_pddl_plan(&self, graph_name: &NamedNode) -> Result<(), Refusal>;
    fn admit_powl_trace(&self, graph_name: &NamedNode) -> Result<(), Refusal>;
    fn trigger_knowledge_hooks(&self, graph_name: &NamedNode) -> Result<(), Refusal>;
    fn generate_receipt(&self, graph_name: &NamedNode) -> Result<Receipt, Refusal>;
}
```

### C. Compile-Time Law Constraints
Located in `/Users/sac/wasm4pm-compat/src/law.rs`.

```rust
// Helper markers for compile-time const checks
pub trait IsTrue {}
pub struct Require<const B: bool>;
impl IsTrue for Require<true> {}

/// ConditionCell constraints capacity. Need9 means split (BITS <= 8).
pub struct ConditionCell<const BITS: usize>
where
    Require<{ BITS <= 8 }>: IsTrue,
{
    _private: (),
}

/// Represents a compile-time metric ratio in [0, 1].
pub struct Between01<const NUMERATOR: usize, const DENOMINATOR: usize>
where
    Require<{ DENOMINATOR > 0 && NUMERATOR <= DENOMINATOR }>: IsTrue;
```

### D. Tape Planning & Conformance Mapping
Located in `crates/praxis-graphlaw/src/bridge/mod.rs` and external crates.

```rust
/// Grounded sequential planning tape (declared in wasm4pm-core/src/pddl.rs)
pub struct Pddl8Tape {
    pub ops: Vec<Pddl8TapeOp>,
}

/// Aligned 64-slot POWL workflow tape (declared in bcinr-powl/src/tape.rs)
pub struct PowlTape {
    pub ops: [Powl64Op; 64],
    pub entry_op: u8,
    pub exit_op: u8,
    pub label_slab: LabelSlab,
}

/// Unified process execution receipt bridging planning, workflow, and hooks.
pub struct ProcessReceipt {
    pub run_id: u64,
    pub pddl_receipt: Pddl8ExecutionReceipt,
    pub powl_receipt: bcinr_powl::typestate::Receipt,
    pub hook_receipts: Vec<HookReceipt>,
    pub state_hash: [u8; 32],
}

/// Maps planning sequence steps directly to POWL places and transitions.
pub struct OrchestratedPlan {
    pub pddl_step_idx: u32,
    pub powl_op_idx: u8,
    pub mapping_rationale: String,
}

pub trait TapeBridge {
    /// Maps a grounded PDDL tape onto a compiled POWL tape.
    fn map_to_workflow(
        pddl_tape: &Pddl8Tape,
        powl_tape: &PowlTape,
    ) -> Result<OrchestratedPlan, Refusal>;
}
```

### E. Events, Causal Receipting & Chaining
Located in `crates/praxis-graphlaw/src/hooks/construct.rs` and `bcinr-powl-receipt`.

```rust
/// Knowledge Hook delta projection receipt
pub struct HookReceipt {
    pub hook_id: String,
    pub pre_state_hash: [u8; 32],
    pub post_state_hash: [u8; 32],
    pub delta_quads_hash: [u8; 32],
}

/// Raw byte-aligned 128-byte causal frame representing one execution step
#[repr(align(16))]
pub struct OcelCausalFrame {
    pub event_id: u64,
    pub parent_hash: [u8; 32],
    pub frame_bytes: [u8; 88],
}

/// Rolling BLAKE3-linked receipt generator
pub struct OcelCausalReceipt {
    pub current_hash: [u8; 32],
    pub frame_count: u64,
}

impl OcelCausalReceipt {
    /// Appends a frame to the causal chain, updating the state hash using BLAKE3.
    /// chain_hash(t+1) = BLAKE3(chain_hash(t) || frame_bytes)
    pub fn append_frame(&mut self, frame: &mut OcelCausalFrame) -> [u8; 32];
}
```
