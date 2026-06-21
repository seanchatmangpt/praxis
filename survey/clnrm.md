---

# Boilerplate Fingerprint Report — `clnrm` & `clnrm_prototype`

---

## Repo 1: `clnrm`

### Identity
- **Visibility:** Public
- **Structure:** Workspace (4 members: `clnrm-cli`, `clnrm-core`, `clnrm-lsp`, `truex-core`)
- **Rust edition:** 2021
- **MSRV / Toolchain:** `nightly-2026-04-15` pinned via `rust-toolchain.toml`; components: `rustfmt`, `clippy`
- **Version scheme:** Calendar versioning — `26.5.28` (year.month.day)
- **Repo size:** ~602 `.rs` files; Cargo.lock is 8,826 lines; root dir has ~100+ files (very cluttered with `.py` fix scripts, `.md` agent reports, `.txt` status files, `.d` demo files, etc.)

### Cargo Metadata Completeness
- `[workspace.package]` used: yes — version, edition, license (`MIT`), authors, repository all inherited
- `[workspace.dependencies]` used: yes, extensively (all shared deps centralized)
- `[workspace.lints]` used: **no** — no workspace-level lints block
- Per-crate `version.workspace = true`, `edition.workspace = true`, etc. used in `clnrm-core` and `clnrm-lsp`; `clnrm-cli` sets its own version without workspace inheritance
- Missing from workspace package: `description`, `homepage`, `keywords`, `categories`, `rust-version`
- `clnrm-core` adds `keywords`, `categories`, `description` locally

### Lints / Quality Config
- `rustfmt.toml`: present but **empty** (only a comment: "# Rustfmt configuration for clnrm")
- `clippy.toml`: not present
- `deny.toml`: present and detailed:
  - advisories: `vulnerability = "deny"`, `yanked = "deny"`, `unmaintained = "warn"`
  - licenses: deny `GPL-3.0`, `AGPL-3.0`; allow MIT, Apache-2.0, BSD-2/3, ISC, CC0, Unicode
  - bans: `wildcards = "deny"`, `multiple-versions = "warn"`
  - sources: `allow-registry` (crates.io only), select git allowlist for opentelemetry repos
- `typos.toml`: not present
- `.editorconfig`: not present
- No `[workspace.lints]` block; lints not set at workspace level

### CI
Workflows (29 files — extremely large CI surface):

| File | Purpose / Jobs |
|------|---------------|
| `ci.yml` | test matrix (ubuntu+macos, stable), fmt, clippy, Weaver install, consistency scan, unit/doc/integration tests, E2E (ubuntu only), security audit, coverage (tarpaulin → Codecov) |
| `unit-tests.yml` | Docker-independent unit tests, ubuntu+macos matrix |
| `fast-tests.yml` | Quick subset tests |
| `integration-tests.yml` | Full integration suite |
| `quality.yml` | Multi-gate: TOML syntax, Weaver schema, additional quality checks |
| `fuzz.yml` | Daily scheduled fuzzing (`cargo-fuzz`), 3 targets, plus PR path trigger |
| `publish-crates.yml` | Manual/tag publish to crates.io with Weaver-first validation |
| `release.yml` | Tag-triggered; build cross-platform binaries + GitHub Release |
| `homebrew-release.yml` | Homebrew tap update |
| `documentation.yml` | mdBook + GitHub Pages |
| `pages.yml` | GitHub Pages deploy |
| `performance.yml` / `performance-regression.yml` | Criterion benchmarks |
| `telemetry-validation.yml` | OTel pipeline validation |
| `schema-validation.yml` | Weaver registry check |
| `contract-tests.yml` | API contract tests |
| `best-practices.yml` | Code standards audit |
| `weaver-*.yml` (4 files) | Weaver refactor, live-check, validation gate, general validation |
| `lib-*.yml` (7 files) | Reusable workflow fragments (install-weaver, command-check, dependency-check, script-check, verify-artifact, port-health, process-health, port-cleanup) |

- **Runner OS matrix:** ubuntu-latest + macos-latest (primary); some workflows include windows-latest
- **Toolchain:** `dtolnay/rust-toolchain@stable`; nightly in fuzz
- **Caching:** Composite action `.github/actions/setup-rust-cache` using `Swatinem/rust-cache@v2`; also `.github/actions/install-cargo-tool` for cargo tool caching
- **Coverage:** `cargo-tarpaulin` → Codecov (`codecov/codecov-action@v4`)
- **Audit:** `cargo-audit`, `cargo-deny`
- **Notable actions:** `actions/checkout@v4`, `dtolnay/rust-toolchain`, `Swatinem/rust-cache@v2`, `taiki-e/install-action@v2`, `softprops/action-gh-release`, `codecov/codecov-action@v4`

### Task Runner
- `Makefile.toml` (cargo-make); `Makefile.toml.full` (full extended version)
- Key recipes: `dev`, `quick`, `fix`, `watch`, `test`, `test-all`, `test-integration`, `test-ci`, `validate`, `validate-crate`, `validate-production-readiness`
- Config: `skip_core_tasks = true`, `default_to_workspace = false`, 1s timeout enforcement on all test tasks via `timeout 1s cargo test`
- `scripts/` directory: 20+ shell scripts (`coverage.sh`, `detect-mura.sh`, `check-consistency.sh`, `scan-fakes.sh`, CI gate scripts, doc validation scripts)

### Docs Set
- README: yes (headings: Overview, v3.0 Architecture, Quick Start, Configuration, Documentation, Code Standards, License)
- SECURITY.md: yes
- CHANGELOG.md: yes (presence only)
- CLAUDE.md: **not present** (despite being referenced in the affidavit project)
- CONTRIBUTING.md: **not present**
- CODE_OF_CONDUCT.md: not present
- LICENSE: yes (MIT only — single file)
- Additional: `PROJECT.md`, `CODE_STANDARDS.md`, `ARCHITECTURE_DIAGRAMS.md`, many agent/evidence `.md` files

### Licensing
- MIT only; single `LICENSE` file

### src Layout
- No top-level `src/`; library code is in workspace members under `crates/`
- `crates/clnrm-core/src/lib.rs` — main library
- `crates/clnrm-cli/src/main.rs` — CLI binary
- `crates/clnrm-lsp/src/main.rs` — LSP binary
- `crates/truex-core/src/` — trust/verification core
- `tests/` — extensive: `.clnrm.toml`-based integration tests (not Rust tests), plus Rust files in `tests/fuzz/`, `tests/contracts/`, `tests/mdbook-examples/`
- `benches/` — 13+ Criterion benchmark files
- `examples/` — mix of `.rs` examples and `.clnrm.toml` config examples

### Common Dependencies
`tokio 1.x`, `serde 1.x`, `serde_json 1.x`, `anyhow 1.x`, `tracing 0.1`, `tracing-subscriber 0.3.20`, `opentelemetry 0.32`, `opentelemetry_sdk 0.32`, `opentelemetry-otlp 0.32`, `opentelemetry-semantic-conventions 0.32`, `tracing-opentelemetry 0.33`, `clap 4.5.49`, `linkme 0.3`, `uuid 1.x`, `chrono 0.4`, `mockall 0.14`, `criterion 0.5.1`, `insta 1.34`, `surrealdb 3.1`, `testcontainers 0.27`, `proptest 1.4`, `serial_test 3.2`, `petgraph 0.8`, `blake3` (noted in architecture but not visible in workspace Cargo.toml), `chicago-tdd-tools 26.6.121`

### House-Specific Tooling
- `ggen.toml` — code generation config using RDF/SPARQL + Tera templates (ontology at `ggen-base.ttl`; generates LSP backend/CLI from TTL ontology)
- `cleanroom.toml` — project-level framework config (present in root and `crates/clnrm-core/`)
- `registry/` — YAML-based OTel semantic convention registry (attributes, metrics, events)
- `Makefile.weaver` — Weaver-specific make tasks
- `scenarios/` — scenario definition files
- `specs/` — spec files
- Many `fix_*.py` / `plan_*.py` scripts (AI-agent artefacts)
- Agent evidence files: `AGENT_10_*`, `AGENT_4_*`, `AGENT_5_*`, `10_AGENTS_DELIVERABLES_SUMMARY.md`

---

## Repo 2: `clnrm_prototype`

### Identity
- **Visibility:** Public
- **Structure:** Single-crate (no workspace)
- **Rust edition:** 2021
- **MSRV / Toolchain:** No `rust-toolchain.toml` present; CI uses `dtolnay/rust-toolchain@stable`
- **Version scheme:** Semver — `0.2.0`
- **Repo size:** ~143 `.rs` files; Cargo.lock is 4,239 lines; repo is clean and well-organized

### Cargo Metadata Completeness
- Single `[package]` section; no workspace
- Set: `name`, `version`, `edition`, `authors`, `description`, `license` (MIT), `repository` (incorrect — points to `https://github.com/sac/ggen`), `keywords`, `categories`
- Missing: `homepage`, `rust-version`
- `[workspace.package]` / `[workspace.dependencies]` / `[workspace.lints]`: N/A (single crate)
- Features block present with: `default`, `coverage`, `signing`, `services`, `advanced_benchmarks`

### Lints / Quality Config
- No `rustfmt.toml`, no `clippy.toml`, no `deny.toml`, no `typos.toml`, no `.editorconfig`
- `[lints.clippy]` in `Cargo.toml`: `expect_used = "deny"`, `unwrap_used = "deny"`, `panic = "warn"`, `indexing_slicing = "warn"`
- `lib.rs` has `#![cfg_attr(test, allow(clippy::unwrap_used, ...))]` exemption for tests

### CI
| File | Purpose / Jobs |
|------|---------------|
| `ci.yml` | `check` (fmt + clippy), `build` (ubuntu+macos+windows × stable+nightly matrix, excludes nightly on mac/windows), `test` (ubuntu+macos+windows, all test types, Docker setup per OS), `coverage` (llvm-cov → Codecov, 80% threshold), `security` (cargo-audit + cargo-deny), `benchmark`, `dependency-review` (PR only), `ci-success` (aggregator) |
| `release.yml` | Tag + manual trigger; validate → build-artifacts (4 targets: linux x86_64, macos x86_64+arm, windows) → changelog (git-log-based) → publish (crates.io) → GitHub Release (softprops) → notify |
| `nightly.yml` | Scheduled 02:00 UTC; nightly Rust, Miri, proptest (10k cases), full benchmarks, GitHub issue on failure |
| `.github/dependabot.yml` | Cargo weekly (grouped: dev-deps minor/patch, prod-deps patch); GH Actions weekly; reviewer `sac`; `chore(deps)` commit prefix |

- **Runner OS matrix:** ubuntu + macos + windows for build; ubuntu + macos for tests
- **Toolchain:** `dtolnay/rust-toolchain@stable` (primary), `@master` for matrix builds, `@nightly` for nightly job
- **Caching:** `actions/cache@v4` with `**/Cargo.lock` hash key; manual cargo cache paths
- **Coverage:** `cargo-llvm-cov` → Codecov; 80% threshold enforced inline
- **Audit:** `cargo-audit`, `cargo-deny`

### Task Runner
- `Makefile.toml` (cargo-make); extensive task list:
  - Core: `check`, `check-release`, `build`, `build-release`, `clean`, `fmt`, `fmt-check`, `lint`, `lint-fix`
  - Testing: `test`, `test-unit`, and more
  - Env sections: `[env.DEV]`, `[env.RELEASE]`, `[env.BENCH]`, `[env.PERF]`, `[env.SECURITY]`, `[env.DOCS]`, `[env.TESTCONTAINERS]`
- `run_fast_tests.sh` — shell script to run fast tests (`SKIP_SLOW_TESTS=1`)

### Docs Set
- README.md: yes (heavy `##` heading structure; 80/20 framing throughout)
- CHANGELOG.md: yes
- CONTRIBUTING.md: yes (sections: Code of Conduct, Getting Started, Development Setup, Contributing Process, Code Style, Testing, Documentation, Release Process)
- LICENSE: yes (MIT)
- SECURITY.md: **not present**
- CLAUDE.md: **not present**
- CODE_OF_CONDUCT.md: not present (referenced in CONTRIBUTING but not committed)

### Licensing
- MIT only; single `LICENSE` file

### src Layout
- Single crate, `src/lib.rs` as main library
- `src/bin/`: 3 binaries — `bench.rs`, `cleanroom.rs`, `micro_cli.rs`
- `src/` subdirectories: `backend/`, `builder/`, `executor/`, `guards/`, `ids/`, `lifecycle/`, `observability/`, `otel/`, `runtime/`, `services/`, `streaming/`, `test_utils/`
- `tests/`: extensive Rust integration tests + fixture files; `benches/`: 4 Criterion files; `examples/`: ~25 `.rs` files + CLI examples in bash/python/js, many `.disabled` variants

### Common Dependencies
`serde 1.x`, `serde_json 1.x`, `anyhow 1.x`, `tokio 1.47`, `tracing 0.1`, `tracing-subscriber 0.3`, `clap 4.5`, `uuid 1.x`, `chrono 0.4`, `regex 1.x`, `tempfile 3.x`, `sha2 0.10`, `testcontainers 0.25`, `testcontainers-modules 0.13`, `opentelemetry 0.31`, `opentelemetry_sdk 0.31`, `opentelemetry-otlp 0.31`, `opentelemetry-jaeger 0.22`, `rand 0.9`, `base64 0.22`, `toml 0.9`, `criterion 0.7`, `proptest 1.x`, `insta 1.x`, `cucumber 0.21`, `mockito 1.x`, `assert_cmd 2.x`

### House-Specific Tooling
- None identified — this is the cleanest of the two repos

---

## Conventions Observed

1. **Calendar versioning** in the production workspace (`clnrm`); semver in prototype — active divergence
2. **MIT-only licensing** (not dual MIT/Apache-2.0) across both repos
3. **`cargo-make` as task runner** (`Makefile.toml`) in both repos; not `just`
4. **`dtolnay/rust-toolchain` action** consistently used for toolchain installation
5. **`actions/cache@v4`** for cargo caching; `clnrm` additionally uses `Swatinem/rust-cache@v2` in composite action
6. **`actions/checkout@v4`** standard
7. **`softprops/action-gh-release`** for GitHub releases in both repos
8. **`cargo-audit` + `cargo-deny`** for security/license checking in both
9. **Coverage → Codecov** via `codecov/codecov-action@v4` in both
10. **Clippy `unwrap_used = "deny"` and `expect_used = "deny"`** in production code in both repos (enforced via `[lints.clippy]` in prototype; in clnrm enforced by scripts/check-best-practices)
11. **OpenTelemetry** as a first-class dependency in both (versions differ: 0.31 vs 0.32)
12. **`clap 4.x` with `derive` feature** for CLI in both
13. **`criterion` for benchmarks** in both
14. **`insta` for snapshot testing** in both
15. **Dependabot** only in prototype (not in `clnrm`)
16. **`ggen.toml` + TTL ontology** as code-generation system — unique to `clnrm`
17. **`linkme 0.3`** for plugin discovery in `clnrm` only
18. **`surrealdb`** as a runtime dependency — `clnrm` only

## Divergences / Inconsistencies

| Concern | `clnrm` | `clnrm_prototype` |
|---------|---------|-------------------|
| Rust toolchain pinning | nightly-2026-04-15 pinned | No pin; uses stable in CI |
| Workspace structure | 4-member workspace with [workspace.package] | Single crate |
| Version scheme | Calendar (26.5.28) | Semver (0.2.0) |
| [workspace.lints] | Absent | N/A |
| rustfmt.toml | Present but empty | Absent |
| deny.toml | Detailed, committed | Absent (cargo-deny run in CI but no config file) |
| Dependabot | Absent | Present and grouped |
| Repo cleanliness | Very cluttered (100+ fix/agent scripts in root) | Clean |
| Repository URL in Cargo.toml | Correct (github.com/seanchatmangpt/clnrm) | Wrong (points to github.com/sac/ggen) |
| CI complexity | 29 workflow files including lib-* reusables + composite actions | 4 workflow files |
| Nightly CI | Via nightly toolchain pin only | Explicit nightly.yml with Miri |
| OS matrix | ubuntu + macos (most jobs) | ubuntu + macos + windows |
| Coverage tool | cargo-tarpaulin | cargo-llvm-cov |
| Coverage threshold | Not enforced in CI | 80% enforced inline in workflow |
| OTel versions | 0.32.x | 0.31.x |
| testcontainers version | 0.27 | 0.25 |
| House-specific tooling | ggen.toml, cleanroom.toml, registry/, Weaver | None |
| CONTRIBUTING.md | Absent | Present |
| SECURITY.md | Present | Absent |
| Local monorepo deps | clap-noun-verb, clnrm-template, lsp-max, chicago-tdd-tools | None |
