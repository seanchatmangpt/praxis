---

## Boilerplate Fingerprint Survey — `seanchatmangpt` Rust Repos

Assigned: **a2a-rs**, **swarmsh-v2**

---

## Repo 1: `a2a-rs`

### Identity
- **Visibility:** Public
- **Type:** Workspace (10 members: `a2a-rs`, `a2a-agents`, `a2a-client`, `a2a-ap2`, `a2a-mcp`, `ggen-optimizer`, `osiris-compiler`, `osiris-macos`, `osiris-edge`, `osiris-marketplace`)
- **Resolver:** `2` (workspace root)
- **Rust edition:** 2024 (most members); `osiris-compiler` is 2021
- **MSRV:** 1.85 (rust-version in most members; no rust-toolchain.toml found)
- **Repo size:** ~550 KB root listing; extensive `.md` documentation files (~15 summary docs at root)

### Cargo Metadata Completeness
- `a2a-rs` (core): `name`, `version` (0.1.0), `edition`, `rust-version`, `authors`, `description`, `license` (MIT), `repository`, `readme`, `keywords`, `categories` — **fully populated**
- Most members inherit the same author (`Emil Lindfors <emil@lindfors.no>`) and `license = "MIT"`
- `a2a-mcp`: `license = "MIT OR Apache-2.0"` (dual; diverges from others)
- `ggen-optimizer`: `license = "MIT OR Apache-2.0"`, version `0.3.0` (ahead of sibling 0.1.0)
- `a2a-ap2`, `osiris-marketplace`: sparse metadata (no description, no license in some)
- **No `[workspace.package]`, `[workspace.dependencies]`, or `[workspace.lints]` blocks** — all dependencies are copy-pasted per member
- Version scheme: 0.1.0 for most, 0.3.0 for `ggen-optimizer`

### Lints / Quality Config
- No `rustfmt.toml`, `clippy.toml`, `deny.toml`, `typos.toml`, or `.editorconfig` at root or in members
- CI enforces `-D warnings` via clippy job; no in-manifest `[lints]` table
- `.vscode/` directory present (no further inspection needed)

### CI (`rust.yml`)
Single workflow file, 4 jobs, all `ubuntu-latest`, all `toolchain: stable`:
| Job | Steps |
|---|---|
| `Build and Test` | build --verbose, test --verbose |
| `Clippy` | clippy -- -D warnings (via `actions-rs/clippy-check@v1`) |
| `Format` | fmt --all -- --check |
| `Doc Check` | doc --no-deps --all-features |

- Trigger: push/PR to `master`
- Caching: `actions/cache@v3` keyed on `Cargo.lock` hash (cargo registry + git + target)
- Actions: deprecated `actions-rs/toolchain@v1`, `actions-rs/cargo@v1`, `actions-rs/clippy-check@v1`
- No build matrix, no audit job, no coverage, no release automation

### Task Runner
- No `justfile`, no `Makefile.toml`, no `Makefile` — build commands documented only in CLAUDE.md

### Docs Set
| File | Present |
|---|---|
| README.md | Yes (workspace root + emoji-heavy, section headings per crate) |
| CLAUDE.md | Yes (root; comprehensive dev guide with ggen workflow, hooks, CI, architecture) |
| CONTRIBUTING.md | No |
| SECURITY.md | No |
| CHANGELOG.md | Yes (in `a2a-rs/` member and `spec/`; Keep a Changelog format) |
| CODE_OF_CONDUCT | No |

- Multiple session-summary `.md` files at root (CLIENT_ENHANCEMENT_SUMMARY, IMPLEMENTATION_COMPLETE, OSIRIS_IMPLEMENTATION_SUMMARY, etc.) — typical of AI-assisted development.

### Licensing
- `license = "MIT"` on most crates; `a2a-mcp` and `ggen-optimizer` use `"MIT OR Apache-2.0"`
- No LICENSE file found at workspace root (license declared in Cargo.toml metadata only)

### Src Layout
- `a2a-rs/src/`: hexagonal architecture — `domain/`, `port/`, `adapter/`, `application/`, `services/`, `observability/`, `lib.rs`
- `tests/`: workspace-level integration tests with `unit/` and `integration/` subdirs
- `examples/`: 2 examples at workspace root (http server demo, push notification demo)
- `spec/`: JSON schema files for A2A Protocol v0.3.0
- `ggen/`: code generation tooling with `ontology/` (RDF/Turtle `.ttl` files), `templates/` (Tera), `ggen.toml`

### Common Dependencies
| Crate | Version | Notes |
|---|---|---|
| serde + serde_json | 1.0 | all members |
| thiserror | 1.0 (core) / 2.0 (a2a-mcp, ggen-optimizer, osiris-marketplace) | inconsistent major |
| anyhow | 1.0 | a2a-client |
| tokio | 1.32–1.43 | varies by member |
| axum | 0.7 (a2a-client) / 0.8 (a2a-mcp, osiris-edge) | inconsistent |
| clap | 4.4 | a2a-agents |
| tracing + tracing-subscriber | 0.1 / 0.3 | most members |
| uuid | 1.4–1.11 | inconsistent |
| bon | 2.3 | builder pattern; multiple members |
| async-trait | 0.1 | most members |
| proptest | 1.4 | ggen-optimizer dev-deps |
| chrono | 0.4 | several members |
| petgraph | 0.6 | a2a-rs core, ggen-optimizer |

### House-Specific Tooling
- `ggen/` + `ggen.toml` + `ggen/ontology/*.ttl` — ontology-driven Rust code generation via SPARQL CONSTRUCT queries + Tera templates; generates into `a2a-rs/src/generated/`
- `.claude/` directory with `settings.json` (detailed permissions: allow cargo/git/gh/jq/ggen cmds; deny cargo publish, credential reads), hooks (`session-init.sh`, `validate-bash.sh`, `enforce-layers.sh`, `post-edit.sh`), `agents/` (rust-implementer subagent), `skills/`, `rules/`, `agent-memory/`
- `osiris-*` crates (compiler, edge, macos, marketplace) — project-specific product tier

---

## Repo 2: `swarmsh-v2`

### Identity
- **Visibility:** Public
- **Type:** Single crate (`swarmsh-v2`, no workspace)
- **Rust edition:** 2021
- **MSRV:** Not set (no `rust-version`, no `rust-toolchain.toml`)
- **Version:** 2.1.0
- **Repo size:** Very large root (~40 files including compiled binaries `simple_ollama_demo`, `simple_roberts_demo`, `validate_core_functionality`, `test_telemetry`, `validate_telemetry`, `weaver_demo_simple` committed to repo); many `.rs` files at root level (loose test/demo files not in src/)

### Cargo Metadata Completeness
- `name`, `version`, `edition`, `description`, `license` (MIT), `authors`, `homepage` (placeholder `user/swarmsh-v2`), `repository` (same placeholder), `keywords`, `categories` — set but `homepage`/`repository` URLs use `user` not actual username
- No `[workspace.package]`, `[workspace.dependencies]`, `[workspace.lints]` (single crate)
- `[features]` block: 11 features including `default` umbrella activating most optional deps (jaeger, prometheus, otlp, stdout, shell-export, ai-integration, cdcs-v8)

### Lints / Quality Config
- No `rustfmt.toml`, `clippy.toml`, `deny.toml`, `typos.toml`, `.editorconfig`
- No `[lints]` table in Cargo.toml
- CI runs `cargo clippy -- -D warnings`

### CI (`ci.yml`)
Single workflow, 11 jobs, all `ubuntu-latest`, triggered on push to `main`/`develop`, PR to `main`, `workflow_dispatch`:
| Job | Description |
|---|---|
| `validate-semantic-conventions` | Install otel-weaver, validate `semantic-conventions/`, generate telemetry code, check generated files exist |
| `test-rust-implementation` | Matrix [stable, beta]; fmt check, clippy, build debug+release, test, integration tests |
| `test-shell-export` | Build via Makefile, validate exported `.sh` files exist and pass `bash -n` |
| `test-cliapi-integration` | Machine-first JSON output, YAML spec processing |
| `test-performance-quality` | Benchmarks, nanosecond ID uniqueness check |
| `test-documentation` | `make docs`, README content checks |
| `security-audit` | `cargo-audit`, unsafe code count check (<5) |
| `test-dlss-optimization` | 8020 analytics, health monitoring |
| `test-deployment` | Full `make setup/generate/build/export`, upload shell-export artifacts |
| `prepare-release` | On main only; package binaries + shell-export as .tar.gz |
| `notify-success` | Echo statements |

- Caching: `actions/cache@v3` keyed on `Cargo.lock`
- Uses deprecated `actions-rs/toolchain@v1`; `actions/checkout@v4`; `actions/upload-artifact@v3`
- No `cargo-deny`, no coverage (though `Makefile` has `coverage` recipe)
- OTEL Weaver (`otel-weaver` tool) installed inline via `cargo install` — slow, no cache

### Task Runner (`Makefile`)
Rich Makefile (~14 KB) with recipes:
`help`, `setup`, `build`, `clean`, `dev`, `test`, `test-80-20`, `test-unit`, `test-all`, `test-property`, `coverage`, `lint`, `fmt`, `fmt-check`, `pre-commit`, `ci-local`, `generate`, `validate`, `docs`, `export`, `export-coord`, `export-telem`, `export-health`, `export-analytics`, `export-ai`, `start`, `agent`, `stop`, `health`, `analyze`, `install`, `install-shell`, `compound`, `infinite`, `scale`

Shell scripts at root: `dev.sh` (large, 27 KB), `full-cycle.sh`, `demo-weaver-integration.sh`, `demo_8020_integration.sh`, `standalone_e2e_demo.sh`, `auto-80-20.sh`, `setup-claude-code.sh`, `generate_telemetry_clean.sh`, `validate_loop.sh`

### Docs Set
| File | Present |
|---|---|
| README.md | Yes (very long, ~28 KB; Diataxis-structured: tutorials, how-to, reference, explanation) |
| README_HONEST.md | Yes (candid status assessment) |
| CLAUDE.md | Yes (very large, ~15 KB; heavy emoji usage, "revolutionary" branding) |
| CHANGELOG.md | Yes (Keep a Changelog; 2 entries: 2.1.0 and 2.0.0) |
| CONTRIBUTING.md | No |
| SECURITY.md | No |
| CODE_OF_CONDUCT | No |

- Additional session/implementation docs: ~30 extra `.md` files at root (FMEA_ANALYSIS, POKA_YOKE_GUIDE, OUTREACH_CAMPAIGN, SALES_PAGE, PRODUCTIZATION, etc.)

### Licensing
- `license = "MIT"` only; no LICENSE file at repo root

### Src Layout
- `src/lib.rs` + `src/bin/` (20+ binaries)
- `src/`: flat module layout — `coordination.rs`, `telemetry.rs`, `shell_export.rs`, `analytics.rs`, `worktree_manager.rs`, `weaver_forge.rs`, `ai_integration.rs`, and many others; plus `src/generated/` for weaver-output
- `src/CLAUDE.md` — CLAUDE.md inside src directory
- `tests/`: 15 integration test files
- `examples/`: 3 Rust examples + 2 YAML specs + demo shell scripts
- `benches/`: `worktree_benchmarks.rs` (criterion)
- `scripts/`: 7 shell validation/test scripts
- Loose compiled binaries committed to repo root (anti-pattern)
- `templates/`: Tera + Jinja2 (`.tera`, `.j2`) templates for code gen
- `semantic-conventions/`: 14 YAML files (OTel semconv format)
- `weaver.yaml`: OTEL Weaver Forge configuration
- `weaver-templates/`: additional template directory
- `generated/`: generated output directory tracked in git

### Common Dependencies
| Crate | Version | Notes |
|---|---|---|
| serde + serde_json + serde_yaml | 1.0 | core |
| thiserror | 1.0 | |
| anyhow | 1.0 | |
| tokio (full) | 1.0 | |
| clap (derive) | 4.0 | |
| tracing + tracing-subscriber | 0.1 / 0.3 | with json, env-filter, chrono features |
| tracing-opentelemetry | 0.24 | |
| opentelemetry + opentelemetry_sdk | 0.23 | heavy OTEL stack |
| opentelemetry-semantic-conventions | 0.15 | |
| uuid | 1.9 | |
| chrono | 0.4 | |
| minijinja | 2.10 | shell template generation |
| regex | 1.10 | |
| criterion | 0.5 | benches |
| proptest | 1.0 | property-based tests |
| insta | 1.39 | snapshot testing |
| chicago-tdd-tools | 1.3 | custom TDD framework (dev-dep) |
| ollama-rs | 0.3.1 | AI integration (optional) |
| metrics + metrics-exporter-prometheus | 0.21 / 0.12 | |

### House-Specific Tooling
- `weaver.yaml` + `semantic-conventions/*.yaml` + `weaver-templates/` — OTel Weaver Forge code generation pipeline
- `src/generated/` tracked in git (auto-gen span builders, attributes, metrics)
- `templates/` with `.tera` and `.j2` files for shell export
- `coordination/coordinator.json`, `context/` (session tracking), `agents/` (agent configs)
- `work/` directory (active work tracking)

---

## Cross-Repo: Conventions Observed

1. **Rust 2021 or 2024 edition** — swarmsh-v2 uses 2021, a2a-rs workspace uses 2024 (most members)
2. **MSRV declared in members** — a2a-rs sets `rust-version = "1.85"`; swarmsh-v2 does not set it
3. **Clippy -D warnings enforced in CI** — both repos
4. **cargo fmt --check in CI** — both repos
5. **ubuntu-latest only** — no macOS/Windows runners in either repo
6. **actions-rs/toolchain@v1** — both use this deprecated action (no replacement with `dtolnay/rust-toolchain`)
7. **actions/cache@v3 keyed on Cargo.lock** — both repos use the same caching strategy
8. **Cargo.lock committed** — both repos commit Cargo.lock (appropriate for binaries/applications)
9. **CLAUDE.md present** — both repos have it (heavy content; serves as dev guide for AI-assisted sessions)
10. **Large volume of root-level `.md` files** — both repos accumulate session/implementation summaries at root
11. **No LICENSE file** — license is declared in Cargo.toml metadata only; no standalone LICENSE file
12. **thiserror + anyhow for errors** — both repos
13. **serde + serde_json** — universal across all crates
14. **tokio async runtime** — both repos use tokio
15. **Keep a Changelog format + Semantic Versioning** — both CHANGELOGs use this format
16. **Custom code generation tooling** — a2a-rs uses `ggen` (SPARQL/RDF/Tera); swarmsh-v2 uses OTel Weaver Forge

## Cross-Repo: Divergences / Inconsistencies

1. **Workspace vs single-crate:** a2a-rs is a 10-member workspace with no `[workspace.dependencies]` sharing (copy-paste deps); swarmsh-v2 is a single crate with 20+ binaries
2. **thiserror major version split:** a2a-rs uses 1.0 in some members and 2.0 in others within the same workspace
3. **axum version split:** 0.7 and 0.8 coexist in a2a-rs workspace
4. **No rustfmt.toml or clippy.toml** in either repo — quality settings exist only in CI invocations
5. **No deny.toml** — neither repo uses `cargo-deny` for license/advisory checking
6. **No justfile / Makefile.toml:** a2a-rs has no task runner; swarmsh-v2 uses a large raw Makefile
7. **CI complexity mismatch:** a2a-rs has 4 simple jobs; swarmsh-v2 has 11 elaborate jobs including OTEL Weaver validation steps
8. **MSRV consistency:** a2a-rs enforces `rust-version = "1.85"` in members; swarmsh-v2 omits it entirely
9. **Compiled binaries committed to repo root** in swarmsh-v2 (anti-pattern absent from a2a-rs)
10. **No `[workspace.package]` inheritance** in a2a-rs — edition, MSRV, license are copy-pasted across all 10 members
11. **Dual MIT/Apache-2.0 inconsistency** — some a2a-rs members use `MIT OR Apache-2.0`, others use just `MIT`
12. **Repository URLs:** swarmsh-v2 has placeholder `user/swarmsh-v2` in both `homepage` and `repository` fields
13. **`.claude/` integration depth:** a2a-rs has a fully elaborated `.claude/` with hooks, sub-agents, skills, and rules; swarmsh-v2 has no `.claude/` directory
