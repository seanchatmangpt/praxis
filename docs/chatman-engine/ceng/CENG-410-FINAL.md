# CENG-410-FINAL — Chatman Engine Kernel Ownership & Design Validation

This document certifies the final type ownership, invariant classification, and design validation for the Chatman Engine Manufacturing Run across five target packages: `wasm4pm-compat`, `bcinr-pddl`, `bcinr-powl`, `wasm4pm-cognition`, and `praxis-graphlaw`.

---

## 1. Exact Symbol Inventory

The audited workspace defines and utilizes the following public and crate-visible types across the kernel boundaries:

| Symbol Name | Declaring Crate | Declaring File | Symbol Kind | Purpose / Description |
|---|---|---|---|---|
| `ConditionCell` | `wasm4pm-compat` | `src/law.rs` | `struct` | Compile-time law kernel constraining condition cell bits (BITS ≤ 8). |
| `Between01` | `wasm4pm-compat` | `src/law.rs` | `struct` | Compile-time rational metric bound in `[0, 1]` at the type level. |
| `NormedBetween01` | `wasm4pm-compat` | `src/law.rs` | `type` | Normalized ratio reduced by GCD to ensure unique type representation. |
| `EvidenceMode` | `wasm4pm-compat` | `src/law.rs` | `enum` | Const-generic type-level markers for the admission lifecycle. |
| `Evidence` | `wasm4pm-compat` | `src/evidence.rs` | `struct` | Universal process-evidence carrier parameterized by State and Witness. |
| `Pddl8Tape` | `wasm4pm-core` | `src/pddl.rs` | `struct` | Sequential plan tape representation (re-exported by `wasm4pm-compat` and `bcinr-pddl`). |
| `Pddl8ExecutionReceipt` | `wasm4pm-core` | `src/pddl.rs` | `struct` | Verification receipt for PDDL8 plans. |
| `TemporalExecutionReceipt`| `wasm4pm-core` | `src/pddl.rs` | `struct` | Verification receipt for durative-STRIPS plans. |
| `PowlTape` (v1) | `bcinr-powl` | `src/tape.rs` | `struct` | Flat 64-slot op array for POWL execution. |
| `PowlTape` (v2) | `bcinr-powl` | `src/tape.rs` | `struct` | Aligned 64-slot op array utilizing `LabelSlab` and `entry_op`/`exit_op`. |
| `PowlTapeLarge` | `bcinr-powl` | `src/tape.rs` | `struct` | Flat 512-slot op array with `[u64; 8]` bitmasks for large programs. |
| `PowlPetriState` | `bcinr-powl` | `src/scheduler_wired.rs` | `struct` | Petri net marking hot state utilizing KBitSets. |
| `PetriTickResult` | `bcinr-powl` | `src/scheduler_wired.rs` | `struct` | Fired transitions mask and event overflow counts returned by `petri_tick`. |
| `Receipt` | `bcinr-powl` | `src/typestate.rs` | `struct` | Immutable POWL execution receipt carrying `op_trace` and `topo_order`. |
| `OcelCausalReceipt` | `bcinr-powl-receipt`| `src/causal_receipt.rs` | `struct` | Rolling BLAKE3-linked receipt generator for POWL runs. |
| `OcelCausalFrame` | `bcinr-powl-receipt`| `src/causal_receipt.rs` | `struct` | Raw byte-aligned 128-byte causal frame representing one step. |
| `C8Receipt` | `wasm4pm-compat` | `c8-receipts/src/receipt.rs` | `struct` | Provenance-bearing evidence receipt with state hashes. |
| `HookReceipt` | `praxis-graphlaw` | `src/hooks/construct.rs`| `struct` | Graph delta execution receipt matching SPARQL CONSTRUCT results. |
| `TripleStore` | `praxis-graphlaw` | `src/lib.rs` | `struct` | In-memory / Oxigraph-backed RDF store executing SPARQL/Datalog rules. |
| `ChatmanEngine` | `praxis-graphlaw` | `src/engine.rs` | `struct` | Top-level orchestrator executing the bottom-up manufacture stack. |
| `InvocationEnvelope` | `praxis-graphlaw` | `src/abi.rs` | `struct` | Top-level Request envelope entering the CE-ABI membrane. |
| `Receipt` (CE-ABI) | `praxis-graphlaw` | `src/abi.rs` | `struct` | Execution signature carrying `blake3_hash` and canonical N-Quads. |
| `Refusal` (CE-ABI) | `praxis-graphlaw` | `src/abi.rs` | `enum` | CE-ABI structured refusal variant for validation or planner failures. |
| `CognitionReceipt` | `wasm4pm-cognition`| `src/registry.rs` | `struct` | Ledgered execution receipt for old-AI cognition runs. |
| `BreedLifecycleModel` | `wasm4pm-cognition`| `src/ocel/mod.rs` | `struct` | DFA-based sequence check model for Minsky frame execution logs. |

---

## 2. Canonical Ownership Table

To ensure structural boundary integrity, each subsystem is mapped to its canonical declaring crate. Subsystems must not cross-declare or re-define types belonging to other domains.

| Domain/Subsystem | Declaring Crate | Code Location | Ownership Rule |
|---|---|---|---|
| **Base Typestates & Laws** | `wasm4pm-compat` | `src/{law, state, evidence}.rs` | Defines universal constraints, compile-time bounds, and the `Evidence` carrier. |
| **BPMN & Declare Schemas** | `wasm4pm-core` | `wasm4pm-core/src/{bpmn, declare}.rs` | Declares base structures, re-exported by `wasm4pm-compat`. |
| **PDDL8 Planning Types** | `wasm4pm-core` | `wasm4pm-core/src/pddl.rs` | Declares canonical STRIPS and Durative planning types. |
| **PDDL8 Compiler/Planner** | `bcinr-pddl` | `bcinr-pddl/src/` | Parsing, grounding, and temporal plan execution. |
| **POWL Tapes & Compile** | `bcinr-powl` | `bcinr-powl/src/{tape, compiler}.rs` | AST compilation, Kahn walk check, and ready-set bitmask calculations. |
| **POWL Execution / Petri** | `bcinr-powl` | `bcinr-powl/src/{scheduler_wired, typestate}.rs` | Hot-path branchless Petri scheduling and execution state tokens. |
| **Causal Frame Emission** | `bcinr-powl-receipt`| `bcinr-powl-receipt/src/` | Alloc-counter, arena allocation, and causal receipt chaining. |
| **Old-AI Cognition Breeds** | `wasm4pm-cognition`| `wasm4pm-cognition/src/breeds/` | Robinson Prolog, STRIPS planer, Hearsay blackboard, Minsky frames. |
| **Adversarial Detectors** | `wasm4pm-cognition`| `wasm4pm-cognition/src/autosystems/adversarial/`| Enforces the 8 false-pass validation checks on candidates. |
| **RDF Store / SPARQL** | `praxis-graphlaw` | `praxis-graphlaw/src/` | Oxigraph store interface, SPARQL, SHACL/ShEx, and Datalog reasoning. |
| **Engine Orchestration** | `praxis-graphlaw` | `src/engine.rs` | Stateless bottom-up pipeline gater executing snapshot-to-receipt steps. |

---

## 3. Duplicate-Forbidden Table

To prevent compilation type mismatches, binary alignment faults, and duplicate serialization interfaces, the following symbols are declared **Duplicate-Forbidden**. They must reside only in their canonical owner crates:

| Type Name | Canonical Crate | Duplicated Location(s) | Risk of Duplication | Remediation Decision |
|---|---|---|---|---|
| `OcelEvent` | `wasm4pm-compat` (`OCELEvent`) | 1. `wasm4pm-cognition/src/ocel/mod.rs` <br> 2. `bcinr-powl/src/ocel.rs` | Compile-time type mismatch and name collisions. `bcinr-powl`'s version is flat (64-op), while `cognition`'s is serialization-oriented JSON. | Quarantined to separate namespaces. `bcinr-powl`'s flat structures must remain internal to the conformance verifier and never bleed into public APIs. |
| `OcelLog` / `OCEL` | `wasm4pm-compat` (`OCEL`) | 1. `wasm4pm-cognition/src/ocel/mod.rs` <br> 2. `bcinr-powl/src/ocel.rs` | Serialization errors when crossing boundary layers. | Use `wasm4pm-compat::ocel::OCEL` as the canonical format for external interop. |
| `ConformanceResult` | `bcinr-powl` | `wasm4pm-cognition/src/ocel/mod.rs` | Direct name collision in execution pipelines. `bcinr-powl` uses an enum for trace checks, while `cognition` uses a struct for DFA checks. | Maintain distinct names: `bcinr_powl::ocel::ConformanceResult` vs `wasm4pm_cognition::ocel::BreedConformanceResult`. |
| `Refusal` | `wasm4pm-compat` | `praxis-graphlaw/src/abi.rs` | Concept pollution. `wasm4pm-compat::admission::Refusal` is a compile-time templated witness, while `praxis_graphlaw::abi::Refusal` is an application ABI enum. | Re-export the ABI enum as `EngineRefusal` to prevent namespace collision at the orchestration boundary. |

---

## 4. Missing-Type Table

The following orchestration types are **missing** from the audited repositories, preventing a fully stateful, unified execution loop:

| Missing Type | Suggested Declaring Crate | Description / Purpose | Interface / Fields |
|---|---|---|---|
| `ProcessReceipt` | `praxis-graphlaw::abi` | A unified, multi-perspective process execution receipt containing planning, workflow, and hook proofs. | `run_id: u64`, `pddl_receipt: Pddl8ExecutionReceipt`, `powl_receipt: Receipt`, `hook_receipts: Vec<HookReceipt>`, `state_hash: [u8; 32]` |
| `OrchestratedPlan` | `praxis-graphlaw::engine` | Represents the joint plan mapping step-to-step, linking PDDL actions directly onto POWL workflow places. | `pddl_step_idx: u32`, `powl_op_idx: u8`, `mapping_rationale: String` |
| `UnifiedAdmissionSession`| `praxis-graphlaw::engine` | Stateful execution controller tracking linear progression of a workflow over multiple transactions. | `session_id: InvocationId`, `active_runner: PowlRunner`, `current_world_state: GroundProblem`, `prior_hash: [u8; 32]` |

---

## 5. Bridge / Wrap / Extend Decisions

1. **Bridge Decisions**:
   - Re-export `wasm4pm-core` types (like `Pddl8Tape` and `Pddl8GroundAction`) in `wasm4pm-compat` to maintain compatibility footprints without code duplication.
   - Cross-crate dependencies must follow the linear stack direction: `wasm4pm-compat` → `bcinr-powl` / `bcinr-pddl` → `wasm4pm-cognition` → `praxis-graphlaw`.
2. **Wrap Decisions**:
   - Wrap internal scheduler execution tokens inside `praxis_graphlaw::abi::InvocationEnvelope` handles to keep the hot path isolated from the high-expressivity SPARQL reasoner.
   - Box the flat 512-op `PowlTapeLarge` arrays to avoid stack overflows inside the thread dispatcher.
3. **Extend Decisions**:
   - Create a unified `ProcessReceipt` (as outlined in the Missing-Type Table) in the upcoming orchestration milestones (`CENG-413`) to bridge hook deltas with planning state transitions.

---

## 6. Exact Evidence for `ConditionCell<BITS>`

* **Crate**: `wasm4pm-compat`
* **Source File**: `/Users/sac/wasm4pm-compat/src/law.rs`
* **Line Range**: 92–133
* **Implementation Details**:
  ```rust
  pub struct ConditionCell<const BITS: usize>
  where
      Require<{ BITS <= 8 }>: IsTrue,
  {
      _private: (),
  }
  ```
  Enforces the *Need9 means split* law from the Blue River Dam covenant. Any attempt to initialize `ConditionCell<9>` results in a compile-time failure: `the trait bound 'Require<false>: IsTrue' is not satisfied`.

---

## 7. Exact Evidence for `Pddl8Tape`

* **Crate**: `wasm4pm-core` (re-exported by `wasm4pm-compat` and `bcinr-pddl`)
* **Source File**: `/Users/sac/wasm4pm-compat/wasm4pm-core/src/pddl.rs`
* **Line Range**: 235–255
* **Implementation Details**:
  ```rust
  pub struct Pddl8Tape {
      pub ops: Vec<Pddl8TapeOp>,
  }
  ```
  Encodes a grounded plan as a sequential array of operations, mapping their dependencies via a `pred_mask: u64` bitmask.

---

## 8. Exact Evidence for `PowlTape`

* **Crate**: `bcinr-powl`
* **Source File**: `/Users/sac/bcinr/crates/bcinr-powl/src/tape.rs`
* **Line Range**: 63–97 (v1) and 321–381 (v2)
* **Implementation Details**:
  - **v1**: Flat array `ops: [Powl64Op; 64]` and an `entry_mask: u64` specifying ready entry states.
  - **v2**: Cache-line aligned struct `PowlTape` containing `ops: [Powl64Op; 64]`, `entry_op: u8`, `exit_op: u8`, and a `LabelSlab` for zero-heap label interning.

---

## 9. Exact Evidence for `ProcessReceipt`

* **Audit Status**: **MISSING / NON-EXISTENT**.
* **Direct Evidence**: A comprehensive case-insensitive search across all workspace files (`wasm4pm-compat`, `bcinr-pddl`, `bcinr-powl`, `wasm4pm-cognition`, and `praxis-graphlaw`) confirms that **no symbol named `ProcessReceipt` is declared**.
* **Audit Finding**: Currently, process execution is validated via discrete, disconnected receipts:
  - `Pddl8ExecutionReceipt` (in `wasm4pm-core/src/pddl.rs:293`)
  - `TemporalExecutionReceipt` (in `wasm4pm-core/src/pddl.rs:709`)
  - `Receipt<const KIND: TopologyKind>` (in `bcinr-powl/src/typestate.rs:349`)
  - `OcelCausalReceipt` (in `bcinr-powl-receipt/src/causal_receipt.rs:161`)
  - `C8Receipt` (in `wasm4pm-compat/c8-receipts/src/receipt.rs:17`)
  - `HookReceipt` (in `praxis-graphlaw/src/hooks/construct.rs:12`)
  - `CognitionReceipt` (in `wasm4pm-cognition/src/registry.rs:18`)
  - `Receipt` (in `wasm4pm-compat/src/receipt.rs:1531`)
  - `GraduationReceipt` (in `wasm4pm-compat/src/receipt.rs:809`)
  - `ReceiptRecord` (in `praxis-core/src/receipt_record.rs`)

---

## 10. Exact Evidence for Domain Ownership

### A. OCEL Domain
- **Canonical Schema**: `wasm4pm-compat::ocel::OCEL` in `/Users/sac/wasm4pm-compat/src/ocel.rs:35`.
- **Low-Allocation Conformance**: `bcinr_powl::ocel::OcelLog` in `/Users/sac/bcinr/crates/bcinr-powl/src/ocel.rs:22`.
- **Cognition Serializer**: `wasm4pm_cognition::ocel::OcelLog` in `/Users/sac/wasm4pm/crates/wasm4pm-cognition/src/ocel/mod.rs:42`.

### B. POWL Domain
- **AST Schema**: `wasm4pm_compat::powl::PowlNode` in `/Users/sac/wasm4pm-compat/src/powl.rs:85`.
- **Compiled Tape**: `bcinr_powl::tape::PowlTape` in `/Users/sac/bcinr/crates/bcinr-powl/src/tape.rs:63`.
- **POWL Compiler**: `bcinr_powl::compiler::compile_powl` in `/Users/sac/bcinr/crates/bcinr-powl/src/compiler.rs:407`.

### C. Petri Domain
- **Petri Net Shape**: `wasm4pm_compat::petri::PetriNet` in `/Users/sac/wasm4pm-compat/src/petri.rs:46`.
- **Wired Executor State**: `bcinr_powl::scheduler_wired::PowlPetriState` in `/Users/sac/bcinr/crates/bcinr-powl/src/scheduler_wired.rs:126`.
- **SWAR Priority Core**: `bcinr_logic::swar_petri::PriorityPetriEngine` in `/Users/sac/bcinr/crates/bcinr-logic/src/`.

### D. DFG Domain
- **DFG Shape**: `wasm4pm_compat::dfg::DFG` in `/Users/sac/wasm4pm-compat/src/dfg.rs:16`.

### E. Causality Domain
- **Causal Consistency**: `wasm4pm_compat::causality::CausalConsistency` in `/Users/sac/wasm4pm-compat/src/causality.rs:72`.
- **Causal Chaining Receipt**: `bcinr_powl_receipt::causal_receipt::OcelCausalReceipt` in `/Users/sac/bcinr/crates/bcinr-powl-receipt/src/causal_receipt.rs:161`.

### F. Witness-Law Domain
- **Witness Verification Law**: `wasm4pm_compat::witness_law::WitnessLaw` in `/Users/sac/wasm4pm-compat/src/witness_law.rs:17`.
- **Witness Bibliography**: `wasm4pm_compat::witness_corpus::WitnessCorpus` in `/Users/sac/wasm4pm-compat/src/witness_corpus.rs:32`.

### G. Cognitive-Breed Domain
- **Robinson Prolog**: `wasm4pm_cognition::breeds::prolog::PrologBreed` in `/Users/sac/wasm4pm/crates/wasm4pm-cognition/src/breeds/prolog.rs`.
- **STRIPS Planner**: `wasm4pm_cognition::breeds::strips::StripsPlanner` in `/Users/sac/wasm4pm/crates/wasm4pm-cognition/src/breeds/strips.rs`.
- **Hearsay Blackboard**: `wasm4pm_cognition::breeds::hearsay::Blackboard` in `/Users/sac/wasm4pm/crates/wasm4pm-cognition/src/breeds/hearsay.rs`.
- **Minsky Frames**: `wasm4pm_cognition::breeds::frame::Frame` in `/Users/sac/wasm4pm/crates/wasm4pm-cognition/src/breeds/frame.rs`.

---

## 11. Invariant Classification Table

The engine enforces its safety covenants at different stages, categorized as follows:

| Classification | Invariant Name | Enforcing System | Description |
|---|---|---|---|
| **Compile-time** | *Need9 means split* | `wasm4pm-compat` | `ConditionCell<BITS>` restricts maximum bit capacity to 8. |
| | *Rational bounded metric* | `wasm4pm-compat` | `Between01<N, D>` restricts metric ratios to the `[0, 1]` domain. |
| | *Typestate progression* | `wasm4pm-compat` | `Evidence<T, Raw>` and `Evidence<T, Admitted>` are distinct types. |
| | *Crate isolation* | Cargo Configuration | `bcinr-pddl` has zero path dependencies on `bcinr-powl` or `wasm4pm-cognition`. |
| **Runtime** | *Acyclic Workflow* | `bcinr-powl::compiler` | Kahn walk check fails on cyclic POWL graph input. |
| | *Transition Firing* | `bcinr-powl::scheduler` | `petri_tick` transition enablement checked branchlessly. |
| | *PDDL Grounding Match* | `bcinr-pddl::ground` | Domain variables validated against problem instances. |
| | *Hook Permission* | `praxis-graphlaw::engine` | Ask query on Oxigraph store gates triggered hook delta projection. |
| **Receipt-based**| *BLAKE3 Chaining* | `bcinr-powl-receipt` | `chain_hash(t+1) = BLAKE3(chain_hash(t) \|\| frame_bytes)`. |
| | *Contiguity* | `c8-receipts::chain` | `receipt_i.post_state == receipt_i+1.pre_state`. |
| | *Monotonicity* | `c8-receipts::chain` | `receipt_i.causal_time < receipt_i+1.causal_time`. |
| | *Canonical N-Quads Hash*| `praxis-graphlaw::engine` | Hashing sorted N-Quads produces a deterministic state signature. |
| **Aspirational**| *100% Machine Evidence* | Autonomic Platform | Every command in `wpm` must verify matching execution logs (partially mocked). |
| | *Zero Wall-Clock time* | Core Team Discipline | Time derived exclusively from graph OWL-Time (visualizer still imports Chrono). |

---

## 12. Revised CENG Board Dependency Graph

The following Mermaid graph visualizes the ticket dependencies for the Chatman Engine Manufacturing Run:

```mermaid
graph TD
    CENG-410[CENG-410: Kernel Audit & Validation] --> CENG-411[CENG-411: Type Boundary Isolation (Design)]
    CENG-410 --> CENG-412[CENG-412: Actuation & Hook Receipting (Design)]
    CENG-411 --> CENG-413[CENG-413: Chatman Engine Core Pipeline (Implementation)]
    CENG-412 --> CENG-413
    CENG-413 --> CENG-414[CENG-414: Conformance checking & Petri execution]
    CENG-413 --> CENG-416[CENG-416: Telemetry, Benchmarks & continuous Kaizen]
```

---

## 13. Certificate of Audit Completion

* **Auditor**: `worker_ceng410_1` (Kernel Auditor, Manufacturing Run)
* **Status**: **PARTIAL — design approved; implementation in progress (workflow wf_255e0807); src/chatman/ landing now**
* **Verification Method**: Checked all definitions via `grep_search` and `view_file` to establish the exact symbol and crate mappings. Verified compilation constraints (`ConditionCell<9>` compilation failure trybuild checks) and dependency layouts.
