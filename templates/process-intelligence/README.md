# Process Intelligence Workspace Template

This workspace is a pre-configured template for building Process Intelligence and process-aware applications. It integrates **Chicago TDD Tools** for advanced development methodologies and telemetry tracking, alongside **wasm4pm** (Wasm-based Process Mining/Manager) for validating runtime process execution logs against formal process models.

---

## Architecture Overview

The Process Intelligence template enables developers to build services that emit execution traces (event logs) conforming to the **OCEL 2.0 (Object-Centric Event Log)** specification. These traces are generated at runtime and can be verified for conformance against a Petri net process model.

### Key Integration Components
1. **Chicago TDD Tools (`chicago-tdd-tools`)**:
   - **TDD Test Runner Macro**: High-performance unit test definitions via `chicago_tdd_tools::test!`.
   - **Snapshot Assertions**: Automated snapshot testing using `SnapshotAssert`.
   - **Property-Based/Fuzz Testing**: Utilities such as `PropertyTestGenerator` and `proptest` integrations to enforce logical invariants.
   - **OCEL Telemetry**: An `OcelCollector` sink that collects runtime diagnostic events (diagnostics) and serializes them into OCEL event logs.
2. **wasm4pm Compatibility (`wpm` CLI)**:
   - A runtime process miner/validation engine that checks prefix-conformance and runs token-replay algorithms on OCEL log streams against a formal Petri net (defined in `petri_net_lawful_dispatch.pnml`).

---

## Bootstrapping Instructions

### 1. Adding Dependencies
All dependencies should be unified in the workspace-level `Cargo.toml` to ensure consistent versioning across crates:
1. Open the root `/Cargo.toml`.
2. Add the dependency under the `[workspace.dependencies]` block:
   ```toml
   [workspace.dependencies]
   my-new-crate = "1.0.0"
   ```
3. Inherit the dependency in your member crate's `Cargo.toml`:
   ```toml
   [dependencies]
   my-new-crate = { workspace = true }
   ```

### 2. Creating a New Crate
To add a new microservice or library to the workspace:
1. Generate the crate in the `crates/` directory:
   ```bash
   cargo new crates/my-service --lib
   ```
2. Register the crate in the root `Cargo.toml` under `[workspace]`:
   ```toml
   [workspace]
   members = [
       "crates/sample-service",
       "crates/my-service"
   ]
   ```
3. Add `chicago-tdd-tools = { workspace = true }` to your new crate's dependencies to leverage telemetry and testing structures.

### 3. Conforming to Telemetry and Conformance Configurations
To ensure your new crate outputs valid traces that can be validated:
* **Register OcelCollector**: Initialize and register the collector sink using `chicago_tdd_tools::core::governance::register_sink`:
  ```rust
  let collector = OcelCollector::new(Some(ocel_log_path));
  register_sink(Box::new(collector));
  ```
* **Emit Diagnostics**: Use `emit_diagnostic` inside your workflows to signal process transitions (e.g., `OrderCreation`, `OrderValidation`, `OrderPayment`, `OrderDispatch`):
  ```rust
  let diag = Diagnostic {
      code: DiagnosticCode::new("activity_name".into(), DiagnosticCategory::Conformance, 100),
      category: DiagnosticCategory::Conformance,
      severity: Severity::Info,
      message: "Activity transition completed".into(),
      run_id: case_id.to_string(),
      // ...
  };
  emit_diagnostic(&diag);
  ```
* **Validate Execution**: Ensure your emitted events match the state transition flow defined in `petri_net_lawful_dispatch.pnml`. Use the `wpm validate` command to verify conformance.

---

## Verification Recipes (Justfile)

This template includes a standardized `Justfile` to automate build compilation, process validation, target pruning, and test execution.

### 1. Compiling the Workspace
To build and check compilation for all crates within the workspace:
```bash
just compile
```
*Behind the scenes*: Runs `cargo build` to compile the active workspace members.

### 2. Validating Process Conformance
To validate the generated runtime OCEL logs against the lawful dispatch Petri net (`petri_net_lawful_dispatch.pnml`):
```bash
just validate [log_path]
```
*Behind the scenes*: Runs `wpm validate <log_path>` (defaulting to `ocel/receipts/latest.json`) to perform prefix-conformance check and token replay.

### 3. Pruning Build Targets
To clean up disk space by pruning target directories:
```bash
just prune
```
*Behind the scenes*: Runs `cargo cicd target prune` to enforce disk space limits by removing stale target assets.

### 4. Running Tests for Changed Code
To execute only tests affected by recent commits or files modified since the base branch (`origin/main`):
```bash
just test-changed
```
*Behind the scenes*: Runs `cargo cicd test changed` to run targeted regression suites on affected code segments.

---

## Testing Instructions

The template demonstrates three complementary testing methodologies under `crates/sample-service/src/lib.rs`:

### 1. Chicago-Style Unit Tests
Instead of standard Rust `#[test]` syntax, this workspace leverages Chicago-style test definitions via the `chicago_tdd_tools::test!` macro. This macro simplifies setup/teardown, supports custom error handling, and automatically records test governance metadata:
```rust
chicago_tdd_tools::test!(test_create_order_success, {
    let order = create_order("customer_1".into(), vec![]);
    assert_eq!(order.status, OrderStatus::Created);
    Ok::<(), TestError>(())
});
```

### 2. Property-Based and Fuzz Testing
For testing robustness and validation edge-cases:
* **Invariants checking**: Powered by the `proptest!` macro, which generates randomized inputs (like randomized item prices and quantities) to assert mathematical correctness (e.g., that total amount calculations match the sum of individual items).
* **Fuzzing and Robustness**: Custom LCG random number generators (`TestLcgRng`) and fuzzer routines (like `test_json_fuzzing_robustness`) test parsing and input boundaries against malicious or malformed inputs.

### 3. Snapshot Testing
To verify complex JSON outputs, object trees, or database records:
* Snapshot testing is implemented using `SnapshotAssert` and `insta`.
* Executing a snapshot test serializes the target object (e.g. `Order`) into JSON and compares it against a saved golden file in `src/snapshots/`.
```rust
SnapshotAssert::with_settings(
    |settings| {
        let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/snapshots");
        settings.set_snapshot_path(path);
    },
    || {
        SnapshotAssert::assert_json_matches(&order_json, "sample_order_snapshot");
    },
);
```
To update snapshots if your models change, run tests with the `INSTA_UPDATE=always` environment variable:
```bash
INSTA_UPDATE=always cargo test
```
