# Ticket: Add praxis-reconciler as Workspace Member

## Title
Add praxis-reconciler as Workspace Member

## Description
The `crates/praxis-reconciler/` directory exists on disk with a complete implementation of the autonomic repair loop (`MeasurementEnvironment` trait, `PraxisReconciler` struct based on the "Chatman Equation" autonomic model), but it is not listed as a workspace member in the root `Cargo.toml`. This means:
1. It is not compiled or tested by `cargo test --workspace`
2. It cannot be published as part of the praxis workspace
3. Its dependency on `genesis_types_v2` is unresolved

Adding it as a workspace member enables full integration testing and allows the autonomic repair capabilities to be part of the official release.

## Acceptance Criteria
- **Workspace Member Entry Added**: Root `/Users/sac/praxis/Cargo.toml` `[workspace].members` array now includes `"crates/praxis-reconciler"`
- **Dependency Resolved**: `genesis_types_v2` dependency in `crates/praxis-reconciler/Cargo.toml` is resolved either as:
  - A path dependency pointing to a local crate (if it exists elsewhere in the filesystem), OR
  - An explicit version pin to a registry crate (if it exists on crates.io)
  - Confirm via `cargo tree -p praxis-reconciler` that the dependency tree is complete
- **Full Workspace Build Success**: `cargo build --workspace` from `/Users/sac/praxis` includes reconciler and completes with no errors
- **Full Workspace Tests Pass**: `cargo test --workspace` from `/Users/sac/praxis` includes and runs tests in `crates/praxis-reconciler/` with all tests passing

## Dependencies
None.

## Verification Mechanism
Execute the following verification steps from `/Users/sac/praxis`:
1. Check workspace members in Cargo.toml:
   ```bash
   grep -A 10 '^\[workspace\]' Cargo.toml | grep members
   ```
   Must include `"crates/praxis-reconciler"`.

2. Check reconciler dependency tree:
   ```bash
   cargo tree -p praxis-reconciler
   ```
   Must show a complete tree with no unresolved dependencies (no `[...]` placeholders).

3. Build the full workspace:
   ```bash
   cargo build --workspace
   ```
   Must complete with exit code 0.

4. Run all tests:
   ```bash
   cargo test --workspace
   ```
   Must complete with exit code 0 (all tests passing).
