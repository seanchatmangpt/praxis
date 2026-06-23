# Getting Started with Praxis

Praxis is a house-style Rust boilerplate kit for the seanchatmangpt fleet. It provides:
- **Canonical project templates** (single-crate, workspace, WASM, MCP, integration)
- **Shared house primitives** (`chatman-common` crate with error handling, hashing, testing)
- **Standardized CI/CD, linting, versioning** (CalVer, forbid unsafe, deny stubs)
- **Migration tooling** (`apply.sh` backfills standards into existing repos)

This guide walks you through choosing a template, scaffolding your project, and publishing your first release.

---

## **1. Choose Your Template**

Praxis provides specialized templates for different project shapes. Pick the one that matches your use case:

| Template | Purpose | When to Use |
|----------|---------|------------|
| **`template/`** | Standard single-crate CLI/library | Most projects: CLIs, libraries, servers |
| **`template-wasm/`** | WebAssembly library + tools | WASM targets (wasm32-*); handles stripping correctly |
| **`template-mcp/`** | LLM Model Context Protocol server | MCP tool servers (Claude, etc.) |
| **`template-integration/`** | Integration test harness | Docker-backed tests, ephemeral services |
| **`template-workspace/`** | Multi-crate workspace | Fleet with 3+ interdependent crates |

**Decision tree:**
- Building a CLI tool or library? → `template/`
- Building a WASM library? → `template-wasm/`
- Exposing tools to Claude or other LLMs? → `template-mcp/`
- Need Docker integration tests? → `template-integration/`
- Managing 3+ related crates? → `template-workspace/`

---

## **2. Generate Your Project**

### Using `cargo generate`

```bash
# Install if not already present
cargo install cargo-generate

# Generate from the canonical template
cargo generate --git https://github.com/seanchatmangpt/praxis \
  --name my-project

# OR specify a different template
cargo generate --git https://github.com/seanchatmangpt/praxis \
  template-mcp \
  --name my-mcp-server

# Follow the prompts:
#   Project name: my-project
#   Description: My awesome CLI tool
```

This creates `my-project/` with all house-standard files in place.

### Using `apply.sh` on an Existing Repo

If you already have a Rust project:

```bash
# Clone praxis
git clone https://github.com/seanchatmangpt/praxis /tmp/praxis

# Dry-run to see what would change
/tmp/praxis/apply.sh . --dry-run

# Apply standards (creates/overwrites config files, CI, etc.)
/tmp/praxis/apply.sh .

# Review changes
git status
git diff --stat

# Commit
git add .
git commit -m "chore: adopt praxis house standards"
```

**Important:** `apply.sh` never overwrites `Cargo.toml`. You may need to manually merge lints and dependencies.

---

## **3. Customize Your `Cargo.toml`**

Praxis templates use `cargo-generate` placeholders. After generating, edit key sections:

```toml
[package]
name = "my-project"                      # ← Updated automatically
version = "26.6.0"                       # ← CalVer: YY.M.patch
edition = "2021"
rust-version = "1.82"                    # ← House MSRV (or bump as needed)
authors = ["Your Name <you@example.com>"]
license = "MIT OR Apache-2.0"            # ← House default
description = "Your project description"
repository = "https://github.com/YOUR_ORG/my-project"

[dependencies]
# For single-crate projects using chatman-common:
chatman-common = { git = "https://github.com/seanchatmangpt/chatman-common" }
# OR when published to crates.io:
# chatman-common = "26.6"

# Add your domain-specific dependencies
```

### Workspace Projects

If you used `template-workspace/`, update the root `Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = ["crates/core", "crates/cli", "crates/wasm"]

[workspace.package]
version = "26.6.0"
edition = "2021"
rust-version = "1.82"

[workspace.lints]
# All crates inherit these lint rules
```

Then each member uses:
```toml
version.workspace = true
edition.workspace = true
lints.workspace = true
```

---

## **4. Run the CI Gate Locally**

Before pushing, ensure all checks pass:

```bash
# Install prerequisites (if not already done)
cargo install cargo-deny typos-cli just

# Run the full CI gate
just ci

# This runs:
#   1. cargo fmt --check
#   2. cargo clippy -- -D warnings
#   3. cargo test --workspace --all-features
#   4. cargo doc --workspace --no-deps
#   5. cargo deny check
#   6. typos --check
```

If any step fails, fix and re-run.

---

## **5. Write Your Code**

### Project Layout

```
my-project/
├── src/
│   ├── lib.rs              # Public API surface
│   ├── main.rs             # Binary (optional; thin wrapper)
│   ├── error.rs            # thiserror enum (domain errors)
│   ├── types.rs            # Domain types (data only, no logic)
│   ├── cli.rs              # Clap configuration
│   ├── handlers.rs         # Dispatch/business logic
│   └── verbs/              # One file per CLI subcommand
│       ├── mod.rs
│       ├── build.rs
│       └── verify.rs
├── tests/                  # Integration tests
├── examples/               # `cargo run --example name`
├── benches/                # Criterion benchmarks
├── justfile                # Task runner (main source of truth)
├── Cargo.toml
├── rust-toolchain.toml     # Pinned stable Rust 1.82
└── CLAUDE.md               # Developer guide for this project
```

### Key Conventions

1. **No `unwrap`/`expect` in libraries.** Return `Result` with proper error types.
2. **Use `?` operator** to propagate errors.
3. **Define errors in `error.rs`** using `thiserror`:
   ```rust
   use thiserror::Error;

   #[derive(Error, Debug)]
   pub enum Error {
       #[error("io error: {0}")]
       Io(#[from] std::io::Error),
       #[error("invalid format: {0}")]
       InvalidFormat(String),
   }
   ```
4. **Seal types that require validation:**
   ```rust
   pub struct Receipt {
       pub hash: String,
       _seal: (),  // Prevents casual construction
   }
   ```
5. **Add rustdoc to public items:**
   ```rust
   /// Verifies the integrity of a sealed receipt.
   pub fn verify(receipt: &Receipt) -> Result<bool> { ... }
   ```

### Adding a CLI Verb

1. Create `src/verbs/my_verb.rs`
2. Define args: `#[derive(Args)]`
3. Implement handler: `pub async fn handle_my_verb(args: MyVerbArgs) -> Result<()>`
4. Add variant to `Cli` enum in `src/cli.rs`
5. Wire match arm in `src/main.rs`

See `CLAUDE.md` in your project for detailed examples.

---

## **6. Test Your Project**

```bash
# Run all tests
just test

# Run a specific test
cargo test my_test_name

# Run with backtrace for failures
RUST_BACKTRACE=1 cargo test

# Determinism check (single-threaded, for hash stability)
cargo test -- --test-threads=1

# Benchmarks
cargo bench

# Code coverage
just coverage
# Opens HTML report in target/coverage/index.html
```

For integration tests using Docker, see `template-integration/CLAUDE.md`.

---

## **7. Document Your Project**

1. **Update `README.md`:**
   - What problem does this solve?
   - Quick start (installation, basic usage)
   - Links to detailed docs

2. **Update `CLAUDE.md`:**
   - Architecture overview
   - Module descriptions
   - Key patterns used
   - Troubleshooting

3. **Add rustdoc:**
   ```bash
   just doc
   # Opens docs in browser
   ```

4. **Update `CHANGELOG.md`:**
   ```markdown
   ## [Unreleased]

   ### Added
   - New `verify` subcommand for receipt validation

   ### Fixed
   - Hash stability issue in parallel test execution

   ### Changed
   - Updated to chatman-common 26.6.0
   ```

---

## **8. Tag & Release**

Praxis uses **CalVer versioning**: `YY.M.patch`

```bash
# Update version in Cargo.toml
# Bump to next month or increment patch for same-month releases
# Example: 26.6.0 (June 2026, patch 0)

# Update CHANGELOG.md: move [Unreleased] section to [26.6.0]

# Commit
git add Cargo.toml CHANGELOG.md
git commit -m "chore: release 26.6.0"

# Tag
git tag v26.6.0

# Push
git push origin main
git push origin v26.6.0

# GitHub releases auto-generated from tag (if .github/workflows/release.yml is in place)
# Or manually create release via GitHub UI
```

**CI automatically:**
- Builds multi-platform binaries (Linux x86_64/aarch64, macOS aarch64, Windows MSVC)
- Uploads artifacts to GitHub Releases
- Publishes to crates.io (if `CARGO_REGISTRY_TOKEN` secret is set)

---

## **9. Integrate with `chatman-common`**

The `chatman-common` crate provides house primitives:

### Error Handling
```rust
use chatman_common::Error;

// Unified error type with FM codes
#[error("failed to verify (FM_VERIFY_001): {0}")]
pub struct VerifyError(String);
```

### Content Addressing (BLAKE3)
```rust
use chatman_common::chain::fold_event;

// Deterministic hash of a blob
let hash = fold_event("previous_hash_hex", b"event_payload");
```

### Testing Helpers
```rust
use chatman_common::testkit::{TestReceipt, assert_golden};

#[test]
fn my_test() {
    let receipt = TestReceipt::capture("my_operation", || {
        // your code here
        Ok(())
    }).unwrap();
    
    assert_golden("receipts/my_operation", &receipt);
}
```

### Telemetry
```rust
use chatman_common::cli::init_tracing;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing("my-project")?;
    // logs and traces now available
    Ok(())
}
```

See `crates/chatman-common/DESIGN.md` for full API reference.

---

## **Troubleshooting**

### "failed to format code"
```bash
cargo fmt --all
just ci
```

### "clippy: unimplemented" error
Remove the `unimplemented!()` stubs and replace with proper error handling.

### "denied license"
Run `cargo deny check` to see which dep. Add exception to `deny.toml` with justification.

### "hash mismatch"
Ensure you're using `chatman_common::canonical_json()` for all serialization before hashing.

### "linkme slice is empty"
Add `use crate::verbs;` in `main.rs` to ensure verb crates are linked.

---

## **Next Steps**

1. **Read the template's `CLAUDE.md`** for architecture and patterns
2. **Review `survey/00-SYNTHESIS.md`** for house-wide design decisions
3. **Check `CHECKLIST.md`** for migration checklist if retrofitting an existing repo
4. **Join the Praxis discussions** for questions or contributions

---

## **Quick Reference**

| Task | Command |
|------|---------|
| Create project | `cargo generate --git https://github.com/seanchatmangpt/praxis --name my-project` |
| Run checks | `just ci` |
| Build | `cargo build --release` |
| Test | `just test` |
| Docs | `just doc` |
| Format | `just fmt` |
| Lint | `just lint` |
| Release | `git tag vYY.M.patch && git push origin vYY.M.patch` |

---

**Questions?** See `docs/faq.md` or `docs/troubleshooting.md`.
