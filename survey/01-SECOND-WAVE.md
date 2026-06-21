# 01 — Second-Wave Survey: 10-Repo Deep Dive

**Scope:** 10 repos scanned locally (`/tmp/fleet/<repo>`) with full file access.
**Repos:** `bcinr`, `clap-noun-verb`, `clnrm`, `pm4py-mcp`, `ggen`, `wasm4pm`, `gitvan`, `wasm4pm-compat`, `chicago-tdd-tools`, `dtr`.
**Goal:** Produce actionable praxis additions — new template variants, `chatman-common` testkit additions, CI improvements, anti-patterns to warn against, and cross-cutting innovations not captured in the first wave.

---

## 1. Critical Praxis Bugs (Fix First)

### BUG-1: `template/[profile.release] strip = true` CORRUPTS WASM BINARIES

**File:** `template/Cargo.toml` (workspace template), `template/Cargo.workspace.toml`
**Impact:** Any repo that inherits the release profile and targets `wasm32-*` will produce a non-loadable `.wasm` binary. `strip = true` passes `--strip-all` to the native toolchain; for WASM targets, stripping must go through `wasm-opt`, not the Rust compiler.
**Evidence:** `wasm4pm` explicitly sets `strip = false` and maintains five size-tuned build profiles (`mobile-wasm`, `standard-wasm`, `performance-wasm`, `analytics-wasm`, `cloud-wasm`). `pm4wasm` (first-wave) has the same workaround.

**Fix:**

```toml
# template/Cargo.toml  — base release profile
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
# strip = true   ← REMOVE. Stripped by wasm-opt for WASM, by the linker for native.

# Per-crate override in WASM crates (in the member Cargo.toml):
[profile.release.package."my-wasm-crate"]
opt-level = "s"
strip = false   # explicit no-op to prevent workspace inheritance
```

Also update the comment in `00-SYNTHESIS.md §5` root-manifest spec (last sentence of the `[profile.release]` bullet).

---

### BUG-2: `template/deny.toml` Blocks BUSL-1.1 Repos

**File:** `template/deny.toml`
**Impact:** Any repo that vendors or depends on `wasm4pm`, `dteam`, `miniml`, or any other BUSL-licensed crate will fail `cargo deny` with no override path.
**Evidence:** `wasm4pm` is BUSL-1.1; `dteam` is BUSL-1.1; `miniml` is BSL-1.1 (same). First-wave synthesis §5 lists BUSL-licensed repos as divergences to reconcile but the `deny.toml` template was not updated.

**Fix:** Add an exceptions block below the `[licenses]` allow list:

```toml
[[licenses.exceptions]]
# wasm4pm, dteam, and miniml use BUSL-1.1 / BSL-1.1 (non-OSI but business-source).
# These are first-party seanchatmangpt repos; their code is not redistributed.
allow = ["BUSL-1.1"]
name = "wasm4pm"
version = "*"
```

Repeat for `dteam` and `miniml` members as needed.

---

## 2. New Template Variants Needed

### VARIANT-1: `template-wasm/`

Three repos (`wasm4pm`, `pm4wasm`, `miniml`) independently solve identical WASM problems:
- `panic = "abort"`, `opt-level = "s"`, `strip = false` in `[profile.release]`
- `[target.'cfg(target_arch = "wasm32")'.dependencies]` for `getrandom { features = ["js"] }`, `uuid { features = ["js"] }`
- `console_error_panic_hook` for browser-readable panics
- `BTreeMap` over `HashMap` for deterministic output (WASM guests are pure functions — hash randomization breaks reproducibility)
- Handle-based API: store Rust objects in a `Store<T>`, expose string handles to JS (never raw pointers)
- `wasm-pack` / `wasm-bindgen` tooling, not `cargo build --target wasm32-unknown-unknown` naked

**Template additions:**

```
template-wasm/
├── Cargo.toml              # WASM-specific profiles, no lto/strip, target dep overrides
├── src/
│   ├── lib.rs              # wasm_bindgen entry points + Store<T> handle pattern
│   └── store.rs            # Handle-based object store
├── .cargo/
│   └── config.toml         # target = "wasm32-unknown-unknown" default
├── .github/workflows/
│   └── ci.yml              # two-phase: test (native) → build (wasm-pack)
└── justfile                # wasm-pack build, wasm-opt, size report recipes
```

Key `justfile` recipes:
```just
# Build WASM with size optimization
build-wasm:
    wasm-pack build --target web --release
    wasm-opt -Os pkg/*.wasm -o pkg/optimized.wasm

# Show size breakdown
wasm-size:
    twiggy top pkg/optimized.wasm
```

### VARIANT-2: `template-integration/`

`clnrm` has the most sophisticated integration-test harness in the corpus. Five patterns missing from the base template:

1. `allocate_ephemeral_port()` — bind port 0, get OS-assigned port, release before test
2. `ContainerGuard<T>` — RAII Docker container lifecycle
3. `skip_without_docker!` — macro that skips test if Docker socket unreachable
4. `serial_test` re-export for tests that cannot run in parallel (DB migrations, port 5432)
5. Async test fixtures with setup/teardown hooks

```
template-integration/
├── Cargo.toml              # testcontainers, serial_test, tokio-test, reqwest
├── tests/
│   └── integration_test.rs # ContainerGuard + skip_without_docker! example
└── .github/workflows/
    └── integration.yml     # separate workflow: needs docker service
```

### VARIANT-3: `template-mcp/`

`ggen-mcp` (Rust, `rmcp` 0.11/1.3) is the reference MCP server implementation. `pm4py-mcp` (Python, FastMCP) proves the value of the pattern but is not the Rust template basis.

Key `ggen-mcp` patterns to encode:
- `#[tool(description = "...")]` on each struct implementing the tool
- Shared `TimeWindowArgs { hours: u32, limit: usize }` (found independently in `pm4py-mcp` too)
- `"passed": bool` always present in response, even on errors (CI gate tool pattern)
- BLAKE3 `result_hash` on every tool response (not in `pm4py-mcp`, should be added)
- Composite "fan-out" tool that calls all analyses and returns aggregated results

```
template-mcp/
├── Cargo.toml              # rmcp, serde_json, tokio, blake3
├── src/
│   ├── main.rs             # McpServer::new().serve_stdio()
│   ├── tools/
│   │   ├── mod.rs
│   │   └── example.rs      # #[tool] impl with TimeWindowArgs + result_hash
│   └── shared_args.rs      # TimeWindowArgs, CommonResponse<T>
└── .github/workflows/
    └── ci.yml
```

---

## 3. `chatman-common` Testkit Additions

From `chicago-tdd-tools` and `dtr`, 11 additions are immediately adoptable into `crates/chatman-common/`.

### From `chicago-tdd-tools`

**3.1 `TestState<Phase>` compile-time AAA enforcement**

```rust
// Enforce Arrange → Act → Assert ordering at compile time
pub struct TestState<Phase> {
    _phase: PhantomData<Phase>,
}
pub struct Arrange;
pub struct Act;
pub struct Assert;

impl TestState<Arrange> {
    pub fn new() -> Self { TestState { _phase: PhantomData } }
    pub fn act(self) -> TestState<Act> { TestState { _phase: PhantomData } }
}
impl TestState<Act> {
    pub fn assert(self) -> TestState<Assert> { TestState { _phase: PhantomData } }
}
```

**3.2 `test!` / `async_test!` macros with SLA enforcement**

```rust
macro_rules! test {
    ($name:ident, $sla_ms:expr, $body:expr) => {
        #[test]
        fn $name() {
            let start = std::time::Instant::now();
            $body;
            let elapsed = start.elapsed().as_millis();
            assert!(elapsed < $sla_ms, "SLA violated: {}ms > {}ms", elapsed, $sla_ms);
        }
    };
}
```

SLA defaults from chicago-tdd-tools: Hot = 8 ticks, Warm = 500K ticks, Cold = unlimited.

**3.3 Thermal test classification**

Add `#[hot_test]`, `#[warm_test]`, `#[cold_test]` proc-macro attributes that set SLA thresholds and (for hot) assert they run in `< 8` clock ticks via `Instant::now()`.

**3.4 `TestReceipt` signed auditable output**

```rust
pub struct TestReceipt {
    pub test_name: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub environment: EnvironmentFingerprint,
    pub chain_hash: String,  // BLAKE3 of (test_name + passed + duration + env)
}

pub struct EnvironmentFingerprint {
    pub os: String,
    pub rust_version: String,
    pub target: String,
    pub timestamp: i64,
}
```

**3.5 `assert_fail!` macro**

```rust
// Capture the error from an expression that should fail, assert on it
macro_rules! assert_fail {
    ($expr:expr, $pat:pat) => {
        match $expr {
            Err($pat) => {},
            Ok(v) => panic!("expected failure, got Ok({:?})", v),
            Err(e) => panic!("wrong error kind: {:?}", e),
        }
    };
}
```

**3.6 Docker retry helper**

```rust
pub async fn wait_for_docker(
    host: &str,
    port: u16,
    timeout: Duration,
    retry_interval: Duration,
) -> Result<()>;
```

**3.7 `TestOutput` trait for `?` in test bodies**

```rust
pub trait TestOutput {
    fn into_test_result(self);
}

impl<T, E: Debug> TestOutput for Result<T, E> {
    fn into_test_result(self) {
        if let Err(e) = self {
            panic!("Test failed: {:?}", e);
        }
    }
}
```

Enables `let x = fallible_op()?;` in test bodies via `#[test] fn foo() -> impl TestOutput`.

### From `dtr`

**3.8 `DocEvent` enum + `DocContext` struct** (feature `"living-docs"`)

```rust
#[non_exhaustive]
pub enum DocEvent {
    Section(String),
    Para(String),
    Code { lang: String, body: String },
    Table { header: Vec<String>, rows: Vec<Vec<String>> },
    KeyValue(Vec<(String, String)>),
    Assertion { label: String, passed: bool },
    Mermaid(String),
    ChainHash(String),   // praxis-exclusive — dtr has no equivalent
}

pub struct DocContext {
    events: Vec<DocEvent>,
    output_path: PathBuf,
}

impl DocContext {
    pub fn for_test(file: &str) -> Self { ... }
    pub fn say(&mut self, text: &str) { ... }
    pub fn say_section(&mut self, heading: &str) { ... }
    pub fn say_code(&mut self, lang: &str, body: &str) { ... }
    pub fn say_table(&mut self, header: &[&str], rows: &[&[&str]]) { ... }
    pub fn say_mermaid(&mut self, dsl: &str) { ... }
    pub fn say_key_value(&mut self, pairs: &[(&str, &str)]) { ... }
    pub fn say_and_assert(&mut self, label: &str, cond: bool) { ... }
    pub fn finish(self) -> Result<()> { ... } // write docs/test/<module>.md
}
```

`finish()` content-addresses the rendered bytes via `Blake3Hash::content_address()` and embeds the hash in the doc footer — a unique praxis advantage over dtr.

**3.9 `doc_assert!` macro**

```rust
macro_rules! doc_assert {
    ($ctx:expr, $label:expr, $cond:expr) => {{
        assert!($cond, "doc assertion failed: {}", $label);
        $ctx.say_and_assert($label, true);
    }};
}
```

This is dtr's most impactful pattern: assertion and documentation are a single atomic call. If the assertion fails, the documentation line is never emitted.

**3.10 Multi-format renderer trait**

```rust
pub trait DocRenderer {
    fn render_event(&mut self, event: &DocEvent) -> Result<()>;
    fn finish(&mut self) -> Result<Vec<u8>>;
}

pub struct MarkdownRenderer { ... }   // default
// pub struct LatexRenderer { ... }  // opt-in, feature "living-docs-latex"
```

**3.11 `docs/test/` as a committed artifact**

- Add `template/docs/test/.gitkeep` so `cargo generate` scaffolds the output directory
- Ensure `template/.gitignore` does **not** contain `docs/test/`
- Add to `justfile`:
  ```just
  docs-test:
      UPDATE_GOLDEN=1 cargo test --features living-docs -- --test-threads=1

  docs-verify:
      cargo test --features living-docs -- --test-threads=1
  ```
- Add to `ci.yml`:
  ```yaml
  living-docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo test --features living-docs -- --test-threads=1
        name: Verify living docs are up to date
  ```

---

## 4. CI Improvements to Adopt

### CI-1: SHA-Pinned GitHub Actions (from `clap-noun-verb`)

`clap-noun-verb` SHA-pins every third-party GitHub Action (supply-chain hardening, stricter than praxis). Praxis template uses tag-pinned (`@v2`). Recommended upgrade for the `ci.yml` and `release.yml` templates:

```yaml
# Current (tag-pinned):
- uses: Swatinem/rust-cache@v2

# Recommended (SHA-pinned):
- uses: Swatinem/rust-cache@82a92a6e8fbeee089604da2575dc567ae9ddeaab  # v2.7.5
```

Update all actions in `template/.github/workflows/`. Add a comment with the human-readable tag next to each SHA.

### CI-2: `ci-success` Gate Job (from `clap-noun-verb`)

```yaml
ci-success:
  name: CI success
  runs-on: ubuntu-latest
  needs: [fmt, clippy, test, docs, deny, typos, msrv]
  if: always()
  steps:
    - name: Require all jobs passed
      run: |
        results="${{ join(needs.*.result, ' ') }}"
        for r in $results; do
          [ "$r" = "success" ] || exit 1
        done
```

This is the correct way to require all jobs: `if: always()` + explicit check, not `needs:` alone (which passes when some jobs are skipped). Praxis `ci.yml` should adopt this gate.

### CI-3: Benchmark Regression Gate (from `bcinr`)

`bcinr` uses `benchmark-action/github-action-benchmark@v1` to post criterion results as PR comments and fail if regression > threshold. Add an **optional** fourth workflow `template/.github/workflows/bench.yml`:

```yaml
bench:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: benchmark-action/github-action-benchmark@v1
      with:
        tool: cargo
        output-file-path: target/criterion/output.json
        fail-on-alert: true
        alert-threshold: "150%"
        github-token: ${{ secrets.GITHUB_TOKEN }}
```

### CI-4: Miri UB-Check Job (from `bcinr` + `wasm4pm-compat`)

**Critical note from `wasm4pm-compat`:** `#![forbid(unsafe_code)]` does **not** cover transitive deps. Miri is the only transitive UB proof. Add an optional `miri.yml`:

```yaml
miri:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@nightly
      with:
        components: miri
    - run: cargo miri test --workspace
```

### CI-5: Unwrap-Check Job (from `chicago-tdd-tools`)

```yaml
unwrap-check:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - run: cargo install cargo-unwrap-check --quiet
    - run: cargo unwrap-check --workspace
```

Complements the `unwrap_used = "deny"` lint in library crates; catches `unwrap()` in test code too.

### CI-6: `andon` Failure Signal Pattern (from `chicago-tdd-tools`)

Any test annotated with `#[andon]` (or `#[fmea(severity = "critical")]`) causes the entire CI run to halt on failure rather than continuing to collect results. Praxis can encode this as a `just andon-check` recipe:

```just
andon-check:
    cargo test -- --test-threads=1 2>&1 | grep -E "(ANDON|CRITICAL)" && exit 1 || true
```

---

## 5. Anti-Patterns to Document in `CHECKLIST.md`

### ANTI-1: `.cargo/config.toml` RUSTFLAGS to suppress lints

**Source:** `wasm4pm`
**Pattern:** Using `[build] rustflags = ["-A", "clippy::all"]` in `.cargo/config.toml` to globally suppress lint warnings instead of using `[workspace.lints]`.
**Why it's bad:** Invisible to CI (doesn't show up in lint reports), silences all clippy diagnostics including safety-critical ones, and propagates across all crates in the workspace.
**CHECKLIST entry:** `[H] Verify .cargo/config.toml does not contain RUSTFLAGS that suppress lints. Move any necessary allows to [workspace.lints.clippy] with justification comments.`

### ANTI-2: Nightly toolchain for non-nightly features

**Source:** `wasm4pm`, `clnrm`, `ggen`, `chicago-tdd-tools`
**Pattern:** Using `channel = "nightly"` in `rust-toolchain.toml` when the actual nightly features used are limited (often just `rustfmt` options or a single proc-macro).
**Why it's bad:** Nightly is unversioned; a new nightly can break CI silently. The `just` pattern seen in `ggen` — using nightly only for `cargo fmt`, stable for everything else — is the correct escape hatch.
**CHECKLIST entry:** `[H] If rust-toolchain.toml pins nightly, document exactly which nightly features are required and why stable is insufficient. Consider using stable for all jobs except fmt.`

### ANTI-3: `proc-macro-error` (RUSTSEC-2024-0370 unmaintained)

**Source:** `clap-noun-verb`
**Pattern:** Depending on `proc-macro-error` crate which is unmaintained and flagged in the RustSec advisory database.
**Fix:** Migrate to `proc-macro-error2` (maintained fork) or `manyhow` (modern replacement).
**CHECKLIST entry:** `[AUTO] Run cargo deny check and fix RUSTSEC-2024-0370 (proc-macro-error) by migrating to proc-macro-error2 or manyhow.`

### ANTI-4: Lone `// TODO:` / `todo!()` with deny lint but open issues as stubs

**Source:** `clnrm` (13 open issues are self-filed AI-generated stub admissions)
**Pattern:** Using `deny(todo)` to gate compile-time stubs is correct, but the issue tracker filling up with `todo!` admissions is a smell. The correct pattern: `todo!` is the compile gate; the issue is only filed when there is a concrete implementation plan.
**CHECKLIST entry:** `[H] Review open issues for self-filed todo! stub admissions. Close any that are fully resolved by the deny(todo) lint or by the implementation plan being captured in CLAUDE.md.`

### ANTI-5: No `[workspace.lints]` in multi-crate workspace

**Source:** `bcinr`, `wasm4pm`, `ggen`, `clnrm` (partial)
**Pattern:** Large workspaces with 12–45 crates that each set their own `[lints]` (or none), causing lint inconsistency across members.
**CHECKLIST entry:** `[AUTO] Add [workspace.lints] to root Cargo.toml. Add lints.workspace = true to all member Cargo.toml files. Verify cargo check --workspace compiles cleanly.`

---

## 6. Innovations Worth Adopting

### INN-1: Typestate Lifecycle with Non-Forgeable Carrier (`wasm4pm-compat`)

`Evidence<T, State: EvidenceState, W>` uses a private `_seal: ()` field to prevent construction outside the module. The `Witness` trait (zero-cost authority label via `const trait`) prevents forgery. This is a more rigorous form of the Seal pattern already documented in praxis `README.md`.

**Praxis addition:** Document the `const trait Witness` pattern in `CLAUDE.md` under "Seal Pattern" as the "full typestate" variant (in addition to the current `_seal: ()` entry-point seal).

### INN-2: `trybuild` ALIVE Gate (`wasm4pm-compat`)

`wasm4pm-compat` maintains 444 compile-fail + 413 compile-pass fixtures and runs them via `trybuild`. This is the gold standard for type-system correctness: tests that verify the type system rejects exactly what it should.

**Praxis addition:** Add to `chatman-common::testkit`:
```rust
// Assert that a code snippet fails to compile with an expected error
pub fn trybuild_fail(fixture_path: &str, expected_error: &str) { ... }
// Assert that a code snippet compiles
pub fn trybuild_pass(fixture_path: &str) { ... }
```

Document `trybuild` in the `template-wasm/` and `template-integration/` CLAUDE.md files.

### INN-3: `anti-llm.toml` AI Integrity Gate (`wasm4pm-compat`)

A manifest listing which patterns, constructs, and files should never be generated by AI tooling (e.g., `witness_marker!` is human-only, `cicd.toml` structure is machine-readable only). CI checks that AI-modified files do not contain banned patterns.

**Praxis consideration:** Add `template/anti-llm.toml` (empty by default) documenting the concept; populate in repos where AI integrity matters (OCEL audit trails, signed receipts, the Seal pattern).

### INN-4: `cicd.toml` Machine-Readable CI State Manifest (`wasm4pm-compat`)

A `cicd.toml` file in the repo root listing CI job names, their expected outcomes, and any known flaky tests. Consumed by tooling to produce a CI health dashboard. Example:

```toml
[ci]
required_jobs = ["fmt", "clippy", "test", "deny", "typos", "msrv", "ci-success"]
flaky = []
known_failing = []
```

**Praxis addition:** Add `template/cicd.toml` with the required job list. This machine-readable format allows `ggen` to generate CI dashboards and `cargo-cicd` to verify CI shape automatically.

### INN-5: Git-as-Runtime Patterns (`gitvan`)

`gitvan` (JavaScript/TypeScript) implements:
- `git update-ref` as distributed CAS locks (atomic across concurrent processes)
- `git notes append` as an immutable audit ledger (NDJSON, append-only)
- Git commit annotations as content-addressed snapshots
- Workflow definitions as versioned RDF (Turtle) with SPARQL queries

The Rust equivalent (possible additions to `chatman-common`):
- `chatman_common::git_lock` — `git update-ref`-based distributed lock (wraps `std::process::Command`)
- `just receipt-commit` recipe — append a receipt JSON to `git notes` via `git notes --ref=receipts append`
- `template/ontology/workflow.ttl` — a Turtle skeleton for workflow definitions

### INN-6: `ontology_id` / Compile-Time KEY Uniqueness Proof (`wasm4pm-compat`)

Ontology-driven codegen that proves at compile time that all ontology keys are unique (no two verbs share an ontology ID). Encoded via `const` arrays and `const fn assert_unique()`. This extends the `linkme` distributed slice pattern by adding compile-time uniqueness checking.

**Praxis addition:** Document in `CLAUDE.md` under "noun-verb CLI" as the "compile-time ontology integrity" extension.

### INN-7: `TimeWindowArgs` Shared Struct for Tool CLIs (from `pm4py-mcp`)

```rust
// Used identically in pm4py-mcp (Python) and should be in ggen-mcp (Rust)
#[derive(Args, Serialize)]
pub struct TimeWindowArgs {
    #[arg(long, default_value = "24")]
    pub hours: u32,
    #[arg(long, default_value = "1000")]
    pub limit: usize,
}
```

Every tool/verb that operates over a time window uses these two parameters. Add to `chatman-common::cli` as a composable `clap::Args` struct.

### INN-8: `cliff.toml` + `git-cliff` for CHANGELOG generation (`clap-noun-verb`)

`clap-noun-verb` uses `git-cliff` to generate CHANGELOG entries from conventional commits. praxis currently specifies hand-authored Keep-a-Changelog entries. Add `template/cliff.toml` and a `just changelog` recipe:

```just
changelog:
    git cliff --output CHANGELOG.md
```

### INN-9: `performance_test!` macro with SLA + Historical Comparison (`chicago-tdd-tools`)

```rust
macro_rules! performance_test {
    ($name:ident, $sla_ms:expr, $body:expr) => { ... }
}
```

The macro runs `$body`, measures wall time, fails if > `$sla_ms`, and optionally writes a `perf_baseline.json` for cross-run comparison. This is a lightweight alternative to criterion for correctness-focused SLA testing (not statistical benchmarking).

### INN-10: `ed25519-dalek` Signing Extends BLAKE3 Receipts (`wasm4pm`)

`wasm4pm` extends BLAKE3 chain receipts with `ed25519-dalek` digital signatures. The signing key is loaded from an environment variable or key file; the signature is included in the receipt JSON. This creates non-repudiable audit trails.

**Praxis addition:** Document in `chatman-common::provenance` as the "signed receipt" extension. The base receipt is BLAKE3-only (already documented); signing is an opt-in feature `"signed-receipts"` that adds `ed25519-dalek` to the dep graph.

---

## 7. Per-Repo Finding Cards

### bcinr

| | |
|---|---|
| **GAPS** | No `LICENSE` files on disk (metadata only); no `deny.toml`, `rustfmt.toml`, `typos.toml`; no `[workspace.lints]`; mass Python codegen scripts not gitignored; `lto = "fat"` (non-standard, should be `lto = true`) |
| **INNOVATIONS** | 3-pipeline CI (ci + bench + miri); benchmark regression gate via `benchmark-action/github-action-benchmark@v1`; dedicated `bcinr-bench/` crate for benchmarks; Miri UB-check CI job; `AGENTS.md` named-agent roster; `fuzz/` with cargo-fuzz; OCEL receipts |
| **ALREADY COMPLIANT** | CalVer, Edition 2021, `Swatinem/rust-cache@v2`, dual MIT OR Apache-2.0 intent, `blake3`, `linkme`, `clap-noun-verb` |
| **RECOMMENDATIONS** | Add `LICENSE-MIT` + `LICENSE-APACHE` files; add `deny.toml` (from template); add `[workspace.lints]`; gitignore Python generated files; adopt `lto = true` not `"fat"`; promote bench + miri CI jobs to praxis template |

### clap-noun-verb

| | |
|---|---|
| **GAPS** | MSRV 1.74 (macros 1.70) vs praxis target 1.82; `thiserror` 1 vs praxis target 2; `unsafe_code = "allow"` (needs justification or removal); `proc-macro-error` RUSTSEC-2024-0370; no dependabot |
| **INNOVATIONS** | Full ADL in `CLAUDE.md` (Architecture Decision Log with rationale); `ci-success` gate job; `cliff.toml` + git-cliff for CHANGELOG; nextest test runner; SHA-pinned GitHub Actions; performance SLO CI job; `.githooks/` pre-commit hooks |
| **ALREADY COMPLIANT** | CalVer, Edition 2021, dual MIT OR Apache-2.0, `[workspace.lints]`, `typos.toml`, `.editorconfig`, `deny.toml`, `Swatinem/rust-cache@v2`, `just` task runner |
| **RECOMMENDATIONS** | Bump MSRV to 1.82; migrate `thiserror` 1 → 2; fix `proc-macro-error` RUSTSEC-2024-0370; adopt SHA pinning in praxis template; adopt `ci-success` gate job in praxis `ci.yml` |

### clnrm

| | |
|---|---|
| **GAPS** | MIT-only license (not dual); nightly toolchain (not pinned stable); no `[workspace.lints]`; 29 CI workflows (should consolidate to ci.yml + release.yml + optional.yml); 13 open issues are self-filed stub admissions |
| **INNOVATIONS** | `allocate_ephemeral_port()`, `ContainerGuard<T>` RAII, `skip_without_docker!`, `serial_test` re-export, async test fixtures — all 5 belong in `chatman-common::testkit`; `deny.toml` (most rigorous in corpus); `integration.yml` separate CI workflow |
| **ALREADY COMPLIANT** | CalVer, Edition 2021, `Swatinem/rust-cache@v2`, `deny.toml`, `cargo-deny` in CI |
| **RECOMMENDATIONS** | Add `LICENSE-APACHE` for dual; consolidate 29 workflows → 2-3; close stub-admission issues; contribute 5 testkit patterns to `chatman-common` |

### pm4py-mcp

| | |
|---|---|
| **GAPS** | Python (FastMCP), not Rust — no direct Cargo applicability; no BLAKE3 result hashing on tool responses; no `result_hash` field |
| **INNOVATIONS** | 27 tools via FastMCP; `TimeWindowArgs { hours, limit }` shared params (adoptable in Rust MCP servers); `"passed": bool` always in response even on error; composite `pm4py_full` fan-out tool |
| **ALREADY COMPLIANT** | CalVer (Python), CI gate via `"passed"` field |
| **RECOMMENDATIONS** | Extract `TimeWindowArgs` to `chatman-common::cli`; extract `CommonResponse<T>` with mandatory `passed: bool` and `result_hash: String` to `template-mcp/src/shared_args.rs`; use `ggen-mcp` (Rust) as the MCP template, not pm4py-mcp |

### ggen

| | |
|---|---|
| **GAPS** | Nightly toolchain (`nightly-2026-04-15`) due to rustfmt options; 200+ `#[allow(clippy::...)]` overrides ("Phase B.1 warn-first" in CLAUDE.md); MIT-only license; no `typos.toml`, no `.editorconfig` |
| **INNOVATIONS** | Composite GitHub Action for sibling-repo provisioning via `[patch.crates-io]`; `oxigraph 0.5` (RDF/SPARQL), `tera 1.20` (Jinja2 templates), `genai 0.5` (LLM client); `[[test]] required-features = ["integration"]`; `human-panic` / `better-panic`; `[package.metadata.deb]` for Debian packaging; SHA-pinned Actions |
| **ALREADY COMPLIANT** | CalVer, Edition 2021, `[workspace.lints]`, `deny.toml`, `CLAUDE.md`, `just` task runner |
| **RECOMMENDATIONS** | Migrate to pinned stable toolchain for non-fmt jobs; add `LICENSE-APACHE` for dual; adopt `[[test]] required-features` pattern in `template-integration/`; document `[package.metadata.deb]` in CLAUDE.md |

### wasm4pm

| | |
|---|---|
| **GAPS** | BUSL-1.1 license (incompatible with praxis `deny.toml` — needs exceptions block); nightly toolchain; no `[workspace.lints]`; lints suppressed via `.cargo/config.toml` RUSTFLAGS (ANTI-1); no justfile |
| **INNOVATIONS** | 5 deployment profiles (mobile 500KB → cloud 2.78MB); handle-based WASM API (store objects Rust-side, pass string handles to JS); `BTreeMap` over `HashMap` for WASM determinism; `ed25519-dalek` signing extends BLAKE3 receipts; `console_error_panic_hook`; `[target.'cfg(target_arch = "wasm32")'.dependencies]` pattern |
| **ALREADY COMPLIANT** | CalVer, BLAKE3 chain receipts, OCEL events, Edition 2021, `Swatinem/rust-cache@v2` |
| **RECOMMENDATIONS** | Fix praxis `deny.toml` with BUSL-1.1 exception (BUG-2); document WASM profiles in `template-wasm/`; move `.cargo/config.toml` RUSTFLAGS to `[workspace.lints]`; add `strip = false` to WASM profile (BUG-1); add signed receipts to `chatman-common::provenance` |

### gitvan

| | |
|---|---|
| **GAPS** | Pure JavaScript/TypeScript — no Rust; no Cargo.toml; no formal release process; TypeScript strict mode inconsistent across files |
| **INNOVATIONS** | Git-as-Runtime: `git update-ref` as distributed CAS locks; `git notes append` as immutable NDJSON audit ledger; content-addressed snapshots via git commit annotation; workflow definitions as versioned RDF (Turtle); git hooks as event bus → PROV-O quads; SPARQL-queryable event store with 90-day/365-day retention tiers; `@noble/hashes` BLAKE3 for JavaScript |
| **ALREADY COMPLIANT** | BLAKE3 content addressing (JavaScript), OCEL-compatible event structure, CalVer intent |
| **RECOMMENDATIONS** | Add `chatman_common::git_lock` (Rust git update-ref lock); add `just receipt-commit` (git notes append); add `template/ontology/workflow.ttl` skeleton; document git-as-runtime in praxis README under "Provenance Patterns" |

### wasm4pm-compat

| | |
|---|---|
| **GAPS** | **Zero CI workflows** (largest gap in entire corpus); nightly toolchain; no deny.toml, no typos.toml |
| **INNOVATIONS** | Typestate lifecycle `Evidence<T, State, W>` with `_seal: ()` + `const trait Witness` zero-cost labels; exactly-3-features discipline (hard ceiling); `trybuild` ALIVE gate (444 compile-fail + 413 compile-pass fixtures); `witness_marker!` macro; ontology-driven codegen with compile-time KEY-uniqueness proof; named refusal enums; `anti-llm.toml` suppression manifest; `cicd.toml` machine-readable CI state manifest; MIRI as transitive UB proof |
| **ALREADY COMPLIANT** | CalVer, Edition 2021, dual MIT OR Apache-2.0, BLAKE3, Seal pattern, `[workspace.lints]` intent, `just` task runner |
| **RECOMMENDATIONS** | Add CI immediately (use praxis `template/.github/workflows/ci.yml` verbatim); add `deny.toml`; add `anti-llm.toml` and `cicd.toml` to praxis template; adopt `trybuild` ALIVE gate in `chatman-common::testkit`; document `const trait Witness` in CLAUDE.md Seal section |

### chicago-tdd-tools

| | |
|---|---|
| **GAPS** | Nightly toolchain; MIT-only license; `cargo-make` not `just`; no `deny.toml`, no `typos.toml`; 85% coverage threshold enforced via `cargo-tarpaulin` (not in praxis template) |
| **INNOVATIONS** | `TestState<Phase>` compile-time AAA enforcement; `test!`/`async_test!`/`performance_test!` macros with SLA; `TestOutput` trait for `?` in test bodies; Thermal classification (Hot/Warm/Cold); `TestContract` + `TestContractRegistry` const test metadata; `TestReceipt` signed auditable output; `EffectTest<E>` type-level effect system; `assert_fail!` error-capture macro; Docker retry helper; `.config/nextest.toml` profiles; Andon CI signal with FMEA annotations; `unwrap-check` CI job |
| **ALREADY COMPLIANT** | CalVer, Edition 2021, BLAKE3 for `TestReceipt`, typestate patterns |
| **RECOMMENDATIONS** | Add `TestState<Phase>`, `TestReceipt`, `assert_fail!`, `doc_assert!`, Docker retry to `chatman-common::testkit`; adopt `unwrap-check` CI job in praxis template; add Andon signal pattern to CI docs; migrate from MIT-only to dual |

### dtr

| | |
|---|---|
| **GAPS** | Java (Maven), not Rust — no Cargo applicability; Apache-2.0 only (not dual); no BLAKE3 provenance on generated doc artifacts; `StructuredTaskScope` uses Java 26 (preview API) |
| **INNOVATIONS** | Tests as the sole source of documentation truth; `say*` fluent API (40+ methods across `RenderMachineCommands`); `SayEvent` sealed interface (28 record variants) — event-queue architecture decouples accumulation from rendering; multi-format fan-out (`MultiRenderMachine` via Java 26 `StructuredTaskScope`); `sayAndAssert` atomic assertion+documentation; doc coverage analysis from reflection; `docs/test/*.md` committed as versioned artifacts; annotation-driven doc structure (`@DocSection`, `@DocDescription`, `@DocCode`); Diataxis section naming conventions |
| **ALREADY COMPLIANT** | CalVer (`2026.4.1`), test-driven correctness gates, structured output format |
| **RECOMMENDATIONS** | Add `DocEvent` enum + `DocContext` + `doc_assert!` to `chatman-common::testkit` (feature `"living-docs"`); add BLAKE3 provenance to doc footer (praxis-exclusive over dtr); commit `docs/test/*.md` as first-class artifacts; add `docs-test`/`docs-verify` `justfile` recipes; add `living-docs` CI job to `ci.yml` |

---

## 8. Cross-Cutting Synthesis

### Priority Matrix

| Priority | Item | Source Repos | Effort | Praxis Files Changed |
|---|---|---|---|---|
| **P0** | Fix `strip = true` WASM corruption bug | wasm4pm, pm4wasm | 5 min | `template/Cargo.toml` |
| **P0** | Fix `deny.toml` BUSL-1.1 block | wasm4pm, dteam, miniml | 10 min | `template/deny.toml` |
| **P1** | Add `ci-success` gate job to `ci.yml` | clap-noun-verb | 15 min | `template/.github/workflows/ci.yml` |
| **P1** | Add `chatman-common::testkit` additions (§3) | chicago-tdd-tools, dtr | 4-8h | `crates/chatman-common/` |
| **P1** | Create `template-wasm/` | wasm4pm, pm4wasm, miniml | 2h | new directory |
| **P2** | Create `template-integration/` | clnrm | 2h | new directory |
| **P2** | Create `template-mcp/` | ggen-mcp, pm4py-mcp | 2h | new directory |
| **P2** | SHA-pin all GitHub Actions in templates | clap-noun-verb | 30 min | `template/.github/workflows/` |
| **P2** | Add `cicd.toml` to template | wasm4pm-compat | 15 min | `template/cicd.toml` |
| **P3** | Add Miri CI job | bcinr, wasm4pm-compat | 15 min | `template/.github/workflows/miri.yml` |
| **P3** | Add `cliff.toml` + `just changelog` | clap-noun-verb | 30 min | `template/cliff.toml`, `template/justfile` |
| **P3** | Document `git-as-runtime` patterns | gitvan | 1h | `README.md` + `chatman-common` |
| **P3** | Add `anti-llm.toml` (empty) | wasm4pm-compat | 10 min | `template/anti-llm.toml` |

### New Section for `CHECKLIST.md`

```markdown
## WASM Checklist (add when target_arch = "wasm32" present)
- [ ] [AUTO] Verify [profile.release] does NOT inherit strip = true for WASM crates
- [ ] [AUTO] Verify [target.'cfg(target_arch = "wasm32")'.dependencies] sets getrandom { features = ["js"] }
- [ ] [H]    Use BTreeMap instead of HashMap in WASM-exposed types (hash randomization)
- [ ] [AUTO] Add console_error_panic_hook initialization in wasm_bindgen_start
- [ ] [H]    Use handle-based API (Store<T>) rather than raw pointer passing to JS

## Anti-Patterns Checklist
- [ ] [H]    Verify .cargo/config.toml does not suppress lints via RUSTFLAGS
- [ ] [H]    If nightly toolchain, document exactly which nightly features are required
- [ ] [AUTO] Run: cargo deny check (will catch RUSTSEC-2024-0370 proc-macro-error)
- [ ] [H]    Review open issues for self-filed todo! stub admissions; close if lint covers them
```

### New Entries for `README.md` "Praxis Patterns"

Patterns identified in this second wave that are not yet in the README:

1. **`const trait Witness` zero-cost authority labels** — `wasm4pm-compat`; extend the existing Seal pattern entry
2. **`trybuild` ALIVE gate** — `wasm4pm-compat`; add under "Testing Patterns"
3. **Git-as-Runtime** — `gitvan`; add under "Provenance Patterns" alongside BLAKE3
4. **Typestate lifecycle with exactly-N-features discipline** — `wasm4pm-compat`
5. **Living docs from tests (`DocContext`)** — `dtr`; add under "Documentation Patterns"

---

*Sources: 10 local-disk agent scans of `/tmp/fleet/<repo>`, 2026-06-21. Critical bugs (BUG-1, BUG-2) verified against `/home/user/praxis/template/`. Per-repo findings cross-checked against `survey/00-SYNTHESIS.md` §2 coverage matrix.*
