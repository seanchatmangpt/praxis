# Ticket: Integrate Hayes-Style Naive Physics Axioms for Workspace Dependency Auditing

## Title
Integrate Hayes-Style Naive Physics Axioms for Workspace Dependency Auditing (PROJ-103)

## Description
To eliminate circular dependency loops and ensure structural stability within the workspace crates, we will integrate a Naive Physics breed solver into our repository auditing process. This approach is inspired by Patrick Hayes's *"Naive Physics Manifesto"* (1979/1985), which models physical scene properties like support stability and containment.

In our system, the repository's dependency graph (parsed from workspace `Cargo.toml` configurations) is modeled as a Hayes physical scene of support chains:
- A dependency of Crate A on Crate B is represented as Crate A being supported by Crate B (`np:on:<crate_a>` = `<crate_b>`).
- Base libraries with no workspace dependencies are treated as grounded (`np:ground:<base_library>`).

The compliance court will invoke the `naive_physics` saturation solver (`crates/wasm4pm-cognition/src/breeds/naive_physics.rs`) inside a WASM execution container. The solver evaluates the scene to a fixpoint over Hayes's axioms (specifically `ax-support` and `ax-unsupported-falls`). If the solver detects a circular dependency (which translates to a cyclic support chain where no crate is grounded), the system experiences a "support failure" (representing a floating object that violates physical gravity laws). The audit must then fail immediately.

## Acceptance Criteria
- **Dependency Parser**: Implement an analyzer that parses all workspace `Cargo.toml` files, extracts the internal crate dependency graph, and serializes it as a Hayes-compatible scene input.
- **Axiomatic Saturation**: Integrate the `naive_physics` breed solver to evaluate the scene using the `ax-support` axiom.
- **Cycle Rejection**: The solver must detect circular support loops (e.g., A on B, B on C, C on A). If any cycle or unsupported crate is found, the audit must refuse execution, throwing an `ObjectLifecycleViolation` error.
- **Cli Tooling**: Extend the `wpm` command-line utility to support the graph auditing flag: `wpm audit --graph <path_to_cargo_toml>`.
- **Actionable Diagnostic Output**: When a cycle is detected, the command must write the complete circular path (e.g., `crate_a -> crate_b -> crate_c -> crate_a`) to standard error before exiting.

## Dependencies
- `ticket_002_wasm_compliance_court` (PROJ-102)

## Verification Mechanism
Verify the Hayes-style dependency auditor using the following steps:
1. Run the workspace audit tool on the current workspace to confirm no cycles exist:
   ```bash
   wpm audit --graph Cargo.toml
   ```
   The command should exit with 0, validating that the existing codebase is structurally stable and cyclic-free.
2. Run the integration test suite:
   ```bash
   cargo test --test dependency_audit_tests
   ```
3. The test suite in `tests/dependency_audit_tests.rs` must:
   - Programmatically construct a mock workspace configuration with a simulated circular crate reference (e.g., Crate X -> Crate Y -> Crate Z -> Crate X).
   - Feed this mock workspace to the audit command.
   - Assert that the command terminates with an exit code indicating a validation failure (specifically raising `ObjectLifecycleViolation`).
   - Assert that the diagnostic error output contains the exact dependency path causing the loop.
