# E2E Test Infra: Knowledge Hooks

## Test Philosophy
- **Opaque-box, requirement-driven**: The E2E tests target the public interfaces of the reasoner and triple store without relying on any specific internal implementation details. Tests must not inspect internal registry structs or mock private functions, ensuring that the internal architecture of the hooks module can be refactored without breaking the E2E verification suite.
- **Methodology**:
  - **Category-Partition**: Input space is partitioned based on parameters like Turtle serialization syntax, SHACL schema validity, trigger dialect kinds, projection queries, and fixpoint recursion bounds. Each partition is tested with specific representatives.
  - **Boundary Value Analysis (BVA)**: Tests are placed at the boundaries of input spaces, such as empty triple sets, threshold limits, window durations, extreme integer values, circular dependency loops, and empty output projections.
  - **Pairwise Testing**: Combines features to ensure that feature interactions (e.g., F3 trigger dialects invoking F4 CONSTRUCT queries that produce F5 BLAKE3 receipts) work correctly without requiring a combinatorial explosion of tests.
  - **Workload Testing**: Simulates real-world processing sequences where multiple hooks are evaluated in a cascaded reasoning loop to verify stability, performance, and correctness under loaded or complex conditions.

## Feature Inventory
| # | Feature | Source (requirement) | Tier 1 | Tier 2 | Tier 3 |
|---|---------|---------------------|:------:|:------:|:------:|
| F1 | Hook Parsing & Registry | R1. Turtle-Native Hook Registry | 5 | 5 | ✓ |
| F2 | Constitutional Gating | R1. Constitutional Guards | 5 | 5 | ✓ |
| F3 | First-Class Trigger Dialects | R2. Trigger Dialects & SPARQL | 6 | 5 | ✓ |
| F4 | Pure Action Projections | R3. Pure Action Projections | 5 | 5 | ✓ |
| F5 | Canonical N-Quads & BLAKE3 Receipts | R4. Canonical N-Quads BLAKE3 Receipts | 5 | 5 | ✓ |
| F6 | Fixpoint Reasoner Integration | R5. Fixpoint Integration | 5 | 5 | ✓ |

### F1: Hook Parsing & Registry
- **Description**: Natively parses `kh:Hook` declarations from loaded graph triples. The parser extracts the hook definition, structural metadata, and target predicates. The registry maintains the set of active hooks and handles dynamic loading and unloading.
- **Tier 1 (5 cases)**: Standard parsing of a single valid hook; registering multiple valid hooks; resolving basic hook prefixes; loading hooks from inline string triples; querying the list of registered hooks.
- **Tier 2 (5 cases)**: Parsing empty hook lists; handling malformed Turtle syntax; parsing hooks with missing mandatory attributes; duplicate hook registrations; registering hooks under non-standard namespaces.
- **Tier 3 (Pairwise)**: Interaction of parsing with reasoner fixpoint loops and receipt generation.

### F2: Constitutional Gating
- **Description**: Validates registered hook packs against a core SHACL Law Pack at load time. If a hook violates the security or structural constraints defined in the Law Pack, the load operation is rejected, and an error is returned.
- **Tier 1 (5 cases)**: Validation of a fully compliant hook pack; rejection of a hook with forbidden external namespaces; validation of basic trigger conditions; rejection of hooks attempting side-effect operations; enforcement of schema types.
- **Tier 2 (5 cases)**: Loading an empty SHACL Law Pack; validating deep nested structures within hooks; error handling on corrupt Law Pack definitions; validation under maximum nesting limits; testing boundary validation errors.
- **Tier 3 (Pairwise)**: Constitutional gating combined with diverse trigger dialects and action projections.

### F3: First-Class Trigger Dialects
- **Description**: Supports multiple trigger dialects to define when a hook fires. The supported dialects include Datalog, Delta, Threshold, Count, Window, SHACL, ShEx, N3, and SPARQL (ASK/SELECT).
- **Tier 1 (6 cases)**: Execution of SPARQL ASK triggers; execution of SPARQL SELECT triggers; evaluation of Count triggers; evaluation of Threshold triggers; evaluation of Delta triggers; validation of Datalog trigger formats.
- **Tier 2 (5 cases)**: Empty trigger conditions; triggers pointing to non-existent variables; threshold limits at maximum/minimum boundaries; window duration bounds (0 and overflow values); malformed SPARQL syntax inside triggers.
- **Tier 3 (Pairwise)**: Combining complex trigger dialects with pure action projections and BLAKE3 receipts.

### F4: Pure Action Projections
- **Description**: Evaluates SPARQL CONSTRUCT queries to project declarative changes into `kh:addQuad` and `kh:deleteQuad` predicates. The reasoner prevents any side-effects outside of the triple store, throwing an error if a hook attempts external system or network operations.
- **Tier 1 (5 cases)**: Successful projection of `kh:addQuad`; successful projection of `kh:deleteQuad`; concurrent projection of both additions and deletions; empty projections resulting in no change; projection using complex SPARQL CONSTRUCT patterns.
- **Tier 2 (5 cases)**: Projections containing invalid quad structures (e.g., literal subjects); projections targeting protected system namespaces; extremely large projection payloads; projections with variables that fail to bind; cyclical addition/deletion projections.
- **Tier 3 (Pairwise)**: Pure action projections generating canonical N-Quads and BLAKE3 receipts.

### F5: Canonical N-Quads & BLAKE3 Receipts
- **Description**: Serializes the projected quad deltas into a lexicographically sorted canonical N-Quads format and hashes them using BLAKE3. This yields a deterministic, cryptographic receipt representing the state change.
- **Tier 1 (5 cases)**: Generating a receipt for a single addition quad; generating a receipt for multiple quads in different order (verifying sort determinism); generating a receipt for a deletion; retrieving receipts from the TripleStore; verifying receipt format.
- **Tier 2 (5 cases)**: Receipt generation for empty deltas; handling blank nodes in canonical N-Quads; processing non-ASCII unicode characters in literals; boundary tests with huge literals; hashing extremely large delta sets.
- **Tier 3 (Pairwise)**: Interaction of receipt generation with fixpoint recursion and rollback.

### F6: Fixpoint Reasoner Integration
- **Description**: Knowledge Hook execution runs inside the reasoner's fixpoint loop (`Reasoner::materialize`). New quads projected by hooks are fed back into the current reasoning cycle until a fixpoint is reached. If any hook triggers a constraint violation, the active transaction rolls back.
- **Tier 1 (5 cases)**: Single-pass hook materialization; multi-pass cascaded materialization (hook A triggers hook B); fixpoint termination with stable state; rollback on validation failure; querying store state post-materialization.
- **Tier 2 (5 cases)**: Infinite reasoning loop detection (max recursion limit); zero-iteration materialization; handling database locks during fixpoint loops; materializing under high concurrency; rollback integrity with deep dependency trees.
- **Tier 3 (Pairwise)**: Interaction of fixpoint loops with constitutional gating and action projections.

## Test Architecture
- **Test runner**: Executes integration tests using the command: `cargo test -p praxis-graphlaw --test knowledge_hooks_e2e`. This ensures the test suite runs in isolation from internal unit tests.
- **Test case format**: Implemented as standard Rust test functions (`#[test]`) invoking public API methods on `TripleStore` and `Reasoner`. The main interfaces used are:
  - `TripleStore::load_hook_pack(&mut self, turtle_data: &str) -> Result<(), HookError>`: Parses and validates hook packs.
  - `Reasoner::materialize(&mut self) -> Result<MaterializationReport, ReasonerError>`: Materializes inferences and executes hooks.
  - `TripleStore::get_hook_receipts(&self) -> Vec<HookReceipt>`: Extracts the generated BLAKE3 receipts.
- **Directory layout**:
  - `crates/praxis-graphlaw/tests/knowledge_hooks_e2e.rs`: The main test suite file containing all integration test cases.
  - Test helper functions and inline data definitions are defined within this file to maintain a self-contained test environment.

## Real-World Application Scenarios (Tier 4)
### S1: Automated Quarantine & Refusal
- **Features Exercised**: F2 (Constitutional Gating), F3 (Trigger Dialects), F4 (Pure Action Projections), F6 (Fixpoint Reasoner Integration)
- **Complexity**: High
- **Description**: Verifies that when a client attempts to insert data violating system constraints (e.g., an unauthorized write to a protected namespace), a constitutional guard automatically catches the write, routes the violating triples to a quarantined graph space, projects a refusal action, and rolls back the client's transaction.

### S2: Ledger Balance Enforcement & Audit Trail
- **Features Exercised**: F3 (Trigger Dialects), F4 (Pure Action Projections), F5 (Canonical N-Quads & BLAKE3 Receipts), F6 (Fixpoint Reasoner Integration)
- **Complexity**: High
- **Description**: Implements a double-entry ledger scenario. The test asserts transaction triples. Triggers check if account balances drop below zero. If balance is valid, the hook projects ledger adjustment deltas and records a cryptographically signed BLAKE3 audit receipt. If invalid, the transaction is refused.

### S3: State Machine Transition Control
- **Features Exercised**: F3 (Trigger Dialects), F6 (Fixpoint Reasoner Integration)
- **Complexity**: Medium
- **Description**: Models a workflow lifecycle (e.g., `Draft` -> `UnderReview` -> `Approved`). The reasoner evaluates current states and triggers, ensuring transitions follow defined paths. Attempts to skip states (e.g., `Draft` -> `Approved`) fail to match triggers, preventing unauthorized state changes.

### S4: Access Control Policy Engine
- **Features Exercised**: F3 (Trigger Dialects), F4 (Pure Action Projections), F6 (Fixpoint Reasoner Integration)
- **Complexity**: Medium
- **Description**: Implements a Role-Based Access Control (RBAC) policy. Triggers evaluate the user's active roles and requested actions. The hook projects authorization decisions (allowing or denying access) and feeds them back into the reasoning cycle to dynamically prune or permit access.

### S5: Materialized View & Cache Maintenance
- **Features Exercised**: F3 (Trigger Dialects), F4 (Pure Action Projections), F6 (Fixpoint Reasoner Integration)
- **Complexity**: Medium
- **Description**: Automatically maintains a pre-computed query cache or materialized view. When base data triples are added or modified, triggers fire and project updates directly into the cache graph, keeping read-optimized queries up to date.

## Coverage Thresholds
- **Tier 1: Feature Coverage**: Requires >=5 test cases per feature (with 6 for F3) to verify core requirements, resulting in a total of 31 test cases.
- **Tier 2: Boundary & Corner Cases**: Requires >=5 test cases per feature targeting limits, errors, and empty inputs, resulting in a total of 30 test cases.
- **Tier 3: Pairwise Coverage**: Requires pairwise coverage of major feature interactions, resulting in a total of 6 test cases.
- **Tier 4: Real-World Application Scenarios**: Requires >=5 realistic application scenarios (S1 through S5) to ensure the system functions correctly as a cohesive whole.
