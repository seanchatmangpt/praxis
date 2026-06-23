# Praxis FAQ

Frequently asked questions about choosing templates, adopting standards, and troubleshooting.

---

## **Template Selection**

### Q: My project uses WASM. Which template?
**A:** Use `template-wasm/`. It:
- Configures `crate-type = ["cdylib", "rlib"]` for WASM + native targets
- Ensures `strip = false` (stripping corrupts WASM binaries)
- Includes `getrandom { features = ["js"] }` for WASM compatibility
- Has size budgets: 500 KB mobile (opt-level="z"), 1 MB standard (opt-level="s")

Never use `strip = true` in WASM crates.

### Q: I'm building a CLI. Template or workspace?
**A:** Start with `template/`:
- **Single binary?** → `template/`
- **CLI + shared library?** → `template/` with internal module
- **3+ crates (CLI, library, tools)?** → `template-workspace/` with members

### Q: How do I expose tools to Claude?
**A:** Use `template-mcp/`. It:
- Scaffolds an MCP server using the `rmcp` SDK
- Uses `#[tool]` derive macros for tool registration
- Implements JSON-RPC over stdio (standard LLM integration)

See the template's `CLAUDE.md` for examples.

### Q: Do I need `template-integration/`?
**A:** Only if you have:
- Docker services (databases, caches, message queues)
- Ephemeral port binding (multiple tests in parallel)
- Network service dependencies

Otherwise, integrate tests in `tests/` in your main template.

### Q: What's the difference between `template/` and `template-workspace/`?
**A:**
- **`template/`**: Single crate (or multi-module lib + binary)
- **`template-workspace/`**: Multiple independently-versioned crates

Use workspace only if crates need independent release cycles. Otherwise, keep related code in a single crate.

---

## **Adoption & Migration**

### Q: I have an existing Rust project. How do I adopt praxis?
**A:** Use `apply.sh`:
```bash
git clone https://github.com/seanchatmangpt/praxis /tmp/praxis
/tmp/praxis/apply.sh . --dry-run    # Preview
/tmp/praxis/apply.sh .              # Apply
git status                          # Review
git add .
git commit -m "chore: adopt praxis standards"
```

This backfills:
- Linting config (`deny.toml`, `rustfmt.toml`, `typos.toml`)
- CI/CD workflows (`.github/workflows/`)
- Build config (`justfile`, `rust-toolchain.toml`)
- Licenses, templates, contributing guide

**What `apply.sh` does NOT touch:**
- `Cargo.toml` — you must manually merge dependencies and lints
- Source code — no refactoring is applied

### Q: Do I have to use CalVer versioning?
**A:** Yes, for fleet consistency. CalVer format: `YY.M.patch`
- `26.6.0` = June 2026, patch 0
- `26.6.1` = June 2026, patch 1 (same month bug fix)
- `26.7.0` = July 2026, patch 0

This standardizes versioning across the fleet and makes release timing clear.

### Q: Can I use a different edition (2020, 2024)?
**A:** House default is 2021. Using 2024 requires:
1. Rust >= 1.79 (2024 edition stabilized then)
2. Update `Cargo.toml`: `edition = "2024"`
3. Update `CLAUDE.md` documenting the change
4. Update MSRV if needed: `rust-version = "1.79"`

Stick with 2021 unless you have a specific need for 2024 features.

### Q: Can I opt out of specific praxis defaults?
**A:** Yes, but document it. Examples:
- **No `unsafe_code` forbid?** Set `unsafe_code = "warn"` in your crate with a comment
- **No `todo = "deny"`?** Change to `todo = "warn"` (but expect pushback in review)
- **Different MSRV?** Update `rust-version` and document in `CLAUDE.md`
- **Different license?** Use your own, but double-check it doesn't violate fleet policy

The house defaults exist for good reasons (security, consistency, stability). Opting out should be intentional and documented.

---

## **Versioning & Release**

### Q: How do I release a new version?
**A:**
1. Update `Cargo.toml`: bump version using CalVer (e.g., `26.6.0` → `26.6.1` or `26.7.0`)
2. Update `CHANGELOG.md`: move `[Unreleased]` section to `[26.6.1]`
3. Commit: `git commit -am "chore: release 26.6.1"`
4. Tag: `git tag v26.6.1`
5. Push: `git push origin main && git push origin v26.6.1`

CI then:
- Builds multi-platform binaries
- Uploads to GitHub Releases
- Publishes to crates.io (if secret is set)

### Q: Can I use semantic versioning instead of CalVer?
**A:** House standard is CalVer for fleet-wide consistency. Use CalVer unless your project specifically needs semantic versioning (e.g., API stability guarantees).

### Q: How do I publish to crates.io?
**A:**
1. Uncomment the `[publish]` job in `.github/workflows/release.yml`
2. Add `CARGO_REGISTRY_TOKEN` secret to your GitHub repo
3. Tag a release; CI publishes automatically

Or manually:
```bash
cargo publish --token <your_token>
```

---

## **Code Quality & Testing**

### Q: Why `todo! = "deny"`?
**A:** Stubs don't belong in shipping code. Instead:
```rust
// ❌ Don't ship this
fn handle_failure() {
    todo!("implement retry logic")
}

// ✅ Use Result instead
fn handle_failure() -> Result<()> {
    Err(anyhow::anyhow!("retry logic not yet implemented"))
}
```

This way, the issue is explicit: you're returning an error, not panicking.

### Q: When do I add tests?
**A:** Always. At minimum:
- Unit tests in the same file as the code
- Integration tests in `tests/`
- Bench tests for performance-critical paths

Run `just ci` to catch gaps.

### Q: How do I test async code?
**A:** Use `#[tokio::test]`:
```rust
#[tokio::test]
async fn my_async_test() {
    let result = my_async_function().await;
    assert_eq!(result, expected);
}
```

For determinism, run serial: `cargo test -- --test-threads=1`

### Q: How do I handle flaky tests?
**A:** If a test is genuinely flaky (timing-dependent), use `serial_test`:
```rust
use serial_test::serial;

#[test]
#[serial]
fn my_timing_test() {
    // runs alone, not in parallel
}
```

Or refactor to remove timing dependencies (e.g., mock time with `tokio::time::pause`).

### Q: How much code coverage do I need?
**A:** Aim for 80%+ of lines, 90%+ of critical paths (crypto, validation). Run:
```bash
just coverage
# Opens HTML report
```

Add tests for uncovered branches, especially error paths.

---

## **Linting & Formatting**

### Q: Clippy says `unwrap_used`. What do I do?
**A:** In libraries, avoid `unwrap`. Use `?` instead:
```rust
// ❌ Library code
fn parse_config(path: &str) -> Config {
    let content = std::fs::read_to_string(path).unwrap();  // panics!
    serde_json::from_str(&content).unwrap()
}

// ✅ Library code
fn parse_config(path: &str) -> Result<Config> {
    let content = std::fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(Into::into)
}
```

In binary-only code, `unwrap` is acceptable with justification.

### Q: Clippy says `todo!`. What do I do?
**A:** Replace with proper error handling:
```rust
// ❌
fn validate() -> bool {
    todo!("add validation")
}

// ✅
fn validate() -> Result<()> {
    Err(anyhow::anyhow!("validation not yet implemented"))
}
```

### Q: How do I suppress a lint?
**A:** Add `#[allow(...)]` at the item level with a comment:
```rust
#[allow(clippy::too_many_arguments)]
// Note: refactoring this function breaks the FFI boundary
pub fn ffi_function(a: i32, b: i32, c: i32, ...) { ... }
```

Never suppress with crate-level `#![allow(...)]` without strong justification.

### Q: Why does `just fmt` require a 100-char line limit?
**A:** Because:
- 100 chars fits on most screens without horizontal scrolling
- Easier for side-by-side diffs in review
- Encourages cleaner abstractions (long lines often indicate poor naming)

If a line is > 100 chars, refactor:
```rust
// ❌ Long line
let result = very_long_function_name(arg1, arg2, arg3, arg4, arg5, arg6, arg7)?;

// ✅ Broken across lines or extracted
let temp = helper_function(arg1, arg2, arg3);
let result = very_long_function_name(temp, arg4, arg5, arg6, arg7)?;
```

---

## **Dependencies**

### Q: Can I use nightly Rust features?
**A:** Only with explicit justification. House default is stable Rust 1.82. If you must use nightly:
1. Use `+nightly` in `rust-toolchain.toml`: `channel = "nightly-YYYY-MM-DD"` (pin the date)
2. Document the required features in `CLAUDE.md`
3. Expect review pushback

Avoid nightly unless the feature is stabilizing soon (check RFC).

### Q: Can I add an optional dependency?
**A:** Yes, via features:
```toml
[dependencies]
my-dep = { version = "1.0", optional = true }

[features]
my-feature = ["dep:my-dep"]
```

Then in code:
```rust
#[cfg(feature = "my-feature")]
use my_dep;
```

### Q: A dependency has a RUSTSEC advisory. What do I do?
**A:** Run `cargo deny check` to see the full issue:
1. **If fixable:** Update the dependency
2. **If critical:** Find an alternative
3. **If benign/time-limited:** Add an ignore entry to `deny.toml` with a comment and issue link

Never ignore security advisories without strong justification.

### Q: Can I depend on a fork or git repo?
**A:** Yes, but only for:
- seanchatmangpt house repos (configured in `deny.toml`)
- Short-term fixes while upstream is updated
- Internal tools not published to crates.io

Prefer crates.io dependencies. Document forks in `CLAUDE.md`.

---

## **License & Legal**

### Q: Why MIT OR Apache-2.0?
**A:** 
- **MIT:** Simple, short, permissive
- **Apache-2.0:** Includes explicit patent grant (useful for corporate users)

Dual licensing gives users choice while maintaining both simplicity and patent protection.

### Q: Can I use a different license?
**A:** Not without fleet consensus. The house default is MIT OR Apache-2.0 for legal clarity and permissiveness.

If you have a business requirement for a different license (e.g., GPL for open-source commitments), discuss with the team.

### Q: What does BUSL-1.1 mean in the exceptions?
**A:** Business Source License 1.1. Some seanchatmangpt repos (wasm4pm, dteam, miniml) use BUSL-1.1:
- Source is available and can be used/modified freely
- Commercial redistribution requires a license
- Converts to open-source (MIT/Apache) 2 years after release

See https://mariadb.com/bsl11/ for details. BUSL-1.1 is allowed as an exception for first-party repos only.

---

## **Performance & Optimization**

### Q: How do I benchmark my code?
**A:**
```bash
cargo bench
# HTML report in target/criterion/
```

Or with Criterion in code:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_hash(c: &mut Criterion) {
    c.bench_function("hash_1mb", |b| {
        b.iter(|| blake3::hash(black_box(&vec![0u8; 1_000_000])))
    });
}

criterion_group!(benches, bench_hash);
criterion_main!(benches);
```

### Q: The release binary is too large. How do I shrink it?
**A:**
- Enable `lto = true` (already in release profile)
- Use `codegen-units = 1` (already in release profile)
- For WASM: use `wasm-opt -Oz` (not `strip = true`)
- For native: add `strip = true` post-build (via packaging step, not Cargo)

### Q: Why does `just test` take so long?
**A:**
- Full `cargo test --workspace --all-features` runs all tests
- For development: `cargo test --lib <test_name>` (single test)
- For CI: full suite is required

Add benchmarks with `#[bench]` if you need performance regression detection.

---

## **Community & Contributing**

### Q: How do I propose a new house pattern?
**A:**
1. Document it in a repo that uses it
2. Survey other repos to see if it's common
3. Open a discussion issue in the Praxis repo
4. Add the pattern to the relevant template once consensus is reached
5. Update the survey/ docs

### Q: Can I fork praxis for my organization?
**A:** Yes. The template is MIT OR Apache-2.0. Maintain attribution to seanchatmangpt/praxis.

### Q: Where do I report issues?
**A:** GitHub issues: https://github.com/seanchatmangpt/praxis/issues

Include:
- Which template(s) affected
- Expected vs. actual behavior
- Steps to reproduce
- OS, Rust version (`rustc --version`)

---

## **Still Stuck?**

- **General questions:** See `docs/troubleshooting.md`
- **Template examples:** Check `template/CLAUDE.md`, `template-mcp/CLAUDE.md`, etc.
- **House patterns:** Read `survey/00-SYNTHESIS.md`
- **Design decisions:** Check `survey/01-SECOND-WAVE.md`
- **Code examples:** See `examples/` in any template

