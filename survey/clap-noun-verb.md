Here is the structured boilerplate fingerprint report for `clap-noun-verb`:

---

# Boilerplate Fingerprint Report: `clap-noun-verb`

**Repo:** `https://github.com/seanchatmangpt/clap-noun-verb`
**Accessibility:** PUBLIC

---

## Identity

| Field | Value |
|---|---|
| Name | `clap-noun-verb` |
| Workspace | Yes — root + `clap-noun-verb-macros` + `utils` (as `clap-noun-verb-utils`); `unibit-cli` present but NOT in workspace members; 9 sub-crates under `crates/` (c8-*) also NOT workspace members |
| Excluded from workspace | `playground`, `examples/greet-demo`, `vendors`, `examples/playground` |
| Rust edition | 2021 (all workspace members) |
| MSRV | `1.74` (root + utils); macros crate sets `1.70`; `.tool-versions` pins `rust stable` |
| Version scheme | CalVer: `26.6.14` (YY.MM.PATCH) |
| Rough size | ~249 `.rs` files, ~29 `.toml` files; large `archive/` tree; extensive `tests/` and `examples/` trees |

---

## Cargo Metadata Completeness

**Root `[package]`:**

| Field | Set? |
|---|---|
| `license` | Yes — `MIT OR Apache-2.0` |
| `description` | Yes |
| `repository` | Yes |
| `homepage` | Yes (same as repository) |
| `documentation` | Yes (`docs.rs/clap-noun-verb`) |
| `keywords` | Yes (5: cli, clap, noun-verb, command-line, typer) |
| `categories` | Yes (2: command-line-utilities, development-tools) |
| `authors` | Yes |
| `rust-version` | Yes (`1.74`) |
| `readme` | Yes |

**Workspace inheritance:**

- `[workspace.dependencies]`: Yes — clap, linkme, serde, serde_json, thiserror, anyhow, once_cell, toml, tokio all centralized
- `[workspace.lints]`: Yes — `[lints] workspace = true` used in root, macros, utils, unibit-cli
- `[workspace.package]`: NOT used (each member repeats `edition`, `license`, `version` manually)
- `[package.metadata.docs.rs]`: Yes (root only) — `rustdoc-args = ["--cfg", "docsrs"]`

---

## Lints / Quality Config

**`rustfmt.toml`:** Present. Key settings: `edition = "2021"`, `max_width = 100`, `tab_spaces = 4`, `use_small_heuristics = "Max"`, `reorder_imports = true`, `use_try_shorthand = true`, `use_field_init_shorthand = true`.

**`[workspace.lints]` (in Cargo.toml):**
- `unsafe_code = "allow"` (needed for linkme)
- `bare_trait_objects = "warn"`
- `unexpected_cfgs = "warn"` with explicit check-cfg allowlist for frontier feature names + kani + docsrs
- `dead_code = "allow"` (v5.1 placeholder justification)
- `async_fn_in_trait = "allow"`
- Clippy: `all = { level = "warn", priority = -1 }` as baseline; `unwrap_used`, `expect_used`, `panic` all set to `"allow"`; `unimplemented`, `todo`, `exit` set to `"deny"`; large allowlist of stylistic warnings suppressed

**`clippy.toml`:** NOT present (clippy config lives entirely in `[workspace.lints.clippy]`)

**`deny.toml`:** Present. Licenses allowed: MIT, Apache-2.0, BSD-2/3-Clause, ISC, Unicode-3.0, CC0-1.0. Advisories: ignores 2 known unmaintained entries. Bans: `multiple-versions = "warn"`, `wildcards = "deny"`. Sources: only crates.io + GitHub index allowed.

**`typos.toml`:** Present. Excludes: `Cargo.lock`, `target`, `.git`, `CONVO.txt`, `archive`, `crates`, `examples/greet-demo`. Extends word allowlist with domain-specific non-words (mape, servises, statu, etc.).

**`.editorconfig`:** Present. UTF-8, LF, final newline, trim trailing. `.rs`/`.toml`: 4-space indent. `.md`: 2-space (no trim). `.yml`/`.yaml`: 2-space.

**`.cargo/config.toml`:** `git-fetch-with-cli = true`; `RUST_BACKTRACE = "1"` as default env.

---

## CI Workflows

| File | Trigger | Key Jobs |
|---|---|---|
| `ci.yml` | push to main/develop/release/**, PR to main/develop | `fmt`, `clippy`, `test` (matrix: stable/beta/nightly), `nextest`, `msrv` (1.74), `docs`, `audit`, `licenses` (deny), `typos`, `ci-success` gate |
| `release.yml` | push tag `v*` | `validate`, `msrv-check`, `security-check`, `license-check`, `publish` (macros then root, then GitHub Release via `softprops/action-gh-release@v1`) |
| `audit.yml` | weekly schedule (Monday 00:00 UTC) | `audit` (cargo-audit, uploads JSON artifact) |
| `performance.yml` | PR, push to main, weekly Sunday | `performance` (binary size SLO ≤10MB, bench comparison), `feature-matrix` (5 feature combos) |
| `frontier-ci.yml` | push to main/develop/frontier/**, nightly | `feature-matrix` (21-point matrix: 9 individual + 3 meta + 5 combos + 3 extremes) |
| `docs-validation.yml` | push/PR on docs/README paths | `doc-build` |
| `pr-feedback.yml` | (not fully read) | PR comment/feedback automation |
| `projection-verification.yml` | (not fully read) | ggen/ontology projection checks |

**Runner OS:** All ubuntu-latest (no Windows/macOS matrix).

**Toolchain actions used:** `dtolnay/rust-toolchain@stable|master|1.74.0`, `Swatinem/rust-cache@v2` (cache key = `hashFiles('**/Cargo.lock')`), `taiki-e/install-action@{cargo-nextest,cargo-audit,cargo-deny,typos}`, `softprops/action-gh-release@v1`, `actions/upload-artifact@v4`.

**No dependabot.yml** found.

---

## Task Runner

**`Justfile`:** Present. Recipes: `test`, `test-full`, `polish`, `build`, `clean`, `doc`, `bench`, `ci`.

**`Makefile.toml` (cargo-make):** Present and extensive. Key recipe groups:
- Format: `format`, `format-check`
- Lint: `clippy`, `lint`
- Test: `test`, `test-default`, `test-no-features`, `test-repl`, `test-feature-combinations`, `test-lib-deterministic`, `test-integration-isolated`, `test-unfailable`, `test-all`, `test-frontier-*`, `test-frontier-matrix` (23-combo shell script)
- Build: `build`, `build-release`, `build-examples`, `check`, `check-all`
- Docs: `doc`, `doc-open`
- CI: `ci`, `verify`, `andon-check` (5-gate stop-the-line protocol)
- Release: `release-check`, `publish-dry-run-macros`, `publish-macros`, `publish`, `publish-all`, `release-validate`
- Security: `audit`, `security-scan`
- Bench: `bench`, `bench-baseline`, `bench-compare`, `bench-phase1/2/3/4`, `slo-check`
- Coverage: `coverage-report` (cargo-tarpaulin, 80% threshold, HTML + Cobertura output)

**`scripts/`:** Present. Shell scripts: `bench.sh`, `generate_360_templates.sh`, `generate_examples.sh`, `measure_performance.sh`, `pre-release-check.sh`, `release-automation.sh`, `release-checklist.sh`, `setup-dx.sh`, `sr_loop.sh`, `test_matrix_21.sh`, `test_timeout.sh`, `validate-docs.sh`, `validate.sh`, plus Rust utility: `doc_example_validator.rs`.

---

## Documentation Set

| Doc | Present? |
|---|---|
| README.md | Yes |
| CLAUDE.md | Yes (extensive — project overview, build commands, architecture, critical rules, recipes, ADL, glossary) |
| CONTRIBUTING.md | Yes |
| SECURITY.md | Yes |
| CHANGELOG.md | Yes |
| CODE_OF_CONDUCT | Not found |
| DEFINITION_OF_DONE.md | Yes (house-specific) |
| DX_GUIDE.md | Yes |
| `docs/` tree | Yes — Diátaxis structure: `tutorial/`, `howto/`, `reference/`, `explanation/`, `_internal/` |

**README sections:** What's New, Installation, The Noun-Verb Pattern, Quick Start, Feature Matrix, Cargo Features, Playground How-To, Additional Library Modules, Learn More (tutorials/how-tos/reference/explanations), Contributing, License.

**CLAUDE.md sections:** Project Overview, Build Commands, Crate Structure, Architecture, Critical Rules, Formatting, Publishing, SLOs, Development Workflows (recipes), Common Recipes, Troubleshooting, Performance Optimization, Contributing Guidelines, Architecture Decision Log (ADL-001 through ADL-010), Glossary.

---

## Licensing

- `LICENSE-MIT`: Present
- `LICENSE-APACHE`: Present
- `LICENSE`: Present (generic)
- All crates declare `MIT OR Apache-2.0`

---

## Src Layout

- **Root:** `src/lib.rs` (lib crate) + `src/bin/clap-noun-verb-gen.rs` (binary)
- **Modules:** `src/{noun,verb,registry,context,format,error,async_verb,builder,deprecation,policies,shell,telemetry,tree,validators,ggen_to_rdf,rdf_to_ggen,ontology_sync,repl}.rs` plus subdirs `src/{capability,clap_ext,cli,diagnostics,federation,graph,logic,macros}/`
- **Macros crate:** `clap-noun-verb-macros/src/{lib.rs,macros/,io_detection.rs,validation.rs,...}` — proc-macro crate (`proc-macro = true`)
- **Utils crate:** `src/` with adapters, completions, display_json, help, mangen, markdown, number_parsing
- `tests/`: Extensive — 50+ test files, organized into `cli/`, `acceptance/`, `frontier/`, `common/`, `macros/`, `performance/`, `scenarios/`
- `examples/`: Organized as `tutorial/`, `howto/`, `reference/`, `c8/`, `ggen/`, `greet-demo/`, `specimen-graph-manager/`, `turtle-specs/`, `specs/`
- `benches/`: `dispatch.rs` (Criterion, `harness = false`), `generate.js`
- `fuzz/fuzz_targets/`: Fuzz testing targets present
- `ontology/`: `.ttl` ontology files + `queries/` SPARQL + `examples/`
- `templates/`: Tera templates for code generation
- `crates/`: 9 non-workspace sub-crates (c8-adversary, c8-bench, c8-core, c8-graph, c8-instruments, c8-market, c8-receipts, c8-time, cargo-cicd)

---

## Common Dependencies (root Cargo.toml)

| Crate | Version | Notes |
|---|---|---|
| `clap` | `>=4.5, <4.6` | features: derive, env, suggestions |
| `linkme` | `0.3` | distributed slices for verb discovery |
| `serde` | `1.0` | features: derive |
| `serde_json` | `1.0` | |
| `thiserror` | `1.0` | |
| `anyhow` | `1.0` | |
| `once_cell` | `1.19` | |
| `toml` | `0.8` | |
| `tokio` | `1.40` | features: full |
| `tracing` | `0.1` | optional, `otel` feature |
| `tracing-subscriber` | `0.3` | optional |
| `tracing-opentelemetry` | `0.28` | optional |
| `opentelemetry` | `0.27` | optional |
| `opentelemetry_sdk` | `0.27` | optional |
| `parking_lot` | `0.12` | |
| `serde_yaml` | `0.9` | |
| `notify` | `6.1` | file watching |
| `regex` | `1.10` | validators |
| `url` | `2.5` | validators |
| `chrono` | `0.4` | |
| `jmespath` | `0.3.0` | |
| `rustyline` | `14.0.0` | optional, `repl` feature |

**Dev:** `criterion 0.5`, `proptest 1.0`, `insta 1.0` (json+yaml), `loom 0.7`, `assert_cmd 2.0`, `predicates 3.0`, `assert_fs 1.0`, `tokio-test 0.4`, `serial_test 3.0`, `tempfile 3.8`, `cargo-make 0.37`, `chicago-tdd-tools 1.0.0`

**Macros crate:** `syn 2.0` (full+parsing), `quote 1.0`, `proc-macro2 1.0`, `proc-macro-error 1.0`

**Utils crate:** `clap_complete 4.5`, `clap_mangen 0.2`, `clap-num 1.1`, `num-traits 0.2`

---

## House-Specific Tooling

| Artifact | Description |
|---|---|
| `ggen.toml` | Pack manifest for `ggen` code generator (RDF ontology → Tera template → Rust source); SPARQL CONSTRUCT (μ₁ inference) + SELECT+template pairs (μ₃ generation); ASK validation rules |
| `ontology/*.ttl` | OWL/RDF ontologies (clap-noun-verb, cli-pattern, cargo-cicd, oshb-morphology, etc.) |
| `ontology/queries/*.rq` | SPARQL SELECT/ASK queries for ggen generation rules |
| `templates/*.tera` | Tera code-gen templates for verb wrappers and mod files |
| `.chatmangpt/state.yaml` | Agent workflow state (line_status, work_state, phase, active_delta) |
| `.claude-flow/metrics/` | JSON metrics from claude-flow agent orchestration |
| `.hive-mind/hive.db`, `memory.db` | SQLite databases from hive-mind agent system |
| `.specify/integrations/portfolio.json` | Portfolio integration config |
| `EVIDENCE.*` | Multiple evidence artifacts: `.report`, `.hash`, `.export` files certifying CI/test outcomes |
| `package.json` / `package.toml` | Node.js package (generate.js bench script) |
| `unrdf.toml` | UnRDF config (RDF-to-ggen reverse pipeline) |
| `concept_coverage.json`, `concept_gaps.json`, `concept_ruleset.yaml` | Coverage/gap analysis for ontology concepts |
| `evidence_graph.json`, `evidence_graph_extended.json` | Evidence provenance graphs |
| `ralph_plan.json` | Agent planning artifact |
| `validation_receipt.yaml`, `implementation_receipt.yaml` | Receipt artifacts |
| `CONVO.txt` | Raw conversation log in repo root |
| `.cursor/commands/` | Cursor AI editor slash-command definitions (kaizen, gemba-walk, poka-yoke, etc.) |
| `.cursorrules` | Cursor AI rules file |
| `.githooks/` | Custom git hooks (commit-msg, pre-commit, pre-push, post-commit) with install/uninstall scripts |
| AGENT_*.md / SUBAGENT_*.md | Multi-agent system design documentation (5 files) |
| `RESEARCH_THESIS.tex` | LaTeX research thesis in repo root |

---

## Conventions Observed

1. **CalVer versioning** (`YY.MM.PATCH`, e.g., `26.6.14`) used across workspace
2. **Dual MIT/Apache-2.0 licensing** with three license files (LICENSE, LICENSE-MIT, LICENSE-APACHE)
3. **`[workspace.dependencies]` + `[workspace.lints]`** for DRY config, but `[workspace.package]` NOT used (edition/license/version repeated per member)
4. **`dtolnay/rust-toolchain` + `Swatinem/rust-cache@v2`** as standard CI toolchain pattern; cache keyed on `Cargo.lock` hash
5. **`taiki-e/install-action`** for cargo-nextest, cargo-audit, cargo-deny, typos
6. **Three-runner test matrix** (stable/beta/nightly) in main CI
7. **MSRV enforced both in CI** (`dtolnay/rust-toolchain@1.74.0` job) and in `Cargo.toml` `rust-version`
8. **`RUSTDOCFLAGS: -D warnings`** for strict doc builds
9. **Diátaxis-structured docs** (`tutorial/`, `howto/`, `reference/`, `explanation/`)
10. **Extensive CLAUDE.md** with ADL, recipes, troubleshooting, glossary
11. **cargo-make** (`Makefile.toml`) as primary task runner with "Andon signal" stop-the-line protocol
12. **Justfile** as thin wrapper delegating to cargo-make
13. **cliff.toml** for conventional commit changelog generation
14. **linkme distributed slices** for zero-registration verb discovery
15. **`otel` feature gate** for OpenTelemetry instrumentation
16. **Evidence artifact files** (`EVIDENCE.*`) as on-disk proof of CI/test outcomes
17. **ggen + RDF ontology** as code-generation source of truth for verb wrappers

---

## Divergences / Inconsistencies

1. **`[workspace.package]` not used**: `edition`, `license`, `authors`, `version` are repeated manually in each member's `[package]` instead of being inherited.
2. **MSRV inconsistency**: root and utils declare `rust-version = "1.74"` but macros crate declares `1.70`; CI tests MSRV 1.74.
3. **Non-workspace crates in repo**: `crates/c8-*` (9 crates) and `unibit-cli` are present in the repo tree but excluded from the workspace; `unibit-cli` references `unibit-*` path deps that are outside the repo entirely.
4. **`clippy.toml` absent**: all clippy configuration lives in `[workspace.lints.clippy]`; no standalone `clippy.toml`.
5. **No `dependabot.yml`**: dependency update automation is absent despite the CI having an `audit.yml` scheduled workflow.
6. **`serde_yaml = "0.9"` not in workspace.dependencies**: declared directly in root `[dependencies]`, not centralized.
7. **`lazy_static` alongside `once_cell`**: both present; `once_cell` is preferred in modern Rust.
8. **`cargo-make` in `[dev-dependencies]`**: `cargo-make = "0.37"` listed as a dev-dep (unusual; it's normally a standalone install, not a library dep).
9. **`proc-macro-error`** used in macros crate despite being listed as unmaintained in `deny.toml`'s advisory ignore list (`RUSTSEC-2024-0370`).
10. **Version skew**: `utils` is at `26.6.1` while root and macros are at `26.6.14`.
11. **`package.json` and `generate.js`** in a Rust repo root; Node.js artifact unexplained.
12. **`CONVO.txt`, `RESEARCH_THESIS.tex`, `.hive-mind/*.db`, `evidence_graph*.json`, `ralph_plan.json`** committed to repo root — significant non-code/non-doc artifact pollution.
13. **`unibit-cli` in workspace.members list comment vs actual**: `unibit-cli` is not in `workspace.members` but is present in the repo; its `Cargo.toml` uses workspace deps, making it appear semi-integrated.
