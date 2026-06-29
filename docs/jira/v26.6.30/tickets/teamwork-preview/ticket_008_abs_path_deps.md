# Ticket: Replace Absolute External Path Dependencies with Registry Pins

## Title
Replace Absolute External Path Dependencies with Registry Pins

## Description
The root `Cargo.toml` (and all playground/template copies) currently reference two external crates via absolute file paths hardcoded to `/Users/sac/`:
- `lsp-max = { path = "/Users/sac/lsp-max", ... }`
- `clap-noun-verb = { path = "/Users/sac/clap-noun-verb" }`

These absolute paths break on any machine other than Sean's workstation and prevent the workspace from building in CI or on contributor machines. `clap-noun-verb` is already published to crates.io at version `26.6.2`; `lsp-max` needs a registry version resolution.

This is a **portability blocker** — CI and contributor onboarding will fail without this fix.

## Acceptance Criteria
- **clap-noun-verb Resolved**: All path deps on `/Users/sac/clap-noun-verb` in root and template/playground `Cargo.toml` files are replaced with:
  ```toml
  clap-noun-verb = "26.6.2"
  clap-noun-verb-macros = "26.6.2"
  ```
- **lsp-max Resolved**: All path deps on `/Users/sac/lsp-max` in root and playground `Cargo.toml` files are replaced with one of:
  - A `version = "X.Y.Z"` registry pin (if lsp-max is published to crates.io), OR
  - A `[patch.crates-io]` override using a relative path (e.g., `path = "../lsp-max"`), OR
  - If the dependency is truly optional, gate it on a feature and document the manual build requirements
- **No Absolute /Users/sac/ Paths**: `grep -r "/Users/sac/" Cargo.toml` from `/Users/sac/praxis` returns 0 matches
- **Build Success on Fresh Checkout**: `cargo build --workspace` and `cargo test --workspace` both succeed without requiring `/Users/sac/` siblings to exist on disk

## Dependencies
None.

## Verification Mechanism
Execute the following verification steps from `/Users/sac/praxis`:
1. Search for absolute paths:
   ```bash
   grep -r "/Users/sac/" Cargo.toml crates/*/Cargo.toml playground/*/Cargo.toml template*/Cargo.toml 2>/dev/null | grep -v "examples\|comments"
   ```
   Must return 0 results.

2. Verify clap-noun-verb versions:
   ```bash
   grep 'clap-noun-verb' Cargo.toml | head -2
   ```
   Must show registry versions (e.g., `"26.6.2"`), not path deps.

3. Build the workspace:
   ```bash
   cargo build --workspace
   ```
   Must complete with exit code 0.

4. Run tests:
   ```bash
   cargo test --workspace
   ```
   Must complete with exit code 0.
