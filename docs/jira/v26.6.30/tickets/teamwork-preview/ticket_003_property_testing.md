# Ticket: Enforce Configuration Invariants with chicago-tdd-tools and Property-Based Testing

## Title
Enforce Configuration Invariants with chicago-tdd-tools and Property-Based Testing

## Description
To ensure fleet auditing reliability, we must enforce robust configuration validation and verify parser resilience. Currently, `praxis-retrofit` deserializes `repos.toml` into raw types (`u8`, `usize`, `String`) without validating invariants (e.g., that `crate_count` is positive or `priority_score` is within 0–100). Additionally, there is no property-based testing verifying that the parser is resilient against malformed or malicious TOML payloads.

This ticket requires:
1. Integrating `chicago-tdd-tools` poka-yoke wrappers (such as `PositiveU32` or `PositiveUsize` / `BoundedU32`) to validate fields in `repos.toml` during deserialization or during registry initialization.
2. Refactoring `RepositoryRegistry::load` and `RegistryDocument::into_registry` to return structured `RetrofitError::ConfigError` (rather than panicking) if validation fails.
3. Adding the `proptest` crate to the dev-dependencies of `praxis-retrofit`.
4. Creating a property-based test suite at `crates/praxis-retrofit/tests/property_tests.rs` using `proptest` and `chicago-tdd-tools` primitives to assert that arbitrary input strings (binary, corrupted TOML, random unicode) do not cause panics and are safely caught as errors.

## Acceptance Criteria
- **Poka-Yoke Field Validation**: Refactor `RepositoryEntry` or the loader in `repo_registry.rs` to validate the following invariants:
  - `crate_count` must be a positive integer, validated via `PositiveUsize::new(crate_count)`.
  - `priority_score` must be between 0 and 100, validated using type-level bounds (e.g. `BoundedU32::new(priority_score as u32)` or an explicit range check).
  - Validation failures must return a detailed `Result::Err(RetrofitError::ConfigError(msg))` stating exactly which field and value failed the invariant.
- **Cargo Manifest Configuration**: Add `proptest` as a dev-dependency in `/Users/sac/praxis/crates/praxis-retrofit/Cargo.toml`:
  ```toml
  [dev-dependencies]
  proptest = "1.4"
  ```
- **Property Test Suite**: The file `/Users/sac/praxis/crates/praxis-retrofit/tests/property_tests.rs` is created and contains property tests validating parser resilience:
  ```rust
  use proptest::prelude::*;
  use praxis_retrofit::repo_registry::RepositoryRegistry;

  proptest! {
      #[test]
      fn test_parsing_resilience(ref s in "\\PC*") {
          // Assert that parsing arbitrary strings never triggers a panic
          // and returns a structured Error for invalid data
          let result = tokio::runtime::Runtime::new()
              .unwrap()
              .block_on(async {
                  // Setup temporary mock file containing string `s` or parse directly
                  RepositoryRegistry::load_str(s).await
              });
          
          // Ensure it either parses successfully or fails gracefully
          assert!(result.is_ok() || result.is_err());
      }
  }
  ```
- **Clean Execution**: Running `cargo test --test property_tests` runs a minimum of 100 iterations of random inputs and passes with zero failures.

## Dependencies
- ticket_002_config_layering

## Verification Mechanism
Execute the following verification steps:
1. Verify that `proptest` is added to dev-dependencies:
   ```bash
   grep "proptest" /Users/sac/praxis/crates/praxis-retrofit/Cargo.toml
   ```
2. Verify that the property test suite exists:
   ```bash
   test -f /Users/sac/praxis/crates/praxis-retrofit/tests/property_tests.rs
   ```
3. Run the property tests:
   ```bash
   cargo test --test property_tests
   ```
   Verify that all tests pass.
