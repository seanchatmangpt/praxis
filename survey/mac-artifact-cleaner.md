---

## Boilerplate Fingerprint: `mac-artifact-cleaner`

### Identity

| Field | Value |
|---|---|
| Repo name | `mac-artifact-cleaner` (GitHub) |
| Package name | `osx-clnr` |
| Binary name | `oclnr` |
| Visibility | Public |
| Structure | Workspace with one sub-crate (`cfab-surface`); root crate is a single binary |
| Rust edition | 2021 (root + member) |
| MSRV | None set explicitly (`rust-version` field absent from Cargo.toml) |
| Toolchain | Nightly (`rust-toolchain.toml` → `channel = "nightly"`) |
| Repo size | ~171 files, ~51 Rust source files, ~11,400 Rust LOC in `src/` |

---

### Cargo Metadata Completeness

Root `Cargo.toml` (`osx-clnr`):
- **Set:** `name`, `version`, `edition`
- **Absent:** `description`, `repository`, `homepage`, `keywords`, `categories`, `authors`, `rust-version`, `license`

Member `cfab-surface/Cargo.toml`:
- **Set:** `name`, `version`, `edition`
- **Absent:** same as root

No `[workspace.package]`, `[workspace.dependencies]`, or `[workspace.lints]` inheritance — the workspace only lists `members`. Sub-crate has its own standalone `Cargo.lock`.

Version scheme: `0.1.0` (initial, not yet released).

---

### Lints / Quality Config

- `rustfmt.toml`: **absent**
- `clippy.toml`: **absent**
- `[lints]` in Cargo.toml: **absent**
- `deny.toml`: **absent**
- `typos.toml`: **absent**
- `.editorconfig`: **absent**
- Clippy is invoked via `cargo clippy --all-targets -- -D warnings` in both `Makefile` and `scripts/sanity.sh` (treating warnings as errors). No configuration file — relying on command-line flags only.

---

### CI

No `.github/` directory present — **zero CI workflows**. No GitHub Actions, no Dependabot, no release automation.

---

### Task Runner

**Dual runner: Makefile + Justfile.**

`Makefile` (primary, canonical):
- `help`, `build`, `release`, `install`, `test`, `lint`, `clippy`, `fmt`, `check`, `clean`
- Notable: `clippy` target runs `-D warnings`; `check` chains `fmt lint test`; `install` calls `cargo install --path .`

`Justfile` (thin wrapper delegating to Makefile):
- `test`, `polish`, `build`, `build-release`, `clean`, `ci`
- All recipes call `make <target>`, so Justfile is a facade.

`scripts/sanity.sh` (7-step DX suite):
1. `cargo fmt -- --check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`
4. `cargo run -- doctor architecture`
5. `cargo run -- doctor substrate`
6. `cargo run -- doctor doctests`
7. `cargo run -- doctor privacy`

`.cargo/config.toml` aliases: `dx` (= `test`), `doctor-arch`, `doctor-sub`, `doctor-doc`, `doctor-priv`, `sanity`.

---

### Docs Set

| File | Present |
|---|---|
| README.md | Yes |
| CLAUDE.md | Yes |
| AGENTS.md | Yes (extensive AI agent operating contract) |
| PROJECT.md | Yes |
| CONTRIBUTING.md | No |
| SECURITY.md | No |
| CHANGELOG.md | No |
| CODE_OF_CONDUCT.md | No |
| DOC_COVERAGE_LOG.md | Yes |
| ORIGINAL_REQUEST.md | Yes |
| `docs/` directory | Yes (GALL_CHECKPOINTS, PRIVACY_MODEL, OCEL_MODEL, TIME_MACHINE_MODEL, CFAB_ALIGNMENT, etc. + `docs/thesis/` with full PhD thesis) |

README sections: Project tagline, "The Old Computing Gap", Architecture & Workflow, Snapshot Management, Emergency Reclaim, Privacy and Safety, Documentation links.

CLAUDE.md sections: Project Identity, Build Commands, Architecture (3-layer rule, CLI structure, execution pipeline, OCEL v2, domain DTOs, traversal model), Doctests as specification, Output files, Key Dependencies, Gall Checkpoints.

AGENTS.md: 15 sections covering non-negotiable operating law, system philosophy, prompt/agent instruction innovations, Gall checkpoints, CLI architecture, doctest discipline, OCEL v2 model, technical reference, root tool classifications, Time Machine awareness, progress UX, privacy rules, development workflow, definition of done, recommended repo shape.

---

### Licensing

No LICENSE file present. No `license` field in `Cargo.toml`.

---

### src Layout

```
src/
  main.rs           — thin CLI entrypoint
  lib.rs            — library root
  domain/           — pure Rust, zero OS calls (15 modules)
  integration/      — all fs/OS/tmutil/docker calls (8 modules)
  nouns/            — CLI subcommand handlers, bridges integration↔domain (18 modules)
tests/
  github_tests.rs
  integration_tests.rs
examples/
  reclaim_check.rs
  size_units.rs
  snapshot_pipeline.rs
cfab-surface/
  src/lib.rs
  tests/validation_tests.rs
ui/                 — Next.js frontend (separate from Rust)
```

Three-layer architectural separation is a hard constraint: domain (pure) → integration (effectful) → nouns (CLI routing).

---

### Common Dependencies

| Crate | Version | Notes |
|---|---|---|
| `clap` | 4 | `features = ["derive"]` |
| `serde` | 1 | `features = ["derive"]` |
| `serde_json` | 1 | |
| `anyhow` | 1 | |
| `blake3` | 1.8.5 | content addressing |
| `linkme` | 0.3 | plugin/handler discovery |
| `chrono` | 0.4 | `features = ["serde"]` |
| `rayon` | 1 | parallel traversal |
| `dashmap` | 6 | concurrent map |
| `jwalk` | 0.8 | fast parallel dir walker |
| `indicatif` | 0.17 | progress bars |
| `dialoguer` | 0.12.0 | interactive prompts |
| `colored` | 3.1.1 | terminal color |
| `clap_complete` | 4.6.5 | shell completions |
| `notify` | 8.2.0 | filesystem watch |
| `sled` | 0.34.7 | embedded KV store |
| `once_cell` | 1.21.4 | lazy statics |
| `toml` | 1.1.2 | config parsing |
| `libc` | 0.2 | `statvfs` free-space |
| `ignore` | 0.4 | gitignore-aware walker |
| `dirs` | 5 | home dir resolution |
| `which` | 8.0.3 | PATH binary lookup |
| `tempfile` | 3 | dev-dep only |
| `thiserror` | 1 | (cfab-surface only) |
| `petgraph` | 0.6 | (cfab-surface, graph structures) |
| `clap-noun-verb` | path dep | local monorepo |
| `clap-noun-verb-macros` | path dep | local monorepo |
| `wasm4pm-compat` | 26.6.8 | `features = ["formats", "strict"]` |
| `affidavit` | git dep | `features = ["core"]`, default-features off |

No `tokio` or async runtime — synchronous Rust with `rayon` for parallelism.

---

### House-Specific Tooling

- `AGENTS.md` — AI agent operating contract (extensive, 15 sections); unique to this project style
- `GALL_CHECKPOINTS.md` — Gall's Law checkpoint-based capability promotion gate
- `DOC_COVERAGE_LOG.md` — documentation coverage tracking
- `ORIGINAL_REQUEST.md` — original project request preserved
- `PROJECT.md` — architecture summary doc
- `maintenance-plan.json` / `maintenance-receipt.json` — plan/receipt files committed to repo (unusual)
- `docs/thesis/` — PhD thesis documents (Sean Chatman / Pentecost)
- Local monorepo path deps: `clap-noun-verb`, `clap-noun-verb-macros` (from `../clap-noun-verb`)
- `affidavit` pulled as a git dependency from `github.com/seanchatmangpt/affidavit`
- `wasm4pm-compat` versioned at `26.6.8` (house package manager compatibility layer)
- `.cargo/config.toml` with domain-specific `doctor-*` aliases
- `doctor` subcommand built into the CLI for self-verification (architecture, substrate, doctests, privacy)

---

### Conventions Observed

1. Nightly toolchain pinned via `rust-toolchain.toml`
2. Three-layer architecture enforced as an explicit hard constraint: `domain` (pure) / `integration` (effectful) / `nouns` (CLI)
3. Noun-verb CLI pattern via house `clap-noun-verb` library
4. Doctests treated as executable specification (mandatory for all public domain functions)
5. OCEL v2 JSON as canonical evidence/receipt format at all external boundaries
6. Plan-bound execution model: scanner and deleter are deliberately separated
7. Dual Makefile + Justfile task runner (Justfile delegates to Makefile)
8. `.cargo/config.toml` aliases for DX commands
9. `scripts/sanity.sh` as 7-step full DX pipeline (fmt → clippy → test → 4 doctor checks)
10. Clippy `-D warnings` enforced at task-runner level (no config file)
11. `AGENTS.md` as AI agent operating contract alongside `CLAUDE.md`
12. `blake3` + `linkme` as recurring house crate pair
13. `rayon` + `jwalk`/`ignore` for parallel, gitignore-aware traversal (no async)
14. Local machine output files protected by `.gitignore` with domain-specific patterns

---

### Divergences / Inconsistencies

1. **No CI at all** — no `.github/workflows/`, no Dependabot; sanity checks exist only locally.
2. **No LICENSE file** and no `license` field in `Cargo.toml` despite the parent `affidavit` project using MIT OR Apache-2.0.
3. **No Cargo.toml metadata** — `description`, `repository`, `homepage`, `keywords`, `categories`, `authors` all absent.
4. **Committed receipt/plan JSON** — `maintenance-plan.json` and `maintenance-receipt.json` are committed, inconsistent with `.gitignore` protecting all other `*-plan.json` and `*-receipt.json` files.
5. **Sub-crate has its own `Cargo.lock`** — `cfab-surface/Cargo.lock` exists separately, which is unusual for a workspace member.
6. **No shared workspace dependencies** — workspace `[package]` and `[dependencies]` tables are unused; each crate declares its own deps without inheritance.
7. **Justfile is a Makefile facade** — redundant layer since every Justfile recipe just calls `make`.
8. **No `rustfmt.toml` or `clippy.toml`** — quality config is CLI-flag-only with no persistent settings.
9. **Git dep for `affidavit`** — uses `git =` URL rather than crates.io or local path, making offline builds impossible without network.
10. **`ui/` subdirectory** is a Next.js app tracked in the same repo but entirely disconnected from the Cargo workspace.
