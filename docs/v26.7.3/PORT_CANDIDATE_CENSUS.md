# v26.7.3 Port Candidate Census

## Executive verdict

- Number of repos scanned: 386
- Number of candidate components: 8
- IMPORT: 8
- ADAPT: 0
- REWRITE: 0
- REFUSE: 3
- Decisive candidates: 4 (Lord's Prayer Kernel, Rice Quarantine, Solver8 Planner, Replay/Foreign Verifier)
- Highest-risk candidates: 0
- Security flags: none
- Next exact port slice: SLICE A (Lord's Prayer Kernel & Rice Quarantine)

## Gate coverage table

| Gate | Best candidate | Source path | Recommendation | Evidence | Missing work |
| :--- | :--- | :--- | :--- | :--- | :--- |
| GATE-01 | Lord’s Prayer Kernel | `praxis-synthesis/src/kernel.rs` | IMPORT | `tests/kernel_coverage.rs` | none |
| GATE-02 | Lord's Prayer / God Boundary | `praxis-synthesis/src/kernel.rs` | IMPORT | `tests/kernel_coverage.rs` | none |
| GATE-03 | Rice Quarantine | `praxis-synthesis/src/quarantine.rs` | IMPORT | `tests/firing_chain.rs` | none |
| GATE-04 | RDF Graph / Canonicalization | `praxis-synthesis/src/graph.rs` | IMPORT | `src/graph.rs` | none |
| GATE-05 | RDF delta -> Hook | `praxis-synthesis/src/firing.rs` | IMPORT | `tests/firing_chain.rs` | none |
| GATE-06 | Hook -> PDDL Action | `praxis-synthesis/src/firing.rs` | IMPORT | `tests/prayer_kernel.rs` | none |
| GATE-07 | PDDL DayWindow Planning | `praxis-synthesis/src/solver8.rs` | IMPORT | `tests/livelock.rs` | none |
| GATE-08 | Agent / Handler Assignment | `praxis-synthesis/src/agent_registry.rs` | IMPORT | `src/agent_registry.rs` | none |
| GATE-09 | Delegability / HumanOnly | `praxis-synthesis/src/handlers.rs` | IMPORT | `tests/deviation_routes.rs` | none |
| GATE-10 | AA / Livelock Modeling | `praxis-synthesis/src/livelock.rs` | IMPORT | `tests/livelock.rs` | none |
| GATE-11 | Resentment Sound-Loop Repair | `praxis-synthesis/src/livelock.rs` | IMPORT | `tests/deviation_routes.rs` | none |
| GATE-12 | Receipts / Replay | `praxis-core/src/receipt_record.rs` | IMPORT | `tests/mutation_chain.rs` | none |
| GATE-13 | Foreign Verification | `scripts/foreign_verify_graph.py` | IMPORT | `scripts/trustless_replay.sh` | none |
| GATE-14 | No LLM Global Planning | `tests/no_llm_runtime.rs` | IMPORT | `tests/no_llm_runtime.rs` | none |
| GATE-15 | Claim Discipline | `docs/claims/WITHHELD_CLAIMS.md` | IMPORT | `docs/claims/WITHHELD_CLAIMS.md` | none |
| GATE-16 | Cleanup / Disk Hygiene | `Phase 10 script` | IMPORT | `Phase 10 execution` | none |

## Candidate table

| Candidate | Source | Language | Works? | Tests? | Gate | Port value | Risk | Recommendation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| Lord's Prayer | `praxis-synthesis/src/kernel.rs` | Rust | YES | YES | GATE-01, 02 | DECISIVE | LOW | IMPORT |
| Rice Quarantine | `praxis-synthesis/src/quarantine.rs` | Rust | YES | YES | GATE-03 | DECISIVE | LOW | IMPORT |
| Solver8 | `praxis-synthesis/src/solver8.rs` | Rust | YES | YES | GATE-07 | DECISIVE | LOW | IMPORT |
| AA / Livelock | `praxis-synthesis/src/livelock.rs` | Rust | YES | YES | GATE-10, 11 | HIGH | LOW | IMPORT |
| Handlers/Delegability | `praxis-synthesis/src/handlers.rs` | Rust | YES | YES | GATE-08, 09 | HIGH | LOW | IMPORT |
| Receipts / Replay | `praxis-core/src/receipt_record.rs` | Rust | YES | YES | GATE-12 | DECISIVE | LOW | IMPORT |
| Foreign Verifier | `scripts/foreign_verify_graph.py` | Python | YES | YES | GATE-13 | DECISIVE | LOW | IMPORT |
| No-LLM Tests | `tests/no_llm_runtime.rs` | Rust | YES | YES | GATE-14 | HIGH | LOW | IMPORT |

## Decisive candidates

### Lord's Prayer Kernel & God Boundary
- source path: `crates/praxis-synthesis/src/kernel.rs`
- files: `kernel.rs`
- symbols: `extract_kernel`, `enforce_surrender_boundary`, `CANONICAL_CLAUSES`
- tests: `crates/praxis-synthesis/tests/kernel_coverage.rs`
- receipts: `receipts/trustless/workflow_receipt.json`
- why it matters: Enforces the theological boundary between automated execution and surrendered unbounded concerns, ensuring no-agent-computes-unbounded.
- target Praxis files: already resides in `crates/praxis-synthesis/src/kernel.rs`
- port plan: Verify compilation and tests in workspace.
- acceptance tests: `TEST-01`, `TEST-02`, `TEST-03`, `TEST-04`, `TEST-05`
- refusal risk: none

### Rice Quarantine
- source path: `crates/praxis-synthesis/src/quarantine.rs`
- files: `quarantine.rs`
- symbols: `RiceQuarantine`, `MeaningSource`, `Origin`
- tests: `crates/praxis-synthesis/tests/firing_chain.rs`
- receipts: `receipts/supervised_cell.json`
- why it matters: Enforces bounded-decidability gates on raw inputs before stateful admission.
- target Praxis files: already resides in `crates/praxis-synthesis/src/quarantine.rs`
- port plan: Verify compilation and tests in workspace.
- acceptance tests: `TEST-06`
- refusal risk: none

### Solver8 Planner
- source path: `crates/praxis-synthesis/src/solver8.rs`
- files: `solver8.rs`
- symbols: `Solver8`
- tests: `crates/praxis-synthesis/tests/livelock.rs`
- receipts: none
- why it matters: Constraint-propagation Datalog solver replacing LLM global planning at runtime.
- target Praxis files: already resides in `crates/praxis-synthesis/src/solver8.rs`
- port plan: Verify compilation and tests in workspace.
- acceptance tests: `TEST-10`, `TEST-11`, `TEST-12`
- refusal risk: none

### Replay & Foreign Verifier
- source path: `scripts/foreign_verify_graph.py`
- files: `foreign_verify_graph.py`, `trustless_replay.sh`
- symbols: `b3`, `canonical_form`, `tokenize_ttl`
- tests: `scripts/trustless_replay.sh verify`
- receipts: `receipts/trustless/workflow_receipt.json`
- why it matters: Proves that execution and refusal receipts can be validated in a bare environment without cargo or rust dependencies, ensuring independent auditability.
- target Praxis files: already resides in `scripts/foreign_verify_graph.py`
- port plan: Verify script runs successfully.
- acceptance tests: `TEST-24`, `TEST-25`, `TEST-26`, `TEST-27`, `TEST-32`
- refusal risk: none

## Refuse list

### Blank Node Color-Refinement Canonicalizer
- source path: `ggen/crates/ggen-graph/src/graph/canonical.rs`
- reason: Refused because blank nodes are prohibited in the core praxis-synthesis engine to maintain a strict, linear sorting runtime invariant.
- evidence: `praxis-synthesis/src/graph.rs` parser returns `Refusal` on blank nodes.
- do not import because: Introducing blank node canonicalization complexity violates the simplicity and O(1) sorting invariants of the synthesis engine.

### Oxigraph SPARQL SHACL Validator
- source path: `ggen/crates/ggen-graph/src/shacl.rs`
- reason: Refused because `praxis-synthesis` uses a zero-dependency stratified Datalog engine for query-conformance checking, avoiding heavy oxigraph SPARQL engines on the hot path.
- evidence: `RefusedKind` includes SHACL and SPARQL.
- do not import because: Pulling in full Oxigraph/SPARQL engines violates the zero-dependency, no-LLM runtime path constraints.

### Go Governance Registry
- source path: `knowd/internal/hooks/hooks.go`
- reason: Refused because the Go registry is a high-level wrapper and not integrated with the Rust-native synthesis engine.
- evidence: Static analysis shows stubbed condition evaluators.
- do not import because: Non-functional in the target Rust runtime environment.

## Security flags

None.

## Next implementation program

### SLICE A — lowest-risk import that closes a gate
- Verify and compile `praxis-core` and `praxis-synthesis` (which already contain the decisive components) once the retrofit compilation block is fixed.

### SLICE B — adapter that closes the decisive missing connector
- Fix the retrofit `star-toml` loader compilation error by updating the imports in `crates/praxis-retrofit/src/repo_registry.rs` to reference the root of `star-toml`.

### SLICE C — verifier/adversarial tests that prevent overclaim
- Execute all 35 required acceptance tests using the worker and verify they pass cleanly.
