Here is the structured boilerplate fingerprint report for the three assigned repos:

---

# Boilerplate Fingerprint Survey — pm4py-rs, pm4wasm, miniml

Cluster: process-mining / WASM. All repos are public.

---

## 1. pm4py-rs

### Identity
- **Name:** `pm4py-rs` (crate name: `pm4py`)
- **Visibility:** Public
- **Structure:** Single crate (lib + bin `mcp_stdio`), `cdylib + rlib` output
- **Edition:** 2021
- **MSRV:** 1.85 (`rust-version` in Cargo.toml; toolchain annotation mentions edition 2024 / a2a-rs)
- **File count:** ~866 non-hidden files (excluding target/) — extremely bloated with generated .md audit reports
- **Version scheme:** CalVer: `2026.3.28`

### Cargo Metadata Completeness
| Field | Present |
|-------|---------|
| `license` | Yes (`AGPL-3.0-or-later`) — **not** MIT/Apache |
| `description` | Yes |
| `repository` | Yes (but points to monorepo subtree, not the standalone repo) |
| `homepage` | Yes (same monorepo URL) |
| `keywords` | Yes (5) |
| `categories` | Yes (3) |
| `authors` | Yes (`Sean Chatman <info@chatmangpt.com>`) |
| `rust-version` | Yes (1.85) |
| `[workspace.*]` inheritance | No (single crate) |

Also has `[package.metadata.docs.rs]` (`all-features = true`, `rustdoc-args = ["--cfg", "docsrs"]`).

### Lints / Quality Config
- **rustfmt.toml:** Absent
- **clippy.toml:** Absent
- **deny.toml:** Absent — cargo-deny used only via CI action
- **typos.toml:** Absent
- **.editorconfig:** Absent
- **`[lints]` table:** Absent — all clippy flags passed inline in CI (`-D warnings`)
- **`.cargo/config.toml`:** Present (sets `git-fetch-with-cli = true`, color auto, verbose off)

### CI — `.github/workflows/`
Three workflow files:

**test.yml** — `Tests and Code Quality`
- Trigger: push/PR to main/master/develop (path-filtered: src/**, tests/**, Cargo.*)
- Jobs: `fmt` (rustfmt check, ubuntu), `clippy` (`-D warnings`, ubuntu, Swatinem/rust-cache@v2), `docs` (RUSTDOCFLAGS=-D warnings), `test-stable` (matrix: ubuntu / macos / windows; `--lib + --test '*' --all-features`; installs python pm4py), `test-beta` (ubuntu, continue-on-error), `test-nightly` (ubuntu, continue-on-error), `msrv` (1.70, `cargo check`), `coverage` (tarpaulin → Codecov), `benches` (compile-only, continue-on-error), `unused-deps` (cargo-udeps nightly, continue-on-error)
- Toolchain action: `dtolnay/rust-toolchain@<channel>`
- Cache: `Swatinem/rust-cache@v2` on all non-fmt jobs

**publish.yml** — `Publish to crates.io`
- Trigger: `v*` tag push or `workflow_dispatch`
- Jobs: `verify` (build release, test, docs, fmt, clippy), `publish` (cargo publish with `CARGO_REGISTRY_TOKEN`), `release` (softprops/action-gh-release@v1, generates changelog from git log)

**security.yml** — `Security Checks`
- Trigger: push/PR (Cargo.toml/lock paths) + daily schedule 02:00 UTC
- Jobs: `audit` (cargo-audit), `cargo-deny` (EmbarkStudios/cargo-deny-action@v1, continue-on-error), `rustsec` (rustsec/audit-check-action@v1), `outdated` (cargo-outdated, continue-on-error), `license-check` (cargo-license)

**dependabot.yml:**
- cargo: weekly Monday 03:00, limit 5 PRs, prefix `deps(cargo)`, reviewer `seanchatmangpt`
- github-actions: weekly Monday 04:00, limit 3 PRs, prefix `ci(actions)`

### Task Runner
**Makefile** (not Makefile.toml / justfile). Targets:
`help, build, release, check, test, test-unit, test-integration, test-all-features, test-verbose, fmt, fmt-check, clippy, clippy-fix, doc, doc-open, doc-check, bench, bench-{discovery,conformance,io,analysis}, coverage, coverage-text, clean, audit, deny, update-deps, outdated, tree, ci, ci-all, pre-commit, watch, watch-test, size, info, logs, debug, profile, status, doctor, weaver-live-check`

### Docs Set
| Doc | Present |
|-----|---------|
| README.md | Yes (many sections) |
| CLAUDE.md | No |
| CONTRIBUTING.md | Yes |
| SECURITY.md | Yes (audit report style) |
| CHANGELOG.md | Yes |
| CODE_OF_CONDUCT | No |
- Also: `00_START_HERE.md`, `ACADEMIC_PUBLICATION_SUMMARY.md`, `paper.tex`, 80+ generated AI session markdown files cluttering root

### Licensing
- **Single license:** AGPL-3.0-or-later (diverges from cluster norm)
- LICENSE file: single file (dual-licensed text in file but Cargo.toml says AGPL only)

### src Layout
- `src/lib.rs` + `src/main.rs` + `src/bin/mcp_stdio.rs`
- Modules: `algorithms/`, `conformance/`, `discovery/`, `statistics/`, `io/`, `filtering/`, `ocpm/`, `llm/`, `ocel/`, `a2a/`, `mcp/`, `connectors/`, `audit/`, `metrics/`, `telemetry/`, etc.
- `tests/` — 60+ integration test files + python test scripts
- `benches/` — 8 bench files (Criterion, `harness = false`)
- `examples/` — sparse (mostly shell and python scripts, few .rs examples)

### Notable Dependencies
`serde 1.0, serde_json 1.0, anyhow 1.0, thiserror 1.0, tokio 1 (full), axum 0.8, tracing 0.1, tracing-subscriber 0.3 (optional), blake3 (absent!), opentelemetry 0.21, pyo3 0.21, redis 0.24, sqlx 0.7, petgraph 0.6, uuid 1.0, chrono 0.4, criterion 0.5, proptest 1.4, a2a-rs (git dep), rmcp 1.3.0 (optional)`

### House-Specific Tooling
- **ggen/** directory with `ggen.toml` — ontology-driven code generation (RDF/SPARQL + Tera templates) generating PyO3 bridge code from `ggen/ontology/pm4py-api.ttl`
- `pyproject.toml` (maturin build backend for Python wheel)
- Weaver live-check telemetry pattern (`WEAVER_LIVE_CHECK` env var, OTLP endpoint)
- `src/semconv/` module (semantic conventions)
- MCP server binary (`rmcp` crate)
- A2A protocol support (`a2a-rs` git dependency)

---

## 2. pm4wasm

### Identity
- **Name:** `pm4wasm`
- **Visibility:** Public
- **Structure:** Two Rust crates (root `pm4wasm` lib + `drift-server` binary sub-directory); JS/TS client in `js/`
- **Edition:** 2021
- **MSRV:** Not set (no `rust-version`, no `rust-toolchain.toml`)
- **File count:** ~207 non-hidden files (excluding target/node_modules)
- **Version scheme:** CalVer: `0.1.0` (Cargo); JS package `26.4.7`

### Cargo Metadata Completeness
| Field | Present (root crate) |
|-------|---------------------|
| `license` | Yes (`Apache-2.0`) |
| `description` | Yes |
| `repository` | No |
| `homepage` | No |
| `keywords` | No |
| `categories` | No |
| `authors` | No |
| `rust-version` | No |
| `[workspace.*]` | No (not a workspace; two separate Cargo.toml files) |

### Lints / Quality Config
- None present (no rustfmt.toml, clippy.toml, deny.toml, typos.toml, .editorconfig, [lints] table)

### CI
- No `.github/workflows/` directory — **zero CI**
- No dependabot.yml

### Task Runner
- No Makefile, justfile, or Makefile.toml
- CLAUDE.md explicitly notes: "No Makefile — use cargo/wasm-pack directly"
- Build steps documented in CLAUDE.md and README

### Docs Set
| Doc | Present |
|-----|---------|
| README.md | Yes |
| CLAUDE.md | Yes (detailed setup + architecture sections) |
| CONTRIBUTING.md | No |
| SECURITY.md | No |
| CHANGELOG.md | No (js/ subdir has one) |
| CODE_OF_CONDUCT | No |

### Licensing
- Apache-2.0 only (single LICENSE file)

### src Layout
- `src/lib.rs` — WASM entry point (`#[wasm_bindgen]`)
- `src/` modules: `algorithms/`, `conformance/`, `conversion/`, `discovery/`, `filtering/`, `footprints.rs`, `ocel/`, `statistics/`, `llm/`, `quality/`, `simulation/`, `transformation/`, `event_log.rs`, `petri_net.rs`, `powl.rs`, `process_tree.rs`, `parser.rs`, etc.
- `tests/web/` — wasm-bindgen-test tests
- No `benches/` in root crate
- `js/` — TypeScript client (`vite`, `vitest`, `tsc`)
- `drift-server/` — separate Rust binary (axum WebSocket server)
- `server/`, `deploy/`, `docs/`, `examples/`

### WASM Toolchain Config
- `crate-type = ["cdylib", "rlib"]` in `[lib]`
- `wasm-bindgen = "0.2"`, `js-sys = "0.3"`, `serde-wasm-bindgen = "0.6"`
- `getrandom = { version = "0.2", features = ["js"] }` (WASM-compatible RNG)
- `chrono` with `wasmbind` feature
- `[package.metadata.wasm-pack.profile.release] wasm-opt = false`
- `[profile.release] opt-level = "s"` (size optimization)
- Build: `wasm-pack build --target bundler --release --out-dir js/pkg`
- JS package `pm4wasm` published to npm (version aligned: `26.4.7`)
- Vite dev server for demo

### Notable Dependencies
`wasm-bindgen 0.2, js-sys 0.3, serde 1, serde-wasm-bindgen 0.6, serde_json 1.0, quick-xml 0.37, chrono 0.4 (wasmbind), rand 0.8, getrandom 0.2 (js), console_error_panic_hook 0.1 (optional), wasm-bindgen-test 0.3`
drift-server: `tokio 1.40, axum 0.7, tracing 0.1, tracing-subscriber 0.3, anyhow 1.0, thiserror 1.0, prometheus 0.13, uuid 1.10, chrono 0.4`

### House-Specific Tooling
- None detected (no ggen, no ontology, no semconv)

---

## 3. miniml

### Identity
- **Name:** `miniml` (monorepo); Rust crate name `wminml` (in `crates/miniml-core/`)
- **Visibility:** Public
- **Structure:** Hybrid monorepo — one Rust crate (`crates/miniml-core/`) + one TS/JS package (`packages/miniml/`); managed by pnpm workspaces + Turborepo
- **Edition:** 2021
- **MSRV:** CONTRIBUTING.md says "Rust 1.75+"; no `rust-version` in Cargo.toml; no rust-toolchain.toml
- **File count:** ~203 non-hidden files (excluding target/node_modules)
- **Version scheme:** CalVer: `26.4.8`

### Cargo Metadata Completeness (miniml-core)
| Field | Present |
|-------|---------|
| `license` | `license-file = "LICENSE"` (BSL 1.1) |
| `description` | Yes |
| `repository` | Yes |
| `homepage` | No |
| `keywords` | Yes (5) |
| `categories` | Yes (3) |
| `authors` | Yes (`Sean Chatman <info@chatmangpt.com>`) |
| `rust-version` | No |
| `[workspace.*]` | No workspace Cargo.toml |

### Lints / Quality Config
- None present (no rustfmt.toml, clippy.toml, deny.toml, typos.toml, .editorconfig)

### CI
- `.github/FUNDING.yml` only — **no workflow files**
- No dependabot.yml

### Task Runner
- **Root:** `package.json` with pnpm + Turborepo (`turbo.json`)
- Turbo tasks: `build`, `test`, `bench`, `typecheck`, `clean`
- Scripts: `build:wasm` (wasm-pack → `packages/miniml/wasm`), `test:rust` (`cargo test`), etc.
- No Makefile, justfile, or Makefile.toml

### Docs Set
| Doc | Present |
|-----|---------|
| README.md | Yes (algorithm-heavy; no quick-setup heading) |
| CLAUDE.md | Yes (architecture + code patterns + testing workflow) |
| CONTRIBUTING.md | Yes |
| SECURITY.md | Yes |
| CHANGELOG.md | Yes |
| CODE_OF_CONDUCT | No |

### Licensing
- BSL 1.1 (Business Source License) — single LICENSE file
- **Diverges significantly** from MIT/Apache cluster norm

### src Layout
- `crates/miniml-core/src/lib.rs` — WASM entry point
- 60+ algorithm modules in `src/` (classification, regression, clustering, neural, ensemble, etc.)
- `crates/miniml-core/benches/` (Criterion, wasm_bench)
- `crates/miniml-core/examples/` — empty or absent
- `packages/miniml/` — TypeScript wrapper (vitest, tsup, vite bench)
- `packages/miniml/src/` — TS bindings
- `docs/` — extensive Diátaxis structure + thesis/

### WASM Toolchain Config
- `crate-type = ["cdylib", "rlib"]`
- `wasm-bindgen = "0.2"` with `serde-serialize` feature
- `js-sys = "0.3"`, `serde-wasm-bindgen = "0.6"`
- `[package.metadata.wasm-pack.profile.release] wasm-opt = false`
- `[profile.release] panic = "abort"` (WASM best practice)
- pnpm workspace: `wasm-pack build --target web --out-dir ../../packages/miniml/wasm`
- TS side uses `vitest`, `tsup` (bundler), `turbo` for task orchestration

### Notable Dependencies
`wasm-bindgen 0.2, js-sys 0.3, serde 1.0, serde_json 1.0, serde-wasm-bindgen 0.6, bincode 1.3, base64 0.21, wasm-bindgen-test 0.3, criterion 0.5`

### House-Specific Tooling
- None (no ggen, no ontology, no semconv, no A2A)

---

## Cross-Repo Conventions Observed

1. **Edition 2021** universally
2. **`crate-type = ["cdylib", "rlib"]`** for all WASM crates
3. **`wasm-pack`** as the WASM build tool (all three reference it)
4. **`[package.metadata.wasm-pack.profile.release] wasm-opt = false`** appears in both WASM crates
5. **`wasm-bindgen 0.2`, `js-sys 0.3`, `serde-wasm-bindgen 0.6`** as the standard WASM bind stack
6. **`serde 1.0` + `serde_json 1.0`** in every crate
7. **CalVer version scheme** (`26.4.x` / `2026.3.x`) — not semver
8. **Single author:** `Sean Chatman <info@chatmangpt.com>` (when set)
9. **CLAUDE.md present** in two of three (pm4wasm, miniml) — used as operational runbook for Claude Code
10. **No `rust-toolchain.toml`** in any repo
11. **`getrandom` with `js` feature** for WASM RNG compatibility
12. **Swatinem/rust-cache@v2** as the standard cache action (pm4py-rs)
13. **`dtolnay/rust-toolchain@<channel>`** as the toolchain installer action (pm4py-rs)

---

## Divergences / Inconsistencies

| Area | Observation |
|------|-------------|
| **License** | Three different licenses: pm4py-rs=AGPL-3.0-or-later, pm4wasm=Apache-2.0, miniml=BSL 1.1. No standard across the cluster. |
| **CI** | Only pm4py-rs has GitHub Actions workflows; pm4wasm and miniml have zero CI pipelines. |
| **MSRV** | pm4py-rs specifies `rust-version = "1.85"` in Cargo.toml (but CI tests against 1.70 MSRV — mismatch). pm4wasm and miniml specify no MSRV. |
| **Task runner** | Inconsistent: pm4py-rs uses Makefile; miniml uses pnpm+Turborepo; pm4wasm has no task runner at all. |
| **Workspace structure** | No repo uses `[workspace.package]` / `[workspace.dependencies]` / `[workspace.lints]` inheritance, even miniml which is a multi-package monorepo. |
| **Metadata completeness** | pm4wasm is severely incomplete (missing repository, keywords, categories, authors, rust-version). |
| **Lints config** | No repo uses `[lints]` table or rustfmt.toml / clippy.toml files — all lint enforcement is inline in CI commands or absent. |
| **WASM size strategy** | pm4wasm uses `opt-level = "s"` (size); miniml uses `opt-level = 3` (speed) — opposite priorities. |
| **House tooling (ggen)** | Only pm4py-rs uses the ggen ontology-driven code generation system; absent in pm4wasm and miniml. |
| **Root bloat** | pm4py-rs has 80+ AI-session-generated markdown files in the root — a maintenance hygiene problem absent in the other two. |
| **JavaScript packaging** | pm4wasm publishes to npm under `pm4wasm` name (v26.4.7); miniml uses a pnpm workspace with Turborepo; pm4py-rs has no JS layer. |
