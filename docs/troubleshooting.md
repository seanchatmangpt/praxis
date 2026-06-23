# Praxis Troubleshooting Guide

Solutions to common issues when using Praxis.

---

## **Installation & Setup**

### "cargo generate: command not found"
```bash
cargo install cargo-generate
cargo generate --git https://github.com/seanchatmangpt/praxis --name my-project
```

### "rust-toolchain.toml: toolchain not installed"
```bash
rustup update
# Rust 1.82 is automatically installed from rust-toolchain.toml
rustc --version  # Should be 1.82.0 or later
```

### "just: command not found"
```bash
cargo install just
```

### "apply.sh: Permission denied"
```bash
chmod +x /tmp/praxis/apply.sh
/tmp/praxis/apply.sh . --dry-run
```

---

## **Build & Compilation**

### "error: failed to resolve: use of undeclared crate"
**Symptom:** `use chatman_common::...` doesn't resolve

**Fix:** Ensure `Cargo.toml` has chatman-common dependency:
```toml
[dependencies]
chatman-common = { git = "https://github.com/seanchatmangpt/chatman-common" }
# OR
chatman-common = "26.6"  # once published
```

Then: `cargo build`

### "error[E0451]: field `_seal` is private"
**Symptom:** Cannot construct a sealed type

**Root cause:** Sealed types have a private `_seal: ()` field that prevents casual construction. This is intentional.

**Fix:** Use the canonical builder:
```rust
// ❌ Won't compile
let receipt = Receipt { events: vec![], chain_hash: String::new(), _seal: () };

// ✅ Correct
let receipt = ChainAssembler::new()
    .push(event1)?
    .push(event2)?
    .finalize()?;
```

### "error: linking with `cc` failed"
**Symptom:** Linker error on native build

**Common causes:**
- Missing system dependencies (libssl-dev, etc.)
- Cross-compilation without proper target installed

**Fix:**
```bash
# Ensure all system dependencies
sudo apt-get install build-essential libssl-dev pkg-config  # Linux
brew install openssl                                        # macOS

# For cross-compilation
rustup target add x86_64-unknown-linux-gnu
cargo build --target x86_64-unknown-linux-gnu
```

### "error: failed to fetch from git repository"
**Symptom:** Cannot clone chatman-common or other git dependencies

**Causes:**
- Network is down
- GitHub SSH key not configured
- Repo is private and you lack credentials

**Fix:**
```bash
# Check network
ping github.com

# Configure SSH (if using git@github)
ssh-keygen -t ed25519 -C "your@email.com"
# Add public key to GitHub: Settings → SSH and GPG keys

# Or use HTTPS with PAT:
# Update Cargo.toml to use https:// URLs
# Credentials in ~/.config/git/credentials (or use gh auth)
```

---

## **Testing**

### "test failed: assertion error"
**Fix:** Read the assertion message carefully. Example:
```
assertion `left == right` failed
  left: "abc123"
 right: "xyz789"
```

Debug by:
```bash
RUST_LOG=debug cargo test <test_name> -- --nocapture
```

### "test timed out"
**Symptom:** Test takes > 30 seconds and is killed

**Causes:**
- Deadlock in async code
- Infinite loop
- Slow I/O (network, disk)

**Fix:**
```rust
#[tokio::test]
#[tokio::time::timeout(5 seconds)]  // Add timeout
async fn my_test() { ... }
```

Or increase the timeout in CI `.github/workflows/ci.yml`.

### "error: `#[test]` fixture not found"
**Symptom:** Test harness can't find the test

**Fix:** Ensure test is marked correctly:
```rust
#[test]
fn my_test() { ... }

#[tokio::test]
async fn my_async_test() { ... }
```

### "hash mismatch: expected X but got Y"
**Symptom:** BLAKE3 hash doesn't match expected value

**Causes:**
- Non-canonical JSON serialization (key order matters)
- Whitespace differences
- Different byte encoding

**Fix:**
```rust
// ❌ Wrong
let json = serde_json::to_string_pretty(&event)?;
let hash = blake3::hash(json.as_bytes()).to_hex().to_string();

// ✅ Correct
let bytes = chatman_common::canonical_json(&event)?;
let hash = blake3::hash(&bytes).to_hex().to_string();
```

Always use `chatman_common::canonical_json` before hashing.

### "tests failed with `--test-threads=1` but passed with parallel"
**Symptom:** Determinism issue

**Causes:**
- Test modifies global state (env vars, files, logging)
- Tests depend on execution order
- Race condition in code being tested

**Fix:**
```rust
#[test]
fn my_test() {
    // Restore env after test
    let original = std::env::var("MY_VAR").ok();
    std::env::set_var("MY_VAR", "test_value");
    // ... test code ...
    if let Some(val) = original {
        std::env::set_var("MY_VAR", val);
    } else {
        std::env::remove_var("MY_VAR");
    }
}
```

Or use `serial_test`:
```rust
#[test]
#[serial]
fn my_serial_test() { ... }
```

---

## **Linting & Formatting**

### "`just ci` fails on `cargo fmt --check`"
**Symptom:** Code not formatted correctly

**Fix:**
```bash
cargo fmt --all
just ci  # Re-run to verify
```

### "error: expected one of ..., found `_`"
**Symptom:** Clippy/rustfmt error after formatting

**Cause:** Usually a macro or macro-generated code issue

**Fix:**
```rust
// Add allow attribute
#[allow(rustfmt::skip)]
macro_rules! my_macro { ... }
```

Or simplify the code to avoid the issue.

### "`just lint` fails: unused variable"
**Symptom:** Clippy warns about an unused variable

**Fix:**
```rust
// ❌
let x = expensive_computation();  // Clippy: unused

// ✅
let _x = expensive_computation();  // If truly unused (e.g., for side effects)

// OR use it
let x = expensive_computation();
println!("{}", x);
```

### "error: `todo!` macro is forbidden"
**Symptom:** Can't ship code with `todo!` or `unimplemented!`

**Fix:** Replace with proper error:
```rust
// ❌
fn my_function() -> Result<i32> {
    todo!("implement this")
}

// ✅
fn my_function() -> Result<i32> {
    Err(anyhow::anyhow!("feature not yet implemented"))
}
```

### "`cargo deny check` fails: license denied"
**Symptom:** Transitive dependency has disallowed license

**Fix:**
1. Check the dependency:
   ```bash
   cargo tree | grep <package>
   ```

2. If it's legitimate, add exception to `deny.toml`:
   ```toml
   [[licenses.exceptions]]
   allow = ["GPL-2.0"]
   name = "legacy-package"
   version = "*"
   # Justification: ...
   ```

3. Or update the dependency to a compatible version.

### "`typos` flags a domain term"
**Symptom:** Spell-checker flags "wasm4pm" or other project-specific terms

**Fix:** Add to `typos.toml`:
```toml
[default.extend-words]
wasm4pm = "wasm4pm"
chatman = "chatman"
```

---

## **CLI & Verbs**

### "error: missing required argument"
**Symptom:** CLI verb is missing an argument

**Example:**
```bash
my-project verify  # Missing --receipt argument
```

**Fix:**
```bash
my-project verify --receipt path/to/receipt.json
# Or check help
my-project verify --help
```

### "error: unknown subcommand"
**Symptom:** Verb doesn't exist

**Fix:**
```bash
my-project --help  # List available verbs
```

Ensure the verb is:
1. Defined in `src/cli.rs`
2. Exported in `src/verbs/mod.rs`
3. Wired in `src/main.rs` match statement
4. The handler file exists

### "linkme distributed_slice is empty"
**Symptom:** Handlers registered via linkme aren't being discovered

**Causes:**
- Handler crate not linked (dead-stripped)
- Module not imported in `main.rs`

**Fix:**
```rust
// src/main.rs
mod verbs;  // Ensure this line exists
use verbs::*;  // Or use explicit imports
```

---

## **Dependencies & Features**

### "feature 'mcp' not found"
**Symptom:** `--features mcp` fails

**Fix:** Ensure `Cargo.toml` has the feature defined:
```toml
[features]
mcp = ["dep:rmcp", "dep:rmcp-macros"]

[dependencies]
rmcp = { version = "0.11", optional = true }
rmcp-macros = { version = "0.11", optional = true }
```

### "cannot find function in nested module"
**Symptom:** Private item expected to be public

**Fix:**
```rust
// ❌ Not accessible
mod helpers {
    fn my_helper() { ... }
}

// ✅ Make public
mod helpers {
    pub fn my_helper() { ... }
}
```

Or import from parent: `use crate::helpers::my_helper`

### "error: unexpected `cfg` condition"
**Symptom:** Feature flag name is misspelled or unknown

**Example:**
```rust
#[cfg(feature = "fase-2")]  // ❌ Typo: should be "phase-2"
```

**Fix:** Match exactly to feature name in `Cargo.toml`:
```toml
[features]
phase-2 = ["phase-1"]
```

```rust
#[cfg(feature = "phase-2")]  // ✅ Correct
```

---

## **WASM-Specific**

### "error: crate `X` is not WASM-compatible"
**Symptom:** Dependency doesn't work on wasm32 target

**Fix:**
1. Check if library supports WASM: `cargo build --target wasm32-unknown-unknown`
2. Or use platform-specific dep:
   ```toml
   [target.'cfg(not(target_arch = "wasm32"))'.dependencies]
   not-wasm-only = "1.0"
   ```

### "`wasm-pack build` produces zero-length .wasm file"
**Symptom:** WASM binary is 0 bytes

**Cause:** Usually `strip = true` in release profile

**Fix:**
```toml
[profile.release]
# strip = false  (must be explicitly false or omitted)
opt-level = "s"  # size-optimized
```

Then: `wasm-pack build --release`

### "TextEncoder is not defined (JS error)"
**Symptom:** WASM calls JS TextEncoder but it's not available

**Fix:** Add to JavaScript that loads WASM:
```javascript
import { MyWasmModule } from './pkg/index.js';

// TextEncoder is available in modern browsers and Node.js 11+
// If needed in older browsers, polyfill:
if (typeof window !== 'undefined' && !window.TextEncoder) {
    // Use a polyfill
}
```

Or use `chatman-common::canonical_json` which handles encoding internally.

---

## **MCP Server**

### "error: tool is not registered"
**Symptom:** MCP tool doesn't appear in LLM

**Fix:**
1. Ensure tool is marked with `#[tool]` macro
2. Check tool struct name and parameters
3. Verify tool is in a loaded module

### "JSON-RPC request timeout"
**Symptom:** LLM call hangs or times out

**Causes:**
- Tool implementation is slow
- Tool is waiting for I/O (database, file, network)
- Infinite loop or deadlock

**Fix:**
1. Add timeouts to I/O operations
2. Use async/await properly
3. Test tool directly: `cargo test`

---

## **Release & Publishing**

### "error: version in Cargo.toml doesn't match git tag"
**Symptom:** `cargo publish` fails because version mismatch

**Example:**
- `Cargo.toml`: version = "26.6.0"
- Git tag: v26.6.1

**Fix:**
```bash
# Update Cargo.toml
# OR re-tag with correct version
git tag -d v26.6.1
git tag v26.6.0
git push origin v26.6.0 --force
```

### "error: unauthorized: need to login"
**Symptom:** Cannot publish to crates.io

**Fix:**
```bash
cargo login <your_crates_io_token>
# Token from: https://crates.io/me

# Then:
cargo publish
```

### "error: cannot overwrite crate"
**Symptom:** Version already published to crates.io

**Solution:** Publish a new patch or minor version (never re-publish the same version).

---

## **Performance Issues**

### "Binary is very large"
**Symptom:** `target/release/<binary>` is > 100 MB

**Causes:**
- Debug symbols included
- LTO not enabled
- Large dependencies

**Fix:**
```toml
[profile.release]
lto = true              # Full LTO
codegen-units = 1      # Single codegen unit
panic = "abort"        # Smaller panic handling
```

Then: `cargo build --release`

### "Tests take too long"
**Symptom:** `just test` takes > 5 minutes

**Fix:**
```bash
# Run specific tests only
cargo test my_test_name

# Skip integration tests
cargo test --lib

# Parallel (default):
cargo test --workspace

# Single-threaded (for debugging):
cargo test -- --test-threads=1
```

---

## **Still Stuck?**

1. **Check `CLAUDE.md`** in your project template
2. **Review `survey/` docs** for design decisions
3. **Read agent findings** in `SYNTHESIS.md`
4. **File an issue** with:
   - Error message
   - Steps to reproduce
   - OS/Rust version
   - Template used

---

**Last resort:** Nuke and rebuild:
```bash
cargo clean
cargo build
just ci
```

This removes stale artifacts and starts fresh.
