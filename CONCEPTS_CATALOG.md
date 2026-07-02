# Concept Catalog: Reusable Design Patterns Across SPARC Ecosystem

## 1. Audit & Compliance Frameworks

**Cluster:** Receipt-backed systems, conformance gates, evidence traces.

| Pattern | Source Repo | Core Idea | Novelty |
|---------|------------|----------|---------| 
| **Receipt Chain with Hash Pointers** | affidavit | Cryptographic ed25519-signed receipts linking inputs→outputs; tamper-evident audit trail via blake3 chaining. | ★★★ High |
| **Type-Gated Workflow Enforcement** | affidavit, chicago-tdd-tools | Rust type system encodes mandatory ordering (e.g., `AdmittedReceipt` forces admission before discovery). Compile-time preconditions. | ★★★ High |
| **Law Object Pattern** | chatmangpt | Obligations as hashable, dispatchable first-class values encoding preconditions, blocking constraints, evidence requirements. Enables deterministic judgment. | ★★★ High |
| **Evidence Gate Invariants (E1–E7)** | cargo-cicd | Formal system separating concerns: cargo-cicd collects evidence; external oracle (wasm4pm) adjudicates. Prevents self-judgment. | ★★★ High |
| **Andon Defect Signaling** | chatmangpt, a2a-rs | Production-line halting system: defects trigger line stop; clearing requires receipt or operator override. 11 defect categories. | ★★ Medium |

**Most Novel:** Law object pattern (chatmangpt) — unifies obligational state, cryptographic identity, and deterministic judgment into a single, auditable construct.

---

## 2. Code Generation from Specifications

**Cluster:** Ontology-driven artifact generation, RDF as source of truth.

| Pattern | Source Repo | Core Idea | Novelty |
|---------|------------|----------|---------| 
| **CONSTRUCT Query Ontology Pipeline** | a2a-rs | SPARQL CONSTRUCT (not SELECT) as minimal-entropy representation; three-way validation (spec ↔ ontology ↔ types). | ★★★ High |
| **Noun-Verb Grammar Manufacturing** | cargo-cicd | Entire CLI surface manufactured from RDF ontology; default verb injection at runtime. Ensures spec ↔ CLI sync. | ★★★ High |
| **Bidirectional RDF ↔ Code Sync** | clap-noun-verb | Code→RDF (extract ontology from Rust signatures) + RDF→Code (generate Rust from specs). Live synchronization. | ★★★ High |
| **Ontology-Driven LSP Generation** | claude-code-config-lsp | RDF (claude-code-config.ttl) as single source; SPARQL→Tera renders hover docs, completions, diagnostics, analyzers in lockstep. | ★★ Medium |
| **Multi-Protocol Binding via Layer** | A2A | Proto3 + ProtoJSON + abstraction layer; new protocol bindings without core changes. Specification, operations, bindings separated. | ★★ Medium |

**Most Novel:** CONSTRUCT pipeline (a2a-rs) — information-theoretic framing elevates codegen from mechanical to mathematically principled.

---

## 3. State & Type-Safety Enforcement

**Cluster:** Phantom types, typestates, immutability, aggregate roots.

| Pattern | Source Repo | Core Idea | Novelty |
|---------|------------|----------|---------| 
| **Typestate with Phantom Types** | cargo-cicd | EngineState<Pending/Adjudicated> enforces state transitions at compile time. 11-dimensional aggregate root. | ★★★ High |
| **Seal Pattern (Private Seals)** | affidavit | Unconstructable receipt via private `_seal: ()` field; forces single canonical seam (ChainAssembler::finalize). | ★★ Medium |
| **Type-Safe DI with TypeMap** | clap-noun-verb | TypeId-keyed HashMap for heterogeneous, zero-reflection dependency injection. No string keys, compile-time safety. | ★★ Medium |
| **Dual-Faceted Receipt** | affidavit | Same bytes serve as OCEL event log AND cryptographic proof. Shape-B fusion prevents split accountability. | ★★★ High |
| **Evidence Lifecycle (Raw→Admitted→Receipted)** | chicago-tdd-tools | Three-state type-level pattern enforcing one-way cryptographic sealing of evidence. | ★★ Medium |

**Most Novel:** Dual-faceted receipt (affidavit) — elegant unification of process mining and cryptographic proof in the same artifact.

---

## 4. Performance & Deterministic Scheduling

**Cluster:** Branchless code, compile-time scheduling, zero-allocation data structures.

| Pattern | Source Repo | Core Idea | Novelty |
|---------|------------|----------|---------| 
| **Branchless Calculus (B-Calculus)** | bcinr | Transform conditionals into arithmetic/bitwise ops; Hoare-logic proofs in comments; cyclomatic complexity = 1. | ★★★ High |
| **SWAR-Marking Petri Nets** | bcinr | Petri net state as u64 bitmask; transitions as bitwise ops; ~20 cycle WCET per step. Autonomic control planes. | ★★★ High |
| **Const Fn Kahn Topology Scheduler** | bcinr | Compile-time DAG topological sort via const fn; static [u8; N] array; 535 ps/op (vs 12.9 ns interpreted). | ★★★ High |
| **Hierarchical Time Wheel** | bcinr | Three-level cascade (A×B×C slots); O(1) amortized deadline scheduling; 100× density improvement. | ★★ Medium |

**Most Novel:** Const Fn Kahn scheduler (bcinr) — moves scheduling overhead to compile time, enabling straight-line execution.

---

## 5. Rule & Policy Enforcement Pipelines

**Cluster:** Observation → diagnosis → enforcement, with staged gates.

| Pattern | Source Repo | Core Idea | Novelty |
|---------|------------|----------|---------| 
| **Multi-Layer Detection Stack** | anti-llm-cheat-lsp | 6+ independent analysis passes (raw text, AST, manifest, markdown, JSON-RPC, BLAKE3); unified observation type. | ★★★ High |
| **Observation → Diagnostic → Enforcement** | anti-llm-cheat-lsp | Observations→AntiLlmDiagnostic (with blocking/required_correction metadata)→LSP Diagnostic + Andon events. | ★★ Medium |
| **Declare Constraint Engine** | anti-llm-cheat-lsp | Van der Aalst LTL-based constraints (Absence, Response, Precedence); activity traces validated against formal specs. | ★★★ High |
| **Staged RDF Admission Gates** | capability-map | Multi-stage pipeline (validate→load→SHACL→version→receipt binding); semantic policy enforcement per gate. | ★★ Medium |
| **Gate-Based Verification Pipeline** | chatmangpt | Independent gates (invocation_split, accepted_delta_required, receipt_required, etc.); structured GateReport with failure classes. | ★★ Medium |

**Most Novel:** Declare constraint engine (anti-llm-cheat-lsp) — brings formal process mining to static code analysis.

---

## 6. Process Mining & Conformance

**Cluster:** Event logs, workflow characterization, discovery algorithms.

| Pattern | Source Repo | Core Idea | Novelty |
|---------|------------|----------|---------| 
| **OCEL 2.0 Conformance Bridge** | capability-map, chicago-tdd-tools | Convert domain events (receipts, traces) to OCEL 2.0; pm4py for discovery/fitness/precision/generalization. | ★★★ High |
| **Evidence-Derived Coverage Matrix** | anti-llm-cheat-lsp | Conformance from on-disk artifacts (transcripts, receipts), not claims. Tri-state axis (Admitted/Refused/Unknown). | ★★ Medium |
| **Chatman Equation** | chicago-tdd-tools | Formal taxonomy of 43 workflow operators: Determinism, Idempotence, Type Preservation, Boundedness + 5 guard types. | ★★★ High |
| **Receipt-Backed Multi-Surface Evidence** | chatmangpt | Four evidence surfaces: file hashes, chain hash, telemetry spans, process-log events. All verified together. | ★★ Medium |

**Most Novel:** Chatman equation (chicago-tdd-tools) — mathematically principled characterization of control-flow operators.

---

## 7. Plugin & Registry Systems

**Cluster:** Auto-discovery, metadata collection, dynamic loading.

| Pattern | Source Repo | Core Idea | Novelty |
|---------|------------|----------|---------| 
| **Linkme Distributed-Slice Discovery** | clap-noun-verb | Compile-time command registration via linkme::distributed_slice; zero-runtime cost, zero boilerplate. | ★★ Medium |
| **Metaclass Auto-Registration** | autotel | Python metaclasses collect metadata at class definition time; capability-based discovery without manual registry. | ★★ Medium |
| **Multi-Origin Dynamic Loader** | clawdbot | Discovers plugins from bundled/global/workspace/config; dynamic code loader (jiti); manifest registry. | ★★ Medium |

---

## 8. Cross-Cutting Themes

1. **Hash-Based Witness Keys:** affidavit, cargo-cicd, chicago-tdd-tools, claude-code-config-lsp all use BLAKE3/ed25519 digests as immutable identifiers for configurations, rules, and evidence.

2. **Determinism as Verifiability:** Multiple repos (affidavit, chicago-tdd-tools, cargo-cicd) ensure pure functions + deterministic hashing so third parties can replay and verify.

3. **Type-State for Ordering:** affidavit, chicago-tdd-tools, cargo-cicd encode workflow preconditions in Rust's type system, making illegal states unrepresentable.

4. **Ontology-Driven Everything:** a2a-rs, A2A, cargo-cicd, claude-code-config-lsp, clap-noun-verb treat RDF/TTL as source of truth; code generation follows.

5. **Evidence Before Verdict:** cargo-cicd (E2 invariant), chatmangpt (receipt-backed), affidavit all enforce that adjudication requires collected evidence.

6. **Admission Gates:** cargo-cicd, capability-map, chicago-tdd-tools, anti-llm-cheat-lsp all use staged validation pipelines to gate transitions.

---

## 9. Recommendations for Praxis

**Absorb into Boilerplate:**
- **Law Object Pattern** (chatmangpt) — for obligation/task representation in SPARC workflow engine.
- **Receipt Chain** (affidavit) — for immutable audit trails across agent executions.
- **Type-State Enforcement** (chicago-tdd-tools/cargo-cicd) — for enforcing workflow preconditions.
- **OCEL Integration** (chicago-tdd-tools, capability-map) — for process conformance checking against DFLSS phases.
- **Andon Defect Signaling** (chatmangpt) — for halting pipelines on integrity violations.
- **RDF Code Generation** (established pattern) — continue leveraging; ensure bidirectional sync.

**Keep Repo-Specific:**
- Branchless calculus (bcinr) — domain-specific optimization for hard real-time.
- Virtual document LSP (anti-llm-cheat-lsp) — diagnostic introspection pattern.
- Declare constraint mining (anti-llm-cheat-lsp) — for governance/cheat-detection specifically.
- Multi-protocol bridging (A2A) — agent-to-agent protocol engineering.

**Emerging Cluster (Monitor):**
- Hash-based determinism as a verification primitive — consider unifying across workflows.

---

**Framework Goal:** Praxis should view itself as orchestrating *obligational workflows* with *type-safe transitions*, *cryptographically witnessed evidence*, and *formal conformance gates* — drawing from the most novel patterns above.
