# Fleet refactor checklist

How to bring each `seanchatmangpt` Rust repo onto the house boilerplate. Per-repo
fixes are derived from the survey (`survey/00-SYNTHESIS.md`). `affidavit` is the
worked reference example (done in-session on branch `claude/jolly-turing-t488iq`).

## Procedure per repo

1. `apply.sh <repo>` to drop in the hygiene files (prints what it skipped/applied).
   - Add `--wasm` for WASM crates (or let it auto-detect from Cargo.toml).
   - Add `--integration` if the repo needs Docker-backed integration tests.
   - Add `--mcp` for MCP server repos.
   - Add `--check` in CI to gate on praxis conformance (exits 1 if anything is missing).
2. Cargo.toml: fix `repository`/`homepage` to `seanchatmangpt/<repo>`, set
   `license = "MIT OR Apache-2.0"`, `edition = "2021"`, `rust-version = "1.82"`,
   trim `keywords`<=5 / `categories`<=5, add the `[lints]`/`[workspace.lints]` block.
3. `cargo clippy --all-targets --all-features -- -D warnings` and fix fallout
   (expect `todo!`/`unwrap` hits — that's the point).
4. `just ci` green, commit on a branch, open a PR with the template.

`[A]` = safe/automatic (apply.sh detects/copies or mechanical). `[H]` = human decision.
`[AUTO]` = apply.sh audit detects and reports (still needs human to fix the root cause).

## Every repo gets (the cheap, near-universal wins)

`deny.toml` (15/18 missing) · `typos.toml` (17/18) · `.editorconfig` (16/18) ·
`[workspace.lints]` (15/18) · `SECURITY.md` (11/18) · CI caching via
`Swatinem/rust-cache@v2` · pinned-stable toolchain.

## Public repos (readable this session)

| Repo | Top fixes |
|------|-----------|
| **affidavit** | DONE in-session — see the commits on this branch. Reference example. |
| **clap-noun-verb** | Closest to conformant already (has rustfmt/lints/deny/typos/.editorconfig/dual). `[A]` add rust-cache if absent; `[H]` align MSRV 1.74→1.82. Use as secondary reference. |
| **ggen** | `[A]` add `typos.toml`+`.editorconfig`; `[H]` consolidate ~45 workflow files → one `ci.yml`; `[H]` MIT → dual-license. |
| **clnrm** | `[A]` add `typos.toml`/`.editorconfig`/`[workspace.lints]`, `CLAUDE.md`+`CONTRIBUTING.md`; `[H]` consolidate ~29 workflows; `[H]` MIT → dual. |
| **clnrm_prototype** | `[A]` fix repo URL (`sac/ggen`→`seanchatmangpt/clnrm_prototype`), add `deny.toml` (CI runs deny with no file), `rustfmt.toml`/`typos.toml`/`.editorconfig`. Already has `[lints]`. |
| **lsp-max** | `[A]` add `[workspace.lints]`/`deny.toml`/`typos.toml`/`.editorconfig`/`SECURITY.md`. Large 34-crate workspace — wire lints via inheritance. |
| **cargo-cicd** | `[A]` add `rustfmt.toml`/`[lints]`/`deny.toml`/`typos.toml`/`.editorconfig`. Good CI already (it's CI tooling) + pinned stable — keep. |
| **wasm4pm-compat** | `[A]` add `ci.yml` (has **zero CI**), `deny.toml`/`typos.toml`/`.editorconfig`/`[lints]`, `CONTRIBUTING.md`; `[H]` pin nightly date or move to stable. |
| **ggen-mcp** | `[A]` add full hygiene set + `SECURITY.md`/`CONTRIBUTING.md`; `[H]` fix repo URL/package name (`PSU3D0/spreadsheet-mcp`), decide edition 2024→2021 and Apache-only → dual. |
| **a2a-rs** | `[A]` replace deprecated `actions-rs` CI with `ci.yml`, add `justfile` (no runner) + license **files**; `[H]` unify `thiserror` 1+2 → 2 across the workspace, reconcile mixed editions. |
| **swarmsh-v2** | `[A]` replace deprecated `actions-rs` CI, add `justfile` (raw Makefile), add `LICENSE-*` files; `[H]` fix placeholder repo URL `user/swarmsh-v2`. |
| **pm4py-rs** | `[A]` add hygiene set + `CLAUDE.md` + `justfile`; `[H]` reconcile license (metadata AGPL vs dual-text file) and the MSRV contradiction (declares 1.85, CI tests 1.70). |
| **pm4wasm** | `[A]` add `ci.yml` (**zero CI**) + `CHANGELOG.md` + full hygiene + `justfile`; `[H]` Apache-only → dual. WASM crate: keep `panic="abort"`/size-opt profile. |
| **miniml** | `[A]` add `ci.yml` (**zero CI**) + hygiene; `[H]` review `BSL-1.1` vs house dual-license (may be intentional). Rust/WASM under pnpm+turbo — CI must build the crate too. |
| **bcinr** | `[A]` add `LICENSE-*` files (metadata dual, no files), `deny.toml` (CI runs deny, no file), `rustfmt.toml`/`typos.toml`/`.editorconfig`/`[workspace.lints]`; `[H]` pin the floating nightly. |
| **dteam** | `[A]` wire the existing `workspace_lints.toml` into `[workspace.lints]`, add `deny.toml`/`rustfmt.toml`/`typos.toml`/`.editorconfig`/`SECURITY.md`/`CHANGELOG.md`; `[H]` `BSL-1.1` review; SemVer→CalVer. |
| **semantic_bit** | Near-empty — `[A]` full bootstrap: `README`, `LICENSE-*`, `ci.yml`, all hygiene, docs; `[H]` edition 2024→2021. |
| **mac-artifact-cleaner** | `[A]` add `ci.yml` (**zero CI**) + `LICENSE-*` files + `CHANGELOG.md`/`CONTRIBUTING.md`/`SECURITY.md` + hygiene. Has just+Makefile already. |

## Private repos (NOT reachable this session)

Not surveyed (no read access). Apply the kit locally from a checkout and review
against the synthesis. See `BROADEN-ACCESS.md`.

- `knhk` · `kcura` · `kgold` · `chatmangpt` · `unibit` · `mcpp` · `stpnt` · `tower-lsp-composition`

For each: `apply.sh .` → fix Cargo metadata + add `[lints]` → `just ci` → PR.

## affidavit reference refactor (this branch)

Applied automatically here as the worked example:

- `[A]` Fixed `Cargo.toml` `repository` (`anthropics`→`seanchatmangpt`), added `homepage`, trimmed `keywords` 9→5.
- `[A]` Added `deny.toml`, `typos.toml`, `SECURITY.md`. (`rustfmt.toml` deferred:
  adding it without a coordinated `cargo fmt` reformat would make the existing
  fmt job flag the whole tree — do it in its own reformat commit.)
- `[A]` Added `Swatinem/rust-cache@v2` to the CI workflow.
- `[A]` Added the `**/*.rs.backup` (+ `*.orig`) ignore rule to `.gitignore`.
- `[A]` Deleted the 18 `src/verbs/*.rs.backup` files (approved).
- `[A]` Removed the root session artifacts `audit_instructions.txt` and `DX_QOL_EXECUTIVE_SUMMARY.txt` (approved).
- `[A]` Moved `portfolio_test_dataset.json` → `fixtures/` (approved).

Deliberately left for human judgment (documented, not done):

- `[H]` Adopt `[lints]` in affidavit's `Cargo.toml` — needs a build with the
  sibling path-deps present to confirm it compiles under `todo`/`unimplemented` deny
  (the `1000x_*` modules are likely offenders). Can't be verified in-session
  (siblings absent).
- `[H]` Triage the 16 `src/1000x_*.rs` modules (non-conventional names) and the
  root one-off Python generators (`gen_thesis.py`, etc.).
- `[H]` MSRV bump 1.78→1.82 and nightly→pinned-stable swap (will change CI signal).

---

## WASM checklist (required when `target_arch = "wasm32"` is present)

Run `apply.sh <repo> --wasm` — it auto-detects WASM from Cargo.toml and src/*.

- `[AUTO]` Verify `[profile.release]` does NOT have `strip = true` for WASM crates.
  **apply.sh detects this (WASM-1).** Fix: remove `strip = true`; use `just build-opt`
  (wasm-opt -Os) instead. Source: survey/01-SECOND-WAVE.md BUG-1.
- `[AUTO]` Verify `[target.'cfg(target_arch = "wasm32")'.dependencies]` sets
  `getrandom { features = ["js"] }` (and `uuid { features = ["js"] }` if used).
  **apply.sh warns if getrandom is present without the js feature (WASM-2).**
- `[H]` Replace `HashMap` with `BTreeMap` in WASM-exposed types.
  Hash randomization makes WASM guest output non-deterministic.
- `[AUTO]` Confirm `console_error_panic_hook::set_once()` is called in
  `#[wasm_bindgen(start)]`. **apply.sh warns if not found in src/ (WASM-3).**
- `[H]` Use handle-based API (`Store<T>` from `template-wasm/`) rather than raw pointer
  passing to JS. No `unsafe` required.
- `[A]` Build with `wasm-pack build --target web --release` and verify binary loads in
  a browser or `wasm-bindgen-test` runner before shipping.

## Anti-patterns checklist

Run `apply.sh <repo>` — the audit section reports ANTI-1/2/3/5 automatically.

- `[AUTO]` Verify `.cargo/config.toml` does **not** suppress lints via `RUSTFLAGS`.
  **apply.sh detects `-A clippy::*` / `--allow` in `.cargo/config.toml` (ANTI-1).**
  Fix: move lint allows to `[workspace.lints.clippy]` with justification comments.
- `[AUTO]` If `rust-toolchain.toml` pins `channel = "nightly"`, apply.sh warns (ANTI-2).
  Fix: pin to `nightly-YYYY-MM-DD` or switch to stable `1.82.0`. Document which
  nightly feature(s) require it in CLAUDE.md. Use stable for all jobs except `cargo fmt`
  where nightly formatter options are needed.
- `[AUTO]` apply.sh scans Cargo.toml and Cargo.lock for `proc-macro-error` (ANTI-3).
  **RUSTSEC-2024-0370 — unmaintained.** Fix: migrate to `proc-macro-error2` or `manyhow`.
  `cargo deny check` will also catch this once deny.toml is in place.
- `[H]` Review open issues for self-filed `todo!()` stub admissions. Close any that
  are fully covered by the `deny(todo)` lint or where a concrete plan now exists
  in CLAUDE.md (ANTI-4). Not detectable automatically.
- `[AUTO]` apply.sh checks all workspace member `Cargo.toml` files for
  `lints.workspace = true` (ANTI-5). Fix: add `[lints]\nworkspace = true` to each
  member that is missing it.
