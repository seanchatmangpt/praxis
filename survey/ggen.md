---

# Boilerplate Fingerprint Report: `ggen`

**Repo:** `seanchatmangpt/ggen` | **Status:** PUBLIC | **Type:** Workspace (root crate + 15+ workspace members)

---

## 1. Identity

| Field | Value |
|---|---|
| Repo name | `ggen` |
| Visibility | Public |
| Type | Cargo workspace (root crate = `ggen`, 15 active workspace members + several dormant) |
| Rust edition | 2021 (workspace-inherited) |
| MSRV / toolchain | `nightly-2026-04-15` (rust-toolchain.toml) |
| Version scheme | CalVer: `YY.M.DD` (e.g. `26.6.11`) |
| Rough size | 736 files at depth ≤2; sprawling monorepo with docs, scripts, data, Docker files |
| Resolver | 2 |

**Active workspace members:**
`ggen-a2a-mcp`, `ggen-config`, `ggen-marketplace`, `ggen-core`, `ggen-cli`, `ggen-graph`, `ggen-lsp`, `ggen-lsp-mcp`, `ggen-lsp-a2a`, `genesis-types-v2`, `genesis-schema-v2`, `genesis-core-v2`, `genesis-core`, `cpmp`, `stpnt`

---

## 2. Cargo Metadata Completeness

| Field | Root `[package]` | Members |
|---|---|---|
| `version` | `workspace = true` | `workspace = true` |
| `authors` | `workspace = true` | `workspace = true` |
| `edition` | `workspace = true` | `workspace = true` |
| `license` | MIT only (not dual) | `workspace = true` |
| `description` | Set | Set (per member) |
| `repository` | Set | `workspace = true` |
| `homepage` | Set (=repository) | Not observed |
| `keywords` | Set | Set per member |
| `categories` | Set | Set per member |
| `rust-version` | NOT set | NOT set |
| `readme` | `README.md` | `README.md` per member |

**Workspace inheritance patterns used:**
- `[workspace.package]`: version, authors, edition, repository, license
- `[workspace.dependencies]`: extensive — all major deps centralized here
- `[workspace.lints]`: Yes — `[workspace.lints.rust]` + `[workspace.lints.clippy]` (verbose allow-heavy config)

**Note:** `rust-version` is not set in `Cargo.toml`; MSRV enforcement is only via `rust-toolchain.toml` (nightly pinned).

---

## 3. Lints / Quality Config

### `rustfmt.toml`
- Present. Edition 2021, `fn_params_layout = "Compressed"`, most nightly-only options commented out but listed. Conservative stable settings only.

### `[workspace.lints]`
- `warnings = "allow"` (warn-first inventory mode)
- `unsafe_code = "warn"`, `missing_docs = "allow"`, `dead_code = "allow"`
- `[workspace.lints.clippy]`: all, pedantic, nursery, cargo at `"warn"`, then ~150+ individual pedantic/nursery lints flipped back to `"allow"` to suppress noise across 30+ crates
- `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented` = `"warn"` (not deny)
- `multiple_crate_versions = "allow"` (explicitly allowed due to complex dep tree)

### `deny.toml`
- Present (cargo-deny). Mostly defaults/template. `bans.multiple-versions = "warn"`, `bans.wildcards = "allow"`. Advisories ignore-list is empty. Sources: crates.io only; git-only sources warned. Not strictly configured.

### `typos.toml` / `.editorconfig`
- NOT observed.

### `clippy.toml`
- NOT a separate file; configured entirely via `[workspace.lints.clippy]`.

---

## 4. CI

**45 workflow files** in `.github/workflows/`. Key ones:

| Workflow | Jobs | Runner | Toolchain |
|---|---|---|---|
| `ci.yml` | file-organization, comprehensive-test (matrix: stable/beta), fmt, clippy, doctests | ubuntu-latest | stable + beta matrix |
| `build.yml` | build + test | ubuntu-latest | stable / beta / nightly matrix; nightly `continue-on-error` |
| `lint.yml` | fmt-check, clippy | ubuntu-latest | stable |
| `test.yml` | unit, integration, BDD, doctests, examples | ubuntu-latest | stable / beta matrix |
| `release.yml` | build-release | macos-latest + ubuntu-latest | stable; targets: x86-darwin, arm64-darwin, x86-linux, arm64-linux |
| `security-audit.yml` | cargo-lock-verify, RUSTSEC audit (scheduled daily at 02:00 UTC) | ubuntu-latest | stable |
| `security.yml` | security-tests (with Redis service container) | ubuntu-latest | stable |
| `dependabot.yml` | Cargo + GitHub Actions; weekly Monday; groups minor+patch; labels "dependencies" | — | — |
| `docs-validation.yml` / `validate-docs.yml` | doc checks | ubuntu-latest | stable |
| `performance.yml` / `performance_benchmarks.yml` | benchmark suite | ubuntu-latest | stable |
| `marketplace.yml` / `marketplace-test.yml` | marketplace-specific gates | ubuntu-latest | stable |
| `deploy-docs.yml` / `marketplace-deploy.yml` | docs + marketplace deploy | ubuntu-latest | stable |
| `release-debian.yml` | `cargo deb` packaging | ubuntu-latest | stable |
| `homebrew-release.yml` | Homebrew tap formula update | ubuntu-latest | stable |
| Various others | erlang, gitops/flux, helm, weaver, gvisor, andon monitoring, secrets-sync, semantic-release | mixed | mixed |

**Caching strategy:** Custom composite action `.github/actions/setup-rust-cached` — caches `~/.cargo/registry/index`, `~/.cargo/registry/cache`, `~/.cargo/git/db`, and `target/`; key on `**/Cargo.lock` hash with `runner.os + toolchain + suffix`.

**Notable actions:**
- `dtolnay/rust-toolchain` (primary), `actions-rust-lang/setup-rust-toolchain` (secondary in some workflows)
- `actions/cache@v4`
- `actions/upload-artifact@v4`
- `softprops/action-gh-release@v2`
- Custom composite actions: `setup-rust-cached`, `install-cargo-tools`, `cargo-security-audit`, `extract-semantic-version`, `pr-comment-upsert`

**Concurrency:** Most workflows use `cancel-in-progress: true` with `group: ${{ github.workflow }}-${{ github.ref }}`; release workflow uses `cancel-in-progress: false`.

---

## 5. Task Runner

### `justfile` (primary)
Recipes: `timeout-check`, `check`, `build`, `build-release`, `clean`, `fmt`, `fmt-check`, `lint`, `test`, `test-lib`, `test-doc`, `test-bdd`, `test-marketplace`, `test-marketplace-full`, `test-mutation`, `pre-commit`

### `Makefile.toml` (cargo-make, secondary/historical)
Recipes include: `timeout-check`, `check`, `check-pre-push`, `build`, `do-build-release`, `build-release`, `verify-binary`, and many more. Described in the justfile as "historical reference only."

### `scripts/` directory
Large collection (~20+ files): `acp.sh`, `act-release.sh`, `ai-demo.sh`, `ci-health-check.sh`, `build-debian-package.sh`, `build-ggen-oci.sh`, `andon_monitor.sh`, Python scripts (`fix.py`, `fix_clippy.py`, `write_models.py`, etc.)

---

## 6. Docs Set

| File | Present |
|---|---|
| `README.md` | Yes |
| `CLAUDE.md` | Yes (extensive agent coordination rules, tool restrictions, architecture map) |
| `AGENTS.md` | Yes |
| `GEMINI.md` | Yes |
| `CONTRIBUTING.md` | Yes |
| `SECURITY.md` | Yes |
| `CHANGELOG.md` | Yes |
| `SKILLS.md` | Yes |
| `MANIFESTO.md` | Yes |
| `CODE_OF_CONDUCT` | Not observed |
| Many `*_REPORT.md`, `*_SUMMARY.md` etc. | Yes (~20+ ephemeral audit/planning docs in root) |

**README.md key sections:** Quick Start, Key Features, OpenTelemetry Tracing, Workspace Audit Dashboard, Architecture, Five-Stage Pipeline, Workspace Organization, Chicago TDD, `ggen init`, `ggen wizard`, `ggen sync`

**CLAUDE.md key sections:** Rules, Architecture Reference, Crate Map, Dormant crates, Cross-Cutting Patterns, Evidence-First Principle, Forbidden/Required Workflow, Tool Restrictions, Agent Coordination Rules, Commands, Verification & Strict Mode, Diagnostic Codes, Cryptographic Receipts, Git Hooks & Build Gotchas, ggen-lsp Intel Log, Remotes, OpenTelemetry Validation

---

## 7. Licensing

- **MIT only** (single `LICENSE` file; `[workspace.package] license = "MIT"`)
- No dual MIT/Apache-2.0

---

## 8. Source Layout

```
src/
├── lib.rs         # root crate lib
├── rdf.rs
└── scanner.rs

crates/            # 15+ workspace members
├── ggen-core/src/lib.rs
├── ggen-cli/src/lib.rs + main.rs (bin = "ggen")
├── genesis-*/
└── ...

tests/             # integration, BDD, contract, proof, archive dirs
examples/          # _archive/, _shared_templates/, 7-agent-validation (excluded), numerous demo dirs
benches/           # multiple bench targets
```

- Root crate exposes a lib; CLI binary lives in `ggen-cli` member as `src/main.rs`
- Integration tests gated on `required-features = ["integration"]` flag (many)
- `tests/proof/` and `tests/contract/` run without feature flags (release-gate tests)

---

## 9. Common Dependencies

| Crate | Version |
|---|---|
| `tokio` | 1.47 (features = full) |
| `serde` | 1.0 (derive) |
| `serde_json` | 1.0 |
| `serde_yaml` | 0.9 |
| `anyhow` | 1.0 |
| `thiserror` | 2.0 |
| `clap` | 4.5 (derive) |
| `tracing` | 0.1 |
| `tracing-subscriber` | 0.3 (env-filter, json, ansi) |
| `blake3` | 1.5 |
| `opentelemetry` | 0.31 |
| `opentelemetry-otlp` | 0.31 |
| `opentelemetry_sdk` | 0.31 |
| `tracing-opentelemetry` | 0.32 |
| `oxigraph` | 0.5.8 (rdf-12) |
| `tera` | 1.20 |
| `uuid` | 1.18 |
| `reqwest` | 0.12 (rustls-tls) |
| `async-trait` | 0.1 |
| `criterion` | 0.5 (html_reports) |
| `proptest` | 1.8 |
| `sqlx` | 0.8 (sqlite, tokio-rustls) |
| `genai` | 0.5 |
| `ed25519-dalek` | 2.1 |

**Custom/project-family crates:** `clap-noun-verb`, `clap-noun-verb-macros`, `lsp-max`, `lsp-max-protocol`, `lsp-max-macros`, `lsp-max-client`, `wasm4pm-compat`, `bcinr`, `chicago-tdd-tools` (all on CalVer scheme)

---

## 10. House-Specific Tooling

- **`ggen.toml`** — project-level config file defining ontology source, generation rules, inference rules, sync settings
- **`ggen.lock`** — lockfile for ggen's own dependency resolution
- **`semconv/`** — semantic convention definitions (live-check, model, policies)
- **`ontologies/`** — RDF/OWL ontology files organized by domain (cached, catalog.ttl, core, project, shapes, sparql)
- **`ontology_catalogue/`** — catalogue of external ontologies (A2A, affidavit, autotel, etc.)
- **`ggen-skills/`** — skill definitions (ggen-audit, ggen-governance, ggen-sync) as `.skill` files
- **`registry/index.json`** — local marketplace/plugin registry
- **`ggen.toml`** — standard project config format for ggen-initialized projects
- **`task_manifest.toml`** — task manifest
- **`audit.json`** — audit artifact
- **`EVIDENCE_SYNTHESIS.md`**, `PATH_A_EVIDENCE_INDEX.md` — evidence-chain documents
- **`data.rdf`**, `*.ttl`, `data.ttl` etc. — ontology data files (committed to root — violation of ci.yml file-org checks)
- **`Dockerfile`**, `Dockerfile.binary`, `Dockerfile.gvisor`, `docker-compose.*.yml` — container support
- **`.ggenrc.yaml`** — in `.gitignore` (user-local override)
- Multiple **Python fix scripts** (`fix.py`, `fix_clippy*.py`, `write_models.py`, etc.) in root
- **`CONVO.txt`**, `FUSION_THESIS.md`, `MANIFESTO.md`, `phd-thesis/`, `fusion-thesis/` — conceptual/research artifacts

---

## Conventions Observed

1. **CalVer versioning** (`YY.M.DD`) applied uniformly via `[workspace.package]` inheritance
2. **`[workspace.dependencies]`** for all major external crates — strict version centralization
3. **`[workspace.lints]`** with a "warn-first inventory" philosophy before enforcing deny
4. **`dtolnay/rust-toolchain` + `actions/cache@v4`** via custom composite action `setup-rust-cached`
5. **Dual task runners**: `justfile` (primary) + `Makefile.toml` (cargo-make, legacy); `just pre-commit` is the gate
6. **Concurrency groups** with `cancel-in-progress: true` on all PR/push workflows
7. **Multi-target release builds**: macOS (x86 + arm64) + Linux (x86 + arm64) via matrix
8. **Debian packaging** via `cargo-deb` with `[package.metadata.deb]`
9. **Integration tests** gated behind `required-features = ["integration"]`
10. **Nightly toolchain** pinned in `rust-toolchain.toml`; CI still tests stable/beta/nightly triad
11. **`clap-noun-verb` pattern** (custom internal crate) for CLI subcommand structure
12. **`[patch.crates-io]`** for redirecting monorepo siblings (`lsp-max`, `wasm4pm`, etc.) to local paths
13. **`ggen.toml`** as project-init config format (generated by `ggen init`)
14. **Evidence-first / receipt pattern** (BLAKE3 cryptographic receipts for audit trails)
15. **Dependabot** configured for both GitHub Actions and Cargo (weekly, Monday; groups minor+patch)

---

## Divergences / Inconsistencies

1. **Root dir is cluttered**: `ci.yml` enforces file-org checks forbidding `*.ttl`, `*.rdf`, `*.sh` in root — yet `data.rdf`, `data.ttl`, `data1.ttl`, `data2.ttl`, `large.turtle`, `entities.ttl`, etc. exist there, along with multiple `.sh` and `.py` scripts.
2. **Dual task-runner confusion**: Both `justfile` and `Makefile.toml` coexist; the `justfile` describes `Makefile.toml` as "historical reference" but CI workflows invoke `cargo-make` directly.
3. **Mixed actions**: Some workflows use `dtolnay/rust-toolchain`, others use `actions-rust-lang/setup-rust-toolchain` — no single canonical action.
4. **No `rust-version` in Cargo.toml**: MSRV is only enforced via `rust-toolchain.toml` pinning nightly; no `rust-version` field in `[workspace.package]` means `cargo msrv` tooling is unsupported.
5. **MIT-only** (not dual MIT/Apache-2.0 as seen in the `affidavit` sibling project).
6. **`typos.toml` and `.editorconfig` absent** — no typo checking or editor whitespace enforcement.
7. **`dev-dependencies` in root partially duplicated** from `workspace.dependencies` (e.g. `serde_json`, `tempfile` declared both ways).
8. **Lints are almost entirely `"allow"`** after setting all groups to warn — the practical effect of the lint config is minimal enforcement; described as "Phase B.1 warn-first" but no evidence Phase B.2 (flip to deny) has happened.
9. **`warnings = "allow"`** at workspace level globally suppresses compiler warnings — atypical and potentially masking real issues.
10. **45 workflow files** — many appear experimental, overlapping, or for one-off purposes (erlang, gvisor, andon, secrets-sync, semantic-release, weaver) suggesting accumulated workflow debt rather than a clean intentional CI surface.
