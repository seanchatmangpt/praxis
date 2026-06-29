# Ticket: Bump All Praxis Crates to v26.6.30

## Title
Bump All Praxis Crates to v26.6.30

## Description
The praxis workspace (3 crates: `my-conforming-project`, `chatman-common`, `praxis-retrofit`) is currently at version `26.6.0`. To complete the v26.6.30 release cycle, all crate versions must be bumped from `26.6.0` → `26.6.30`. Additionally, the `chicago-tdd-tools` dependency in `praxis-retrofit` must be bumped from the exact pin `"=26.6.29"` to `"=26.6.30"` to match the release version.

This is a **release blocker** — crates cannot be published at v26.6.30 if their `Cargo.toml` version fields still show `26.6.0`.

## Acceptance Criteria
- **Root Crate Version Bump**: `/Users/sac/praxis/Cargo.toml` `[package].version` changed from `"26.6.0"` to `"26.6.30"`
- **chatman-common Bump**: `/Users/sac/praxis/crates/chatman-common/Cargo.toml` `[package].version` changed from `"26.6.0"` to `"26.6.30"`
- **praxis-retrofit Bump**: `/Users/sac/praxis/crates/praxis-retrofit/Cargo.toml` `[package].version` changed from `"26.6.0"` to `"26.6.30"`
- **Dependency Pin Bump**: `/Users/sac/praxis/crates/praxis-retrofit/Cargo.toml` `chicago-tdd-tools` dependency changed from `"=26.6.29"` to `"=26.6.30"`
- **Build Success**: `cargo build --workspace` from `/Users/sac/praxis` executes with no errors or compilation failures

## Dependencies
None.

## Verification Mechanism
Execute the following verification steps from `/Users/sac/praxis`:
1. Check version fields in all three crates:
   ```bash
   grep '^version' Cargo.toml
   grep '^version' crates/chatman-common/Cargo.toml
   grep '^version' crates/praxis-retrofit/Cargo.toml
   ```
   All three must show `version = "26.6.30"`.

2. Check chicago-tdd-tools pin in praxis-retrofit:
   ```bash
   grep 'chicago-tdd-tools' crates/praxis-retrofit/Cargo.toml | head -1
   ```
   Must show `chicago-tdd-tools = "=26.6.30"`.

3. Build the workspace:
   ```bash
   cargo build --workspace
   ```
   Must complete with exit code 0.
