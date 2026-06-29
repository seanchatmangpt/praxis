# Ticket: Configure cargo-cicd for Praxis Workspace and CI/CD Pipeline

## Title
Configure cargo-cicd for Praxis Workspace and CI/CD Pipeline

## Description
To establish standardized local CI/CD execution and target directory size policies across the Praxis workspace, we must configure `cargo-cicd`. Currently, the `/Users/sac/praxis` workspace does not have a `cicd.toml` configuration at its root directory, which prevents local-first automated checks and size enforcement. 

This ticket involves:
1. Creating a `cicd.toml` configuration file in the root of the Praxis workspace (`/Users/sac/praxis/cicd.toml`) to enforce target directory size pruning policies and configure autonomic feedback loops.
2. Modifying or creating the root `justfile` at `/Users/sac/praxis/justfile` to integrate automated developer commands (`test-changed` and `clean-stale`) powered by `cargo-cicd`.
3. Integrating the `cargo cicd workspace doctor` health checker in the GitHub Actions pipeline. If no root workflow exists, create `.github/workflows/ci.yml` to run the checker.

## Acceptance Criteria
- **Root Configuration File**: A new file is created at `/Users/sac/praxis/cicd.toml` with the following content:
  ```toml
  [target]
  max_size_gb = 5.0
  prune_after_days = 7

  [test.changed]
  base = "origin/main"

  [autonomic]
  enabled = true
  mode = "suggest"
  ```
- **Justfile Integration**: The workspace root `justfile` at `/Users/sac/praxis/justfile` is updated (or created) to contain the following two recipes:
  ```just
  # Run only tests affected by changes
  test-changed:
      cargo cicd test changed

  # Check target directory size and prune
  clean-stale:
      cargo cicd target prune
  ```
- **CI/CD Pipeline Integration**: The GitHub Actions workflow file `/Users/sac/praxis/.github/workflows/ci.yml` is updated (or created) to run the workspace doctor:
  ```yaml
  name: CI
  on: [push, pull_request]
  jobs:
    doctor:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
        - name: Install Rust
          uses: dtolnay/rust-toolchain@stable
        - name: Install cargo-cicd
          run: cargo install cargo-cicd
        - name: Run Workspace Doctor
          run: cargo cicd workspace doctor
  ```
- **Pruning Verification**: The target directory size limits and pruning logic must be recognized by `cargo-cicd`. Running `cargo cicd target prune` must process target directories and complete with success.

## Dependencies
None.

## Verification Mechanism
Execute the following verification steps:
1. Verify the existence and content of `/Users/sac/praxis/cicd.toml`:
   ```bash
   cat /Users/sac/praxis/cicd.toml
   ```
2. Verify the existence and content of `/Users/sac/praxis/justfile`:
   ```bash
   cat /Users/sac/praxis/justfile
   ```
3. Run the following CLI commands from the root `/Users/sac/praxis` directory:
   ```bash
   cargo cicd workspace doctor
   cargo cicd target prune
   ```
   Both commands must terminate with exit code 0.
