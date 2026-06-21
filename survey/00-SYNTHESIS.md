# 00 — Consolidated Rust Boilerplate Synthesis

**Owner:** `seanchatmangpt` · **Scope:** ~18 readable repos (8 private/unreadable) · **Goal:** one house-style boilerplate to refactor toward.

This spec is derived strictly from the 10 per-repo survey reports in this directory. Where the surveys are silent, it is flagged `[best-practice default]`. Decisions are concrete and ranked.

---

## 1. Coverage Matrix

One row per individual repo (multi-repo survey files split into constituents). Legend: ✓ present · ✗ absent · `—` N/A · values inline. Docs column = R(EADME)/CL(AUDE.md)/CO(NTRIBUTING)/SE(CURITY)/CH(ANGELOG), each ✓/✗.

| Repo | pub/priv | ws/single | ed | MSRV/toolchain | version | CI (jobs) | rustfmt | clippy/[lints] | deny | typos | editorcfg | license | task-runner | docs R/CL/CO/SE/CH |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **affidavit** | pub | single | 2021 | 1.78 / nightly(unpinned) | CalVer 26.6.17 | 2 (rust, web) — all `continue-on-error` | ✗ | ✗/✗ | ✗ | ✗ | ✓ | MIT OR Apache | just | ✓/✓/✓/✗/✓ |
| **ggen** | pub | ws (15+) | 2021 | none / nightly-2026-04-15 | CalVer 26.6.DD | 45 files (ci, build, lint, test, release, sec…) | ✓ | warn-mostly/✓ ws.lints | ✓ (loose) | ✗ | ✗ | MIT | just + Makefile.toml | ✓/✓/✓/✓/✓ |
| **clnrm** | pub | ws (4) | 2021 | none / nightly-2026-04-15 | CalVer 26.5.28 | 29 files (ci, fuzz, release, weaver…) | ✓ (empty) | scripts/✗ | ✓ (detailed) | ✗ | ✗ | MIT | Makefile.toml | ✓/✗/✗/✓/✓ |
| **clnrm_prototype** | pub | single | 2021 | none / stable | SemVer 0.2.0 | 3 (ci, release, nightly) | ✗ | deny/✓ [lints.clippy] | ✗ (CI runs deny, no file) | ✗ | ✗ | MIT | Makefile.toml | ✓/✗/✓/✗/✓ |
| **clap-noun-verb** | pub | ws (3+) | 2021 | 1.74 (macros 1.70) / stable | CalVer 26.6.14 | 8 (ci, release, audit, perf, frontier…) | ✓ | warn/✓ ws.lints | ✓ (detailed) | ✓ | ✓ | MIT OR Apache | Makefile.toml + Justfile | ✓/✓/✓/✓/✓ |
| **lsp-max** | pub | ws (34) | 2021 | 1.82 / nightly-2026-04-15 | CalVer 26.6.18 | 2 (ci, release) | ✓ | -Dwarn CLI/✗ | ✗ | ✗ | ✗ | MIT OR Apache | just | ✓/✓/✓/✗/✓ |
| **cargo-cicd** | pub | ws (3) | 2021 | 1.86 root/1.85 core / stable@1.94.1 pin | CalVer 26.6.2 | 4 (ci, pr-checks, release, weekly-audit) | ✗ | -Dwarn CLI/✗ | ✗ | ✗ | ✗ | MIT OR Apache | Makefile.toml | ✓/✓/✓/✓/✓ |
| **wasm4pm-compat** | pub | single | 2021 | none / nightly-2026-05-04 | CalVer 26.6.14 | **0** (no workflows) | ✓ (min) | comment/✗ | ✗ | ✗ | ✗ | MIT OR Apache | just + Makefile.toml | ✓/✓/✗/✓/✓ |
| **ggen-mcp** | pub | single | **2024** | none / stable | SemVer 1.0.0 | 4 (ci, release, coverage, docker) | ✗ | ✗/✗ | ✗ | ✗ | ✗ | **Apache only** | Makefile.toml | ✓/✓/✗/✗/✓ |
| **a2a-rs** | pub | ws (10) | **2024** (1 member 2021) | 1.85 / none | SemVer 0.1.0 (1 at 0.3.0) | 1 (rust.yml, 4 jobs) — deprecated `actions-rs` | ✗ | -Dwarn CLI/✗ | ✗ | ✗ | ✗ | MIT (2 dual) | **none** | ✓/✓/✗/✗/✓ |
| **swarmsh-v2** | pub | single | 2021 | none / none | SemVer 2.1.0 | 1 (ci.yml, 11 jobs) — deprecated `actions-rs` | ✗ | -Dwarn CLI/✗ | ✗ | ✗ | ✗ | MIT (no file) | Makefile (raw) | ✓/✓/✗/✗/✓ |
| **pm4py-rs** | pub | single | 2021 | 1.85 (CI tests 1.70) / stable | CalVer 2026.3.28 | 3 (test, publish, security) | ✗ | -Dwarn CLI/✗ | ✗ (CI action only) | ✗ | ✗ | **AGPL-3.0** | Makefile (raw) | ✓/✗/✓/✓/✓ |
| **pm4wasm** | pub | 2 crates | 2021 | none / none | SemVer 0.1.0 | **0** | ✗ | ✗/✗ | ✗ | ✗ | ✗ | **Apache only** | none | ✓/✓/✗/✗/✗ |
| **miniml** | pub | monorepo (1 crate) | 2021 | 1.75 (doc only) / none | CalVer 26.4.8 | **0** (FUNDING.yml only) | ✗ | ✗/✗ | ✗ | ✗ | ✗ | **BSL-1.1** | pnpm+turbo | ✓/✓/✓/✓/✓ |
| **bcinr** | pub | ws (12) | 2021 | 1.70 / nightly | CalVer 26.6.x | 3 (ci, bench, miri) | ✗ | -Dwarn CLI/✗ | ✗ (CI runs deny) | ✗ | ✗ | MIT OR Apache (no file) | Makefile.toml + Makefile | ✓/✓/✗/✗/✓ |
| **dteam** | pub | ws (12+nested) | 2021 | none / nightly (+pin for gate) | SemVer 1.3.0 | 2 (rust-dod, matrix-tv) | ✗ | `workspace_lints.toml`(unwired)/✗ | ✗ | ✗ | ✗ | **BSL-1.1** | Makefile.toml + just(nested) | ✓/✓/✓/✗/✗ |
| **semantic_bit** | pub | single | **2024** | none / none | SemVer 0.1.0 | **0** (no .github) | ✗ | ✗/✗ | ✗ | ✗ | ✗ | **none** | none | ✗/✗/✗/✗/✗ |
| **mac-artifact-cleaner** | pub | ws (2) | 2021 | none / nightly | SemVer 0.1.0 | **0** | ✗ | -Dwarn CLI/✗ | ✗ | ✗ | ✗ | **none** | Makefile + Justfile | ✓/✓/✗/✗/✗ |

`clnrm_prototype` is folded in from `clnrm.md`. Total readable rows: **18**.

---

## 2. De-facto House Style (already consistent)

These are the points where the corpus already agrees — the boilerplate should **lock them in, not invent them**.

- **Edition 2021** — 14/18 repos. Only `ggen-mcp`, `a2a-rs`, `semantic_bit` use 2024 (the three newest).
- **Dual `MIT OR Apache-2.0`** is the modal intent: affidavit, clap-noun-verb, lsp-max, cargo-cicd, wasm4pm-compat, bcinr (6 explicit dual). The single-license outliers (`ggen`/`clnrm` MIT, `ggen-mcp`/`pm4wasm` Apache, `pm4py-rs` AGPL, `miniml`/`dteam` BSL-1.1, `semantic_bit`/`swarmsh-v2`/`a2a-rs`/`mac-artifact-cleaner` none-or-MIT) are divergences to reconcile.
- **CalVer `YY.M.patch`** is the house version scheme where deliberate: affidavit, ggen, clnrm, clap-noun-verb, lsp-max, cargo-cicd, wasm4pm-compat, bcinr, pm4py-rs, miniml (10). SemVer survives only in prototypes/forks/early crates.
- **Recurring core deps** (near-universal): `serde` 1 + `serde_json` 1 (every Rust crate), `anyhow` 1, `thiserror` (1 or 2), `tokio` 1 (async repos), `tracing` 0.1 + `tracing-subscriber` 0.3, `blake3` 1 (provenance repos: affidavit, lsp-max, bcinr, dteam, wasm4pm-compat, mac-artifact-cleaner), `clap` 4 (derive), `criterion` 0.5 (benches), `proptest`/`insta` (tests), `uuid` 1, `chrono` 0.4.
- **`linkme` 0.3** for zero-registration plugin/verb discovery: affidavit, clap-noun-verb, clnrm, mac-artifact-cleaner.
- **`clap-noun-verb` + `ggen.toml` ecosystem** — the signature house pattern. `clap-noun-verb` is consumed by affidavit, cargo-cicd, mac-artifact-cleaner (+ referenced in ggen). `ggen.toml` (RDF/Turtle ontology → SPARQL → Tera → generated Rust) appears in affidavit, ggen, clnrm, clap-noun-verb, cargo-cicd, wasm4pm-compat (`ggen-witness.toml`), ggen-mcp, pm4py-rs, a2a-rs. `unrdf.toml` is the same idea in semantic_bit.
- **OpenTelemetry / semconv** as a first-class concern: ggen, clnrm, lsp-max, swarmsh-v2, pm4py-rs, ggen-mcp, dteam, plus affidavit's `semconv/` + `otel` feature.
- **`CLAUDE.md` as mandatory dev runbook** — 15/18 (absent only in clnrm, clnrm_prototype, pm4py-rs, semantic_bit). Companion `AGENTS.md`/`GEMINI.md` multi-LLM docs recur (lsp-max, cargo-cicd, bcinr, dteam, a2a-rs, mac-artifact-cleaner, semantic_bit).
- **`[profile.release]` hardening** where set: `lto`, `codegen-units = 1`, `panic = "abort"`, `strip` — affidavit and bcinr identical in spirit; WASM crates use `panic = "abort"` + size opt.
- **CI toolchain action** where modern: `dtolnay/rust-toolchain` (ggen, clnrm, clap-noun-verb, lsp-max, cargo-cicd, pm4py-rs, bcinr, dteam). `Swatinem/rust-cache@v2` is the modal cache action.
- **`Cargo.lock` committed** everywhere (binary/app convention).

---

## 3. Divergences & Inconsistencies (need reconciling)

- **Edition split:** 2021 (14) vs 2024 (3: ggen-mcp, a2a-rs, semantic_bit). a2a-rs is *internally* split (osiris-compiler 2021, rest 2024).
- **MSRV spread:** declared values are 1.70 (bcinr, clap-noun-verb-macros), 1.74 (clap-noun-verb), 1.75 (miniml doc), 1.78 (affidavit), 1.82 (lsp-max), 1.85 (cargo-cicd-core, a2a-rs, pm4py-rs), 1.86 (cargo-cicd root). 8 repos set **no** MSRV. pm4py-rs declares 1.85 but CI tests 1.70 — a contradiction. cargo-cicd declares 1.86 root / 1.85 core in the same workspace.
- **Toolchain channel:** nightly-pinned (ggen, clnrm, lsp-max, wasm4pm-compat dates differ: `2026-04-15` vs `2026-05-04`), nightly-unpinned (affidavit, bcinr, dteam, mac-artifact-cleaner), or stable/none (clnrm_prototype, clap-noun-verb, cargo-cicd, ggen-mcp, a2a-rs, swarmsh-v2, pm4py-rs, pm4wasm, miniml, semantic_bit). No single answer.
- **thiserror 1 vs 2:** thiserror 2 → affidavit, ggen, lsp-max, cargo-cicd, dteam. thiserror 1 → clap-noun-verb, ggen-mcp, pm4wasm, pm4py-rs, mac-artifact-cleaner(cfab). a2a-rs mixes **both majors inside one workspace**.
- **Lint config presence:** `[workspace.lints]`/`[lints]` exists in only 3 (ggen, clap-noun-verb ws.lints; clnrm_prototype [lints.clippy]). dteam has `workspace_lints.toml` **not wired** into Cargo.toml (declared, unenforced). Everyone else passes `-D warnings` inline in CI or has nothing.
- **CI shape:** ranges from **0 workflows** (wasm4pm-compat, pm4wasm, miniml, semantic_bit, mac-artifact-cleaner) → 1 deprecated `actions-rs` file (a2a-rs, swarmsh-v2) → clean 2–4 (affidavit, lsp-max, clnrm_prototype, cargo-cicd, ggen-mcp, pm4py-rs, bcinr) → 29–45 sprawling (clnrm, ggen). Caching: `Swatinem/rust-cache@v2` (modal) vs raw `actions/cache@v4` (clnrm_prototype, dteam, ggen-mcp) vs `actions/cache@v3` (a2a-rs, swarmsh-v2, deprecated). OS matrix: most ubuntu-only; clnrm/clnrm_prototype/pm4py-rs/bcinr add macOS/Windows. cargo-deny in CI: clnrm, clnrm_prototype, clap-noun-verb, pm4py-rs, bcinr only. Release workflow: present in ~9, absent in 9.
- **Workspace inheritance:** only ggen, clnrm (partial), lsp-max use `[workspace.package]`. clap-noun-verb, cargo-cicd, a2a-rs, bcinr, dteam, mac-artifact-cleaner have multi-crate workspaces but **copy-paste** edition/license/version per member. `[workspace.dependencies]` is more common than `[workspace.package]`.
- **Version scheme:** CalVer (10) vs SemVer (clnrm_prototype, ggen-mcp, a2a-rs, swarmsh-v2, dteam, semantic_bit, pm4wasm, mac-artifact-cleaner). Even CalVer disagrees on shape: `YY.M.patch` (affidavit, cnv, lsp-max) vs `YY.M.DD` (ggen, clnrm) vs `YYYY.M.DD` (pm4py-rs).
- **LICENSE file presence:** declared dual but **no LICENSE file at all**: bcinr, a2a-rs, swarmsh-v2 (license in Cargo.toml metadata only), mac-artifact-cleaner & semantic_bit (no field either). pm4py-rs's LICENSE file text mismatches its Cargo.toml (`AGPL` field, dual text in file).
- **Wrong / placeholder repository URLs:** affidavit → `github.com/anthropics/affidavit` (should be `seanchatmangpt`). clnrm_prototype → `github.com/sac/ggen` (wrong repo). swarmsh-v2 → `user/swarmsh-v2` placeholder. ggen-mcp → `PSU3D0/spreadsheet-mcp` (fork origin, package name `spreadsheet-mcp` ≠ repo). pm4py-rs → monorepo subtree URL, not standalone.
- **Task runner:** just (affidavit, lsp-max), Makefile.toml/cargo-make (clnrm, clnrm_prototype, cargo-cicd, ggen-mcp, dteam, bcinr), both (ggen, clap-noun-verb, wasm4pm-compat, mac-artifact-cleaner), raw Makefile (swarmsh-v2, pm4py-rs), pnpm+turbo (miniml), none (a2a-rs, pm4wasm, semantic_bit). No house standard.

---

## 4. Gaps (missing almost everywhere)

Quantified across the 18 readable repos:

- **`deny.toml`: 15/18 missing.** Present only in ggen, clnrm, clap-noun-verb. (bcinr, clnrm_prototype, pm4py-rs run `cargo deny`/audit in CI with **no config file**.)
- **`typos.toml`: 17/18 missing.** Present only in clap-noun-verb.
- **`.editorconfig`: 16/18 missing.** Present only in clap-noun-verb and **affidavit** (survey under-counted this — verified present in repo).
- **`[workspace.lints]`/`[lints]`: 15/18 missing or unenforced.** Real ones: ggen, clap-noun-verb, clnrm_prototype. dteam has an unwired `workspace_lints.toml`.
- **`SECURITY.md`: 11/18 missing.** Present: ggen, clnrm, clap-noun-verb, cargo-cicd, wasm4pm-compat, pm4py-rs, miniml (7).
- **`rust-cache`/any CI cache: ~8/18 weak or absent.** No CI at all: wasm4pm-compat, pm4wasm, miniml, semantic_bit, mac-artifact-cleaner (5). Deprecated `actions/cache@v3`: a2a-rs, swarmsh-v2 (2). affidavit CI has **no cache action**.
- **No CI whatsoever: 5/18** (wasm4pm-compat, pm4wasm, miniml, semantic_bit, mac-artifact-cleaner). Another 2 (a2a-rs, swarmsh-v2) run deprecated `actions-rs/*` actions.
- **Release workflow: ~9/18 missing.**
- **MSRV verification job in CI: ~14/18 missing** (real `rust-version` job: clap-noun-verb, clnrm_prototype, cargo-cicd weekly, pm4py-rs — and pm4py-rs's is wrong).
- **`CONTRIBUTING.md`: 9/18 missing.** **`CHANGELOG.md`: 4/18 missing** (pm4wasm, dteam, semantic_bit, mac-artifact-cleaner).
- **`rust-toolchain.toml`: 9/18 missing.**
- **Dependabot: ~12/18 missing** (present: affidavit, ggen, clnrm_prototype, lsp-max, pm4py-rs).
- **`CODE_OF_CONDUCT.md`: ~16/18 missing** (lsp-max yes; referenced-but-absent in clnrm_prototype/miniml).

**Takeaway:** the boilerplate's biggest leverage is the cheap, currently-absent hygiene files (`deny.toml`, `typos.toml`, `.editorconfig`, `[workspace.lints]`, `SECURITY.md`) plus a single canonical `ci.yml` with caching.

---

## 5. Recommended Canonical Boilerplate — MANIFEST

Exact file set. For each: the decisions to encode (bodies abbreviated).

### Root manifest
- **`Cargo.toml`** — `[workspace.package]` with `version` (CalVer), `edition = "2021"`, `rust-version = "1.82"`, `authors = ["Sean Chatman <xpointsh@gmail.com>"]`, `license = "MIT OR Apache-2.0"`, `repository`/`homepage = "https://github.com/seanchatmangpt/<repo>"`, `keywords` (≤5), `categories` (≤5). `[workspace.dependencies]` centralizing serde/serde_json/anyhow/thiserror(=2)/clap/tokio/tracing/tracing-subscriber/blake3/linkme/criterion/proptest/insta + house path/registry deps. `[workspace.lints]` (see §6). `[profile.release]` = `lto = true`, `codegen-units = 1`, `panic = "abort"`, `strip = true` (WASM crates override: `opt-level = "s"`, keep `panic="abort"`, drop `lto`/`strip`). `resolver = "2"`. Members use `version.workspace = true`, `edition.workspace = true`, `license.workspace = true`, `lints.workspace = true` — **no copy-paste**.
- **`rust-toolchain.toml`** — `channel = "1.82.0"` (pinned stable, see §6), `components = ["rustfmt", "clippy"]`, `profile = "minimal"`. Nightly variant documented for the ggen/specgen subset that needs `#![feature(...)]`.

### Lint / quality config
- **`rustfmt.toml`** — `edition = "2021"`, `max_width = 100`, `tab_spaces = 4`, `use_small_heuristics = "Max"`, `reorder_imports = true`, `use_field_init_shorthand = true`, `use_try_shorthand = true`, `ignore = ["generated/", "**/generated/"]`. (Adopt clap-noun-verb's, which is the only fleshed-out one.)
- **`clippy.toml`** — only if needed beyond `[lints]`: `msrv = "1.82"`, optional `disallowed-methods`/`disallowed-types`. Otherwise omit; keep clippy config in `[workspace.lints.clippy]`.
- **`deny.toml`** — adopt clnrm's (the most rigorous): `advisories` `vulnerability="deny"`, `yanked="deny"`, `unmaintained="warn"`; `licenses` allow `MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Unicode-3.0, CC0-1.0`, deny `GPL-3.0, AGPL-3.0`; `bans` `wildcards="deny"`, `multiple-versions="warn"`; `sources` crates.io only + explicit git allowlist (opentelemetry).
- **`typos.toml`** — adopt clap-noun-verb's: exclude `Cargo.lock`, `target`, `.git`, `archive`, `crates`, generated dirs, `*.ttl`; extend the project-noun allowlist.
- **`.editorconfig`** — adopt affidavit's verbatim (already correct): UTF-8/LF/final-newline/trim; `.rs`/`.toml` 4-space; web 2-space; `.md` no-trim.
- **`.gitignore`** — `target/`, `**/*.rs.backup`, `.claude/` (or scope to local), `/.ggen/keys/`, secrets (`*.pem`, `*.key`, `.env`), OS cruft, LaTeX artifacts (`*.aux *.pdf *.blg`), agent session artifacts (`*_SUMMARY.md` root, `AGENT_*`, `checkpoints/`, `receipts/` if runtime).

### GitHub
- **`.github/workflows/ci.yml`** — triggers `push` + `pull_request` to `main`/`master`/`develop`. `concurrency: group: ${{ github.workflow }}-${{ github.ref }}`, `cancel-in-progress: true`. `permissions: contents: read`. Global `env: CARGO_TERM_COLOR: always`, `RUST_BACKTRACE: 1`. Jobs (all `ubuntu-latest`, toolchain via `dtolnay/rust-toolchain@1.82.0`, cache via `Swatinem/rust-cache@v2` keyed on `Cargo.lock`):
  - `fmt` — `cargo fmt --all -- --check` (pinned toolchain, no cache).
  - `clippy` — `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
  - `test` — `cargo test --workspace --all-features`, 15-min timeout. **Optional matrix** `stable + beta` (not nightly in required path).
  - `docs` — `cargo doc --no-deps --all-features` with `RUSTDOCFLAGS: -D warnings`.
  - `deny` — `EmbarkStudios/cargo-deny-action@v2` (uses the committed `deny.toml`).
  - `typos` — `crate-ci/typos` action.
  - `msrv` — `dtolnay/rust-toolchain@1.82.0` + `cargo check --workspace`.
  - `ci-success` — gate job, `needs:` all above, fails if any failed.
- **`.github/workflows/release.yml`** — trigger `v*.*.*` tags. `concurrency` with `cancel-in-progress: false`. Jobs: `validate` (re-run fmt/clippy/test), `publish` (crates.io in dependency order, `--locked`, with polling loop for path-dep crates), `github-release` (`softprops/action-gh-release@v2`, multi-target binaries: x86_64/aarch64 × linux/macos, + checksums, CHANGELOG section extraction).
- **`.github/dependabot.yml`** — adopt affidavit's: `cargo` (weekly), `github-actions` (weekly), `npm` per `web/` if present; `open-pull-requests-limit: 10`; commit prefixes `cargo`/`ci`/`npm`; group minor+patch.
- **`.github/pull_request_template.md`** — adopt affidavit's "what I verified vs. did not" honest-checklist framing.

### Task runner & docs
- **`justfile`** — **canonical runner** (see §6). Recipes: `default` (`just --list`), `fmt`, `fmt-check`, `lint` (clippy -D warnings), `test`, `test-full`, `build`, `build-release`, `doc`, `audit`, `deny`, `typos`, `ci` (fmt-check→lint→test→deny→typos), `clean`. Honest disclaimers for path-dep repos.
- **`CONTRIBUTING.md`** — adopt clnrm_prototype's structure (Getting Started, Dev Setup, Process, Code Style, Testing, Release).
- **`SECURITY.md`** — adopt wasm4pm-compat's (Scope, Supported Versions, Reporting → xpointsh@gmail.com, Known Non-Issues).
- **`CHANGELOG.md`** — Keep-a-Changelog + CalVer headers `## [YY.M.patch] — YYYY-MM-DD` with Added/Changed/Fixed.
- **`README.md`** — sections: tagline/doctrine, Install/Quick Start, Concepts, CLI/API surface, Build & Test, Sibling deps disclaimer, License.
- **`CLAUDE.md`** — keep house standard (Overview, Architecture, Key Concepts, Dev Workflow, Conventions, Testing, Troubleshooting). Keep tight.
- **`LICENSE-MIT`** + **`LICENSE-APACHE`** — both files always present (dual). Stop relying on Cargo metadata alone.

---

## 6. Opinionated Defaults to Lock In

| Axis | RECOMMENDATION | One-line rationale |
|---|---|---|
| **Version scheme** | **CalVer `YY.M.patch`** (e.g. `26.6.17`), not `YY.M.DD` | Already modal (10 repos); `.patch` allows >1 release/day, which `.DD` cannot — and matches affidavit/lsp-max/clap-noun-verb. |
| **Edition** | **2021** | 14/18 already; 2024 only where forced (ggen-mcp/a2a-rs/semantic_bit). Allow opt-in 2024 per repo, default 2021. |
| **MSRV** | **`1.82`** | Median of the *declared* values; matches lsp-max (the largest workspace) and clears 2021-edition needs. Set `rust-version` + a CI `msrv` job everywhere. |
| **Toolchain** | **Pinned stable `1.82.0`** in CI + `rust-toolchain.toml`; **nightly opt-in** only for crates using `#![feature]` (ggen-specgen/wasm4pm-compat subset) | cargo-cicd already pins stable explicitly to stop "new-lint-turns-CI-red" churn; nightly-everywhere is the chief reason affidavit/bcinr CI can't go green. Pin, don't float. |
| **thiserror** | **2.x** | Newer repos (affidavit, ggen, lsp-max, cargo-cicd, dteam) already on 2; one version kills the a2a-rs intra-workspace split. |
| **Template shape** | **Offer BOTH**: a single-crate template and a workspace template | Corpus is ~half-half (8 single, 10 workspace). Workspace template mandates `[workspace.package]`+`[workspace.dependencies]`+`[workspace.lints]` inheritance. |
| **Task runner** | **`just`** as canonical (thin), with optional generated `Makefile.toml` parity for cargo-make holdouts | just is simplest and already canonical in affidavit/lsp-max; the cargo-make repos can keep a delegating wrapper during migration. |
| **Lint set** | **rust:** `unsafe_code = "forbid"` (default) / `"warn"` for `linkme`/WASM/specgen crates; `missing_docs = "warn"` on libs, `"allow"` on bins; `unexpected_cfgs = "warn"` with check-cfg allowlist. **clippy:** `all` + `pedantic` at `warn` (priority −1); `unwrap_used`/`expect_used`/`panic` = `"deny"` in **library** crates (clnrm_prototype precedent) but `"warn"` in bins/tests; `todo`/`unimplemented`/`exit` = `"deny"`; `multiple_crate_versions = "allow"`. Wire via `[workspace.lints]`, never a stray file. | Encodes the strictest behavior actually shipped (clnrm_prototype deny-unwrap, clap-noun-verb deny-todo) while staying buildable; replaces the inline `-D warnings`-only approach in 13 repos. |

---

## 7. Shared Common Crate Proposal

**Name: `chatman-common`** (alt: `house-core`; avoid `cnv-*` to not collide with `clap-noun-verb`). Single crate, `MIT OR Apache-2.0`, CalVer, edition 2021, MSRV 1.82. Justified by patterns repeated in nearly every repo:

- **`error` module — unified `Error`/`Result`** via `thiserror` 2: a house `#[derive(Error)]` enum + `pub type Result<T> = core::result::Result<T, Error>;`. Every repo hand-rolls `src/error.rs` + `anyhow`/`thiserror` (affidavit, ggen-mcp, lsp-max, cargo-cicd, semantic_bit, …). Re-export `anyhow::Context` for bins.
- **`telemetry` module — tracing/OTel init helper**: one `init_tracing(opts)` / `init_otel(endpoint)` wrapping `tracing-subscriber` (env-filter+json) + optional `opentelemetry-otlp`. 7+ repos duplicate this (ggen, clnrm, lsp-max, swarmsh-v2, pm4py-rs, ggen-mcp, dteam, affidavit `otel` feature). Feature-gate `otel` so non-telemetry repos pay nothing.
- **`cli` module — clap-noun-verb bootstrap**: a `bootstrap()` that wires `clap-noun-verb` + `linkme` verb discovery + standard global flags (`--format {json,yaml}`, `--color`, `-v`). Pattern is identical in affidavit, cargo-cicd, mac-artifact-cleaner.
- **`provenance` module — BLAKE3 helpers** (feature `provenance`): content-address + rolling-chain-hash + hex-digest validation. Six repos reimplement blake3 receipt/commitment logic.
- **`testkit` module (dev-facing)**: re-export `assert_cmd`, `predicates`, `tempfile`, plus golden-file + trybuild harness helpers and a `proptest` config shim. Standardizes the test deps every repo copies.

Ship behind features (`telemetry`/`otel`, `cli`, `provenance`, `testkit`) with a near-empty `default` so WASM/library-only crates (pm4wasm, miniml, semantic_bit) can depend on just `error`.

---

## 8. Affidavit Refactor Checklist

Ordered. `[AUTO]` = safe to script with no judgment; `[HUMAN]` = needs a decision/review. Grounded in `affidavit.md` + verified against the live repo.

**Metadata fixes (Cargo.toml)**
1. `[AUTO]` Fix `repository` URL: `https://github.com/anthropics/affidavit` → `https://github.com/seanchatmangpt/affidavit`. Add matching `homepage` (currently missing).
2. `[AUTO]` Trim `keywords` from **9 → ≤5** (crates.io limit). Recommend `["provenance", "receipt", "blake3", "verification", "ocel"]`; drop `sealed`, `otel`, `wasm4pm`, `compliance`.
3. `[HUMAN]` Decide MSRV: bump `rust-version` 1.78 → **1.82** to match house default, or keep 1.78 — verify it still builds.

**Add missing hygiene files**
4. `[AUTO]` Add `rustfmt.toml` (house template; affidavit currently has none).
5. `[HUMAN]` Add `[workspace.lints]`/`[lints]` to Cargo.toml with the §6 lint set — needs a pass to confirm the codebase compiles under deny-`todo`/`unimplemented` (the `1000x_*` modules are likely offenders).
6. `[AUTO]` Add `deny.toml` (clnrm template), `typos.toml` (clap-noun-verb template). `.editorconfig` **already present and correct — no action** (survey said absent; it exists).
7. `[AUTO]` Add `SECURITY.md` (wasm4pm-compat template; reporting → xpointsh@gmail.com).

**CI**
8. `[AUTO]` Add `Swatinem/rust-cache@v2` to both jobs in `.github/workflows/rust.yml` (currently no caching).
9. `[HUMAN]` Once nightly→pinned-stable decision (§6) lands, swap toolchain and remove the blanket `continue-on-error: true` from `fmt`/`build-and-test` so CI gives a real signal. Add `deny`/`typos`/`msrv` jobs + `ci-success` gate.

**Quarantine / delete cruft** (each its own commit; nothing here is part of the crate)
10. `[AUTO]` Delete the **18** `src/verbs/*.rs.backup` files (plus `src/assemble.rs.backup`, `src/verbs/mod.rs.backup` if present) and add `**/*.rs.backup` to `.gitignore`.
11. `[HUMAN]` Triage the **16** `src/1000x_*.rs` modules (`1000x_auto_remediate_dx.rs`, `1000x_autonomous_governance.rs`, `1000x_chaos_e2e.rs`, `1000x_cli_telepathy_qol.rs`, `1000x_distributed_sharding.rs`, `1000x_formal_verification_spec.rs`, `1000x_gpu_verifier.rs`, `1000x_holographic_lsp_dx.rs`, `1000x_nlp_query_qol.rs`, `1000x_otel_hyper_spec.rs`, `1000x_post_quantum_sealing.rs`, `1000x_receipt_to_wasm_qol.rs`, `1000x_semantic_isomorphism_e2e.rs`, `1000x_swarm_e2e.rs`, `1000x_tdd_synthesizer_dx.rs`, `1000x_time_travel_dx.rs`): the `1000x_` prefix violates module naming; either delete, move under a `experimental/` feature-gated module with conventional names, or extract to a separate crate. Needs judgment on which are load-bearing.
12. `[AUTO]` Remove root session artifacts: `audit_instructions.txt`, `DX_QOL_EXECUTIVE_SUMMARY.txt`. `[HUMAN]` decide on `IMPLEMENTATION_SUMMARY.md` / `STATUS.md` (likely delete or move to `docs/`).
13. `[HUMAN]` Remove/relocate the root one-off Python generators: `gen_thesis.py`, `generate_bib.py`, `generate_conclusion.py`, `generate_verbs.py`, `remediate_licenses.py` — move to `tools/` or delete; they are not crate scaffolding.
14. `[AUTO]` Move `portfolio_test_dataset.json` from repo root → `fixtures/` (update any path references).
15. `[HUMAN]` Gitignore compiled LaTeX artifacts under `thesis/` (`*.aux`, `*.pdf`, `*.blg`, `*.log`) that aren't already excluded.

**Adopt inheritance (if affidavit ever splits into a workspace)**
16. `[HUMAN]` Not currently a workspace; if/when the `web/`+crate split or sibling vendoring happens, hoist edition/license/version/lints into `[workspace.package]`+`[workspace.lints]` per the §5 workspace template.

---

*Sources: the 10 survey reports in this directory. `affidavit` rows additionally verified against the live repo (`.editorconfig` present, 9 keywords, 18 `.rs.backup`, 16 `1000x_*.rs`, repository URL `anthropics/affidavit`).*
