# Milestone Overview: teamwork-preview

This document provides the execution sequence and dependency graph for the `teamwork-preview` milestone tickets, which integrate `cargo-cicd`, `chicago-tdd-tools`, and OCEL 2.0 tracing into the Praxis workspace.

## Milestone Objective
The objective of this milestone is to establish local-first CI/CD checks, configure target directory pruning, introduce configuration path layering, enforce TOML parsing invariants through type-level validations and property-based testing, and capture Object-Centric Event Logs (OCEL 2.0) across the compilation and test execution pipelines.

## Execution Sequence & Dependency Graph

```
[ticket_001_setup_cicd] (Configure cargo-cicd & target directory size targets)
       │
       ▼
[ticket_002_config_layering] (Integrate configuration layering & path resolution)
       │
       ▼
[ticket_003_property_testing] (Adopt type-level invariants & proptest resilience)
       │
       ▼
[ticket_004_ocel_tracing] (Implement OCEL 2.0 tracing for builds and test suites)
```

---

## Ticket Index

### 1. [ticket_001_setup_cicd.md](ticket_001_setup_cicd.md)
* **Title**: Configure cargo-cicd for Praxis Workspace and CI/CD Pipeline
* **Description**: Setup a `cicd.toml` file at the root of the workspace to limit target directory growth to 5GB, configure autonomic feedback modes, and integrate local developer tasks (`test-changed`, `clean-stale`) in `justfile` and the standard `cargo cicd workspace doctor` check in GitHub Actions.
* **Dependencies**: None.
* **Primary Verification**: Run `cargo cicd workspace doctor` and `cargo cicd target prune` from the workspace root.

### 2. [ticket_002_config_layering.md](ticket_002_config_layering.md)
* **Title**: Integrate chicago-tdd-tools Configuration Layering in repo-registry
* **Description**: Incorporate the `chicago-tdd-tools` workspace library as a dependency in `praxis-retrofit`. Refactor the TOML file loader in `repo_registry.rs` to support path resolution layering via the `PRAXIS_REGISTRY_PATH` environment variable or by performing parent-directory upward directory scans for `repos.toml`.
* **Dependencies**: `ticket_001_setup_cicd`
* **Primary Verification**: Run `cargo test --package praxis-retrofit` to execute unit tests verifying the fallback locator and environment variable override logic.

### 3. [ticket_003_property_testing.md](ticket_003_property_testing.md)
* **Title**: Enforce Configuration Invariants with chicago-tdd-tools and Property-Based Testing
* **Description**: Replace raw data structures with type-level poka-yoke wrappers (`PositiveUsize`, `BoundedU32`) to enforce TOML fields (such as `crate_count` and `priority_score`) invariants, making invalid states unrepresentable. Add `proptest` dev-dependencies and build a property-based test harness under `crates/praxis-retrofit/tests/property_tests.rs` to assert parser resilience against arbitrary invalid payloads.
* **Dependencies**: `ticket_002_config_layering`
* **Primary Verification**: Run `cargo test --test property_tests` to execute 100+ random fuzz iterations of the TOML parsing interface.

### 4. [ticket_004_ocel_tracing.md](ticket_004_ocel_tracing.md)
* **Title**: Implement Ocel Event Logging and Build Trace Generation
* **Description**: Embed a `build.rs` compile-time hook to log compilation metrics. Extend `cargo-cicd` trace commands to support the `build` profile. Integrate `OcelCollector` as a `DiagnosticSink` in the test suites to write structural test assertions, durations, and cryptographic receipts to `target/praxis/evidence/events.ocel.json`.
* **Dependencies**: `ticket_003_property_testing`
* **Primary Verification**: Build the crate to verify `build_event.jsonl` output, and run the test suite to verify generated output in `target/praxis/evidence/events.ocel.json`.
