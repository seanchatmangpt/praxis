# praxis

The `seanchatmangpt` Rust **house style**, as a kit: one place that defines how
every Rust repo in this account should be scaffolded, linted, built, and
released — plus the tooling to apply it to new and existing repos.

Derived empirically from 18 repos (17 public + `affidavit`). Ten agents
surveyed the fleet; findings were consolidated into
[`survey/00-SYNTHESIS.md`](survey/00-SYNTHESIS.md).

---

## What is praxis?

`praxis` is a house-style Rust project template and standardization kit. It
captures the rulings that emerged from surveying the fleet: which crates,
which lint levels, which CI shape, which versioning scheme. Rather than
re-deciding these things per repo, every new or existing repo starts from
`praxis` and inherits the defaults.

The two entry points are:

- **`cargo generate`** — scaffold a new project from `template/`
- **`apply.sh`** — backfill the project-agnostic hygiene files into an
  existing repo

---

## What's included

| Path | What it is |
|------|------------|
| `template/` | Canonical single-crate scaffold. Ships `Cargo.toml` (with `[lints]`), `Cargo.workspace.toml` (workspace variant), `rustfmt.toml`, `deny.toml`, `typos.toml`, `.editorconfig`, `rust-toolchain.toml`, `.github/` (CI + release + dependabot + PR template), `justfile`, dual license files, doc skeletons, and `src/` with the standard module layout. |
| `crates/chatman-common/` | Shared **house crate** — unified `Error`/`Result`, tracing/otel init, CLI bootstrap, BLAKE3 provenance helpers, testkit. Feature-gated; `cargo check`-clean. |
| `Cargo.workspace.toml` | Drop-in workspace manifest for multi-crate repos. Includes `[workspace.lints]` inheritance and shared dependency versions. |
| `apply.sh` | Copies the project-agnostic files from `template/` into an existing repo (dry-run available). Prints what it skipped and what manual `Cargo.toml` steps remain. |
| `CHECKLIST.md` | Per-repo refactor checklist for all 18 surveyed repos. Distinguishes automatic `[A]` fixes from human-judgment `[H]` decisions. |
| `survey/` | The raw 10-agent reports and the consolidated synthesis (`00-SYNTHESIS.md`). Evidence behind every ruling. |
| `BROADEN-ACCESS.md` | How to give a Claude Code session write access to repos beyond `affidavit` so the kit can be applied to the full fleet. |

---

## Key patterns encoded in the template

### BLAKE3 chain (content addressing)

Every receipt and artifact is identified by its BLAKE3 digest of canonical
bytes (deterministic JSON, sorted keys, no whitespace). Same inputs always
produce the same hash. The template wires this via `chatman-common`'s
provenance helpers:

```rust
let hash = blake3::hash(canonical_bytes);
let hex  = hash.to_hex().to_string();  // 64 lowercase hex chars — the identity
```

### Noun-verb CLI

Commands follow `<noun> <verb>` via `clap-noun-verb` (or plain Clap
subcommands). Each verb lives in `src/verbs/<verb>.rs`:

```rust
pub async fn handle_<verb>(args: <Verb>Args) -> anyhow::Result<()>
```

`cli.rs` owns argument parsing only; no business logic leaks there.

### linkme plugin discovery

Zero-cost distributed slices let downstream crates register handlers without a
central registry:

```rust
use linkme::distributed_slice;

#[distributed_slice(HANDLERS)]
pub static MY_HANDLER: Handler = Handler::new("my-type", handle_my_type);
```

`src/discovery.rs` iterates `HANDLERS` at startup. The slice is filled at
link time — no runtime scanning, no `inventory` crate.

### CalVer versioning

All repos use `YY.M.patch` (e.g., `26.6.0`). The release workflow in
`.github/workflows/release.yml` derives the tag from this scheme.

### Seal pattern (immutable domain objects)

Structs that must pass through a validation or hashing stage before they are
trustworthy get a private `_seal: ()` field:

```rust
pub struct Receipt {
    pub events: Vec<Event>,
    pub chain_hash: String,
    _seal: (),   // struct-literal construction fails at compile time (E0451)
}
```

Construction is only possible through the canonical builder path. Outside
callers get the public fields but cannot fabricate the type.

---

## How to use

### New project (cargo generate)

```bash
cargo install cargo-generate   # once
cargo generate --git https://github.com/seanchatmangpt/praxis template
```

Choose `Cargo.toml` for a single crate or `Cargo.workspace.toml` for a
workspace. The generator substitutes `{{project-name}}` and `{{description}}`
throughout.

### Existing project (apply.sh)

```bash
# Preview what would change (no writes)
/path/to/praxis/apply.sh . --dry-run

# Apply
/path/to/praxis/apply.sh .
```

After running, follow the printed checklist for the manual `Cargo.toml` steps:

1. Set `repository` / `homepage` to `seanchatmangpt/<repo>`.
2. Set `license = "MIT OR Apache-2.0"`, `edition = "2021"`,
   `rust-version = "1.82"`.
3. Trim `keywords` ≤ 5 / `categories` ≤ 5.
4. Add the `[lints]` / `[workspace.lints]` block (copy from `template/`).
5. Run `cargo clippy --all-targets --all-features -- -D warnings` and fix
   fallout (`todo!` / `unwrap` hits are expected — that is the point).
6. `just ci` green, commit on a branch, open a PR using the house template.

### Shared crate

```toml
# In your repo's Cargo.toml or workspace Cargo.toml
chatman-common = { git = "https://github.com/seanchatmangpt/chatman-common", features = ["telemetry", "provenance"] }
```

---

## House defaults (the rulings)

| Axis | Default |
|------|---------|
| Version | CalVer `YY.M.patch` |
| Edition | 2021 (opt-in 2024 per repo) |
| MSRV | 1.82, verified in CI |
| Toolchain | pinned stable `1.82.0` (`rust-toolchain.toml`) |
| License | dual `MIT OR Apache-2.0`, both files committed |
| Errors | `thiserror` 2 |
| Task runner | `just` |
| Lints | `[workspace.lints]`; `unsafe_code = forbid`; clippy `all` + `pedantic` warn; `todo` / `unimplemented` / `dbg_macro` deny |
| CI cache | `Swatinem/rust-cache@v2` |
| Spell check | `typos` |
| Supply chain | `cargo deny check` (license + advisory) |

The evidence behind each ruling is in `survey/00-SYNTHESIS.md` (§6).

---

## Survey methodology

Ten agents independently surveyed 18 repos:

- 17 public repos: `clap-noun-verb`, `ggen`, `clnrm`, `clnrm_prototype`,
  `lsp-max`, `cargo-cicd`, `wasm4pm-compat`, `ggen-mcp`, `a2a-rs`,
  `swarmsh-v2`, `pm4py-rs`, `pm4wasm`, `miniml`, `bcinr`, `dteam`,
  `semantic_bit`, `mac-artifact-cleaner`
- 1 private writable repo: `affidavit` (the worked reference example)

Each agent reported on: toolchain pins, lint posture, CI shape, dependency
hygiene, license files, versioning scheme, and documentation coverage. The
raw reports are in `survey/`. The synthesis (`survey/00-SYNTHESIS.md`)
consolidates findings into ranked gaps and the rulings above.

`affidavit` was refactored in-session as the reference implementation. See
`CHECKLIST.md §affidavit reference refactor` for the exact changes made.

---

## Why this lives inside `affidavit`

Built in a session scoped to the single writable repo `affidavit`. The kit is
self-contained and depends on nothing in `affidavit`; it is meant to graduate
into its own `seanchatmangpt/praxis` repo. See `BROADEN-ACCESS.md` for how to
roll it out to the rest of the fleet.

---

## License

MIT OR Apache-2.0
