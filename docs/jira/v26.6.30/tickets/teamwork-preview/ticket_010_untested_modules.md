# Ticket: Add Tests for Untested Core Modules

## Title
Add Tests for Untested Core Modules

## Description
Several critical modules have no test coverage:
1. **`src/discovery.rs`** — ecosystem discovery (currently a stub)
2. **`src/types.rs`** — integrity primitives and typestate machinery (the largest API surface)
3. **`crates/praxis-retrofit/src/apply.rs`** — retrofit application logic
4. **`crates/praxis-retrofit/src/audit.rs`** — compliance auditing
5. **`crates/praxis-retrofit/src/models.rs`** — core data types

These modules implement critical business logic and integrity guarantees. Without tests, refactoring is risky and regressions can silently occur. This ticket adds baseline test coverage to all untested modules.

## Acceptance Criteria
- **src/discovery.rs**: At least 1 `#[cfg(test)]` test module added covering the discovery flow (e.g., `test_discover_local_repos`, `test_discovery_empty_dir`, or similar)
- **src/types.rs**: At least 3 tests covering:
  - `Blake3Hash::content_address()` — verify hash correctness for known inputs
  - `Evidence<T, State, Witness>` — verify typestate transitions (Raw → Validated → Admitted)
  - `ValidationChain` — verify error accumulation and reporting (part of error.rs but interacts with types)
- **praxis-retrofit/src/apply.rs**: At least 1 test covering retrofit application logic (e.g., `test_apply_single_file`, `test_apply_with_permissions`)
- **praxis-retrofit/src/audit.rs**: At least 1 test covering audit logic (e.g., `test_audit_empty_repo`, `test_audit_compliance_gap`)
- **praxis-retrofit/src/models.rs**: At least 1 test covering model serialization/deserialization or validation (e.g., `test_retrofit_plan_roundtrip`)
- **All Tests Pass**: `cargo test --workspace` completes with exit code 0 and all new tests pass
- **Coverage Reported**: Optional but recommended: run `cargo tarpaulin --workspace --out Html` to generate coverage reports

## Dependencies
Depends on: **PRAXIS-009** (src/types.rs should be documented before tests are written to clarify intent).

## Verification Mechanism
Execute the following verification steps from `/Users/sac/praxis`:
1. Verify test modules exist:
   ```bash
   grep -n "^#\[cfg(test)\]" src/discovery.rs src/types.rs crates/praxis-retrofit/src/{apply,audit,models}.rs
   ```
   All 5 files must show at least one `#[cfg(test)]` line.

2. Count tests per module:
   ```bash
   cargo test --workspace --list 2>&1 | grep -E "discovery::|types::|apply::|audit::|models::" | wc -l
   ```
   Must show at least 1 test per module (5+ total).

3. Run all tests:
   ```bash
   cargo test --workspace
   ```
   Must complete with exit code 0 (all tests passing).

4. Optional coverage report:
   ```bash
   cargo tarpaulin --workspace --out Html
   ```
   Open `tarpaulin-report.html` and verify coverage increased for untested modules.
