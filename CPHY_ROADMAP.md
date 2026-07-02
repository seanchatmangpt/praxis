# CPhy Phased Delivery Roadmap
## Prototype → Production (2026-06 through 2026-Q4)

**Project Vision**: CPhy (Cryptographic Physical Law) is a deterministic, receipt-driven workflow governance system fusing **Law Objects** (obligation + lifecycle + receipt + event audit) with **planning** (PDDL) and **execution geometry** (POWL) to enable cryptographically auditable, compliance-first infrastructure automation.

**Philosophical Foundation**: Every artifact and action is judged against law (MUST satisfy obligations), receipted (cryptographically bound to state), and replayable (deterministic re-execution). OCEL event logs provide causal attribution. Erlang/OTP supervision enables cold-path recovery and distribution.

---

## Phase 1: MVP Foundation ✅ COMPLETE
**Timeline**: 2026-06 (shipped)  
**Status**: Ready for integration testing

### Deliverables
- **LawObject** (`crates/praxis-core/src/law.rs`): Fused type binding obligation + lifecycle + receipt + OCEL
  - Obligation enum: Precondition, BlockingConstraint, EvidenceRequired
  - Andon signal (halt/override/green)
  - Typestate pattern enforcing Raw → Validated → Admitted → Receipted transitions
  - Chain hash (BLAKE3) + signature (Ed25519) binding
  - Feature-gated OCEL event logging (`law-ocel`)
  
- **Rice Quarantine** (observation admission boundary in MCP+)
  - Inspect/judge entrypoint in `src/bin/mcp_lawobject_server.rs`
  - Receipt validation surface
  - Admission gate preventing unbounded ingress
  
- **MCP+ Surfaces** (Praxis → Model Context Protocol)
  - `mcp_lawobject_server.rs`: Capability inspection (law list), judgment (obligation check), admission (promote to Admitted), receipt (issue signed receipt)
  - Resource discovery and deep inspection via MCP protocol
  - Lifecycle enforcement at RPC boundary
  
- **PDDL Model** (`docs/ggen_rdf_to_pddl_sketch.rs` + planning library foundations)
  - Domain model mapping Law Objects → PDDL states and predicates
  - Action model for obligation satisfaction, override, receipt issuance
  - Ground problem generation from LawObject instance graph

### Critical Files
```
crates/praxis-core/src/law.rs          # Core LawObject + Obligation + Andon
crates/praxis-core/src/lifecycle.rs    # Typestate stages
src/bin/mcp_lawobject_server.rs        # MCP+ rice quarantine surfaces
docs/ggen_rdf_to_pddl_sketch.rs        # PDDL planning model
research/mu5_receipt_design.md         # Receipt determinism + BLAKE3/Ed25519 crypto
```

### Success Criteria
✅ LawObject compiles and passes unit tests with all features enabled  
✅ MCP+ server exposes law/obligation/receipt endpoints and responds to inspection  
✅ Typestate prevents unsafe lifecycle transitions at compile time  
✅ Receipt chain hash is deterministic and reproducible  
✅ PDDL sketch demonstrates obligation → planning action mapping  

### Effort Estimate
- **Past effort**: ~120 dev-hours (research, design, implementation, iteration)
- **Status**: Shipped in v26.6.30

---

## Phase 2: Deterministic Receipt Validation & Replay
**Timeline**: 2026-07 to 2026-08 (2–3 weeks, 1 sprint)  
**Dependencies**: Phase 1 (LawObject foundation)

### Deliverables

#### 2a. BCINR Receipt Validation Engine
- **Low-latency, branchless receipt validation**
  - Implement `ReceiptValidator` struct in `crates/praxis-core/src/receipt_validation.rs`
  - BLAKE3 digest recomputation (no allocation; streaming hash)
  - Ed25519 signature verification (ed25519-dalek or equivalent)
  - Chain integrity check (hash chain confirms no tampering)
  - Target: **<5ms validation per receipt** (single-threaded, no syscalls)
  
- **Deterministic Hashing Proof**
  - File discovery: stable sorted order (by path string)
  - Serialize strategy: canonical JSON or CBOR for reproducibility
  - Salt/nonce handling: explicit in receipt, not random
  - Cross-platform reproducibility: test on Linux/macOS/Windows CI
  
- **Branchless Design**
  - Avoid branching on validation result (use Result type + early return)
  - No conditional logging or allocations in hot path
  - Benchmark: propose `criterion` benchmark in `benches/receipt_validation.rs`

#### 2b. Receipt Replay Infrastructure
- **Deterministic Re-execution**
  - Implement `ReplayContext` struct: binds receipt → replay entrypoint
  - State snapshot extraction: law object state from receipt metadata
  - Action replay: re-run obligation checks, override signals, receipt issuance in identical order
  - Output matching: verify replayed events match original receipt chain hash
  
- **Time-Travel Semantics**
  - Configurable clock: replace `SystemTime` with injectable `Clock` trait
  - Replay mode: all nondeterminism frozen (RNG seeded from receipt entropy)
  - Auditability: every replayed step logged to intermediate receipt (breadcrumb trail)

#### 2c. Standing Receipt Cache
- Persistent storage of validated receipts (`data/validated_receipts/`)
  - Indexed by: `(project_name, timestamp, content_hash)`
  - Schema: receipt JSON + validation metadata (validated_at, validator_id)
  - Retention: unlimited (append-only audit log)
  
- **Standing Credential** (non-repudiation)
  - Receipt can be promoted to "standing" (auditor-signed endorsement)
  - Standing endorsement proves: "receipt was issued; we verified it; we vouch"
  - Use case: prove compliance retroactively in audit

### Critical Files
```
crates/praxis-core/src/receipt_validation.rs     # ReceiptValidator + BLAKE3/Ed25519
crates/praxis-core/src/replay.rs                 # ReplayContext + time-travel
benches/receipt_validation.rs                    # Criterion benchmark (<5ms target)
src/bin/dod.rs                                   # (extend) replay sub-command
research/mu5_receipt_design.md                   # (update) replay section
```

### Success Criteria
✅ `cargo test --lib` passes all receipt validation + replay tests  
✅ Benchmark shows receipt validation **< 5ms** (averaged over 1000 runs)  
✅ Replay produces byte-identical output (same obligation checks, same receipt hash)  
✅ Determinism proof: re-hash identical source tree → identical BLAKE3 digest (3 platforms)  
✅ MCP+ server exposes `receipt/validate` and `receipt/replay` operations  
✅ CLI verb `praxis receipt validate` works end-to-end  

### Effort Estimate
- **Estimated effort**: 80–100 dev-hours (implementation, benchmarking, determinism testing)
- **Team sizing**: 1 core engineer (receipt validation) + 1 (replay + infrastructure)
- **Dependencies**: Phase 1 (complete)
- **Risks**: Determinism across platforms (Windows file paths, floating-point rounding if any); edge case in Ed25519 library behavior

---

## Phase 3: Code Generation (ggen) & CLI Verbs
**Timeline**: 2026-08 to 2026-09 (3–4 weeks, 1.5 sprints)  
**Dependencies**: Phase 1, Phase 2 (receipt validation available for generated artifacts)

### Deliverables

#### 3a. ggen Ontology → Rust Types + PDDL
- **RDF/OWL → Rust Code**
  - Extend `ggen.toml` template system (currently RDF → Cargo.toml/Rust stubs)
  - Implement `TypeEmitter`: SPARQL query → Rust `struct` definitions with serde
  - Obligation schema extraction: query ontology for `law:obligation` predicates → emit Rust enum variants
  - Andon state machine: generate typestate machine from SHACL constraints
  
- **RDF/OWL → PDDL**
  - Implement `PddlEmitter`: SPARQL CONSTRUCT query → PDDL domain + problem
  - Domain model: predicates from RDF properties, actions from law transitions
  - Problem grounding: concrete objects from RDF individuals
  - Output: emit standard PDDL (STRIPS or Level 1) to `generated/domain.pddl` + `generated/problem.pddl`
  
- **Code Generation Pipeline**
  - Entry point: `ggen compile --ontology=ontology.ttl --target-lang=rust,pddl`
  - Validation: emit SHACL validation errors if ontology violates constraints
  - Receipt: generate receipt (BLAKE3 hash of ontology + PDDL output)

#### 3b. CLI Verbs (praxis law judge|admit|receipt|show|promote)
- **`praxis law judge <payload-file> [--law=<law-id>]`**
  - Parse payload → construct raw LawObject
  - Evaluate obligation preconditions (against provided evidence)
  - Report: green (all satisfied) or red (unmet obligations list)
  - Exit code: 0 (judge pass), 1 (judge fail)
  
- **`praxis law admit <payload-file> --law=<law-id> [--override-reason=<reason>]`**
  - Upgrade LawObject: Raw → Validated → Admitted
  - If --override-reason: emit Andon::Overridden signal (for critical path waiver)
  - Check: must be previously judged green OR have override authorization
  - Output: write admitted object to stdout (JSON)
  
- **`praxis receipt issue <payload-file> --law=<law-id>`**
  - Admit object (if not already)
  - Compute BLAKE3 digest of payload + law context
  - Sign with local Ed25519 key (or load from keyring)
  - Emit: receipt JSON with signature + chain hash
  - Persist: save to `~/.praxis/receipts/` with deterministic filename
  
- **`praxis receipt validate <receipt-file> [--replay]`**
  - Load receipt → verify signature + chain integrity
  - Recompute digest → assert matches receipt
  - If --replay: execute ReplayContext, compare outputs
  - Report: green (valid + unreplayed) or amber (valid + replayed + divergent)
  
- **`praxis receipt show <receipt-file> [--format=json|yaml|human]`**
  - Pretty-print receipt with chain history
  - Show obligations that were satisfied
  - Show andon state at issuance time
  - Format: human-readable table (default) or structured (JSON/YAML)
  
- **`praxis law promote <receipt-file> --reason=<audit-reason>`**
  - Mark receipt as standing (auditor-endorsed)
  - Append auditor signature and reason to receipt metadata
  - Use case: "this receipt is good; we vouch it" (compliance proof)

#### 3c. CLI Infrastructure
- Extend `src/bin/dod.rs` with noun-verb architecture (using `clap-noun-verb`)
  - Nouns: `law`, `receipt`, `obligation`, `andon`
  - Verbs per noun: judge, admit, issue, validate, show, promote
  - Flag groups: `--law-id`, `--evidence-dir`, `--keyring`, `--override-reason`
  
- Configuration file: `~/.praxis/config.toml`
  - Default keyring location, law library paths, receipt storage root
  - Pluggable validators (list of external CLI tools to invoke)

### Critical Files
```
crates/praxis-core/src/ggen/emitter.rs          # RDF → Rust types + PDDL
src/bin/dod.rs                                  # CLI verbs (judge, admit, receipt, promote)
ggen.toml                                       # (update) ontology compilation targets
examples/sample_law.ttl                         # Sample RDF ontology
tests/integration/cli_e2e.rs                    # E2E tests for all verbs
```

### Success Criteria
✅ `praxis law judge <test-payload> --law=compliance` → exit 0 if obligations satisfied  
✅ `praxis law admit <test-payload> --law=compliance` → emits Admitted LawObject  
✅ `praxis receipt issue <test-payload> --law=compliance` → signed receipt written  
✅ `praxis receipt validate <receipt>` → verifies signature + chain hash (< 5ms)  
✅ `praxis receipt show <receipt> --format=json` → structured output parseable  
✅ E2E test: law → judge → admit → receipt → validate → replay all pass  
✅ ggen emits valid Rust code (compiles without warnings)  
✅ ggen emits valid PDDL (passes pddl-linter or equivalent)  

### Effort Estimate
- **Estimated effort**: 100–140 dev-hours
  - ggen Rust emitter: 50–70 hours
  - ggen PDDL emitter: 30–40 hours
  - CLI infrastructure: 20–30 hours
- **Team sizing**: 1–2 engineers (1 ggen-focused, 1 CLI)
- **Dependencies**: Phase 1 (LawObject), Phase 2 (receipt validation) to validate generated receipts
- **Risks**: PDDL standard compliance; SPARQL query complexity; error messages in CLI

---

## Phase 4: POWL Execution Geometry & Partial-Order Scheduling
**Timeline**: 2026-09 to 2026-10 (4–5 weeks, 1.5 sprints)  
**Dependencies**: Phase 1, Phase 2, Phase 3 (ggen PDDL output)

### Deliverables

#### 4a. POWL Core Data Structure
- **Partial-Order Workflow Language (POWL)**
  - Implement in `crates/praxis-core/src/powl.rs`
  - Node types: Task, Decision, Join, Fork, Loop, Callback
  - Edges: precedence (before), concurrency (parallel), choice (XOR)
  - Attributes: guardians (who can execute), resources (semaphores/locks)
  
- **POWL → Graph Representation**
  - DAG (directed acyclic graph) with transitive reduction
  - Serialize to GraphML or DOT for visualization
  - Compile to LawObject obligations (each node → obligation to execute)

#### 4b. Scheduling Engine
- **Partial-Order Scheduler**
  - Implement `ScheduleContext` struct in `crates/praxis-core/src/scheduler.rs`
  - Input: POWL graph + constraints (resource limits, deadline, guardian assignments)
  - Output: valid total order (linearization) satisfying all precedence + concurrency rules
  - Algorithm: topological sort with constraint propagation (no full solver needed; greedy heuristic OK for Phase 4)
  
- **Conflict Resolution**
  - Detect resource conflicts (same resource requested by concurrent nodes)
  - Propose serialization (fallback to sequential) or queuing discipline (FIFO, priority)
  - Emit warning: "nodes X and Y cannot run in parallel due to resource Y"

#### 4c. Execution Geometry
- **Execution Trace**
  - Record: (node_id, timestamp, status, guardian, resources_held)
  - Emit to OCEL event log (one event per node execution)
  - Support: pause/resume (checkpoint state), abort (cleanup resources)
  
- **Visualization**
  - Generate SVG execution gantt chart (time vs. concurrent nodes)
  - Color code: green (done), yellow (in progress), red (failed/blocked)
  - Interactive: hover node → show receipt chain for that node

#### 4d. Deterministic Replay
- **Replay with POWL**
  - Use original schedule (from receipt) to re-execute in identical order
  - Enforce: if original execution was serial (due to conflict), replay serially even if resources now free
  - Benefit: exactly reproduces original behavior (critical for audit)

### Critical Files
```
crates/praxis-core/src/powl.rs                  # POWL data structure + graph ops
crates/praxis-core/src/scheduler.rs             # ScheduleContext + topological sort
crates/praxis-core/src/execution_trace.rs       # Execution log + OCEL emission
benches/scheduler_perf.rs                       # Benchmark scheduling (target: <100ms for 1000-node graph)
examples/powl_example.rs                        # Sample POWL workflow
```

### Success Criteria
✅ POWL parses and serializes without error  
✅ Topological sort produces valid linearization (respects all edges)  
✅ Scheduler detects resource conflicts and proposes resolution  
✅ Execution trace maps to OCEL events correctly  
✅ Replay produces byte-identical execution trace (same order, same guardians)  
✅ Gantt chart visualization is readable and correct  
✅ Scheduling < 100ms for 1000-node workflow  

### Effort Estimate
- **Estimated effort**: 120–160 dev-hours
  - POWL data structure: 30–40 hours
  - Scheduler: 40–50 hours
  - Trace + visualization: 30–40 hours
  - Testing: 20–30 hours
- **Team sizing**: 1–2 engineers
- **Dependencies**: Phase 1–3 complete
- **Risks**: Scheduling complexity (NP-hard for general CSP; mitigate with greedy heuristic); visualization library choice

---

## Phase 5: Erlang/OTP Supervision & AtomVM Bridging
**Timeline**: 2026-10 to 2026-11 (5–6 weeks, 2 sprints)  
**Dependencies**: Phase 1–4 complete; receipt infrastructure (Phase 2) stable

### Deliverables

#### 5a. Cold-Path Supervision Model
- **Erlang/OTP Design Patterns in Rust**
  - Implement `SupervisionTree` in `crates/praxis-core/src/supervision.rs`
  - Supervisor node: monitors child processes, restarts on failure
  - Worker nodes: execute POWL tasks; linked to supervisor for fault propagation
  - Restart strategy: exponential backoff (2^n seconds, capped)
  
- **Process Linking & Monitoring**
  - Link: if child dies, supervisor receives signal
  - Monitor: supervisor watches process health (heartbeat every 30s)
  - Clustering: multiple supervisors across machines via eventual consistency + receipts
  
- **Recovery Mechanics**
  - Checkpoint: save POWL execution state to disk before each task
  - Recovery: on restart, replay from last checkpoint (using Phase 2 replay infrastructure)
  - Determinism: replay guarantees identical behavior (idempotence required in task design)

#### 5b. AtomVM Bridging
- **AtomVM as Edge Runtime**
  - Research: explore AtomVM BEAM VM for resource-constrained execution
  - Interface: define RPC protocol for Rust ↔ AtomVM communication
  - Task dispatch: send minimal POWL task to AtomVM, retrieve result + receipt
  - Use case: edge devices (IoT, 5G-MEC) execute specialized tasks, phone home with receipts
  
- **Receipt Flow**
  - Edge device executes task → emits local receipt (BLAKE3 + time-limited Ed25519 key)
  - Send receipt to central supervisor
  - Supervisor: validates + includes in chain (downstream receipt meta)

#### 5c. Distribution & Fault Tolerance
- **Multi-Node Coordination**
  - Implement `ClusterState` struct: leader election (via Raft or simpler quorum-based)
  - State sync: replicate POWL queue + receipt log across nodes
  - Failure: if leader dies, follower takes over without losing in-flight tasks
  
- **Receipt Chaining Across Nodes**
  - Each node: maintains local receipt chain
  - Global chain: merge via causal ordering (vector clocks or OCEL timestamp)
  - Audit view: single coalesced receipt proving full computation path

#### 5d. Observability
- **Structured Logging**
  - Emit events: supervisor bootstrap, task scheduled, task failed, restart triggered, recovery complete
  - Format: OCEL-compatible (task_id, node_id, timestamp, status, supervisor_action)
  - Sink: stdout + optional gRPC trace sink (Jaeger-compatible)
  
- **Health Dashboard**
  - Query: `praxis supervision status [--node=<node-id>]`
  - Output: uptime, tasks executed, failures + recovery count, last receipt hash
  - Real-time: use streaming HTTP or WebSocket

### Critical Files
```
crates/praxis-core/src/supervision.rs           # SupervisionTree + restart logic
crates/praxis-core/src/atomvm_bridge.rs         # AtomVM RPC + receipt flow
src/bin/dod.rs                                  # (extend) supervision sub-command
examples/supervision_example.rs                 # Multi-node supervision setup
tests/integration/fault_tolerance.rs            # Failure injection tests
```

### Success Criteria
✅ SupervisionTree starts, monitors, and restarts children without panic  
✅ Child restart triggers replay of last task (idempotence verified via same output receipt)  
✅ Multi-node setup: leader election works, state syncs, leader failure triggers failover  
✅ Receipt chain merges correctly (no duplicate or missing receipts)  
✅ AtomVM bridge: task dispatch + result retrieval works (latency < 100ms)  
✅ Health dashboard shows: uptime, task count, failure count, last receipt  
✅ Failure injection test: kill random node → system recovers, audit trail intact  

### Effort Estimate
- **Estimated effort**: 160–200 dev-hours
  - Supervision tree: 60–80 hours
  - AtomVM bridge: 40–60 hours
  - Clustering: 40–50 hours
  - Observability: 20–30 hours
- **Team sizing**: 2 engineers (1 supervision/OTP, 1 clustering/distribution)
- **Dependencies**: Phase 1–4; external: AtomVM documentation + tooling
- **Risks**: Erlang/OTP semantics in Rust (subtle differences); AtomVM stability; network partition handling

---

## Phase 6: Promotion Gates & Enterprise Integrations
**Timeline**: 2026-11 to 2026-12 (6–8 weeks, 2–2.5 sprints)  
**Dependencies**: Phase 1–5 complete; receipt validation and compliance framework stable

### Deliverables

#### 6a. Promotion Gates
- **Replay → Standing Credential Pipeline**
  - Gate 1: Automated Audit
    - Replay task execution using Phase 2 replay infrastructure
    - Assertion: replayed task output matches original receipt (bit-identical)
    - If pass: flag receipt as `AUDIT_PASS`, emit audit log
    - If fail: flag as `AUDIT_FAIL`, trigger investigation
    
  - Gate 2: Compliance Check
    - Scan receipt + task log against regulatory policies (e.g., HIPAA, SOC2)
    - Policies: configurable (RDF rules) or built-in (PDDL domain model)
    - Pass: receipt promoted to "auditor-reviewed"
    
  - Gate 3: Guardian Approval
    - Route to human guardian (via CLI: `praxis receipt pending-approval`)
    - Guardian: review audit trail, sign off with `praxis law promote --reason="<approval>"`
    - Receipt: marked as `STANDING_CREDENTIAL` (non-repudiation achieved)

- **Standing Credential Semantics**
  - Proof artifact: receipt + guardian signatures + audit logs (all chained)
  - Use case: retroactively prove compliance ("deployment X is auditable back to source ontology")
  - Export: emit as compliance report (PDF + JSON) for auditors

#### 6b. Compliance Audit Surfaces
- **Audit Query DSL**
  - Language: simple SPARQL-like queries over receipt graph + OCEL log
  - Example: `SELECT task, duration, guardian WHERE supervisor="auth-checker" AND status="FAIL"`
  - Endpoint: `praxis audit query "<dsl-query>" [--format=json|csv|html]`
  
- **Audit Reports**
  - Template: compliance report (Diátaxis; auto-generated from logs)
  - Sections: execution summary, failure log, remediation timeline, standing credentials
  - Export: PDF (via markdown → pandoc) or static HTML
  
- **Trail Integrity Proof**
  - Merkle tree of receipts: compute root hash of all receipts in date range
  - Assertion: "all receipts between T0..T1 form unbroken chain (no deletion, no splicing)"
  - Output: Merkle root proof + individual receipt hashes (auditor can verify)

#### 6c. Enterprise Integrations
- **SIEM Integration (Splunk, ELK, Datadog)**
  - Emit OCEL events to HTTP endpoint (configurable)
  - Format: JSON or CEF (Common Event Format)
  - Fields: task_id, status, duration, guardian, resource_used, supervisor_node
  
- **Ticketing System Integration (Jira, ServiceNow)**
  - On task failure: auto-create ticket with receipt + error details
  - Link: receipt chain embedded in ticket for auditing
  - Resolution workflow: ticket closure triggers `praxis law promote` (gate 3 approval)
  
- **Secrets Management (Vault, AWS Secrets Manager)**
  - Keyring abstraction: pluggable Ed25519 signing key storage
  - Interface: `KeyProvider` trait with backends (local file, Vault, AWS Secrets)
  - Phase 6 implementation: Vault backend + AWS Secrets backend
  
- **Provisioning/IaC Frameworks (Terraform, Pulumi, CloudFormation)**
  - CPhy as policy layer: Terraform plan → Law Objects (authorization checks)
  - Integration: invoke `praxis law judge` in CI/CD pipeline before `terraform apply`
  - Output: receipt proves plan was authorized before deployment

#### 6d. Web Dashboard (Optional but Recommended)
- **Read-Only Compliance Dashboard**
  - Tech stack: React or Vue.js + Rust backend (warp/actix-web)
  - Views:
    - Execution timeline (POWL + task status gantt)
    - Receipt explorer (tree view of chain)
    - Compliance scorecard (% of tasks with audit_pass)
    - Alert list (audit_fail, missing receipts, pending approvals)
  
- **Guardian Portal**
  - List: pending approvals (receipts awaiting promotion)
  - Action: review audit trail → sign → promote
  - Audit: all guardian actions logged (who, when, reason)

### Critical Files
```
crates/praxis-core/src/promotion_gates.rs       # Replay audit + compliance check
crates/praxis-core/src/audit.rs                 # Query DSL + report generation
src/bin/compliance_server.rs                    # (new) HTTP server for dashboards + integrations
src/bin/dod.rs                                  # (extend) audit sub-command
integrations/splunk_emitter.rs                  # SIEM integration
integrations/vault_keyring.rs                   # Secrets management
integrations/terraform_hook.rs                  # IaC provisioning integration
tests/integration/e2e_compliance.rs             # End-to-end compliance workflow
```

### Success Criteria
✅ Receipt promotion: replay audit → compliance check → guardian approval, all gates fire  
✅ Audit query: `praxis audit query "SELECT task WHERE status='FAIL'"` returns correct results  
✅ Compliance report: PDF generated with all required sections (execution, failures, remediation)  
✅ Merkle root proof: independently verifiable (auditor can recompute)  
✅ SIEM integration: events flow to Splunk (test with splunk-hec endpoint)  
✅ Vault integration: Ed25519 key loaded from Vault, receipt signed correctly  
✅ Terraform hook: phase validated before apply (success: deploy blocked by failed judge)  
✅ Dashboard: real-time timeline + receipt tree rendering, guardian approval workflow functional  
✅ E2E test: law → judge → admit → task execute → receipt → audit → promote → report, all pass  

### Effort Estimate
- **Estimated effort**: 200–280 dev-hours
  - Promotion gates: 50–70 hours
  - Audit DSL + reporting: 60–80 hours
  - Enterprise integrations (Vault, Splunk, Terraform, ServiceNow): 60–100 hours
  - Dashboard (if included): 80–120 hours (optional; can defer)
- **Team sizing**: 2–3 engineers (1 gates/audit, 1 integrations, 1 optional dashboard)
- **Dependencies**: Phase 1–5 complete; external APIs (Vault, Splunk) stable
- **Risks**: Integration testing with external systems; API versioning; user experience of dashboard

---

## Integration Timeline & Critical Path

```
Phase 1 (MVP)     [===] Complete (v26.6.30)
  ↓
Phase 2 (Receipt) [======] 2026-07 to 2026-08 (2-3 weeks)
  ↓
Phase 3 (ggen+CLI)[=========] 2026-08 to 2026-09 (3-4 weeks, overlaps Phase 2)
  ↓
Phase 4 (POWL)    [===========] 2026-09 to 2026-10 (4-5 weeks)
  ↓
Phase 5 (Erlang)  [=============] 2026-10 to 2026-11 (5-6 weeks, overlaps Phase 4)
  ↓
Phase 6 (Audit)   [===============] 2026-11 to 2026-12 (6-8 weeks)
```

**Critical Path**: Phase 2 (receipt validation) → Phase 3 (CLI+ggen) → Phase 4 (POWL) → Phase 5 (supervision).  
**Parallelization opportunity**: Phase 3 (CLI) can start mid-Phase 2; Phase 5 (Erlang) can start mid-Phase 4.

---

## Success Metrics & Verification

### Phase 2 Success
- **Metric**: Receipt validation latency < 5ms (averaged over 1000 runs)
- **Verification**: Benchmark: `cargo bench --bench receipt_validation`
- **Metric**: Replay produces byte-identical output
- **Verification**: Automated test: replay 100 random law objects, compare output hashes

### Phase 3 Success
- **Metric**: All CLI verbs functional end-to-end
- **Verification**: E2E test: `praxis law judge` → `admit` → `receipt issue` → `validate` → `replay`
- **Metric**: ggen emits valid, compilable Rust
- **Verification**: Generated code passes `cargo check` without warnings

### Phase 4 Success
- **Metric**: POWL scheduling latency < 100ms for 1000-node graph
- **Verification**: Benchmark: `cargo bench --bench scheduler_perf`
- **Metric**: Replay with POWL produces identical execution trace
- **Verification**: Automated test: compare original + replayed gantt charts

### Phase 5 Success
- **Metric**: Supervised task failure → restart → recovery within 30 seconds
- **Verification**: Fault injection test: kill child process, measure recovery time
- **Metric**: Multi-node cluster survives leader failure
- **Verification**: Chaos test: kill leader, verify followers elect new leader, no task loss

### Phase 6 Success
- **Metric**: Compliance report generated in < 10 seconds
- **Verification**: Load test: 10000 receipts, measure report generation latency
- **Metric**: SIEM integration: events delivered within 1 second of task completion
- **Verification**: End-to-end: task → OCEL event → SIEM HTTP endpoint, measure latency

---

## Risk & Mitigation Summary

| Phase | Risk | Severity | Mitigation |
|-------|------|----------|-----------|
| 2 | Determinism across platforms (Windows paths) | High | CI matrix: Linux/macOS/Windows; explicit platform-agnostic serialization |
| 2 | Ed25519 library stability | Medium | Use well-tested library (ed25519-dalek); pin version; extensive signature tests |
| 3 | SPARQL complexity (query optimization) | Medium | Start with simple queries; optimize if profiling shows bottleneck |
| 3 | PDDL standard compliance | Medium | Use pddl-linter or Downward parser to validate generated plans |
| 4 | Scheduling NP-hardness | Medium | Accept greedy heuristic for Phase 4; propose exact solver (SAT/SMT) for Phase 5+ |
| 5 | Erlang/OTP semantics in Rust | High | Leverage existing crates (tokio, parking_lot); study OTP docs; prototype early |
| 5 | AtomVM stability | Medium | Evaluate AtomVM maturity; consider Lua VM alternative if needed |
| 5 | Network partition handling | High | Implement eventual consistency model; document split-brain scenarios |
| 6 | SIEM vendor API churn | Low | Abstract via trait; support major vendors (Splunk, Datadog, ELK) |
| 6 | Secrets management key rotation | Medium | Implement key versioning; rotate keys without downtime (version negotiation) |

---

## Resource Allocation (Recommended Team Structure)

### Phase 2–3 (Receipt + CLI)
- **Engineer A** (Core, Senior): Receipt validation + replay infrastructure, benchmarking
- **Engineer B** (CLI, Mid-level): ggen Rust/PDDL emitters, CLI verb design
- **QA**: E2E testing, determinism verification (cross-platform)

### Phase 4 (POWL)
- **Engineer A** (continued): POWL data structure, scheduler
- **Engineer C** (Execution, Mid-level): Execution trace, visualization
- **QA**: Scheduling correctness, replay verification

### Phase 5 (Erlang/OTP)
- **Engineer D** (Distributed, Senior): SupervisionTree, clustering, leader election
- **Engineer E** (Systems, Mid-level): AtomVM bridge, fault injection tests
- **QA**: Chaos testing, multi-node failure scenarios

### Phase 6 (Audit + Enterprise)
- **Engineer B** (continued): Audit DSL, compliance reporting
- **Engineer F** (Integration, Mid-level): Vault, Splunk, Terraform, ServiceNow adapters
- **Engineer G** (optional, Frontend): Dashboard (React/Vue.js)
- **QA**: Integration testing with external systems, user acceptance testing

**Total Effort**: ~820–1180 dev-hours across all phases (6–8 months, 1.5–2 engineers average, or 3 engineers full-time for 4–5 months)

---

## Credibility & Achievability Assessment

### Why This Roadmap Is Credible

1. **Phases 1–2 are grounded in existing code**: LawObject, MCP+ server, receipt design all exist in v26.6.30. Receipt validation is a straightforward extension of BLAKE3/Ed25519 primitives (well-understood crypto).

2. **Phase 3 (ggen + CLI) is achievable in 1–2 sprints**: Templates exist; SPARQL → Rust codegen is commodity work. CLI verbs are straightforward I/O; no algorithmic complexity.

3. **Phase 4 (POWL) leverages existing scheduler research**: Topological sort is O(V+E); resource conflict detection is a constraint satisfaction problem. No novel algorithms needed; leverage existing crates (petgraph for DAG operations).

4. **Phase 5 (Erlang/OTP) is proven pattern**: Tokio provides async runtime; libraries exist for exponential backoff, process monitoring. AtomVM is exploratory but optional (can ship Phase 5 without it, add later).

5. **Phase 6 (Audit) is integration-focused**: No algorithmic risk; primarily glue code + report generation. Dashboard is nice-to-have (can defer or use existing tools like Grafana).

6. **CPhy vision is internally consistent**: Law Objects → PDDL planning → POWL execution → Erlang supervision forms a coherent stack. Each phase adds value independently; phases don't require moonshot innovations.

### Achievability in 1–2 Sprints (Phase 2–3)

**Assumptions**:
- 2 engineers, 2-week sprints, ~80 dev-hours available per engineer per sprint
- Phase 1 already shipped (LawObject stable)
- Crypto libraries (BLAKE3, Ed25519) already integrated

**Phase 2 breakdown (2 weeks)**:
- Week 1: Implement `ReceiptValidator`, BLAKE3 streaming, determinism tests
- Week 1: Implement `ReplayContext`, time-travel clock
- Week 2: Benchmarking, cross-platform determinism proof, MCP+ integration
- **Outcome**: Receipt validation + replay shipped, tested, benchmarked

**Phase 3 breakdown (3–4 weeks, overlaps Phase 2)**:
- Week 2–3 (parallel to Phase 2 week 2): ggen Rust emitter (SPARQL → struct definition)
- Week 3: ggen PDDL emitter (ontology → PDDL domain/problem)
- Week 3–4: CLI infrastructure (noun-verb arch), implement `praxis law judge|admit|receipt|promote`
- Week 4: E2E testing, CLI refinement
- **Outcome**: Full `praxis` CLI functional; ggen emits valid Rust + PDDL; all integration tests pass

**Risk factors**:
- Determinism edge cases (Windows file I/O, floating-point non-determinism) → mitigate with early cross-platform CI setup
- SPARQL query complexity → start with simple queries; profile + optimize if needed
- PDDL validation tooling → identify + integrate pddl-linter early (week 1)

---

## Appendix: Key Research References

- **Chatman Equation**: $A = \mu(O)$ (artifacts are deterministic projection of ontology)
- **5-Stage Pipeline**: $\mu_1$ (Normalize) → $\mu_2$ (Extract) → $\mu_3$ (Emit) → $\mu_4$ (Canonicalize) → $\mu_5$ (Receipt)
- **Law Objects**: Fused obligation + lifecycle + receipt + OCEL event logging (blend of Contract Law, typestate, Andon)
- **PDDL**: Planning Domain Definition Language (AI classical planning; expressiveness: STRIPS)
- **POWL**: Partial-Order Workflow Language (DAG-based concurrent workflow model)
- **Erlang/OTP**: Fault-tolerant distributed system pattern (process linking, supervisor restart trees, eventual consistency)
- **BLAKE3**: Fast cryptographic hash; deterministic content addressing
- **Ed25519**: Modern EdDSA signature scheme; non-repudiation proof

**Related Documents**:
- `/Users/sac/praxis/research/mu5_receipt_design.md` — Receipt generation + verification
- `/Users/sac/praxis/research/ggen_first_transition.md` — Ontology-driven code generation vision
- `/Users/sac/praxis/research/post_chatman_research.md` — Chatman Equation formalization
- `/Users/sac/praxis/crates/praxis-core/src/law.rs` — LawObject implementation
- `/Users/sac/praxis/src/bin/mcp_lawobject_server.rs` — MCP+ rice quarantine surfaces

---

**Document Version**: 1.0  
**Date**: 2026-07-01  
**Author**: Claude Code (praxis agent)  
**Status**: Ready for team review and sprint planning
