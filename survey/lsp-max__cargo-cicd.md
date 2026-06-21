---

# Boilerplate Fingerprint Survey — `lsp-max` & `cargo-cicd`

---

## Repo 1: `lsp-max`

### Identity

| Field | Value |
|---|---|
| Visibility | Public |
| Structure | Workspace (34 members) |
| Rust edition | 2021 (via `[workspace.package]`) |
| MSRV | 1.82.0 (`rust-version` in `[workspace.package]`) |
| Toolchain file | `rust-toolchain.toml` — `nightly-2026-04-15`, components `rustfmt clippy`, profile `minimal` |
| Version scheme | CalVer `YY.M.D` (e.g., `26.6.18`), declared in `[workspace.package]` |
| File count | ~1,456 non-git files |
| Language mix | Rust + Next.js webapp (`web/`) + Python script (`write_thesis.py`) |

**Workspace members (34):** root `lsp-max`, `lsp-max-macros`, `lsp-max-protocol`, `lsp-max-runtime`, `lsp-max-agent`, `lsp-max-base`, `lsp-max-cli`, `lsp-max-client`, `lsp-max-live`, `lsp-max-lsif`, `lsp-max-compositor`, `lsp-max-specgen`, `lsp-max-anti-cheat`, `lsp-max-anti-cheat-cli`, `anti-llm-cheat-lsp`, `lsp-max-ast`, `lsp-max-ast-core`, `lsp-max-ast-codegen`, `lsif-rust`, `lsif-typescript`, `lsif-linker`, `lsif-cli`, `lsif-tests`, `playground`, plus several examples as workspace members (`powl-lsp`, `wasm4pm-lsp`, `pattern-lsp`, `clap-noun-verb-lsp`, `wasm4pm-compat-lsp`, `axum-lsp`, `bevy-lsp`, `tex-lsp`, `gc005-wasm4pm-adapter`, `justfile-lsp`).

### Cargo Metadata Completeness

| Field | Set |
|---|---|
| `license` | Yes — `MIT OR Apache-2.0` (workspace-inherited) |
| `description` | Yes (per-crate, non-inherited) |
| `repository` | Yes (workspace) — `https://github.com/seanchatmangpt/lsp-max` |
| `homepage` | Yes (workspace) — same as repository |
| `keywords` | Yes (workspace) — `["language-server","lsp","tower","lsp-3-18"]` |
| `categories` | Yes (workspace) — `["asynchronous","development-tools"]` |
| `authors` | Yes (workspace) — `["Eyal Kalderon <ebkalderon@gmail.com>"]` |
| `rust-version` | Yes (workspace) — `1.82.0` |
| `documentation` | Yes (per-crate, e.g., `https://docs.rs/lsp-max/`) |
| `[workspace.package]` | Yes — full set |
| `[workspace.dependencies]` | Yes — extensive (anyhow, thiserror, tracing, serde, dashmap, rayon, salsa, regex, tree-sitter, bon, rstest, etc.) |
| `[workspace.lints]` | No |
| `[package.metadata.docs.rs]` | Yes — `all-features = true`, `rustdoc-args = ["--cfg", "docsrs"]` |

### Lints / Quality Config

| File | Present | Notes |
|---|---|---|
| `rustfmt.toml` | Yes | `edition = "2021"`, ignores `generated/` |
| `clippy.toml` | No | Clippy flags passed inline via CI |
| `[lints]` in Cargo.toml | No | — |
| `deny.toml` | No | — |
| `typos.toml` | No | — |
| `.editorconfig` | No | — |

CI uses `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### CI

**Workflows: `ci.yml`, `release.yml`**

`ci.yml` (triggers: push/PR to `master`, `main`, `develop`):

| Job | Runner | Toolchain | Cache | Notes |
|---|---|---|---|---|
| `fmt` | ubuntu-latest | `dtolnay/rust-toolchain@stable` + rustfmt | — | `cargo fmt -- --check` |
| `clippy` | ubuntu-latest | `dtolnay/rust-toolchain@stable` + clippy | `Swatinem/rust-cache@v2` | `--workspace --all-targets --all-features -D warnings` |
| `test` | ubuntu-latest | `dtolnay/rust-toolchain@stable` | `Swatinem/rust-cache@v2` | `cargo test --workspace`, timeout 10m |
| `docs` | ubuntu-latest | `dtolnay/rust-toolchain@stable` | `Swatinem/rust-cache@v2` | `cargo doc --no-deps --all-features`, `RUSTDOCFLAGS=-D warnings`, optional `cargo-deadlinks` |
| `result` | ubuntu-latest | — | — | Gate job: fails if any required job fails |

Notable: Multi-checkout pattern — each job checks out **3 sibling repos** (`lsp-types-max`, `wasm4pm-compat`, `wasm4pm`) alongside the main repo because of `[patch.crates-io]` and path deps. Uses `SIBLING_REPO_TOKEN` secret for private siblings. Global env: `CARGO_TERM_COLOR=always`, `RUST_BACKTRACE=1`, `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24=true`.

`release.yml` (trigger: `v*.*.*` tags): Same fmt/clippy/test/docs jobs, plus `publish` (crates.io in dependency order with polling loop) and `github-release` (binary artifacts + checksums + `softprops/action-gh-release@v1`).

No matrix builds (single OS, single toolchain version for tests — stable).

**Dependabot:** `dependabot.yml` present — daily at 21:00, Cargo ecosystem, limit 10 PRs, ignores specific old `tokio` and `lsp-types` versions.

### Task Runner

**Justfile** (shell: `bash -c`):

Recipe groups:
- `bench-admit`, `bench-compositor`, `test-perf-admission`, `test-compositor-admission` — BLAKE3-signed admission receipts
- `test`, `test-e2e`, `test-pre-publish`
- `dx-verify` — architectural boundary grep (forbids `legacy`, `deprecated`, `shim`, `facade`, intermediary type crates)
- `dx-polish` — `cargo fmt --all` + strict clippy
- `dx-all` — polish + verify + clean + intel
- `qol-clean` — `cargo cache`, deep clean if target > 10 GB
- `qol-sync` — `git fetch --prune` across sibling repos
- `spec-graph` — regenerate LSP 3.18 spec artifacts
- `check-laws`, `bench-hot`, `check-fmt`, `doc-coverage`, `health-check` (delegate to `scripts/`)
- `etc-intel` — generates `.autodx/ecosystem-intel.md` manifest
- `release-validate`, `release-dry-run`, `release-publish VERSION`, `release-version-bump NEWVERSION`, `release-notes-extract DATE VERSION`

**scripts/**: `bench-hot-paths.sh`, `check-law-compliance.sh`, `format-and-check.sh`, `generate-lsp-receipts.sh`, `health-check.sh`, `update-doc-coverage.sh`, `write_bench_receipt.sh`, `write_compositor_bench_receipt.sh`

### Docs Set

| Doc | Present |
|---|---|
| README.md | Yes — headings: Quick start, Key features, Directory structure, Published crates, Design principles, Build & test, Sibling dependencies, Documentation, License |
| CLAUDE.md | Yes — full project guide with law-enforcement notes |
| AGENTS.md | Yes — "project constitution" (unique to this ecosystem) |
| CONTRIBUTING.md | Yes |
| CODE_OF_CONDUCT.md | Yes |
| CHANGELOG.md | Yes |
| SECURITY.md | No |
| DOC_COVERAGE_LOG.md | Yes — auto-updated |
| HOOKS_*.md, MCP_*.md | Yes — extensive Claude Code hook and MCP docs |

### Licensing

Dual `MIT OR Apache-2.0`. Both `LICENSE-MIT` and `LICENSE-APACHE` present.

### `src/` Layout

- Root crate: primarily a **lib** (`src/lib.rs`), no top-level `main.rs`
- Subdirectory modules: `composition/`, `coverage/`, `diagnostics/`, `jsonrpc/`, `language_server/`, `primitives/`, `service/`
- Companion crates as separate members (not subdirs of `src/`)
- `tests/` — extensive integration tests with subdirs (`e2e/`, `lsp318_capabilities/`, `max_rpc_handlers/`, `playground/`)
- `examples/` — mix of `.rs` files and workspace member crates
- `benches/` — present in `crates/anti-llm-cheat-lsp/benches/`

### Key Dependencies

`serde 1.0`, `serde_json 1.0`, `thiserror 2.0.18`, `anyhow 1.0`, `tracing 0.1`, `tokio 1` (optional feature-gated), `blake3 1`, `dashmap 6.1`, `parking_lot 0.12`, `rayon 1.11`, `salsa 0.26`, `bon 3.6`, `rstest 0.25`, `regex 1.11`, `tree-sitter 0.26`, `linkme` (not in workspace deps but used per CLAUDE.md), `async-trait 0.1`, `tower 0.4`, `url 2.5`, `ed25519-dalek 2.0`, `oxigraph 0.5.8`, `ureq 3`.

### House-Specific Tooling

- **AGENTS.md** — "project constitution" with law enforcement
- **`crates/anti-llm-cheat-lsp/`** — LSP server that detects law violations
- **`.claude/settings.json`** — `PreToolUse` hook calls `lsp-max-cli gate check` before every Edit/Write/Bash; `PostToolUse` takes diagnostic snapshot
- **`.anti-llm-ignore`** — custom ignore file
- **`.mesh_state.json`** — autonomic mesh state tracking
- **`receipts/`** directories — BLAKE3-signed provenance receipts
- **`generated/`** directory in `anti-llm-cheat-lsp` — spec-graph artifacts from `lsp-max-specgen`
- **`web/`** — Next.js dashboard for receipt/conformance visualization
- **Forbidden vocabulary** enforced in CI and hooks: `tower-lsp`, `tower_lsp`, "done", "fully admitted", "solved", "guaranteed" etc.

---

## Repo 2: `cargo-cicd`

### Identity

| Field | Value |
|---|---|
| Visibility | Public |
| Structure | Workspace (3 members: root, `crates/cargo-cicd-core`, `crates/cargo-cicd-lsp`) |
| Rust edition | 2021 (per-crate; no `[workspace.package]`) |
| MSRV | `1.86` in root; `1.85` in `cargo-cicd-core`; weekly audit tests MSRV `1.85` |
| Toolchain file | None |
| Version scheme | CalVer `YY.M.D` (e.g., `26.6.2`) |
| Resolver | `"2"` |
| File count | ~928 non-git files |

**Bins:** `cargo-cicd` (`src/main.rs`), `cicd-evidence-gen` (`src/bin/cicd-evidence-gen.rs`)

### Cargo Metadata Completeness

| Field | Set |
|---|---|
| `license` | Yes — `MIT OR Apache-2.0` (per-crate, not inherited) |
| `description` | Yes |
| `repository` | Yes — `https://github.com/seanchatmangpt/cargo-cicd` |
| `homepage` | Yes |
| `keywords` | Yes — `["cargo","ci","testing","workspace","cleanup"]` |
| `categories` | Yes — `["command-line-utilities","development-tools"]` |
| `authors` | No |
| `rust-version` | Yes |
| `readme` | Yes |
| `exclude` | Yes — extensive list (`/receipts`, `/ontology`, `/templates`, `/cicd.toml`, `/ggen.toml`, etc.) |
| `[workspace.package]` | No — fields not hoisted |
| `[workspace.dependencies]` | Yes — `anyhow`, `serde`, `serde_json`, `thiserror`, `tokio`, `toml`, `tracing`, `tracing-subscriber`, `walkdir`, `assert_cmd`, `predicates`, `tempfile`, internal crates, external git dep (`lsp-max-anti-cheat`) |
| `[workspace.lints]` | No |

### Feature Flags

Extensive feature system: `default` (empty), `process-data`, `autonomic` (→`process-data`), `autoarch` (→`autonomic`), `contrib` (→`process-data`), `wasm4pm` (→`process-data`), `anti-llm-cheat` (→`lsp-max-anti-cheat`), `lsp`, `affidavit` (→`process-data`), `advanced` (→ pulls in `ignore`, `rayon`, `blake3`, `tracing`, `tracing-subscriber`, `miette`, `thiserror`, `moka`, `bitcode`, `petgraph`, `jiff`, `hdrhistogram`, `aho-corasick`).

### Lints / Quality Config

| File | Present | Notes |
|---|---|---|
| `rustfmt.toml` | No | — |
| `clippy.toml` | No | — |
| `[lints]` | No | — |
| `deny.toml` | No | — |
| `typos.toml` | No | — |
| `.editorconfig` | No | — |

CI uses `cargo clippy -- -D warnings` (no `--workspace --all-targets --all-features`).

### CI (Detailed)

**4 workflow files:** `ci.yml`, `pr-checks.yml`, `release.yml`, `weekly-audit.yml`

**`ci.yml`** (trigger: push all branches, PR to `main`):

Concurrency cancel-in-progress on same ref.

| Job | Runner | Toolchain | Cache | Notes |
|---|---|---|---|---|
| `fmt` | ubuntu-latest | `dtolnay/rust-toolchain@1.94.1` + rustfmt | — | Pinned toolchain for reproducible fmt |
| `check-and-test` | ubuntu-latest | `dtolnay/rust-toolchain@1.94.1` + clippy | `Swatinem/rust-cache@v2` | Clippy + 6 feature-targeted test runs + 3 named test targets (`feature_projection`, `autonomic_policies`, `invariants --nocapture`, `ggen_customization_guard --nocapture`) |
| `evidence-gate` | ubuntu-latest | `dtolnay/rust-toolchain@stable` | `Swatinem/rust-cache@v2` | `continue-on-error: true`; checks `wpm` oracle availability; runs `wasm4pm_evidence_gate`, `wasm4pm_evidence_mutation`, `wasm4pm_refusal_cases`; audits `.xes` artifacts if `wpm` available |

Note: Toolchain **pinned** to `1.94.1` (not `@stable`) in fmt/check jobs with rationale: "a new stable Rust release adds lints that would otherwise turn this gate red with no code change."

**`pr-checks.yml`** (trigger: PR events):

| Job | What it does |
|---|---|
| `validate-title` | Enforces Conventional Commits: `<type>(<scope>): <desc>`, allowed types/scopes listed |
| `validate-description` | Requires ≥ 30 chars body; warns if all checkboxes unchecked |
| `forbidden-terms-pr` | Greps only changed `.rs` files + `README.md` for terms: `ALIVE|Nehemiah|CONSTRUCT8|Inspection Gate|Cargo Court|Truex|Field8|Instinct8` |
| `auto-label` | Maps commit type prefix to GitHub labels (enhancement, bug, documentation, etc.) via `actions/github-script@v7` |
| `request-review` | Comments on PR if sensitive paths touched (`src/engine/`, `src/adapters/`, `tests/invariants.rs`, `Cargo.toml`, `crates/cargo-cicd-core/`) |

**`release.yml`** (trigger: `v*.*.*` tags):

| Job | Notes |
|---|---|
| `validate-tag` | Parses semver, marks `is_prerelease` for `-rc`/`-beta` suffixes |
| `run-gates` | `cargo make test` + `cargo test --test invariants` + build with `autonomic,wasm4pm,contrib` + optional evidence gate; uses `actions/cache@v4` (manual key: `**/Cargo.lock`) |
| `build-release` | `cargo build --release`, renames to `cargo-cicd-linux-amd64` |
| `create-release` | Extracts CHANGELOG.md section, `gh release create` with artifact and pre-release flag |

**`weekly-audit.yml`** (trigger: cron Monday 08:00 UTC + manual):

| Job | Notes |
|---|---|
| `cargo-audit` | Installs `cargo-audit --locked`, runs audit, captures report, uploads artifact (90-day retention) |
| `cargo-outdated` | Installs `cargo-outdated --locked`, runs outdated scan |
| `msrv-check` | Builds on Rust `1.85` via `dtolnay/rust-toolchain@master` + `toolchain: "1.85"` |
| `open-issue` | Creates/updates GitHub issue labelled `weekly-audit` if any check fails; uses `actions/github-script@v7` |

**No dependabot.yml.**

**CODEOWNERS:** `* @seanchatmangpt` globally, plus explicit rules for `CLAUDE.md`, `.claude/`, `src/engine/`, `src/evidence.rs`, `tests/invariants.rs`, evidence gate tests, `ontology/`.

**PR template:** Checklist with Evidence Pattern Compliance section (event emission, forbidden terms, adapter statelessness, policy suggest-mode only), test plan referencing `cargo make test`.

### Task Runner

**Makefile.toml** (`cargo-make`, `default_to_workspace = false`):

Recipes: `build`, `build-release`, `check`, `test`, `test-verbose`, `test-features`, `lint`, `fmt`, `fmt-check`, `invariants`, `gate`, `ci` (aggregates fmt-check + lint + test + invariants), `release-check` (ci + test-features).

No Justfile, no bare Makefile.

### `src/` Layout

Fully binary-focused (`src/main.rs` + `src/lib.rs`). Module tree:

- `src/nouns/` — one file per CLI noun (`affidavit.rs`, `analyze.rs`, `autoarch.rs`, `evidence.rs`, `git.rs`, `lsp.rs`, `pipeline.rs`, `publish.rs`, `status.rs`, `target.rs`, `test.rs`, `trybuild.rs`, `ui.rs`, `workspace.rs`)
- `src/state/` — domain state projections (`changed.rs`, `event.rs`, `git_phase.rs`, `policy.rs`, `projection.rs`, `target.rs`, `test_plan.rs`, `toolchain.rs`, `workspace.rs`)
- `src/policies/` — autonomic policy modules (one per policy)
- `src/engine/`, `src/adapters/`, `src/autonomic/`, `src/advanced/`, `src/certification/`, `src/integrations/`, `src/ui/` — capability domains
- `src/ocel.rs`, `src/oracle_keys.rs`, `src/receipt_validation.rs`, `src/session.rs`, `src/evidence.rs`, `src/evidence_xes_v2.rs`, etc.

Tests: `tests/` with ~35 named `[[test]]` entries, fixture subdirs, `tests/cli/` subdir.
Examples: 3 numbered examples (`01_first_clean.rs`, `02_ocel_evidence.rs`, `03_max_pipeline.rs`).
Benches: Not present at root (none declared).
Templates: `templates/` with Tera templates for README, noun scaffold, CLI tests, docs.

### Docs Set

| Doc | Present |
|---|---|
| README.md | Yes — generated from `ggen.toml` + Tera template |
| CLAUDE.md | Yes — extensive (mission, forbidden terms, evidence patterns, policies, invariants, test strategy) |
| CONTRIBUTING.md | Yes |
| SECURITY.md | Yes |
| CHANGELOG.md | Yes |
| CODE_OF_CONDUCT.md | No |
| CODEOWNERS | Yes (`.github/CODEOWNERS`) |
| PR template | Yes (`.github/pull_request_template.md`) |
| `.claude/` docs | Extensive: `ARCHITECTURE.md`, `PATTERNS.md`, `PLUGINS.md`, `QUICKSTART.md`, `TESTING.md` |
| Thesis chapters | Yes — `thesis_chapter1-5.md`, `thesis.pdf`, `thesis.html` |

### Licensing

Dual `MIT OR Apache-2.0`. Both `LICENSE-MIT` and `LICENSE-APACHE` present.

### Key Dependencies

`clap 4` (derive), `clap-noun-verb 26.6.2`, `serde 1`, `serde_json 1`, `toml 0.8`, `anyhow 1`, `walkdir 2`, `tracing 0.1` (optional), `tracing-subscriber 0.3` (optional), `blake3 1` (optional/advanced), `rayon 1` (optional/advanced), `miette 7` (optional/advanced), `thiserror 2` (optional/advanced), `petgraph 0.6` (optional), `jiff 0.2` (optional), `moka 0.12` (optional), `bitcode 0.6` (optional), `hdrhistogram 7` (optional), `aho-corasick 1` (optional). Dev: `assert_cmd 2`, `predicates 3`, `tempfile 3`.

### House-Specific Tooling

- **`ggen.toml`** — ontology-driven code generation (SPARQL queries → Tera templates → output files); generates README.md, docs from TTL ontology
- **`ontology/`** — Turtle/RDF files (`cargo-cicd.ttl`, `cicd-process.ttl`, `public/`)
- **`.ggen/`** — ggen state cache (SHA256 manifests, signing keys, sync receipts in JSON)
- **`.claude/`** — extensive Claude Code configuration: `agents/`, `commands/`, `hooks/`, `plugins/`, `skills/`, `subagents/`, `mcp-servers.json`, `settings.json`
- **`.claude/hooks/`** — `session-start.sh`, `cargo-check.sh`, `pre-commit-check.sh`, `public-boundary-guard.sh`
- **`.claude/settings.json`** — model pinned to `claude-sonnet-4-6`, context files, env vars, extensive allow permissions for cargo/git/wpm
- **`.claude/skills/`** — JSON/YAML skill definitions (benchmark-pipeline, evidence-audit, invariant-audit, noun-scaffold, release-checklist, ui-component, etc.)
- **`.claude/subagents/`** — YAML subagent definitions (governance-auditor, performance-profiler, release-coordinator, test-coordinator)
- **`cicd.toml`** — workspace state carrier (excluded from `cargo publish`)
- **`templates/`** — Tera templates for scaffold generation
- **`tests/wasm4pm_evidence/`** — XES evidence test harness with cases/expected/fixtures
- **Forbidden vocabulary list** enforced in PR checks and CLAUDE.md (ALIVE, Nehemiah, CONSTRUCT8, Inspection Gate, Cargo Court, Truex, Field8, Instinct8, AGI, etc.)

---

## Conventions Observed (Both Repos)

1. **CalVer `YY.M.D`** as version scheme (not SemVer).
2. **Dual MIT/Apache-2.0 licensing** with both LICENSE files present.
3. **`dtolnay/rust-toolchain` action** (not `actions/rust-toolchain`) for toolchain installation.
4. **`Swatinem/rust-cache@v2`** as the standard caching action.
5. **`actions/checkout@v4`** throughout.
6. **Clippy with `-D warnings`** as the lint gate.
7. **`clap-noun-verb`** pattern for CLI command grammar (noun + verb, not subcommand trees).
8. **CLAUDE.md as mandatory first read** — both repos insist on it with enforcement mechanisms.
9. **`.claude/settings.json`** present with hooks and permissions.
10. **BLAKE3 content-addressing** for provenance/receipts.
11. **`wasm4pm` (`wpm` binary)** as external oracle for evidence adjudication; tests declare `Blocked` status when oracle is unavailable.
12. **Forbidden vocabulary enforcement** — both repos maintain lists of banned terms in public output/code.
13. **`tests/invariants.rs`** as a named, release-gating invariant test suite.
14. **Edition 2021** universally.
15. **`CARGO_TERM_COLOR=always` + `RUST_BACKTRACE=1`** as standard CI env.
16. **`cargo publish` exclusion list** for internal tooling artifacts (`cicd.toml`, `ggen.toml`, `ontology/`, `templates/`, etc.).

## Divergences / Inconsistencies

| Item | `lsp-max` | `cargo-cicd` |
|---|---|---|
| Workspace resolver | `"3"` | `"2"` |
| `[workspace.package]` | Yes — full field hoisting | No — fields set per-crate |
| Toolchain file | Yes — nightly pinned (`nightly-2026-04-15`) | None |
| Toolchain in CI | `@stable` (floating) | `@1.94.1` (pinned by version) |
| PR validation workflow | None | Yes — conventional commits + forbidden terms + auto-label + CODEOWNERS |
| Weekly audit workflow | None | Yes — cargo-audit + cargo-outdated + MSRV check + auto-issue |
| Task runner | `just` (Justfile) | `cargo-make` (Makefile.toml) |
| Dependabot | Yes — daily | None |
| `rustfmt.toml` | Yes | None |
| Sub-crate MSRV | 1.82.0 (all via workspace) | 1.86 root / 1.85 core (inconsistent, not hoisted) |
| `[workspace.lints]` | Neither — but lsp-max has more linting surface to benefit |
| `ontology/` + `ggen.toml` | None | Yes — full ontology-driven generation |
| Numbered examples | No | Yes (`01_`, `02_`, `03_`) |
| Authors field | Yes (workspace) | No |
| `concurrency:` cancel-in-progress | No | Yes in `ci.yml` |
| Release artifact upload | Checksums + `softprops/action-gh-release@v1` | `actions/upload-artifact@v4` + `gh release create` (CLI) |
