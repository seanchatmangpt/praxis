---

## Boilerplate Fingerprint Survey — seanchatmangpt: bcinr, dteam, semantic_bit

---

### Repo 1: `bcinr`

**Identity**
- Public, accessible. Workspace with 12 members: `bcinr`, `bcinr-core`, `bcinr-bench`, `crates/bcinr-api`, `crates/bcinr-logic`, `crates/wasm4games`, `crates/wasm4games-capi`, plus 5 `tools/` crates. `crates/wasm4games-wasm4pm` excluded (external git dep).
- Rust edition: **2021** (all members). MSRV: `rust-version = "1.70"` (set on publishable crates).
- Toolchain: `nightly`, profile `minimal`, components `rustfmt` + `clippy` (rust-toolchain.toml).
- File count: ~1102 total, 654 `.rs`. Large repo with many Python scripts for batch codegen/audit.

**Cargo metadata completeness**
- Per-member `[package]` sets: `name`, `version` (26.6.x CalVer), `edition`, `description`, `repository`, `license`, `readme`, `rust-version`, `keywords`, `categories`. No `homepage` or `authors` on most members (root workspace has no `[workspace.package]`).
- No `[workspace.package]` inheritance, no `[workspace.dependencies]`, no `[workspace.lints]`.
- Root Cargo.toml: `[workspace]` only; `[profile.release]` with `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`.
- One `[patch.crates-io]` override: `encode_unicode` → local path.
- Version scheme: **CalVer** `26.6.x` (year.month.patch), consistent across all crates.

**Lints/quality config**
- No `rustfmt.toml`, no `clippy.toml`, no `.editorconfig`, no `deny.toml`, no `typos.toml`.
- CI enforces `-D warnings` via `RUSTFLAGS` in Makefile.toml env and direct clippy args.

**CI (`.github/workflows/`)**
- `ci.yml`: jobs: `style` (fmt + clippy, stable, ubuntu), `security` (cargo-audit + cargo-deny, stable), `test` (matrix: ubuntu/macos/windows × stable/beta + ubuntu/nightly via `cargo make ci`), `doc` (cargo doc, stable, `RUSTDOCFLAGS=-D warnings`). Cache: `Swatinem/rust-cache@v2`.
- `bench.yml`: Criterion benchmark on push to main, stores results via `benchmark-action/github-action-benchmark@v1`, `fail-on-alert: true`.
- `miri.yml`: Miri UB check on push/PR, nightly toolchain, `cargo miri test --workspace --all-features`.
- Uses `dtolnay/rust-toolchain`, `actions/checkout@v4`, `Swatinem/rust-cache@v2`.

**Task runner**
- `Makefile.toml` (cargo-make): `check`, `build`, `test`, `bench`, `bench-report`, `clippy`, `fmt`, `audit`, `deny`, `docs`, `clean`, `scan-cheats`, `contract-gate`, `wasm-check`, `ggen-sync`, `ci` (default). `ci` task chains fmt→check→clippy→scan-cheats→contract-gate→test→audit→deny.
- `Makefile`: thin wrapper delegating all targets to `cargo make`.

**Docs set**
- README.md (present, headings: Key Features, Installation, Quick Start, Documentation, Audit & Remediation, Performance & Architecture, Development & Testing, Formal Basis, License).
- CLAUDE.md (present, full project guide).
- GEMINI.md (present, parallel LLM guidance doc).
- AGENTS.md (present, agentic protocol doc defining named AI agents: `@hoare_oracle`, `@turing_machine`, etc.).
- CHANGELOG.md (present).
- No CONTRIBUTING.md, no SECURITY.md, no CODE_OF_CONDUCT.
- `book/SUMMARY.md` (mdBook stub), `docs/diataxis/` (Diátaxis structure), `ocel/` (OCEL receipts).

**Licensing**
- `license = "MIT OR Apache-2.0"` on all crates. No LICENSE file at workspace root (unusual).

**src layout**
- No root `src/`; each workspace member has `src/lib.rs` (facade). Algorithmic core in `crates/bcinr-logic/src/` with rich submodule tree (`abstractions/`, `algorithms/`, `autonomic/`, `patterns/`, flat `.rs` modules). Tools are bin crates. `fuzz/` present with Cargo.toml.
- No top-level `tests/` or `examples/`. Benchmarks in `bcinr-bench/` and per-crate.

**Notable deps** (bcinr-logic/wasm4games/tools): `criterion 0.5`, `proptest 1.x`, `serde 1.0`, `serde_json 1.0`, `regex 1.9`, `prettytable-rs 0.10`.

**House-specific tooling**
- `ocel/` — OCEL event log receipts.
- `fuzz/` — cargo-fuzz targets.
- AGENTS.md — AI agent roster/protocol.
- GEMINI.md — Gemini-specific guidance.
- `tools/bcinr-cheat-scanner` + `tools/bcinr-contract-gate` — custom lint gates.
- `ggen-sync` task references external `ggen` tool.
- Mass Python scripts at root (`fix_*.py`, `generate_*.py`, `implement_*.py`, etc.) — large-scale codegen/batch-fix scaffolding.
- `proof/` directory for wasm4games proofs.

---

### Repo 2: `dteam`

**Identity**
- Public, accessible. Mixed layout: single root `[package]` (`dteam` v1.3.0) plus `[workspace]` with 4 member groups: `crates/ccog`, `crates/ccog-bridge`, `crates/autoinstinct`, `crates/insa/*` (7 insa crates). An inner `crates/insa/` sub-workspace with its own justfile.
- Rust edition: **2021** (all). No MSRV (`rust-version` not set in root Cargo.toml; AGENTS.md says "use current stable").
- Toolchain: `nightly`, profile `minimal`, components `rustfmt` + `clippy` (rust-toolchain.toml). CI uses pinned `nightly-2026-04-15` for ccog gate.
- File count: ~2224 total, 496 `.rs`. Very large with many research/whitepaper artifacts.

**Cargo metadata completeness**
- Root `[package]`: `name`, `version`, `edition`, `license` (BUSL-1.1), `authors`, `description`, `repository`, `homepage`. Missing `keywords`, `categories`, `rust-version`.
- Sub-crates: `name`, `version`, `edition`, `license`, `authors`, `description`. Most have `publish = false`.
- No `[workspace.package]`, no `[workspace.dependencies]`. Has separate `workspace_lints.toml` (not wired via `[workspace.lints]` in Cargo.toml — standalone file).
- `[profile.release]` not set at workspace level.
- Heavy use of local monorepo path deps: `unibit-*`, `wasm4pm-*` (sibling repo paths).
- Root crate: `crate-type = ["cdylib", "rlib"]` (WASM-dual), many `[[bin]]` and `[[bench]]` entries.

**Lints/quality config**
- `workspace_lints.toml` (standalone, not Cargo-integrated): denies `unsafe_code`, `missing_docs`, `expect_used`, `unwrap_used`; warns `pedantic`, `nursery`, `cargo`, `unexpected_cfgs`.
- No `rustfmt.toml`, no `clippy.toml`, no `.editorconfig`, no `deny.toml`, no `typos.toml`.

**CI (`.github/workflows/`)**
- `rust-dod.yml`: path-filtered (src/, Cargo.toml, Cargo.lock, Makefile.toml), concurrency cancel-in-progress. Jobs: `dod` (cargo-make dod, nightly, 20m timeout, soft-fail), `ccog-nightly-gate` (pinned nightly-2026-04-15 + miri + llvm-tools, runs conformance-replay/boundary-tests/pack-coverage/claude-md-fresh/dendral-bundle-roundtrip/bench-baseline-compare), `ccog-stable-still-builds` (stable, no-default-features build of ccog). Cache: `actions/cache@v4` manual (registry + git + target, keyed on Cargo.lock hash).
- `matrix-tv.yml`: Node.js 20/22 matrix for `apps/matrix-tv/` (Next.js + Playwright). Not Rust.
- Uses `dtolnay/rust-toolchain`, `actions/checkout@v4`, `actions/cache@v4`, `actions/setup-node@v4`, `actions/upload-artifact@v4`.
- `.github/CODEOWNERS`: `@seanchatmangpt` owns everything.

**Task runner**
- `Makefile.toml` (cargo-make): `ci` (fmt-check→lint→check→test→conformance-replay→boundary-tests→dendral-bundle-roundtrip→pack-coverage→claude-md-fresh→bench-baseline-compare), `pre-merge` (ci+doc+dod+doctor), `fmt`, `fmt-check`, `lint`, `audit`, `check`, `build`, `build-debug`, `wasm-build`, `wasm-check`, `test`, `test-all`, `bench`, `doctor`, `doctor-dev`, `doctor-target`, `doctor-json`, `plan-diff`, `plan-schema`, `plan-report`, `run`, `pdc`, `pdc-debug`, `pdc-stem`, `pdc-check-data`, `doc` (pdflatex), `dod`, `dod-dev`, `clean`.
- `Makefile`: thin wrapper with 8 targets delegating to cargo/cargo-make.
- `scripts/`: 6 shell/Python helper scripts.
- `crates/insa/justfile`: `dx` (fmt+lint+test-unit+test-golden+test-replay+layout+bench-smoke), `fmt`, `lint`, `test`, `test-unit`, `test-prop`, `test-compile-fail`, `test-golden`, `test-replay`, `test-jtbd`, `bench-smoke`, `clean`, `layout`, `truthforge`, `fuzz`, `miri`, `doctor`.

**Docs set**
- README.md (present, terse: headings Architecture: INSA, Workspace Layout).
- CLAUDE.md (present; sections: Build Commands, Architecture, Critical Performance Constraints, Configuration, Test Organization, Code Conventions, See Also).
- GEMINI.md (present).
- AGENTS.md (present, at root and `crates/insa/`).
- CONTRIBUTING.md (present).
- No CHANGELOG.md, no SECURITY.md, no CODE_OF_CONDUCT.
- `dteam.toml` — project config (kernel tier, autonomic mode, integrity hash).
- `docs/` — large multi-section: architecture, explanation, how-to, reference, tutorials, thesis, whitepaper, opus, regulatory, papers, rust-patterns.

**Licensing**
- `license = "BUSL-1.1"` (Business Source License 1.1, changes to Apache-2.0 on 2029-04-18). Single `LICENSE` file.

**src layout**
- Root `src/lib.rs` (lib) + `src/bin/` (12 binaries). Rich `src/` with flat modules and sub-dirs (`agentic/`, `autonomic/`, `b_yawl/`, `conformance/`, `discovery/`, `io/`, `ml/`, `models/`, `ocpm/`, `powl/`, `probabilistic/`, `ralph_plan/`, `ref_conformance/`, `ref_models/`, `reinforcement/`, `simd/`, `utils/`). `tests/` (12 integration tests), `examples/` (4), `benches/` (26).
- Sub-crate `crates/insa/` is effectively a nested workspace with its own justfile, xtask, docs, AGENTS.md, GEMINI.md.

**Notable deps**: `serde 1.0`, `serde_json 1.0`, `anyhow 1.0`, `thiserror 1`, `tracing 0.1`, `tracing-subscriber 0.3`, `blake3 1.5`, `blake2 0.10`, `tokio 1.52`, `opentelemetry 0.31`, `rayon 1.10`, `quick-xml 0.37`, `uuid 1.0`, `chrono 0.4`, `hashbrown 0.14`, `ed25519-dalek 2`, `oxigraph 0.4` (dev), `criterion 0.5`, `divan 0.1.14`, `iai-callgrind 0.11`, `proptest 1.2`. Depends on `bcinr = "26.4.18"` from crates.io.

**House-specific tooling**
- `dteam.toml` — autonomic kernel configuration (tier, determinism, allocation policy, guards).
- `ontologies/` — negknowledge ontology.
- `crates/insa/` nested sub-workspace with justfile and xtask.
- `crates/insa/xtask` — custom xtask for layout/golden/replay/truthforge.
- `insa/AGENTS.md` — TCPS (Toyota Code Production System) operating contract.
- GEMINI.md + AGENTS.md at both root and sub-workspace level.
- `apps/matrix-tv/` — Next.js dashboard.
- `proptest-regressions/` — persisted proptest regression cases.
- `data/`, `artifacts/`, `dev-worktree/`, `ostar_ref/` — data and research artifacts.

---

### Repo 3: `semantic_bit`

**Identity**
- Public, accessible. Single-crate (no workspace). No `src/` sub-modules via workspace.
- Rust edition: **2024** (the newest edition; notably different from the other two).
- MSRV: not set. No rust-toolchain.toml.
- File count: ~374 total, 18 `.rs`. Small repo, but includes large ontology/codegen scaffolding.

**Cargo metadata completeness**
- `[package]`: `name`, `version` (0.1.0), `edition` only. No `license`, `description`, `repository`, `homepage`, `keywords`, `categories`, `authors`, `rust-version`.
- `[dependencies]`: empty.
- No `[profile.*]`, no `[workspace]`, no features.

**Lints/quality config**
- None: no rustfmt.toml, clippy.toml, .editorconfig, deny.toml, typos.toml.

**CI**
- None: no `.github/` directory at all.

**Task runner**
- None: no Makefile, Makefile.toml, or justfile.

**Docs set**
- No README.md, no CLAUDE.md, no CONTRIBUTING.md, no SECURITY.md, no CHANGELOG.md, no LICENSE.
- GEMINI.md present (sections: Core Philosophy, Rust Architectural Standards, Adversarial Trust, Structure).
- `book.toml` + `book/src/` — mdBook configuration ("The Semantic Bit: A Field Manual for Bounded Meaning in Rust").

**Licensing**
- No license file or license field.

**src layout**
- `src/lib.rs` + `src/main.rs` (both lib and bin). Flat module files: `access.rs`, `access_generated.rs`, `channel.rs`, `checkpoint.rs`, `cog8.rs`, `condition8.rs`, `condition_code8.rs`, `construct8.rs`, `contract.rs`, `control.rs`, `dispatch.rs`, `journal.rs`, `operation64.rs`, `relation64.rs`, `restart.rs`, `status8.rs`. No tests/, examples/, benches/.

**Notable deps**: None (empty `[dependencies]`).

**House-specific tooling**
- `unrdf.toml` — code-generation config: maps RDF ontology (Turtle/SPARQL) → Rust source via Nunjucks templates. Key generator for the `access_generated.rs` and similar files.
- `ontology/fields/` — `spine.ttl` + `access.ttl` (INSA semantic field ontology in Turtle/RDF).
- `queries/rust/field.sparql` — SPARQL query driving codegen.
- `templates/rust/field.rs.njk` — Nunjucks template producing Rust field types from ontology.
- GEMINI.md only (no CLAUDE.md).

---

### Conventions Observed (across all three repos)

1. `cargo-make` + `Makefile.toml` is the standard task runner in every repo that has CI (bcinr, dteam). Companion thin `Makefile` delegates to it.
2. `dtolnay/rust-toolchain` is used universally for toolchain installation in CI.
3. `Swatinem/rust-cache@v2` (bcinr) or manual `actions/cache@v4` on registry+git+target (dteam) for Cargo caching.
4. Both CI repos use nightly as primary channel; bcinr adds a stable/beta test matrix. dteam pins a specific nightly for the conformance gate.
5. `actions/checkout@v4` everywhere.
6. `CLAUDE.md` present in bcinr and dteam (not semantic_bit). Complementary `GEMINI.md` and `AGENTS.md` appear in all three (GEMINI) or two (AGENTS). Multi-LLM guidance doc pattern is a house convention.
7. CalVer `YY.M.patch` versioning in bcinr. Semantic versioning in dteam/semantic_bit.
8. `blake3` appears as a dependency in both bcinr (indirectly via logic) and dteam (direct). Content-addressing is a cross-cutting concern.
9. `serde` + `serde_json` present in all Rust crates with any serialization. `anyhow` + `thiserror` in dteam and tools.
10. Ontology-driven codegen (`unrdf.toml`, `spine.ttl`, SPARQL + Nunjucks) in semantic_bit is an emerging shared infrastructure also referenced by the `INSA` namespace in dteam.
11. `[profile.release] lto = "fat"`, `codegen-units = 1`, `panic = "abort"` is the bcinr release profile; not replicated elsewhere.
12. `publish = false` on tooling/internal crates is consistent in bcinr.

### Divergences / Inconsistencies

- **Edition mismatch**: bcinr and dteam use `edition = "2021"`; semantic_bit uses `edition = "2024"` (no consistency).
- **Toolchain file**: bcinr and dteam have `rust-toolchain.toml`; semantic_bit has none.
- **CI coverage**: bcinr has full CI (fmt/clippy/audit/deny/test-matrix/bench/miri); dteam has narrower CI (no fmt/clippy standalone job, no audit/deny, no cross-platform matrix); semantic_bit has no CI at all.
- **License**: bcinr uses `MIT OR Apache-2.0`; dteam uses `BUSL-1.1`; semantic_bit has no license. No dual MIT/Apache across the board.
- **Workspace `[workspace.package]` / `[workspace.dependencies]` / `[workspace.lints]` inheritance**: unused in all three repos despite multi-crate workspaces.
- **`workspace_lints.toml`** in dteam is a standalone file not wired into Cargo.toml `[workspace.lints]` — lints are declared but not enforced by Cargo tooling.
- **MSRV**: only bcinr sets `rust-version = "1.70"`. dteam and semantic_bit have none.
- **Security/audit**: bcinr runs cargo-audit + cargo-deny in CI; dteam has `audit` task in Makefile.toml but no CI job for it; semantic_bit has nothing.
- **Caching strategy**: bcinr uses `Swatinem/rust-cache@v2` (simpler); dteam uses raw `actions/cache@v4` with manual key construction.
- **Test layout**: bcinr keeps tests inline and in bench crate (no top-level `tests/`); dteam has a rich `tests/` directory (12 integration tests); semantic_bit has no tests.
- **No `deny.toml`** in any repo despite bcinr CI running `cargo deny check`.
