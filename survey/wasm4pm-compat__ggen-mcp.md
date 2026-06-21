---

## Boilerplate Fingerprint Report — wasm4pm-compat & ggen-mcp

---

### Repo 1 — `wasm4pm-compat`

**Status:** PUBLIC

#### Identity

| Field | Value |
|---|---|
| Crate name | `wasm4pm-compat` |
| Version | `26.6.14` (date-based: YY.M.D) |
| Workspace | Single-crate workspace (`resolver = "2"`, excludes `wasm4pm-compat-ts` and `wasm4pm-compat-lsp`) |
| Rust edition | 2021 |
| MSRV / toolchain | **Nightly-pinned** — `nightly-2026-05-04`; components: `rustfmt`, `clippy`, `rust-src` |
| Crate type | `rlib` (library, no binary) |
| Approx. source size | 70 files in `src/`, ~28 K LOC; 4 bench files; 20+ integration test files; 20+ example files |

#### Cargo Metadata Completeness

- Set: `name`, `version`, `edition`, `description`, `license` (`MIT OR Apache-2.0`), `repository`, `keywords` (5), `categories` (2), `readme`, `documentation`
- Not set: `authors`, `homepage`, `rust-version` (nightly-first; intentionally omitted)
- `[workspace.package]` / `[workspace.dependencies]` / `[workspace.lints]` inheritance: **not used** (single-crate, no shared table needed)
- Large `[package].exclude` list filtering agent artifacts, paper/research directories, generated files

#### Features

Three public features exactly: `default = ["formats"]`, `strict`, `wasm4pm` — intentionally capped at three per documented invariant.

#### Lints / Quality Config

| File | Content |
|---|---|
| `rustfmt.toml` | `edition = "2021"` only |
| `clippy.toml` | Comment only — "nightly-first, no MSRV-aware lints" |
| `deny.toml` | Not present |
| `typos.toml` | Not present |
| `.editorconfig` | Not present |
| `anti-llm.toml` | **House-specific** — scanner suppression manifest for the `anti-llm-cheat-lsp` tool |

#### CI

**No `.github/workflows/` directory.** CI is described in `cicd.toml` (a house-specific `cargo make`-style state file, not a GitHub Actions definition). The `.github/` directory contains only issue templates and a PR template — no workflow YAML.

| GitHub config | Present |
|---|---|
| Issue templates | 3 (bug_report, paper_coverage, type_law_gap) |
| PR template | Yes (commit-class checklist) |
| Dependabot | No |
| Workflows | **None** |

#### Task Runner

Dual task runners: `Justfile` + `Makefile.toml` (cargo-make).

**Justfile** recipes: `test`, `test-full`, `polish`, `build`, `clean`, `publish`, `ci`, `anti-cheat-gate`

**Makefile.toml** tasks: `check`, `check-all`, `build`, `build-all`, `test`, `test-all`, `test-minimal`, `alive`, `doc-test`, `doc`, `clippy`, `fmt`, `fmt-fix`, `build-formats`, `build-strict`, `build-wasm4pm`, `ggen-witnesses` (many variants), `ci`, `miri-setup`, `miri`

**scripts/** contains ~25 shell audit scripts: `audit_*.sh`, `validate*.sh`, `emit_receipts.sh`, `crown_audit_runner.sh`, etc.

#### Docs Set

| Doc | Present |
|---|---|
| README.md | Yes — headings: version alignment, architecture mandate, evidence lifecycle, boundary laws, OCEL, receipt shapes, feature contract, verification commands, examples, license |
| CLAUDE.md | Yes — headings: ggen provision, nightly Rust, testing surfaces, build commands, architecture, type-law receipts, DX surfaces, invariants |
| CHANGELOG.md | Yes (milestone-based, not semver) |
| SECURITY.md | Yes (scope, supported versions, reporting, known non-issues) |
| CONTRIBUTING.md | Not found |
| CODE_OF_CONDUCT.md | Not found |

#### Licensing

Dual **MIT OR Apache-2.0** — both `LICENSE-MIT` and `LICENSE-APACHE` files present.

#### src Layout

Library crate (`rlib`). `src/` has ~70 modules including `lib.rs`, `admission.rs`, `law.rs`, `loss.rs`, `receipt.rs`, `witness.rs`, `witnesses.rs` (ggen-generated), `verifier/`, `ocel/`, `parity/`, `import/`. `tests/` has 20+ integration test files plus `ui/` trybuild fixtures. `benches/` has 4 Criterion benchmarks. `examples/` has 20+ runnable demos.

#### Common Dependencies

`blake3 1.8.5`, `serde 1.0.228` + `serde_json 1.0`, `chrono 0.4.45`, `uuid 1.23.2`, `quick-xml 0.36`, `hashbrown 0.17`, `rustc-hash 2`, `trybuild` (dev), `criterion 0.5` (dev). No `tokio`, `clap`, `anyhow`, `thiserror`, `tracing`, or `opentelemetry` at the crate level.

#### House-Specific Tooling

| Artifact | Description |
|---|---|
| `.ggen/` | ggen runtime cache: `inference_state.sha256`, `manifest.sha256`, `ontology.sha256`, `rules.sha256`; + receipts/ of JSON sync receipts |
| `ggen-witness.toml` | ggen project manifest — ontology sources, SPARQL queries, Tera template → `src/witnesses.rs` generation pipeline |
| `cicd.toml` | House-specific CI state tracking (target size, changed files, trybuild snapshot mode) |
| `anti-llm.toml` | Anti-cheat scanner suppression config |
| `ggen/` | Ontology sources (`.ttl`), SPARQL queries, Tera templates, projection manifests, emitted artifacts, audit scripts |
| `AGENT_REPORTS/`, `AGENT_*.txt` | Multi-agent delivery receipts |
| `checkpoints/`, `receipts/` | Agent-phase checkpoint and validation receipt JSON files |
| `.cargo/config.toml` | Empty (nightly features via `src/lib.rs` `#![feature(...)]`; no RUSTFLAGS needed) |

---

### Repo 2 — `ggen-mcp`

**Status:** PUBLIC

#### Identity

| Field | Value |
|---|---|
| Package name | `spreadsheet-mcp` (Cargo name) / repo name `ggen-mcp` |
| Version | `1.0.0` (semver) |
| Workspace | Single-crate — no `[workspace]` table in Cargo.toml; local path deps (`ggen/crates/*`, `chicago-tdd-tools`) referenced via `path=` |
| Rust edition | **2024** |
| MSRV / toolchain | `stable` (no `rust-toolchain.toml` found) |
| Crate type | Binary + lib (`src/main.rs`, `src/lib.rs`, `src/bin/`) |
| Approx. source size | 162 files in `src/`, ~11 K LOC; 2 bench files; 10+ integration test files; 10+ example files |

#### Cargo Metadata Completeness

- Set: `name`, `version`, `edition`, `description`, `license` (`Apache-2.0`), `repository`, `authors`
- Not set: `keywords`, `categories`, `homepage`, `readme`, `documentation`, `rust-version`
- Note: `name = "spreadsheet-mcp"` / `repository` point to a **different upstream** (`PSU3D0/spreadsheet-mcp`) — this appears to be a fork with ggen-mcp overlay
- `[workspace.package]` / `[workspace.dependencies]` / `[workspace.lints]`: **not used**

#### Lints / Quality Config

| File | Present |
|---|---|
| `rustfmt.toml` | Not found |
| `clippy.toml` | Not found |
| `[lints]` in Cargo.toml | Not present |
| `deny.toml` | Not found |
| `typos.toml` | Not found |
| `.editorconfig` | Not found |

Quality enforcement is entirely process-based (cargo-make tasks, `.claude/hooks/`).

#### CI

4 workflow files under `.github/workflows/`:

| Workflow | Jobs | Runner matrix | Toolchain | Notable |
|---|---|---|---|---|
| `ci.yml` | test-and-build (ubuntu+macos), docker-integration | ubuntu-latest, macos-latest | `stable` via `actions-rust-lang/setup-rust-toolchain@v1` | `cargo fetch --locked`, uploads binary artifacts; `actions/cache@v4` on `Cargo.lock` hash |
| `release.yml` | build (4 targets), release, publish | ubuntu, macos, windows | `stable` | Cross-compile aarch64-apple-darwin; `softprops/action-gh-release@v2`; `cargo publish --locked` |
| `coverage.yml` | coverage, coverage-check | ubuntu-latest | `stable` + `llvm-tools-preview` | `taiki-e/install-action` for cargo-llvm-cov; Codecov upload; 50% line threshold; daily cron schedule; PR comment with emoji table |
| `docker.yml` | build-slim (+ implied full) | ubuntu-latest | — | GHCR push via `docker/login-action@v3`; `docker/metadata-action@v5`; `docker/setup-buildx-action@v3` |

Dependabot: not present.

#### Task Runner

`Makefile.toml` (cargo-make only; no Justfile).

Tasks: `sync`, `sync-dry-run`, `sync-validate`, `sync-force`, `sync-script`, `gen`, `check`, `build`, `build-release`, `fmt`, `fmt-check`, `lint`, `test`, `test-all`, `test-integration`, `test-ggen`, `test-traceability`, `test-ddd`, `test-determinism`, `validate-sparql-limits`, `pre-commit`, `ci`, `dev`, `validate-pipeline`, `clean`, `clean-generated`, `regenerate`, `doc`, `doc-open`, `ggen-build`

**scripts/** (10 shell scripts): `coverage.sh`, `ggen-sync.sh`, `load-test.sh`, `local-docker-mcp.sh`, `snapshot_manager.sh`, `start-monitoring.sh`, `stop-monitoring.sh`, `test_validators.sh`, `validate_sparql_limits.sh`, `verify_entitlement.sh`, `verify_manifest.sh`

#### Docs Set

| Doc | Present |
|---|---|
| README.md | Yes — extensive: architecture, tool surface, VBA, recalc, Docker, MCP config, testing, coverage, ggen integration |
| CLAUDE.md | Yes — SPR protocol, TPS philosophy, proof-first, poka-yoke, Chicago-TDD |
| CHANGELOG.md | Yes (Keep a Changelog / semver format) |
| SECURITY.md | Not found |
| CONTRIBUTING.md | Not found |
| CLAUDE-DESKTOP.md | Yes (additional Claude tooling guide) |
| Many `*_SUMMARY.md`, `*_IMPLEMENTATION.md` | Yes — 80+ agent-generated implementation docs at root |

#### Licensing

Single **Apache-2.0** — one `LICENSE` file.

#### src Layout

Binary/library hybrid. `src/` has `main.rs`, `lib.rs`, `server.rs`, `state.rs`, `config.rs`, `error.rs`, `model.rs`, and subdirectories: `analysis/`, `audit/`, `bin/`, `codegen/`, `diff/`, `dod/`, `domain/`, `entitlement/`, `formula/`, `generated/`, `guards/`, `health.rs`, `logging.rs`, `metrics.rs`, `ontology/`, `recalc/`, `recovery/`, `sparql/`, `styles.rs`, `template/`, `tools/`, `utils.rs`, `validation/`, `workbook.rs`. `tests/` has integration tests + snapshot fixtures. `benches/` has 2 Criterion files.

#### Common Dependencies

`anyhow 1.0`, `thiserror 1.0`, `serde 1.0` + `serde_json 1.0` + `serde_yaml 0.9` + `serde_with 3.8`, `tokio 1.37` (full), `tracing 0.1` + `tracing-subscriber 0.3` (json feature), `tracing-appender 0.2`, `opentelemetry 0.22` + `opentelemetry-otlp 0.15` + `opentelemetry_sdk 0.22` + `tracing-opentelemetry 0.23`, `clap 4.5` (derive + env), `axum 0.8`, `rmcp 0.11` (MCP transport), `rmcp-macros 0.11`, `schemars 1.0`, `tera 1.20`, `oxigraph 0.5.1`, `chrono 0.4`, `uuid 1.10`, `prometheus-client 0.22`, `reqwest 0.12`, `criterion 0.5` (dev), `proptest 1.5` (dev), `testcontainers 0.23` (dev).

#### House-Specific Tooling

| Artifact | Description |
|---|---|
| `ggen.toml` | ggen v6 project manifest — ontology source, SPARQL prefixes, code generation pipeline |
| `ggen-mcp.ttl` | Primary Turtle ontology for the MCP domain |
| `ontology/` | `mcp-domain.ttl`, `shapes.ttl` |
| `ggen/` | Sub-directory with local ggen crates (path deps: `ggen-ontology-core`, `ggen-core`, `ggen-domain`, `ggen-config` all at `0.2.0`) |
| `chicago-tdd-tools/` | Local crate used in dev-dependencies |
| `.claude/` | Claude Code settings: `settings.json` (model config sonnet-4-5, haiku subagents), agents/, hooks/, rules/, scripts/ |
| `.cursor/` | Cursor IDE commands and rules |
| `.cursorrules` | SPR × TPS × ontology-driven coding standards |
| `profiles/` | `enterprise-strict.toml`, `ggen-mcp-default.toml` |
| `snapshots/`, `generated/` | Snapshot test artifacts and generated Rust code |
| `observability/`, `grafana/`, `prometheus/`, `loki/`, `alertmanager/` | Full observability stack configs |
| `docker-compose.monitoring.yml`, `docker-compose.observability.yml` | Docker observability stack |
| `Dockerfile`, `Dockerfile.full` | Container images (slim read-only vs full with recalc) |
| `.env.example` | Environment variable template |
| `verify_poka_yoke.sh` | Poka-yoke verification script |
| `meta/ggen.toml` | Meta-level ggen config |
| `queries/`, `templates/` | SPARQL query files and Tera templates for code generation |

---

### Conventions Observed (both repos)

1. **ggen ecosystem** is central: `ggen.toml` manifest, SPARQL → Tera → Rust generation pipeline, ontology as single source of truth
2. **cargo-make** (`Makefile.toml`) is the universal task runner; both repos use it
3. **BLAKE3** for content addressing appears across the ecosystem (direct dep in wasm4pm-compat, conceptually in ggen-mcp via proof-first receipts)
4. **`actions-rust-lang/setup-rust-toolchain@v1`** is the standard toolchain action in CI
5. **`actions/cache@v4`** keyed on `Cargo.lock` hash is the standard caching pattern
6. **`taiki-e/install-action`** used for cargo-llvm-cov in coverage workflows
7. **Criterion 0.5** with `html_reports` feature is the bench harness
8. **Date-based versioning** (YY.M.D) in wasm4pm-compat; semver in ggen-mcp
9. **CLAUDE.md** present in both repos with structured sections for agent/LLM guidance
10. **Receipt / provenance receipts** as agent-phase artifacts are tracked in both repos
11. **MIT OR Apache-2.0** is the preferred dual license (wasm4pm-compat); ggen-mcp deviates with Apache-2.0-only
12. **`serde` + `serde_json`** present everywhere; `chrono`, `uuid` appear in both
13. **`cargo fetch --locked` + `--locked` on all cargo commands** in CI
14. **`cargo llvm-cov`** + Codecov upload for coverage

### Divergences / Inconsistencies

| Issue | wasm4pm-compat | ggen-mcp |
|---|---|---|
| Rust edition | 2021 | **2024** (cutting edge) |
| Toolchain | Nightly-pinned (`nightly-2026-05-04`) | Stable (no rust-toolchain.toml) |
| CI workflows | **None** (no `.github/workflows/`) | 4 workflows (ci, release, coverage, docker) |
| License | MIT OR Apache-2.0 (dual) | Apache-2.0 only |
| Justfile | Yes | No |
| Package name | Matches repo | **Mismatched** (`spreadsheet-mcp` vs repo `ggen-mcp`) — fork origin exposed |
| `rustfmt.toml` depth | Minimal (`edition = "2021"`) | Not present |
| Clippy config | Comment-only | Not present |
| `deny.toml` | Not present | Not present |
| `[workspace.lints]` | Not used | Not used |
| Observability stack | None (no otel dep) | Full OTEL + Prometheus + Grafana + Loki |
| Docker | None | Dockerfile + docker-compose |
| Anti-cheat gate | Yes (`anti-llm.toml`, audit scripts) | No |
| Authors field | Not set | Set (upstream author name) |
| `homepage` / `documentation` fields | `documentation` set | Neither set |
| Crate type | `rlib` (lib only) | Binary + lib |
| Local ggen crates | External (CLI tool) | Embedded as path deps in `ggen/crates/` |
