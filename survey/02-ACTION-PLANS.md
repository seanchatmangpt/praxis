# 02 — Per-Repo Action Plans

**Generated:** 2026-06-23
**Scope:** 18 surveyed Rust repos + 4 second-wave repos (2 cross-language).
**Legend:** `[A]` = automatic / scriptable with `apply.sh` or a one-liner. `[H]` = requires human judgment. Effort is per-item wall-clock estimate assuming a clone is on disk.

`apply.sh <REPO>` drops in: `deny.toml`, `typos.toml`, `rustfmt.toml`, `rust-toolchain.toml`, `SECURITY.md`, `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `.github/dependabot.yml`, `.editorconfig`. It never touches `Cargo.toml` — instead it prints a diff of missing `[lints]` and `[profile.release]` sections.

---

## Priority Order

### Tier 0 — Zero CI (highest urgency: no safety net at all)

| Repo | Missing |
|---|---|
| **wasm4pm-compat** | Zero workflows; nightly toolchain; large trybuild suite running nowhere |
| **pm4wasm** | Zero workflows; Apache-only license; bare WASM crate |
| **miniml** | Zero workflows; BSL-1.1; pnpm+turbo monorepo |
| **semantic_bit** | Zero workflows; no license; no README; near-empty Cargo metadata |
| **mac-artifact-cleaner** | Zero workflows; no license; no Cargo metadata |

### Tier 1 — Deprecated CI (broken safety net)

| Repo | Problem |
|---|---|
| **a2a-rs** | `actions-rs/toolchain@v1` + `actions-rs/cargo@v1` deprecated; no task runner; mixed thiserror 1+2 |
| **swarmsh-v2** | `actions-rs/toolchain@v1` deprecated; placeholder repo URLs; committed binaries at root |

### Tier 2 — Security / License Concerns

| Repo | Concern |
|---|---|
| **pm4py-rs** | AGPL-3.0 in Cargo.toml but dual-text in LICENSE file; `deny.toml` absent; MSRV contradiction (1.85 declared, 1.70 tested) |
| **ggen-mcp** | Apache-only license; wrong package name (`spreadsheet-mcp`); wrong repo URL (`PSU3D0/spreadsheet-mcp`) |
| **dteam** | BUSL-1.1 license; unwired `workspace_lints.toml`; no `deny.toml` |
| **miniml** | BSL-1.1 license; zero CI |
| **clap-noun-verb** | RUSTSEC-2024-0370 (`proc-macro-error` unmaintained) |

### Tier 3 — Easiest Wins (closest to conformant)

| Repo | Gap |
|---|---|
| **clap-noun-verb** | Already has rustfmt/lints/deny/typos/.editorconfig/dual; just MSRV bump + RUSTSEC fix |
| **cargo-cicd** | Good CI; just needs rustfmt.toml/deny.toml/typos.toml/.editorconfig/[workspace.lints] |
| **lsp-max** | Good workspace.package; needs [workspace.lints]/deny.toml/typos.toml/.editorconfig/SECURITY.md |
| **affidavit** | Reference example; mostly done; human judgment items remain |
| **bcinr** | Good CI; needs LICENSE files, deny.toml, workspace.lints, pin nightly |

### Tier 4 — Structural Debt (require more work)

ggen (45 workflow files), clnrm (29 workflow files), a2a-rs (workspace consistency), dteam (nested sub-workspace), pm4py-rs (license confusion + MSRV contradiction).

---

## affidavit

**Status:** Reference example — partially done on branch `claude/jolly-turing-t488iq`.
**Remaining work:** Lints + toolchain + 1000x module cleanup.

### Automatic fixes (`apply.sh .`)

1. `[A][5min]` `apply.sh` was already run. Verify by running `apply.sh <affidavit> --dry-run` — all files should report SKIPPED (already present).
2. `[A][5min]` Add `**/*.rs.backup` and `*.orig` to `.gitignore` (done in reference commit; verify it is present).
3. `[A][5min]` Delete `src/verbs/*.rs.backup` files (done; 18 files removed).
4. `[A][5min]` Remove root session artifacts `audit_instructions.txt`, `DX_QOL_EXECUTIVE_SUMMARY.txt` (done).
5. `[A][5min]` Move `portfolio_test_dataset.json` → `fixtures/` (done).

### Human judgment required

1. `[H][30min]` Add `[lints]` block to `Cargo.toml` (single-crate). Paste from `template/Cargo.toml`. Then run `cargo clippy --all-targets --all-features -- -D warnings` with sibling path-deps present to validate. The `1000x_*` modules and any `todo!()` calls will surface as errors.
2. `[H][2h]` Triage the 16 `src/1000x_*.rs` modules: `1000x_auto_remediate_dx.rs`, `1000x_autonomous_governance.rs`, `1000x_chaos_e2e.rs`, `1000x_cli_telepathy_qol.rs`, `1000x_distributed_sharding.rs`, `1000x_formal_verification_spec.rs`, `1000x_gpu_verifier.rs`, `1000x_holographic_lsp_dx.rs`, `1000x_nlp_query_qol.rs`, `1000x_otel_hyper_spec.rs`, `1000x_post_quantum_sealing.rs`, `1000x_receipt_to_wasm_qol.rs`, `1000x_semantic_isomorphism_e2e.rs`, `1000x_swarm_e2e.rs`, `1000x_tdd_synthesizer_dx.rs`, `1000x_time_travel_dx.rs`. Options: delete, move under `experimental/` behind a feature gate, or extract to a separate crate.
3. `[H][1h]` Triage root Python generators: `gen_thesis.py`, `generate_bib.py`, `generate_conclusion.py`, `generate_verbs.py`, `remediate_licenses.py`. Move to `tools/` or delete; they are not crate scaffolding.
4. `[H][30min]` Decide MSRV: bump `rust-version = "1.78"` → `"1.82"` (house default). Verify build still passes.
5. `[H][1h]` Swap nightly→pinned-stable toolchain in `rust-toolchain.toml`: `channel = "1.82.0"`. Remove `continue-on-error: true` from `.github/workflows/rust.yml` once fmt + clippy pass cleanly. Add `deny`/`typos`/`msrv` jobs + `ci-success` gate job.
6. `[H][30min]` Remove or relocate `IMPLEMENTATION_SUMMARY.md` / `STATUS.md` (decide: delete or move to `docs/`).
7. `[H][5min]` Fix `repository` field if still `anthropics/affidavit`: change to `https://github.com/seanchatmangpt/affidavit`.
8. `[H][30min]` Gitignore compiled LaTeX artifacts under `thesis/` (`*.aux`, `*.pdf`, `*.blg`, `*.log`).

---

## ggen

**Status:** Strongest CI in corpus (45 workflow files) but single MIT license, nightly toolchain, 200+ clippy allows, no typos/editorconfig.

### Automatic fixes (`apply.sh .`)

1. `[A][5min]` Run `apply.sh <ggen>`. New files dropped: `typos.toml`, `.editorconfig`. Files that will be **skipped** (already present): `deny.toml`, `rustfmt.toml`, `SECURITY.md`, `ci.yml`, `release.yml`, `dependabot.yml`.
2. `[A][10min]` Add `[workspace.lints]` block to root `Cargo.toml` and `lints.workspace = true` to all 15+ member `Cargo.toml` files. Template block at `praxis/template/Cargo.toml`. Run `cargo check --workspace` to validate.
3. `[A][5min]` Add `LICENSE-APACHE` file: `cp praxis/template/LICENSE-APACHE <ggen>/LICENSE-APACHE`.

### Human judgment required

1. `[H][4h]` Consolidate 45 CI workflow files → 3 canonical files (`ci.yml`, `release.yml`, `bench.yml`). The praxis `ci.yml` from `apply.sh` can be the starting point. Audit the existing sprawl for any jobs not covered (weaver validation, build matrix). Keep only what is not duplicated.
2. `[H][30min]` License: change `license = "MIT"` → `"MIT OR Apache-2.0"` in all workspace member `Cargo.toml` files. Add `[workspace.package] license = "MIT OR Apache-2.0"` so members inherit.
3. `[H][2h]` Toolchain: migrate to pinned stable for all non-fmt jobs. The current `nightly-2026-04-15` is needed only for rustfmt nightly options. Pattern: use `dtolnay/rust-toolchain@stable` for clippy/test/docs; `dtolnay/rust-toolchain@nightly` for fmt job only.
4. `[H][4h]` Resolve the 200+ `#[allow(clippy::...)]` overrides ("Phase B.1 warn-first"). Work through them methodically; inline `#[allow]` at the call site with justification comment, or fix the underlying issue.

---

## clnrm

**Status:** Second most elaborate CI (29 files), MIT-only, no workspace.lints, no CLAUDE.md. Has the most rigorous `deny.toml` in corpus.

### Automatic fixes (`apply.sh .`)

1. `[A][5min]` Run `apply.sh <clnrm>`. New files dropped: `typos.toml`, `.editorconfig`, `SECURITY.md` (if absent), `dependabot.yml`. Files skipped (already present): `deny.toml`, `rustfmt.toml`, `ci.yml` (custom; do not overwrite — see human section).
2. `[A][5min]` Add `LICENSE-APACHE`: `cp praxis/template/LICENSE-APACHE <clnrm>/LICENSE-APACHE`.
3. `[A][15min]` Add `[workspace.lints]` to root `Cargo.toml`. Add `lints.workspace = true` to all 4 member `Cargo.toml` files (`clnrm`, plus members). Run `cargo check --workspace`.
4. `[A][10min]` Add `CLAUDE.md` from template: `cp praxis/template/CLAUDE.md <clnrm>/CLAUDE.md`. Edit project-specific sections (Architecture, Key Concepts, Dev Workflow).
5. `[A][10min]` Add `CONTRIBUTING.md`: `cp praxis/template/CONTRIBUTING.md <clnrm>/CONTRIBUTING.md`.

### Human judgment required

1. `[H][4h]` Consolidate 29 CI workflow files → 3 canonical files (`ci.yml`, `release.yml`, optionally `integration.yml`). The praxis `ci.yml` is the baseline. Preserve the fuzz and weaver jobs in a third optional workflow file.
2. `[H][30min]` License: `license = "MIT"` → `"MIT OR Apache-2.0"` in all member `Cargo.toml` files. Add `[workspace.package] license` so members can inherit.
3. `[H][1h]` Toolchain: `nightly-2026-04-15` → pinned stable. Identify which exact nightly features are used; move them to feature flags or pin nightly only for fmt.
4. `[H][1h]` Close or triage the 13 open issues that are self-filed `todo!()` stub admissions. The `deny(todo)` lint covers them if wired via `[workspace.lints]`.

---

## clnrm_prototype

**Status:** Leanest of the clnrm cluster. Has `[lints.clippy]` already. Wrong repo URL.

### Automatic fixes (`apply.sh .`)

1. `[A][5min]` Run `apply.sh <clnrm_prototype>`. New files dropped: `deny.toml`, `typos.toml`, `.editorconfig`, `SECURITY.md`, `rust-toolchain.toml`, `dependabot.yml`. File skipped (already present): `rustfmt.toml` (if present).
2. `[A][5min]` Fix `repository` URL in `Cargo.toml`: `https://github.com/sac/ggen` → `https://github.com/seanchatmangpt/clnrm_prototype`. Add `homepage = "https://github.com/seanchatmangpt/clnrm_prototype"`.
3. `[A][5min]` In the CI workflow, upgrade `actions/cache@v3` → `Swatinem/rust-cache@v2` (simpler and more correct). Replace the manual key construction.
4. `[A][5min]` Add `ci-success` gate job to `.github/workflows/ci.yml` (copy the `ci-success` stanza from `praxis/template/.github/workflows/ci.yml`).

### Human judgment required

1. `[H][30min]` Decide MSRV: `rust-version` is absent; add `rust-version = "1.82"`. Verify build passes. The CI `msrv` job in the template will test this.
2. `[H][15min]` The `[lints.clippy]` block is already present. Reconcile it with the house `[workspace.lints]` shape from `template/Cargo.toml` (single-crate: use `[lints]` not `[workspace.lints]`).

---

## clap-noun-verb

**Status:** Most conformant repo in corpus. Closest to house style already. Key remaining issues: MSRV 1.74 vs house 1.82, `proc-macro-error` RUSTSEC advisory, no dependabot.

### Automatic fixes (`apply.sh .`)

1. `[A][5min]` Run `apply.sh <clap-noun-verb>`. Most files already exist; expect **SKIPPED** for: `deny.toml`, `typos.toml`, `.editorconfig`, `rustfmt.toml`, `SECURITY.md`, `ci.yml`, `release.yml`. Only `dependabot.yml` may be new (absent per survey).
2. `[A][5min]` Add `dependabot.yml`: it will be dropped by `apply.sh` if absent.

### Human judgment required

1. `[H][30min]` Fix RUSTSEC-2024-0370: `proc-macro-error` is unmaintained. In `Cargo.toml` (macros crate), replace `proc-macro-error = "1"` with `proc-macro-error2 = "1"` (drop-in maintained fork) or `manyhow` (modern replacement). Run `cargo deny check` to verify advisory is cleared.
2. `[H][1h]` Bump MSRV 1.74 → 1.82 (macros: 1.70 → 1.82). Update `rust-version` in all member `Cargo.toml` files. Run `cargo check` on Rust 1.82 to confirm. Then update the CI `msrv` job to test 1.82.
3. `[H][30min]` Migrate `thiserror 1` → `thiserror 2` in all members. `thiserror 2` is backwards-compatible for most usages; verify with `cargo test`.
4. `[H][15min]` `unsafe_code = "allow"` in `[workspace.lints]`: either add a justification comment explaining which code needs unsafe, or change to `"warn"` if `linkme` is the only reason (linkme requires `unsafe` internally but this can be allowed per-call-site).

---

## lsp-max

**Status:** Excellent workspace.package / workspace.dependencies. Missing: [workspace.lints], deny.toml, typos.toml, .editorconfig, SECURITY.md. Nightly toolchain but CI uses stable.

### Automatic fixes (`apply.sh .`)

1. `[A][5min]` Run `apply.sh <lsp-max>`. New files dropped: `deny.toml`, `typos.toml`, `.editorconfig`, `SECURITY.md`, `dependabot.yml` (verify; daily at 21:00 already present per survey). Files skipped (already present): `rustfmt.toml`, `ci.yml`, `release.yml`.
2. `[A][15min]` Add `[workspace.lints]` block to root `Cargo.toml`. Add `lints.workspace = true` to all 34 member `Cargo.toml` files. Run `cargo check --workspace`. Use this shell snippet from the workspace root:
   ```bash
   for f in $(find . -name 'Cargo.toml' -not -path '*/target/*'); do
     grep -q 'lints.workspace' "$f" && continue
     grep -q '^\[package\]' "$f" || continue
     printf '\n[lints]\nworkspace = true\n' >> "$f"
   done
   ```
3. `[A][5min]` In the just-dropped `deny.toml`, add BUSL-1.1 exception blocks for any BUSL-licensed path or git deps pulled from sibling repos (`wasm4pm`, `dteam`, `miniml`). Edit `deny.toml`:
   ```toml
   [[licenses.exceptions]]
   allow = ["BUSL-1.1"]
   name = "wasm4pm"
   version = "*"
   ```

### Human judgment required

1. `[H][1h]` CI: `rust-toolchain.toml` pins `nightly-2026-04-15` but CI jobs use `dtolnay/rust-toolchain@stable`. Reconcile: either update `rust-toolchain.toml` to `channel = "1.82.0"` (if nightly is only needed for fmt) or document why nightly is required. The multi-checkout pattern (sibling repos) makes this non-trivial — verify siblings build on stable first.
2. `[H][30min]` Add `deny` and `typos` jobs to `ci.yml` (already templated in `praxis/template/.github/workflows/ci.yml`). The `ci-success` gate job is already present (named `result`); ensure it covers the new jobs.
3. `[H][30min]` The `authors` field in `[workspace.package]` reads `Eyal Kalderon <ebkalderon@gmail.com>` (upstream author). Change to `Sean Chatman <xpointsh@gmail.com>`.

---

## cargo-cicd

**Status:** Best-in-class CI tooling (4 workflows, pinned toolchain, weekly audit). Missing: rustfmt.toml, deny.toml, typos.toml, .editorconfig, [workspace.lints]. MSRV inconsistency (1.86 root / 1.85 core).

### Automatic fixes (`apply.sh .`)

1. `[A][5min]` Run `apply.sh <cargo-cicd>`. New files dropped: `rustfmt.toml`, `deny.toml`, `typos.toml`, `.editorconfig`, `SECURITY.md` (already present per survey — will skip). Files skipped (already present): `ci.yml`, `release.yml`.
2. `[A][15min]` Add `[workspace.lints]` to root `Cargo.toml`. Add `lints.workspace = true` to `crates/cargo-cicd-core/Cargo.toml` and `crates/cargo-cicd-lsp/Cargo.toml`. Run `cargo check --workspace`.
3. `[A][5min]` Wire `[workspace.package]` for `edition` and `license` so the 3 crates inherit instead of copy-pasting. The `version`, `name`, `description` remain per-crate. Add to root `Cargo.toml`:
   ```toml
   [workspace.package]
   edition = "2021"
   license = "MIT OR Apache-2.0"
   ```
4. `[A][5min]` In the just-dropped `deny.toml`, add BUSL-1.1 exception for any BUSL deps (e.g., dteam, wasm4pm siblings pulled via `[patch.crates-io]`).

### Human judgment required

1. `[H][30min]` Reconcile MSRV: root declares `1.86`, `cargo-cicd-core` declares `1.85`, the `weekly-audit.yml` tests `1.85`. Decide on one value. Recommendation: set `rust-version = "1.86"` everywhere, pin the weekly audit to `1.86`. This requires all deps to be compatible with 1.86.
2. `[H][15min]` Add `authors = ["Sean Chatman <xpointsh@gmail.com>"]` to all three crates (currently absent).
3. `[H][30min]` Add `clippy --workspace --all-targets --all-features` (instead of bare `clippy -- -D warnings`) to the `check-and-test` CI job. The current invocation may miss workspace member lints.

---

## wasm4pm-compat

**Status:** ZERO CI despite being the most sophisticated codebase in the corpus (444 compile-fail + 413 compile-pass trybuild fixtures, Miri, typestate correctness). Highest urgency in Tier 0.

### Automatic fixes (`apply.sh .`)

1. `[A][5min]` Run `apply.sh <wasm4pm-compat>`. All hygiene files dropped: `deny.toml`, `typos.toml`, `.editorconfig`, `SECURITY.md`, `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `.github/dependabot.yml`. `rustfmt.toml` skipped (already present, minimal).
2. `[A][5min]` In the just-dropped `deny.toml`, add BUSL-1.1 exception (this repo's own license is MIT OR Apache-2.0 but it depends on BUSL crates from sibling repos):
   ```toml
   [[licenses.exceptions]]
   allow = ["BUSL-1.1"]
   name = "wasm4pm"
   version = "*"
   ```
3. `[A][5min]` In the just-dropped `ci.yml`, the `trybuild` test suite will run as part of `cargo test --workspace`. No special step needed — `trybuild` fixtures run under `cargo test` automatically.
4. `[A][10min]` Add `CONTRIBUTING.md`: `cp praxis/template/CONTRIBUTING.md <wasm4pm-compat>/CONTRIBUTING.md`.

### Human judgment required

1. `[H][2h]` Toolchain: `rust-toolchain.toml` pins `nightly-2026-05-04`. The trybuild and miri suites likely need nightly. Decide per job: use `dtolnay/rust-toolchain@nightly` for miri job; test whether `cargo test --workspace` (trybuild) passes on stable. If any `#![feature(...)]` annotations are needed in the lib itself, stable is not viable — keep nightly but pin the date in CI explicitly and update it monthly.
2. `[H][30min]` Add the `miri` job from `praxis/template/.github/workflows/miri.yml` as a separate optional workflow (not gating — run on schedule or push to `main`). This preserves the existing local miri setup.
3. `[H][15min]` Add `rust-version` to `Cargo.toml` — currently absent. If staying on nightly, this must be omitted; if moving to stable, set `rust-version = "1.82"`.
4. `[H][30min]` Verify `.cargo/config.toml` does not suppress any lints via `RUSTFLAGS` (per ANTI-1). The survey noted the file is empty; just confirm.

---

## ggen-mcp

**Status:** Wrong package name (`spreadsheet-mcp`), wrong repo URL (`PSU3D0/spreadsheet-mcp`), Apache-only license, edition 2024, no hygiene files. Fork origin needs cleanup.

### Automatic fixes (`apply.sh .`)

1. `[A][5min]` Run `apply.sh <ggen-mcp>`. All hygiene files dropped: `deny.toml`, `typos.toml`, `.editorconfig`, `rustfmt.toml`, `SECURITY.md`, `rust-toolchain.toml`, `.github/dependabot.yml`. Files skipped (already present): `ci.yml`, `release.yml`, `coverage.yml`, `docker.yml`.
2. `[A][5min]` Fix `name = "spreadsheet-mcp"` → `name = "ggen-mcp"` in `Cargo.toml`.
3. `[A][5min]` Fix `repository` → `https://github.com/seanchatmangpt/ggen-mcp`. Fix `homepage` → same.
4. `[A][5min]` Add `keywords` (currently absent). Suggest: `["mcp", "code-generation", "ontology", "sparql", "rust"]`.
5. `[A][5min]` Add `categories` (currently absent). Suggest: `["development-tools", "web-programming"]`.
6. `[A][5min]` Add `readme = "README.md"` and `documentation = "https://docs.rs/ggen-mcp"`.
7. `[A][5min]` Add `authors = ["Sean Chatman <xpointsh@gmail.com>"]` (currently set to upstream author).
8. `[A][10min]` Add `CONTRIBUTING.md`: `cp praxis/template/CONTRIBUTING.md <ggen-mcp>/CONTRIBUTING.md`.
9. `[A][5min]` Remove the 80+ agent-generated `*_SUMMARY.md` / `*_IMPLEMENTATION.md` session artifacts at root. Add to `.gitignore`: `*_SUMMARY.md` and `*_IMPLEMENTATION.md` (root-level only).

### Human judgment required

1. `[H][30min]` License: `Apache-2.0` → `MIT OR Apache-2.0`. Add `LICENSE-MIT`: `cp praxis/template/LICENSE-MIT <ggen-mcp>/LICENSE-MIT`. Rename existing `LICENSE` → `LICENSE-APACHE`.
2. `[H][30min]` Edition: 2024 is used here. Acceptable — leave as-is but document in `CLAUDE.md` which edition-2024 features are required.
3. `[H][1h]` CI: upgrade `actions-rust-lang/setup-rust-toolchain@v1` → `dtolnay/rust-toolchain@stable` and `actions/cache@v4` (manual key) → `Swatinem/rust-cache@v2` in all 4 workflow files.
4. `[H][30min]` Add `[lints]` to `Cargo.toml` (single-crate). Use house lint block from `template/Cargo.toml`.

---

## a2a-rs

**Status:** Deprecated `actions-rs` CI, no task runner, mixed thiserror 1+2, mixed editions (2024 most / 2021 in osiris-compiler), no LICENSE files, MIT-only on most crates.

### Automatic fixes (`apply.sh .`)

1. `[A][5min]` Run `apply.sh <a2a-rs>`. New files dropped: `deny.toml`, `typos.toml`, `.editorconfig`, `rustfmt.toml`, `SECURITY.md`, `rust-toolchain.toml`, `.github/dependabot.yml`. **Overwrite** existing CI: rename `rust.yml` first (`mv .github/workflows/rust.yml .github/workflows/rust.yml.bak`), then run `apply.sh <a2a-rs>` to drop the canonical `ci.yml`.
2. `[A][5min]` Add `LICENSE-MIT`: `cp praxis/template/LICENSE-MIT <a2a-rs>/LICENSE-MIT`. Add `LICENSE-APACHE`: `cp praxis/template/LICENSE-APACHE <a2a-rs>/LICENSE-APACHE`.
3. `[A][10min]` Add `justfile` to root: `cp praxis/template/justfile <a2a-rs>/justfile`. Edit to add workspace-specific recipes for the `ggen/` codegen pipeline.
4. `[A][10min]` Add `[workspace.package]` block hoisting `edition`, `license`, `rust-version`, `authors` so members stop copy-pasting. The `version`, `name`, `description`, `repository`, `keywords`, `categories` remain per-crate.
5. `[A][15min]` Add `[workspace.dependencies]` for the deps shared across all 10 members: `serde`, `serde_json`, `tokio`, `tracing`, `tracing-subscriber`, `uuid`, `chrono`, `anyhow`, `thiserror`. Use `thiserror = "2"` in the workspace dep (see human item 1).
6. `[A][15min]` Add `[workspace.lints]` to root `Cargo.toml`. Add `lints.workspace = true` to all 10 member `Cargo.toml` files.

### Human judgment required

1. `[H][2h]` Unify `thiserror` major version: some members use 1.0, others use 2.0. Pin `thiserror = "2"` in `[workspace.dependencies]`, bump all members to 2.0. `thiserror 2` is mostly source-compatible; check for any `#[from]` annotation changes.
2. `[H][1h]` Reconcile editions: most members are 2024, `osiris-compiler` is 2021. Decide: migrate `osiris-compiler` to 2024 (preferred) or leave it as the sole 2021 outlier with a comment in its `Cargo.toml`. If migrating, run `cargo fix --edition` in that crate.
3. `[H][1h]` Unify `license`: most crates are `MIT`; `a2a-mcp` and `ggen-optimizer` are `MIT OR Apache-2.0`. Upgrade all to `MIT OR Apache-2.0` via `[workspace.package]`.
4. `[H][30min]` Fix `osiris-marketplace` and `a2a-ap2`: currently missing `description` and some `license` fields. Once `[workspace.package]` is wired, these will inherit.
5. `[H][1h]` Clean up root session-summary `.md` files (`CLIENT_ENHANCEMENT_SUMMARY.md`, `IMPLEMENTATION_COMPLETE.md`, `OSIRIS_IMPLEMENTATION_SUMMARY.md`, etc.). Add `*_SUMMARY.md` and `*_IMPLEMENTATION.md` to `.gitignore`.

---

## swarmsh-v2

**Status:** Deprecated `actions-rs` CI, placeholder repo URLs, committed binaries, MIT-only (no LICENSE file), raw Makefile only.

### Automatic fixes (`apply.sh .`)

1. `[A][5min]` Run `apply.sh <swarmsh-v2>`. Files dropped: `deny.toml`, `typos.toml`, `.editorconfig`, `rustfmt.toml`, `SECURITY.md`, `rust-toolchain.toml`, `.github/dependabot.yml`. **Overwrite** CI: rename `ci.yml` to `ci.yml.bak`, then run `apply.sh --force` to drop the canonical `ci.yml` (replacing the deprecated `actions-rs` 11-job variant). The semantic-conventions weaver validation should be preserved in a separate `weaver.yml` workflow.
2. `[A][5min]` Add `LICENSE-MIT`: `cp praxis/template/LICENSE-MIT <swarmsh-v2>/LICENSE-MIT`.
3. `[A][5min]` Add `LICENSE-APACHE`: `cp praxis/template/LICENSE-APACHE <swarmsh-v2>/LICENSE-APACHE`. Update `license = "MIT"` → `"MIT OR Apache-2.0"` in `Cargo.toml`.
4. `[A][5min]` Fix `homepage` and `repository` placeholder URLs: `user/swarmsh-v2` → `https://github.com/seanchatmangpt/swarmsh-v2`.
5. `[A][5min]` Add `rust-version = "1.82"` to `Cargo.toml`.
6. `[A][10min]` Add `justfile` from template. Supplement with `generate` recipe (wraps `otel-weaver forge`) and `export` recipe (existing Makefile targets).
7. `[A][10min]` Add `[lints]` to `Cargo.toml` (single-crate).

### Human judgment required

1. `[H][1h]` Remove committed compiled binaries from repo root: `simple_ollama_demo`, `simple_roberts_demo`, `validate_core_functionality`, `test_telemetry`, `validate_telemetry`, `weaver_demo_simple`. Run `git rm --cached <binaries>` and commit. Add binary names to `.gitignore`.
2. `[H][30min]` Clean up root session docs: 30+ `.md` files (`FMEA_ANALYSIS.md`, `POKA_YOKE_GUIDE.md`, `OUTREACH_CAMPAIGN.md`, `SALES_PAGE.md`, `PRODUCTIZATION.md`, etc.). Keep `README.md`, `README_HONEST.md`, `CLAUDE.md`, `CHANGELOG.md`. Delete or move the rest to `docs/`.
3. `[H][2h]` Consolidate the 11-job CI into 2-3 workflows: `ci.yml` (fmt + clippy + test + deny + typos), `weaver.yml` (semantic-conventions validation), optionally `release.yml`. Install `otel-weaver` via `cargo install --locked` in CI — consider caching the binary with `actions/cache@v4`.
4. `[H][30min]` Remove `src/CLAUDE.md` (CLAUDE.md inside src directory is non-standard). Merge any unique content into root `CLAUDE.md`.

---

## pm4py-rs

**Status:** AGPL-3.0 in `Cargo.toml` but dual-license text in LICENSE file — contradiction. MSRV contradiction (1.85 declared, 1.70 tested). No CLAUDE.md. Root bloat with 80+ AI-generated markdown files.

### Automatic fixes (`apply.sh .`)

1. `[A][5min]` Run `apply.sh <pm4py-rs>`. New files dropped: `deny.toml`, `typos.toml`, `.editorconfig`, `rustfmt.toml`, `rust-toolchain.toml`. Files skipped (already present): `SECURITY.md`, `ci.yml` (test.yml), `release.yml` (publish.yml), `dependabot.yml`.
2. `[A][10min]` Add `CLAUDE.md`: `cp praxis/template/CLAUDE.md <pm4py-rs>/CLAUDE.md`. Edit the Architecture and Key Concepts sections to document the `ggen/` pipeline, PyO3 bridge, and MCP server.
3. `[A][5min]` Add `justfile` from template: `cp praxis/template/justfile <pm4py-rs>/justfile`. It will coexist with the existing Makefile — edit to delegate to `make <target>` for equivalence, or replicate the key Makefile targets.
4. `[A][5min]` Add `[lints]` to `Cargo.toml` (single-crate). **Exception:** `unsafe_code = "forbid"` cannot apply here because of `pyo3`'s FFI; use `unsafe_code = "warn"` instead.
5. `[A][15min]` Remove the 80+ root AI-generated session `.md` files (`00_START_HERE.md`, `ACADEMIC_PUBLICATION_SUMMARY.md`, and the large pile of audit/session reports). Add gitignore patterns for session reports. Only keep: `README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CHANGELOG.md`, `CLAUDE.md`.

### Human judgment required

1. `[H][1h]` **License contradiction (critical):** `Cargo.toml` declares `license = "AGPL-3.0-or-later"` but the LICENSE file contains dual MIT/Apache-2.0 text. Decide which is correct:
   - If AGPL is intentional (binds all Rust FFI wrappers): keep AGPL, remove MIT/Apache text from LICENSE file, update `deny.toml` to allow AGPL-3.0 from own crate.
   - If dual MIT/Apache is correct: change `license = "MIT OR Apache-2.0"` in `Cargo.toml`, add `LICENSE-MIT` and `LICENSE-APACHE` files.
   - Note: the house `deny.toml` blocks AGPL for transitive deps — update the exceptions block if AGPL is kept.
2. `[H][30min]` **MSRV contradiction:** `rust-version = "1.85"` in `Cargo.toml`, but the CI `msrv` job tests `1.70`. Fix the CI msrv job toolchain to `1.85`. Or lower the Cargo.toml MSRV to 1.70 if that is the true minimum.
3. `[H][30min]` Fix `repository` and `homepage` URLs — currently point to a monorepo subtree URL. Change to `https://github.com/seanchatmangpt/pm4py-rs`.
4. `[H][15min]` The `authors` field uses `info@chatmangpt.com` instead of `xpointsh@gmail.com`. Standardize.
5. `[H][2h]` Consider upgrading `thiserror 1` → `thiserror 2` and replacing the `a2a-rs` git dep with a versioned crates.io dep once a2a-rs publishes a stable release.

---

## pm4wasm

**Status:** Zero CI, Apache-only, no task runner. WASM crate with correct size-opt profile already. Missing all hygiene files.

### Automatic fixes (`apply.sh .`)

1. `[A][5min]` Run `apply.sh <pm4wasm>`. All hygiene files dropped: `deny.toml`, `typos.toml`, `.editorconfig`, `rustfmt.toml`, `SECURITY.md`, `rust-toolchain.toml`, `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `.github/dependabot.yml`.
2. `[A][5min]` In the just-dropped `ci.yml`, add a WASM build step after `cargo test`:
   ```yaml
   - name: Build WASM
     run: wasm-pack build --target bundler --release --out-dir js/pkg
   ```
3. `[A][5min]` Verify `[profile.release]` does NOT have `strip = true`. Per BUG-1, `strip = true` corrupts WASM binaries. The existing `opt-level = "s"` is correct; confirm `strip` is absent or explicitly `strip = false`.
4. `[A][5min]` Add `CHANGELOG.md` at repo root: `cp praxis/template/CHANGELOG.md <pm4wasm>/CHANGELOG.md`.
5. `[A][10min]` Add `justfile` with WASM-specific recipes:
   ```just
   build-wasm:
       wasm-pack build --target bundler --release --out-dir js/pkg
   test-wasm:
       wasm-pack test --headless --chrome
   ```
6. `[A][10min]` Add `CONTRIBUTING.md` and `SECURITY.md` (both dropped by apply.sh automatically).
7. `[A][10min]` Fill in missing `Cargo.toml` metadata fields: `repository`, `homepage`, `keywords`, `categories`, `authors`, `rust-version`.

### Human judgment required

1. `[H][30min]` License: `Apache-2.0` → `MIT OR Apache-2.0`. Add `LICENSE-MIT`: `cp praxis/template/LICENSE-MIT <pm4wasm>/LICENSE-MIT`. Rename `LICENSE` → `LICENSE-APACHE`.
2. `[H][30min]` `drift-server/` is a separate crate with its own `Cargo.toml` but is not a workspace member. Decide: (a) convert root + drift-server into a proper workspace — add `[workspace]` block to root, list members, create shared `[workspace.dependencies]` for the shared deps; or (b) leave as two separate Cargo roots.
3. `[H][15min]` Set `rust-version` once a target stable version is chosen. If no nightly features are used, `rust-version = "1.82"` is appropriate.

---

## miniml

**Status:** Zero CI. BSL-1.1 license (possibly intentional). pnpm+turbo monorepo with one Rust crate under `crates/miniml-core/`.

### Automatic fixes (`apply.sh .`)

1. `[A][5min]` Run `apply.sh <miniml>/crates/miniml-core` (apply.sh requires the directory containing `Cargo.toml`). Files dropped inside `crates/miniml-core/`: `deny.toml`, `typos.toml`, `.editorconfig`, `rustfmt.toml`, `rust-toolchain.toml`.
2. `[A][5min]` For GitHub CI, place workflow files at repo root:
   ```bash
   mkdir -p <miniml>/.github/workflows
   cp praxis/template/.github/workflows/ci.yml <miniml>/.github/workflows/ci.yml
   ```
   Then add a `build-ts` job that runs `pnpm install && pnpm turbo build`.
3. `[A][5min]` Add `dependabot.yml`:
   ```bash
   cp praxis/template/.github/dependabot.yml <miniml>/.github/dependabot.yml
   ```
   Edit to add an `npm` ecosystem block for `packages/miniml/`.
4. `[A][5min]` Add `rust-version = "1.82"` to `crates/miniml-core/Cargo.toml` (currently absent; CONTRIBUTING says "1.75+").
5. `[A][5min]` Verify `[profile.release]` in `crates/miniml-core/Cargo.toml` does NOT have `strip = true`. Current profile has `panic = "abort"` only — correct. No action needed.

### Human judgment required

1. `[H][30min]` License review: BSL-1.1 is intentional (business-source). Keep if business decision. If changing to MIT OR Apache-2.0: add both LICENSE files, update `crates/miniml-core/Cargo.toml`. Also update `deny.toml` to allow BSL-1.1 if any sibling depends on miniml.
2. `[H][1h]` CI Rust jobs: the Rust crate is at `crates/miniml-core/`. CI must run with `--manifest-path crates/miniml-core/Cargo.toml` or via a workspace root. Adjust all `cargo` commands in `ci.yml` accordingly.
3. `[H][30min]` The Cargo.toml in `crates/miniml-core/` is not part of a workspace (no root workspace Cargo.toml). For hygiene files and workspace lints to apply cleanly, consider adding a root `Cargo.toml` with `[workspace] members = ["crates/miniml-core"]`.

---

## bcinr

**Status:** Good 3-pipeline CI (ci + bench + miri). Missing: LICENSE files (on disk), deny.toml, rustfmt.toml, typos.toml, .editorconfig, [workspace.lints]. Floating nightly toolchain.

### Automatic fixes (`apply.sh .`)

1. `[A][5min]` Run `apply.sh <bcinr>`. New files dropped: `deny.toml`, `typos.toml`, `.editorconfig`, `rustfmt.toml`. Files skipped (already present): `SECURITY.md` (if present), `ci.yml`, `release.yml` (if present). `dependabot.yml` dropped if absent.
2. `[A][5min]` Add `LICENSE-MIT`: `cp praxis/template/LICENSE-MIT <bcinr>/LICENSE-MIT`. Add `LICENSE-APACHE`: `cp praxis/template/LICENSE-APACHE <bcinr>/LICENSE-APACHE`.
3. `[A][5min]` Add `CONTRIBUTING.md` (absent per survey): `cp praxis/template/CONTRIBUTING.md <bcinr>/CONTRIBUTING.md`. Add `SECURITY.md` if absent: `cp praxis/template/SECURITY.md <bcinr>/SECURITY.md`.
4. `[A][15min]` Add `[workspace.lints]` to root `Cargo.toml`. Add `lints.workspace = true` to all 12 member `Cargo.toml` files. Run `cargo check --workspace`. Note: `crates/wasm4games` needs `unsafe_code = "warn"` not `"forbid"` (WASM FFI).
5. `[A][5min]` Fix `lto = "fat"` → `lto = true` in `[profile.release]`. `"fat"` is non-standard; `true` (equivalent to `"thin"` in most contexts) is the house default.
6. `[A][5min]` Gitignore the mass Python scripts at root if they are generated: add `fix_*.py`, `generate_*.py`, `implement_*.py` to `.gitignore` or move to `tools/python/`.

### Human judgment required

1. `[H][1h]` Toolchain: `rust-toolchain.toml` pins `nightly`. Identify which nightly features are needed. If rustfmt is the only reason, use `dtolnay/rust-toolchain@stable` for all CI jobs except `fmt`. If any crate uses nightly features (e.g., `#![feature(...)]`), list them in `CLAUDE.md`.
2. `[H][30min]` MSRV: currently `rust-version = "1.70"`. The house default is 1.82. Decide: keep 1.70 for maximum compatibility or upgrade to 1.82. Update CI accordingly.
3. `[H][30min]` `wasm4games` and `crates/wasm4games-capi`: verify `strip = false` is set in their release profile (BUG-1 concern). Add explicitly to WASM member profiles if absent.

---

## dteam

**Status:** BUSL-1.1 license, `workspace_lints.toml` exists but is NOT wired into Cargo.toml, SemVer 1.3.0 vs house CalVer, no deny.toml, no typos.toml, no CHANGELOG.md, complex nested structure.

### Automatic fixes (`apply.sh .`)

1. `[A][5min]` Run `apply.sh <dteam>`. New files dropped: `deny.toml`, `typos.toml`, `.editorconfig`, `rustfmt.toml`, `SECURITY.md`, `dependabot.yml`. Files skipped (already present): `rust-toolchain.toml`, existing CI workflows (named differently than `ci.yml`).
2. `[A][20min]` Wire the existing `workspace_lints.toml` into `Cargo.toml`. Move its contents into `[workspace.lints]` in root `Cargo.toml`. Then add `lints.workspace = true` to all 11+ member `Cargo.toml` files:
   ```bash
   # From dteam repo root — update each member
   for f in crates/*/Cargo.toml crates/insa/*/Cargo.toml; do
     grep -q 'lints.workspace' "$f" && continue
     printf '\n[lints]\nworkspace = true\n' >> "$f"
   done
   ```
3. `[A][5min]` Add `CHANGELOG.md` (absent per survey): `cp praxis/template/CHANGELOG.md <dteam>/CHANGELOG.md`.
4. `[A][5min]` Add `authors = ["Sean Chatman <xpointsh@gmail.com>"]` to root `[package]` if absent.
5. `[A][5min]` In the just-dropped `deny.toml`, add BUSL-1.1 exception for first-party BUSL crates (dteam itself is BUSL-1.1):
   ```toml
   [[licenses.exceptions]]
   allow = ["BUSL-1.1"]
   name = "dteam"
   version = "*"
   ```

### Human judgment required

1. `[H][30min]` License review: BUSL-1.1 with expiry to Apache-2.0 on 2029-04-18. This is intentional — do not change unless business decision warrants it. Update `deny.toml` exceptions to allow `BUSL-1.1` for all first-party repos that depend on each other.
2. `[H][30min]` Version scheme: SemVer `1.3.0` vs house CalVer. Recommend switching to CalVer `YY.M.patch` for consistency. This requires a version bump and CHANGELOG entry.
3. `[H][1h]` Toolchain: pins nightly. Identify which nightly features are required specifically for the `ccog-nightly-gate` conformance job. Document in `CLAUDE.md`. Use stable for all other CI jobs.
4. `[H][1h]` Add `[workspace.package]` to hoist `edition`, `license`, `authors` — currently each crate copies them. The nested sub-workspace in `crates/insa/` requires its own `[workspace.package]`.
5. `[H][30min]` Add `keywords` and `categories` to root `Cargo.toml` (currently absent). Suggest: `keywords = ["process-mining", "otel", "conformance", "autonomic", "blake3"]`.

---

## semantic_bit

**Status:** Near-empty repo. No README, no LICENSE, no CI, no CLAUDE.md, no task runner. Empty Cargo.toml metadata. Edition 2024. Zero dependencies.

### Automatic fixes (`apply.sh .`)

1. `[A][5min]` Run `apply.sh <semantic_bit>`. All hygiene files dropped: `deny.toml`, `typos.toml`, `.editorconfig`, `rustfmt.toml`, `rust-toolchain.toml`, `SECURITY.md`, `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `.github/dependabot.yml`.
2. `[A][5min]` Add `LICENSE-MIT`: `cp praxis/template/LICENSE-MIT <semantic_bit>/LICENSE-MIT`. Add `LICENSE-APACHE`: `cp praxis/template/LICENSE-APACHE <semantic_bit>/LICENSE-APACHE`.
3. `[A][10min]` Bootstrap `Cargo.toml` metadata (currently only `name`, `version`, `edition`). Add:
   ```toml
   description = "Semantic field definitions for bounded meaning in Rust — ontology-driven codegen via unrdf.toml"
   repository = "https://github.com/seanchatmangpt/semantic_bit"
   homepage = "https://github.com/seanchatmangpt/semantic_bit"
   license = "MIT OR Apache-2.0"
   authors = ["Sean Chatman <xpointsh@gmail.com>"]
   keywords = ["ontology", "codegen", "sparql", "semantic", "rdf"]
   categories = ["development-tools"]
   rust-version = "1.82"
   ```
4. `[A][5min]` Add `[lints]` block to `Cargo.toml` (single-crate, no workspace needed).
5. `[A][10min]` Add `README.md` from template: `cp praxis/template/README.md <semantic_bit>/README.md`. Edit sections for the `unrdf.toml` / RDF codegen purpose.
6. `[A][10min]` Add `CLAUDE.md` from template: `cp praxis/template/CLAUDE.md <semantic_bit>/CLAUDE.md`. Merge relevant sections from existing `GEMINI.md`.
7. `[A][5min]` Add `CONTRIBUTING.md` and `CHANGELOG.md` from template.
8. `[A][10min]` Add `justfile` from template: `cp praxis/template/justfile <semantic_bit>/justfile`. Supplement with codegen recipe: `gen: unrdf generate`.

### Human judgment required

1. `[H][30min]` Edition 2024 is the only identifier in `Cargo.toml`. Decide: keep 2024 (acceptable for newer crates) or migrate to 2021 (house default). Document the decision.
2. `[H][30min]` Add `[profile.release]` block (currently absent). Use house default: `lto = true`, `codegen-units = 1`, `panic = "abort"`. Only if WASM is a target: add `strip = false`.
3. `[H][30min]` No test suite exists. Add at minimum a `tests/smoke_test.rs` that imports `semantic_bit` and verifies the ontology-generated types compile. This is a prerequisite for the CI `test` job to produce any signal.

---

## mac-artifact-cleaner

**Status:** Zero CI despite a sophisticated sanity.sh DX pipeline. No LICENSE file, no Cargo metadata, sub-crate has its own Cargo.lock (anti-pattern), nightly toolchain.

### Automatic fixes (`apply.sh .`)

1. `[A][5min]` Run `apply.sh <mac-artifact-cleaner>`. All hygiene files dropped: `deny.toml`, `typos.toml`, `.editorconfig`, `rustfmt.toml`, `SECURITY.md`, `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `.github/dependabot.yml`. `rust-toolchain.toml` will be **skipped** (already present).
2. `[A][5min]` Add `LICENSE-MIT`: `cp praxis/template/LICENSE-MIT <mac-artifact-cleaner>/LICENSE-MIT`. Add `LICENSE-APACHE`: `cp praxis/template/LICENSE-APACHE <mac-artifact-cleaner>/LICENSE-APACHE`.
3. `[A][15min]` Bootstrap `Cargo.toml` metadata (root crate: `osx-clnr`). Add:
   ```toml
   description = "macOS artifact and build cache cleaner with OCEL v2 provenance"
   repository = "https://github.com/seanchatmangpt/mac-artifact-cleaner"
   homepage = "https://github.com/seanchatmangpt/mac-artifact-cleaner"
   license = "MIT OR Apache-2.0"
   authors = ["Sean Chatman <xpointsh@gmail.com>"]
   keywords = ["macos", "cleanup", "artifact", "cache", "cli"]
   categories = ["command-line-utilities", "filesystem"]
   rust-version = "1.82"
   ```
4. `[A][10min]` Same metadata fields for `cfab-surface/Cargo.toml`. Hoist common fields via `[workspace.package]`.
5. `[A][10min]` Add `[workspace.lints]` to root `Cargo.toml`. Add `lints.workspace = true` to `cfab-surface/Cargo.toml`. Note: `unsafe_code = "warn"` not `"forbid"` (if `linkme` or `libc` need unsafe).
6. `[A][10min]` Add `CONTRIBUTING.md` and `CHANGELOG.md` from template (both absent per survey).
7. `[A][5min]` Delete `cfab-surface/Cargo.lock` — workspace members must not have their own lockfile: `git rm cfab-surface/Cargo.lock`. Add `cfab-surface/Cargo.lock` to `.gitignore`.
8. `[A][5min]` Remove committed plan/receipt JSON that should be runtime-only: `git rm maintenance-plan.json maintenance-receipt.json`. Add `*-plan.json` and `*-receipt.json` to `.gitignore` (excluding fixture files).

### Human judgment required

1. `[H][1h]` Toolchain: nightly — for what features? Check `src/lib.rs` for `#![feature(...)]`. If only rustfmt nightly options are needed, switch to `channel = "1.82.0"` in `rust-toolchain.toml` and use `dtolnay/rust-toolchain@nightly` only for the `fmt` CI job.
2. `[H][30min]` Justfile is a facade over Makefile: every recipe calls `make <target>`. Consolidate: either expand the Justfile to be self-contained, or remove it (keep only the Makefile). Having both adds confusion.
3. `[H][30min]` `affidavit` is pulled as a git dep. Once affidavit publishes to crates.io, switch to a versioned dep: `affidavit = { version = "26.6", features = ["core"], default-features = false }`.
4. `[H][30min]` The `ui/` Next.js app is in the same repo but not mentioned in `dependabot.yml`. Add an `npm` ecosystem block to the just-dropped `dependabot.yml`.

---

## wasm4pm _(second-wave)_

**Status:** BUSL-1.1, nightly toolchain, lints suppressed via `.cargo/config.toml` RUSTFLAGS (ANTI-1), no `[workspace.lints]`, no justfile. Sophisticated WASM profiles and ed25519 signing.

### Automatic fixes (`apply.sh .`)

1. `[A][5min]` Run `apply.sh <wasm4pm>`. New files dropped: `deny.toml`, `typos.toml`, `.editorconfig`, `rustfmt.toml`, `SECURITY.md`, `.github/dependabot.yml`. Files skipped (already present — verify): `rust-toolchain.toml`, `ci.yml`.
2. `[A][5min]` In the just-dropped `deny.toml`, add BUSL-1.1 exception for this repo:
   ```toml
   [[licenses.exceptions]]
   allow = ["BUSL-1.1"]
   name = "wasm4pm"
   version = "*"
   ```
3. `[A][15min]` Add `[workspace.lints]` to root `Cargo.toml`. Add `lints.workspace = true` to all workspace members. **Critical:** remove the `[build] rustflags = ["-A", "clippy::all"]` line from `.cargo/config.toml` (ANTI-1). Move any necessary `#[allow(clippy::...)]` to `[workspace.lints.clippy]` with justification comments.
4. `[A][5min]` Add `justfile` from template; supplement with WASM recipes from `template-wasm/justfile`:
   ```just
   build-wasm profile="release":
       wasm-pack build --target web --{{profile}}
   wasm-size:
       twiggy top pkg/*.wasm
   ```
5. `[A][5min]` Verify all 5 WASM profiles (`mobile-wasm`, `standard-wasm`, `performance-wasm`, `analytics-wasm`, `cloud-wasm`) have `strip = false` explicitly set (BUG-1 concern).

### Human judgment required

1. `[H][30min]` BUSL-1.1 is intentional. Confirm the license expiry date and target open-source date. Document in `SECURITY.md` and `README.md`.
2. `[H][1h]` Nightly toolchain: document in `CLAUDE.md` which exact nightly features are required. Consider using stable for non-nightly-feature code paths and feature-flagging the nightly-only modules.
3. `[H][1h]` Add `[workspace.package]` to hoist `edition`, `license`, `rust-version` across all members.
4. `[H][1h]` Consider adding the `ed25519-dalek` signed receipt pattern to `chatman-common::provenance` (feature `"signed-receipts"`) so other repos can reuse it without copying.

---

## chicago-tdd-tools _(second-wave)_

**Status:** MIT-only, nightly toolchain, cargo-make (not just), no deny.toml, no typos.toml. Rich testkit patterns (TestState, TestReceipt, thermal classification) that belong in `chatman-common`.

### Automatic fixes (`apply.sh .`)

1. `[A][5min]` Run `apply.sh <chicago-tdd-tools>`. New files dropped: `deny.toml`, `typos.toml`, `.editorconfig`, `rustfmt.toml`, `SECURITY.md`, `.github/dependabot.yml`. Files skipped (already present): `rust-toolchain.toml`, `ci.yml`.
2. `[A][5min]` Add `LICENSE-APACHE`: `cp praxis/template/LICENSE-APACHE <chicago-tdd-tools>/LICENSE-APACHE`. Update `license = "MIT"` → `"MIT OR Apache-2.0"` in `Cargo.toml`.
3. `[A][15min]` Add `[workspace.lints]` (if workspace) or `[lints]` (if single-crate) to `Cargo.toml`. Add `lints.workspace = true` to all members.
4. `[A][5min]` Add `justfile` from template. Supplement with `unwrap-check: cargo unwrap-check --workspace` recipe.

### Human judgment required

1. `[H][30min]` Toolchain: nightly for what features? Check `src/` for `#![feature(...)]`. If only for rustfmt, switch core CI jobs to stable.
2. `[H][1h]` Migrate `TestState<Phase>`, `TestReceipt`, `assert_fail!`, `docker_retry`, `TestOutput` trait into `chatman-common::testkit` (feature `"testkit"`). Leave chicago-tdd-tools as the original but add it as a dev-dep in the praxis template.
3. `[H][30min]` MSRV: set `rust-version` (currently absent). Recommend 1.82.
4. `[H][30min]` The 85% coverage threshold via `cargo-tarpaulin` is a good pattern. Consider adding `tarpaulin` as an optional CI job in the praxis `ci.yml` template.

---

## dtr _(second-wave — Java / Maven)_

**Status:** Java + Maven, not a Rust project. Cross-language. No Cargo applicability.

### Notes

- dtr is a Java 26 (preview APIs: `StructuredTaskScope`) Maven project. `apply.sh` is not applicable.
- The `DocEvent` / `DocContext` / `say*()` patterns from dtr are the inspiration for `chatman-common::testkit` feature `"living-docs"` (see `01-SECOND-WAVE.md §3.8-3.11`).
- **Action for praxis (not for the dtr repo):** Implement the `DocEvent` enum, `DocContext` struct, and `doc_assert!` macro in `crates/chatman-common/src/testkit/living_docs.rs` behind `feature = "living-docs"`.
- Apache-2.0 only license in dtr — no Rust action needed.

---

## gitvan _(second-wave — JavaScript / TypeScript)_

**Status:** Pure JavaScript/TypeScript, not a Rust project. Cross-language. No Cargo applicability.

### Notes

- gitvan is a JavaScript/TypeScript repo. `apply.sh` is not applicable.
- The git-as-runtime patterns (CAS locks via `git update-ref`, NDJSON audit ledger via `git notes append`) should be documented in praxis README under "Provenance Patterns" and implemented as a Rust helper module `chatman_common::git_lock`.
- **Actions for praxis (not for the gitvan repo):**
  1. Add `chatman_common::git_lock` module (wraps `std::process::Command` for `git update-ref` + `git notes`).
  2. Add `just receipt-commit` recipe to `praxis/template/justfile`.
  3. Add `template/ontology/workflow.ttl` as a Turtle skeleton for workflow definitions.

---

## Cross-Cutting Automatic Scripts

These shell one-liners can be run against any cloned repo to apply mechanical fixes not covered by `apply.sh`.

### Fix deprecated actions-rs in any CI file

```bash
# Replace actions-rs/toolchain@v1 with dtolnay/rust-toolchain@stable
find .github/workflows -name '*.yml' -exec sed -i \
  's|actions-rs/toolchain@v1|dtolnay/rust-toolchain@stable|g' {} \;

# Upgrade actions/cache@v3 to v4
find .github/workflows -name '*.yml' -exec sed -i \
  's|actions/cache@v3|actions/cache@v4|g' {} \;
# Note: actions-rs/cargo@v1 requires manual replacement with direct `run:` steps
```

### Add lints.workspace = true to all member crates

```bash
# From workspace root
find . -name 'Cargo.toml' -not -path '*/target/*' | while read f; do
  grep -q 'lints.workspace' "$f" && continue
  grep -q '^\[package\]' "$f" || continue  # skip workspace root
  printf '\n[lints]\nworkspace = true\n' >> "$f"
done
```

### Remove session-summary markdown artifacts from root

```bash
# Dry run: list files that look like AI session artifacts at root
find . -maxdepth 1 -name '*.md' | grep -E \
  '(SUMMARY|IMPLEMENTATION|COMPLETE|ANALYSIS|CAMPAIGN|PRODUCTIZATION)' | sort

# After review, remove tracked copies:
# git rm <listed files>

# Prevent future commits:
cat >> .gitignore <<'EOF'
*_SUMMARY.md
*_IMPLEMENTATION.md
*_COMPLETE.md
AGENT_*.md
EOF
```

### Verify no WASM crate inherits strip = true

```bash
# From any repo with WASM crates
if grep -r 'strip = true' Cargo.toml */Cargo.toml 2>/dev/null; then
  echo "WARN: strip = true found — remove for WASM crates (see BUG-1 in 01-SECOND-WAVE.md)"
else
  echo "OK: no strip = true in Cargo.toml files"
fi
```

### Verify .cargo/config.toml does not suppress lints (ANTI-1)

```bash
if grep -r 'RUSTFLAGS.*-A\|rustflags.*-A' .cargo/config.toml 2>/dev/null; then
  echo "ANTI-1: RUSTFLAGS in .cargo/config.toml suppresses lints — move to [workspace.lints]"
fi
```

---

*Sources: `survey/00-SYNTHESIS.md`, `survey/01-SECOND-WAVE.md`, individual survey reports in `survey/`, `CHECKLIST.md`, and `apply.sh` source. All repo-specific findings are cross-referenced against the coverage matrix in `00-SYNTHESIS.md §1`.*
