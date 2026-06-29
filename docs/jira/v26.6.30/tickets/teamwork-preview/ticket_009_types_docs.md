# Ticket: Document src/types.rs Public API

## Title
Document src/types.rs Public API

## Description
The `/Users/sac/praxis/src/types.rs` module contains 25+ public items (structs, traits, type aliases, functions) but has **zero `///` documentation comments** on any of them. This is the single largest documentation gap in the codebase. Key undocumented types include:
- `Blake3Hash` — BLAKE3 content-addressing wrapper
- `ObjectRef` — OCEL-conformant object references
- `Evidence<T, State, Witness>` — typestate-driven generics for evidence tracking (Raw, Validated, Admitted states)
- `Admit` trait — evidence admission protocol
- `AdmittedReceipt` — cryptographic receipt structure
- `ProfileId` — profile identifier enum

All of these are core to praxis's integrity guarantees. Documentation is necessary for users of the library and for `cargo doc` coverage.

## Acceptance Criteria
- **Every pub Item Has `///` Doc Comment**: All `pub` structs, enums, traits, functions, type aliases, and const items in `src/types.rs` have a `///` comment block above them explaining what they are
- **Key Types Have `# Examples` Blocks**:
  - `Blake3Hash::content_address()` — example showing BLAKE3 hash generation
  - `Evidence<T, State, Witness>` — example showing state transitions (Raw → Validated → Admitted)
  - `Admit` trait — example showing custom implementation
- **Doc Build Passes**: `cargo doc --no-deps 2>&1 | grep -E "(warning|error).*types\.rs"` returns 0 matches
- **Missing Documentation Lint Clean**: Running `cargo clippy --all-targets -- -W missing_docs` does not produce warnings for `src/types.rs`

## Dependencies
None.

## Verification Mechanism
Execute the following verification steps from `/Users/sac/praxis`:
1. Check for missing doc comments:
   ```bash
   cargo doc --no-deps 2>&1 | grep "missing documentation" | grep -i types
   ```
   Must return 0 lines.

2. Verify specific items are documented:
   ```bash
   grep -A 2 "pub struct Blake3Hash\|pub trait Admit\|pub struct Evidence\|pub struct ObjectRef" src/types.rs | head -20
   ```
   All must have a `///` comment above them.

3. Check for Examples blocks:
   ```bash
   grep -E "^/// #.*[Ee]xample" src/types.rs
   ```
   Must show at least 3 `# Examples` blocks.

4. Run clippy lint:
   ```bash
   cargo clippy --all-targets -- -W missing_docs 2>&1 | grep "src/types.rs"
   ```
   Must return 0 lines.

5. Build and view docs locally:
   ```bash
   cargo doc --no-deps --open
   ```
   Navigate to `my_conforming_project::types` and verify all public items are documented and readable.
