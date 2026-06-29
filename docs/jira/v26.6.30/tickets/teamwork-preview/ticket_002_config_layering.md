# Ticket: Integrate chicago-tdd-tools Configuration Layering in repo-registry

## Title
Integrate chicago-tdd-tools Configuration Layering in repo-registry

## Description
Currently, the parsing of `repos.toml` in `/Users/sac/praxis/crates/praxis-retrofit/src/repo_registry.rs` loads configuration from a hardcoded relative path provided as a parameter to the `load` function. This makes it difficult to run the audit tools from nested crate directories or customize the registry location via the environment.

This ticket requires:
1. Adding the `chicago-tdd-tools` crate as a dependency to the `/Users/sac/praxis/crates/praxis-retrofit/Cargo.toml` manifest.
2. Refactoring `RepositoryRegistry::load` in `/Users/sac/praxis/crates/praxis-retrofit/src/repo_registry.rs` to load the TOML configuration using layered path locations:
   - Check the `PRAXIS_REGISTRY_PATH` environment variable. If set, load from that path.
   - If not set, check for `repos.toml` in the current directory and search upwards through parent directories (up to 5 levels), mirroring the configuration file locator pattern in `chicago-tdd-tools`.
   - Fall back to the passed path parameter if neither of the above resolves a file.
3. Adding unit tests within `repo_registry.rs` that verify the environment override behavior, the upward traversal fallback behavior, and default fallback behavior.

## Acceptance Criteria
- **Dependency Inclusion**: `chicago-tdd-tools` is added as a dependency in `/Users/sac/praxis/crates/praxis-retrofit/Cargo.toml` pointing to the workspace copy (e.g., `chicago-tdd-tools = { path = "../../../chicago-tdd-tools" }` or equivalent).
- **Layering Resolution Logic**: Refactor `RepositoryRegistry::load` to implement path resolution:
  ```rust
  let registry_path = if let Ok(env_path) = std::env::var("PRAXIS_REGISTRY_PATH") {
      if !env_path.is_empty() {
          std::path::PathBuf::from(env_path)
      } else {
          resolve_layered_path(path.as_ref())
      }
  } else {
      resolve_layered_path(path.as_ref())
  };
  ```
  Where `resolve_layered_path` searches upwards from the current directory for `repos.toml` up to 5 levels, falling back to the parameter.
- **Unit Tests**:
  - `test_load_with_env_var_override`: Sets `PRAXIS_REGISTRY_PATH` to a temporary file path, writes a mock registry, calls `RepositoryRegistry::load`, and asserts it loads from the env path.
  - `test_load_with_parent_directory_search`: Creates a temp directory hierarchy, places `repos.toml` in the parent directory, executes the resolver from a child directory, and asserts it locates the parent registry file.
  - `test_load_fallback_to_parameter`: Asserts that if no environment variable or parent-directory files exist, it falls back to the exact path parameter.
- **Clean Compilation**: The `praxis-retrofit` crate compiles cleanly with no warnings or errors.

## Dependencies
- ticket_001_setup_cicd

## Verification Mechanism
Execute the following verification steps:
1. Verify the dependency entry in `/Users/sac/praxis/crates/praxis-retrofit/Cargo.toml`:
   ```bash
   grep "chicago-tdd-tools" /Users/sac/praxis/crates/praxis-retrofit/Cargo.toml
   ```
2. Verify that the new unit tests compile and pass successfully:
   ```bash
   cargo test --package praxis-retrofit --lib repo_registry::tests
   ```
   The test suite must compile and report all tests as passed.
