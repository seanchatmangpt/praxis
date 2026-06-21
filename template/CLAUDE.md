# {{project-name}} — Developer Guide

**Version:** (CalVer `YY.M.patch`)
**Language:** Rust (2021 edition)
**License:** MIT OR Apache-2.0

---

## Overview

{{description}}

> Scaffolded from [`praxis`](https://github.com/seanchatmangpt/praxis).
> House style: `just` orchestrates everything; `cargo` is the engine underneath.

---

## Architecture

### Source Layout

```
{{project-name}}/
├── src/
│   ├── lib.rs          # Public API, module tree, crate-level docs
│   ├── main.rs         # Binary entrypoint (thin — delegates to lib)
│   ├── error.rs        # thiserror error enum; one type per domain boundary
│   ├── types.rs        # Domain structs, newtypes, enums (no logic)
│   ├── cli.rs          # Clap configuration (noun-verb pattern)
│   ├── handlers.rs     # Event/command dispatch & routing
│   ├── discovery.rs    # linkme-based plugin / type discovery
│   └── verbs/          # One file per CLI subcommand
│       ├── mod.rs
│       └── <verb>.rs
├── tests/              # Integration tests (CLI round-trips, property tests)
├── benches/            # Criterion benchmarks
├── examples/           # Runnable demonstrations (`cargo run --example`)
├── justfile            # Task runner (source of truth for all commands)
├── Cargo.toml
├── rust-toolchain.toml # Pinned stable toolchain
└── CLAUDE.md           # This file
```

**Rule:** `types.rs` holds data; `handlers.rs` holds dispatch; `verbs/` holds CLI glue.
Logic that does not fit neatly into those layers gets its own module (`chain.rs`,
`verifier.rs`, etc.) rather than growing a god-module.

---

## Key Concepts

### Seal Pattern

Immutable domain objects are **sealed**: their construction is only possible through
a canonical builder path. The private `_seal` field (unit type `()`) makes
struct-literal construction fail at compile time (`E0451`):

```rust
pub struct Receipt {
    pub events: Vec<Event>,
    pub chain_hash: String,
    _seal: (),   // private — only constructible via ChainAssembler::finalize
}
```

Use this pattern for any value that must pass through a validation or hashing stage
before it is trustworthy. Outside users get the public fields but cannot fabricate a
`Receipt` without going through the assembler.

### Noun-Verb CLI

Commands follow the `<noun> <verb>` pattern via `clap-noun-verb` (or plain Clap with
a subcommand per verb). Each verb lives in `src/verbs/<verb>.rs` and exposes:

```rust
pub async fn handle_<verb>(args: <Verb>Args) -> anyhow::Result<()>
```

The `cli.rs` module owns argument parsing only; no business logic leaks there.

### linkme Plugin Discovery

Zero-cost distributed slices let downstream crates register handlers without a
central registry:

```rust
use linkme::distributed_slice;

#[distributed_slice(HANDLERS)]
pub static MY_HANDLER: Handler = Handler::new("my-type", handle_my_type);
```

Discovery (in `src/discovery.rs`) iterates `HANDLERS` at startup. The slice is
filled at link time, so there is no `inventory` crate or runtime scanning.

### BLAKE3 Content Addressing

All receipts and artifacts are identified by their BLAKE3 digest:

```rust
let hash = blake3::hash(canonical_bytes);
let hex  = hash.to_hex().to_string();  // 64 lowercase hex chars
```

Canonical bytes come from deterministic JSON serialization (sorted keys, no
whitespace). Same inputs always produce the same hash; the hash is the identity.

---

## Build & Test

### Prerequisites

```bash
cargo install cargo-deny typos-cli just
rustup show   # toolchain auto-installed from rust-toolchain.toml
```

### Common Tasks

| Task | Command | Notes |
|------|---------|-------|
| List all tasks | `just` | |
| Format | `just fmt` | |
| Lint | `just lint` | clippy `-D warnings` |
| Test | `just test` | `cargo test` |
| Docs | `just doc` | opens browser |
| Full CI gate | `just ci` | fmt-check + lint + test + deny + typos |
| Run binary | `cargo run --bin {{project-name}} -- <args>` | |
| Run example | `cargo run --example <name>` | |
| Benchmark | `cargo bench` | Criterion HTML report in `target/criterion/` |

### Running Tests

```bash
# All tests
just test

# Single test by name
cargo test <test_name>

# With log output
RUST_LOG=debug cargo test -- --nocapture

# Determinism (single-threaded)
cargo test -- --test-threads=1

# Integration tests only
cargo test --test '*'
```

---

## Development Workflow

### Pre-commit Gate

Before every push, run:

```bash
just ci
```

This mirrors what CI runs:
1. `cargo fmt --check` — formatting is law
2. `cargo clippy -- -D warnings` — no lint regressions
3. `cargo test` — all tests green
4. `cargo deny check` — license / advisory / duplicate audit
5. `typos` — spell-check source and docs

### Adding a New Verb (Subcommand)

1. Create `src/verbs/<verb>.rs` with `pub async fn handle_<verb>(args: <Verb>Args) -> anyhow::Result<()>`.
2. Add `pub mod <verb>;` to `src/verbs/mod.rs`.
3. Register the subcommand in `src/cli.rs`.
4. Wire the match arm in `src/main.rs`.
5. Add at least one integration test in `tests/`.
6. (Optional) Add a runnable example in `examples/`.

### Adding a New Domain Type

1. Define the struct/enum in `src/types.rs`.
2. Derive `Debug`, `Clone`, `serde::Serialize`, `serde::Deserialize` unless there is
   a specific reason not to.
3. If the type must be sealed, add a private `_seal: ()` field and expose a builder.
4. Add a unit test in the same file.

### Adding a Plugin Handler

1. Create `src/handlers/<name>.rs`.
2. Implement the handler signature matching `handlers.rs`.
3. Register via `#[linkme::distributed_slice(HANDLERS)]`.
4. Add a unit test confirming the handler is discovered.

### Branching & Commits

- Branch from `main`.
- Conventional commit format: `type(scope): description`
  - `feat`, `fix`, `refactor`, `test`, `docs`, `chore`
- One logical change per commit.
- Add a `CHANGELOG.md` entry under `## [Unreleased]` for user-visible changes.

### CI Gate (GitHub Actions)

Pull requests must pass:
- `just ci` (runs the same steps as local pre-commit)
- `cargo deny check` (license + advisory)
- `typos` (spell check)

The CI definition lives in `.github/workflows/ci.yml`.

---

## Troubleshooting

### `cargo test` fails with `E0451` on a sealed type

You are trying to construct a sealed struct with a struct literal. Use the
canonical builder instead (e.g., `ChainAssembler::finalize`, `Builder::build`).
The private `_seal` field is intentional.

### `just ci` fails on `cargo deny`

Run `cargo deny check` for the detailed report. Common causes:

- New transitive dependency with a disallowed license — add an `allow` entry to
  `deny.toml` with a justification comment.
- Unmaintained crate advisory — update the dependency or add a `skip` entry with a
  linked issue.

### `typos` flags a false positive

Add the word to `typos.toml` under `[default.extend-words]`:

```toml
[default.extend-words]
mything = "mything"   # domain term, not a typo
```

### Clippy fires `clippy::pedantic` on generated or macro code

Add `#[allow(clippy::<lint>)]` at the item level with a comment explaining why.
Do not add crate-level `#![allow(...)]` suppression without a strong reason.

### Hash mismatch at runtime

Canonical serialization order matters. Ensure you use `serde_json::to_vec` on a
type whose fields serialize in a stable order (alphabetical or declaration order
with `#[serde(rename_all = "...")]`). Verify with the `--test-threads=1` flag.

---

## Code Conventions

- **No `unwrap`/`expect`/`panic` in library code.** Use `?` and `thiserror`.
- **Public items get rustdoc.** `missing_docs` is a warning; keep it clean.
- **`unsafe_code = "forbid"`.** Relax only when crate semantics require it (linkme,
  WASM bindgen) and document the exception.
- **`todo!`/`unimplemented!` are denied.** Stub with a returning `Err(...)` instead.
- **`dbg!` is denied.** Remove debug prints before committing.

---

## License

MIT OR Apache-2.0
