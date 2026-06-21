Now I have enough information to produce the comprehensive boilerplate fingerprint report.

---

# Affidavit — Boilerplate Fingerprint Report

## Identity

- **Type:** Single crate (not a Cargo workspace). One `Cargo.toml` at root.
- **Edition:** `2021`
- **Version scheme:** CalVer-flavored (`26.6.17` → year.month.patch)
- **MSRV:** `rust-version = "1.78"` in `Cargo.toml`
- **Toolchain:** `rust-toolchain.toml` pins `channel = "nightly"` with `components = ["rustfmt", "clippy"]` — nightly is unpinned intentionally to track latest

---

## Cargo.toml — Detail

### `[package]` metadata completeness

All metadata fields present and populated:

| Field | Value |
|---|---|
| `name` | `affidavit` |
| `version` | `26.6.17` |
| `edition` | `2021` |
| `rust-version` | `1.78` |
| `description` | Present (descriptive) |
| `license` | `MIT OR Apache-2.0` |
| `authors` | `Sean Chatman <xpointsh@gmail.com>` |
| `repository` | `https://github.com/anthropics/affidavit` |
| `documentation` | `https://docs.rs/affidavit` |
| `readme` | `README.md` |
| `keywords` | 9 keywords (exceeds crates.io 5-keyword limit — a lint issue) |
| `categories` | `security`, `development-tools`, `cryptography` |
| `homepage` | Missing |

### Multiple binaries

- `affi` (always compiled)
- `affi-shell` (requires `shell` feature)

### Bench entries

- `receipt_operations` (harness = false, Criterion)
- `quality_western_electric` (harness = false, requires `quality-monitor`)

### Feature flags

Organized in labeled phases:

| Feature | Purpose |
|---|---|
| `default` | pulls in `core` only |
| `core` | pulls `wasm4pm-compat` |
| `inspection` | `chicago-tdd-tools`, `glob`, `colored` |
| `discovery` | `wasm4pm`, `chrono`, `hex` |
| `conformance` | depends on `discovery` |
| `predictive` | depends on `conformance` |
| `lsp` | `lsp-max`, `tokio` |
| `file-watch` | `notify`, `shell`, `tokio` |
| `mutation` | `clnrm-core`, `tera`, `quickcheck`, `uuid`, `rand` |
| `fixture-db` | `chrono` |
| `otel` | `opentelemetry`, `opentelemetry-jaeger`, `tracing`, `tracing-subscriber` |
| `metrics` | extends `otel` with Prometheus |
| `shell` | `rustyline`, `tokio`, `shlex` |
| `ui` | `colored`, `indicatif` |
| `benchmarking` | `criterion`, `rayon` |
| `gpu` | `wgpu`, `bytemuck`, `pollster` |
| `remediation` | `syn`, `quote`, `proc-macro2`, `tempfile` |
| `quality-monitor` | `syn`, `quote`, `shell`, `tokio` |
| `webhook` | `reqwest`, `shell`, `tokio` |
| `all` | All of the above combined |
| `dev` | Alias for `all` |

### External crate versions (core mandatory)

| Crate | Version |
|---|---|
| `clap-noun-verb` | `26.6` (local ecosystem, CalVer) |
| `clap-noun-verb-macros` | `26.6` |
| `linkme` | `0.3` |
| `serde` | `1` (derive feature) |
| `serde_json` | `1` |
| `blake3` | `1` |
| `anyhow` | `1` |
| `thiserror` | `2` |
| `walkdir` | `2` |
| `regex` | `1` |
| `wasm-encoder` | `0.252.0` (oddly pinned, non-optional) |

### Local path-deps (all optional, ecosystem siblings)

- `clap-noun-verb`, `clap-noun-verb-macros` — `../clap-noun-verb`
- `wasm4pm`, `wasm4pm-compat` — `../wasm4pm/wasm4pm`, `../wasm4pm-compat`
- `lsp-max` — `../lsp-max`
- `clnrm-core` — `../clnrm/crates/clnrm-core`
- `chicago-tdd-tools` — `../chicago-tdd-tools` (dev-dep too)

### `[dev-dependencies]`

`assert_cmd`, `predicates`, `tempfile`, `trybuild`, `criterion` (html_reports), `quickcheck`, `quickcheck_macros`, `tokio`, `chicago-tdd-tools`

### `[profile.release]`

```toml
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

No `[profile.dev]` overrides. No `[lints]` table.

---

## Lints / Quality Config

- **`rustfmt.toml`:** Not present (formatting uses nightly defaults via `cargo +nightly fmt`)
- **`clippy.toml`:** Not present; no `[lints]` table in Cargo.toml
- **`deny.toml`:** Not present
- **`typos.toml`:** Not present
- **`.editorconfig`:** Not present

Lint enforcement is minimal — only `cargo clippy --all-targets -- -D warnings` in the non-blocking CI job.

---

## CI

### `.github/workflows/rust.yml`

- **Trigger:** `push` + `pull_request` (all branches)
- **Concurrency:** cancel-in-progress per ref
- **Permissions:** `contents: read`
- **Jobs:**
  - `fmt` — `ubuntu-latest`, installs stable + nightly, runs `cargo +nightly fmt --all -- --check`; marked `continue-on-error: true` (formatting is the only honest signal since path-deps are absent)
  - `build-and-test` — `ubuntu-latest`, nightly + clippy, attempts `cargo build`, `cargo test`, `cargo clippy`; all `continue-on-error: true` by design — expected to fail in CI without sibling repos
- **No caching** (no `Swatinem/rust-cache` or similar)
- **No matrix** (single OS/toolchain)
- **No release workflow**

### `.github/workflows/web.yml`

- **Trigger:** `push`/`pull_request` scoped to `web/**` paths
- **Job:** `web` — `ubuntu-latest`, Node 22 (`actions/setup-node@v4`), `npm ci` cache via `web/package-lock.json`
- **Steps:** `npx tsc --noEmit` (typecheck) + `next build`
- **Timeout:** 15 minutes
- **Failure surfacing:** Build log tail appended to `$GITHUB_STEP_SUMMARY`
- **No lint step** (no ESLint config in repo)

### `.github/dependabot.yml`

Three ecosystems, all weekly: `cargo` (root), `npm` (`/web`), `github-actions` (root). PRs limited to 10 per ecosystem. Commit message prefixes: `cargo`, `npm`, `ci`.

### `.github/pull_request_template.md`

Two-section checklist: Web gates (real CI signal) + Rust notes (honest disclaimer about sibling path-dep requirement). Ethos note: unchecked boxes more useful than falsely checked ones.

---

## Task Runner

### `justfile`

Recipes:

| Recipe | Description |
|---|---|
| `default` | `just --list` |
| `web-dev` | `cd web && npm run dev` |
| `web-build` | `cd web && npm ci && npm run build` |
| `web-check` | `cd web && npx tsc --noEmit` |
| `fmt` | `cargo fmt --all` |
| `fmt-check` | `cargo fmt --all -- --check` |
| `rust-build` | `cargo build` (requires siblings) |
| `rust-test` | `cargo test` (requires siblings) |
| `golden` | `bash examples/golden_run.sh` (requires siblings) |

No `Makefile.toml`. `scripts/` contains: `bootstrap.sh`, `check.sh`, `golden.sh`, `web-dev.sh`. `tools/` contains: `dx-report.mjs`.

---

## Docs Set

### README.md headings

- Doctrine: Certify, Don't Decide
- The 1000x Initiative
- Installation & Quick Start
- Core Concepts (Receipt, 7-Stage Pipeline)
- CLI Surface (59 verbs listed)

### CLAUDE.md sections

Overview, Architecture, Key Concepts, CLI Surface, Development Workflow, Integration Points, Code Conventions, Common Tasks, Documentation Ecosystem, Testing Strategy, Performance Characteristics, Troubleshooting, Roadmap & Future Work, License, References

### CONTRIBUTING.md

Present. Headings: Doctrine in brief (4 principles), Admission criterion, Building & testing, How to submit a PR.

### CHANGELOG.md

Present. Keep-a-Changelog-adjacent format (`## [version] — date`) with `### Added`, `### Changed`, `### Fixed` subsections. CalVer versions.

### IMPLEMENTATION_SUMMARY.md, STATUS.md

Present at root — project-state tracking docs.

### SECURITY.md

Not present.

### LICENSE-MIT, LICENSE-APACHE

Both present — dual license confirmed.

---

## src Layout & Test Strategy

### Module organization

Main modules: `lib.rs`, `cli.rs`, `chain.rs`, `verifier.rs`, `types.rs`, `admission.rs`, `discovery.rs`, `ocel.rs`, `handlers.rs`, `tracing.rs`, `lsp/` (subdirectory with `mod.rs`, `hover.rs`, `goto_definition.rs`, `diagnostics.rs`), plus heavy expansion: `bench.rs`, `catalog.rs`, `diff.rs`, `error.rs`, `fixture_db.rs`, `generation.rs`, `metrics.rs`, `mining.rs`, `mutation.rs`, `quality.rs`, `sbom*.rs` (6 files), `visualize.rs`, `predict_maximalist.rs`, and 16 `1000x_*.rs` files.

### `src/verbs/` — 69 files total

Core verbs per CLAUDE.md: `emit.rs`, `assemble.rs`, `verify.rs`, `show.rs`, `inspect.rs`, `diagnose.rs`, `stats.rs`, `graph.rs`, `replay.rs`, `model.rs`, `conformance.rs`. Plus 50+ generated/expanded verbs (e.g., `gdpr_proof.rs`, `sbom_attest.rs`, `security_debt.rs`). **18 `.rs.backup` files** also present.

### `src/bin/`

`affi.rs` (main entrypoint), `affi-shell.rs` (REPL, feature-gated)

### `tests/` — ~115 files

Includes core integration (`e2e.rs`, `cli_dispatch.rs`), property-based (`property_based.rs`), trybuild UI tests (`tests/ui/compile_fail/`, `tests/ui/compile_pass/`), and 90+ `reference_*.rs` files covering OCEL, BPMN, POWL, Petri nets, quality rules, SBOM, etc.

### `examples/` — 20 files

Core: `golden_run.sh`, `chain_build.rs`, `full_pipeline.rs`, `admission_gate.rs`, `verify_stages.rs`, `ocel_events.rs`, `verdict_diagnostics.rs`, `observable_spans.rs`, `receipt_determinism.rs`, `discover_shapeb.rs`. Also: `ci_github_actions.yml`, `ci_gitlab_ci.yml`, `git_hook_monitor.sh`, `webhook_slack_integration.rs`, and others.

### `benches/` — `receipt_operations/`

Files: `receipt_operations.rs`, `throughput.rs`, `variance.rs`, `profile.rs`, `quality_western_electric.rs`. Supporting: `FLAMEGRAPH_GUIDE.md`, `TIMING_TABLE.md`, shell scripts (`run_quality_benchmarks.sh`, `gen_summary.sh`, `check_regression.sh`).

### `completions/`

Shell completions: `affi.bash`, `affi.fish`, `affi.zsh`

---

## House-Specific Tooling

### `ggen.toml` + ontology-driven code generation

- `ggen.toml` declares `affidavit` as a consumer of the `clap-noun-verb` pack
- Ontology source: `ontology/affi-cli.ttl` (Turtle RDF) + `ontology/registry/` for additional TTL files
- SPARQL inference rule to normalize noun→verb back-links
- Two generation rules: `verb-wrappers` (fans out one `src/verbs/{verb_name}.rs` per verb from the ontology) and `verbs-mod` (generates `src/verbs/mod.rs`)
- Four SPARQL-ASK validation rules enforced at generation time
- No `.ggen/` directory at root (path not present)

### `semconv/`

Files: `affidavit.yaml`, `manifest.yaml`, plus `registry/` and `registry_broken/` subdirectories — OpenTelemetry semantic conventions registry

### `ontology/`

`affi-cli.ttl` (main CLI ontology), `1000x_w3c_provo_spec.ttl`, `registry/` and `registry_broken/` subdirectories

### `web/` — Next.js 15 / React 19 / Node 22

Self-contained, no Rust dependency. Structure: `app/` (routes: anatomy, api, capabilities, coverage, diff, globals, learn, observe), `lib/` (affidavit.ts, blake3.ts, verify-client.ts). TypeScript strict, no ESLint config.

### `thesis/`

Full LaTeX thesis: `main.tex`, `frontmatter.tex`, `chapters/`, `appendices/`, `bibliography.bib`, generated auxiliary files (`main.aux`, `main.pdf`, etc.)

### `reference/`

`COVERAGE.md`, `sbom/`, `cache/`, `ocel/`, `receipts/`, `sync-state.json` — runtime data, cache, and fixture storage (not source)

### `fixtures/` 

`cache/`, `ocel/`, `receipts/`, `sync-state.json` — test fixture storage

---

## What's Clean vs. What's Cruft

### Clean (boilerplate-worthy)

- `Cargo.toml` metadata completeness (all standard fields populated)
- `[profile.release]` settings (LTO, 1 codegen unit, panic=abort, strip)
- Dual license (`LICENSE-MIT` + `LICENSE-APACHE`)
- `rust-toolchain.toml` (explicit toolchain pinning)
- `CONTRIBUTING.md` with doctrine-first framing and honest admission criterion
- `CHANGELOG.md` in Keep-a-Changelog format with CalVer
- `.gitignore` (well-structured: target, keys, .claude, OS cruft, secrets, LaTeX artifacts)
- `justfile` (clean recipe set, honest disclaimers about sibling deps)
- `.github/dependabot.yml` (three ecosystems, weekly, commit prefixes)
- `tests/ui/` with `trybuild` compile-fail tests (sealing invariant `E0451` enforced)
- Shell completions in `completions/` (`bash`, `fish`, `zsh`)
- `examples/golden_run.sh` as runnable E2E smoke test
- `ggen.toml` as ontology-driven code generation config (novel but structured)
- `semconv/` OTEL semantic conventions registry
- `.github/pull_request_template.md` with honest verification checklist
- CI `concurrency` + `cancel-in-progress` on all workflows
- `permissions: contents: read` on all workflows

### Cruft (not boilerplate-worthy)

| File / Dir | Reason |
|---|---|
| `gen_thesis.py`, `generate_bib.py`, `generate_conclusion.py`, `generate_verbs.py`, `remediate_licenses.py` | One-off Python generators for thesis/content; not part of crate scaffolding |
| `audit_instructions.txt` | Ad-hoc Claude session instruction file left in repo root |
| `DX_QOL_EXECUTIVE_SUMMARY.txt` | Agent session artifact, not project documentation |
| `portfolio_test_dataset.json` | Large dataset at root (312-repo test data); should live in `fixtures/` |
| `IMPLEMENTATION_SUMMARY.md`, `STATUS.md` | Agent session status tracking; project-specific, not boilerplate |
| `src/1000x_*.rs` (16 files) | "1000x Initiative" experimental modules with `1000x_` prefix names; violate Rust module naming conventions |
| `src/verbs/*.rs.backup` (18 files) | Generated file backups; should be gitignored or deleted |
| `src/handlers_stubs.rs`, `src/assemble.rs`, `src/assemble.rs.backup` | Stubs and backups at src root |
| `thesis/` with `.aux`, `.pdf`, `.blg`, etc. | Compiled LaTeX artifacts; `.gitignore` partially excludes `thesis/chapters/*.tex` but not all aux files |
| `tests/reference_*.rs` (~90 files) | Extremely broad reference test suite; useful but overwhelming for a boilerplate |
| `WEBHOOK_INTEGRATION.md` in `examples/` | Misplaced doc in examples |
| `src/verbs/mod.rs.backup`, `src/verbs/*.backup` | Stale backups |

---

## Conventions Observed

1. CalVer version scheme (`YY.M.patch`) applied to both `Cargo.toml` and local ecosystem crates
2. Noun-verb CLI pattern driven from RDF/SPARQL ontology via `ggen.toml`
3. "Certify, don't decide" doctrine enforced at the type system level (private `_seal` field, `E0451`)
4. Honest CI: gates only what can genuinely pass; non-blocking jobs explicitly annotated with `continue-on-error: true` and explanatory commentary
5. `trybuild` compile-fail tests for type-level invariants
6. Admission criterion: "a green that is true whether or not the work happened carries no information"
7. Cargo.lock committed (binary crate convention)
8. No `[lints]` table despite nightly toolchain — missed opportunity
9. Feature flags organized in named phases with inline comments
10. `all` and `dev` mega-features for combinatorial maximalism
11. Shell completions ship alongside the binary
12. Benches use Criterion with `harness = false`
13. Dual MIT/Apache-2.0 license on every file

---

## Boilerplate-Worthy Elements

These are the scaffolding pieces from this repo worth standardizing across the monorepo:

1. **`Cargo.toml` template** — `[package]` completeness (all fields), `[profile.release]` settings (lto/codegen-units/panic=abort/strip), dual license string
2. **`rust-toolchain.toml`** — nightly channel, `["rustfmt", "clippy"]` components
3. **`justfile` structure** — `default: just --list`, clearly grouped `fmt`/`fmt-check` + ecosystem-specific recipes, honest disclaimers in comments
4. **`.github/dependabot.yml` pattern** — three-ecosystem weekly updates (cargo, npm, github-actions) with commit-message prefixes
5. **CI concurrency + cancel-in-progress** per workflow, `permissions: contents: read`
6. **Honest CI comment block** explaining non-blocking jobs when build requirements can't be met in isolation
7. **`.github/pull_request_template.md`** — "what I verified vs. did not" framing
8. **`completions/`** — `affi.bash`, `affi.fish`, `affi.zsh` shell completion pattern
9. **`examples/golden_run.sh`** — full lifecycle smoke test as first-class artifact
10. **`tests/ui/compile_fail/`** — trybuild tests for type-system invariants
11. **`ggen.toml`** — ontology-driven generation config (for repos in the `clap-noun-verb` ecosystem)
12. **`semconv/`** — OTEL semantic conventions registry with `manifest.yaml`
13. **`.gitignore` entries** — `/.ggen/keys/`, `/.claude/`, secrets patterns, Cargo `target/`, LaTeX artifacts
14. **Feature flag phase organization** — labeled `# Phase N:` sections with `all` and `dev` aliases
15. **`[dev-dependencies]`** set — `assert_cmd`, `predicates`, `trybuild`, `criterion`
