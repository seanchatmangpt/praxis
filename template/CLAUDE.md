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
│   ├── bin/{{project-name}}.rs  # Binary entrypoint (thin — delegates to lib)
│   ├── error.rs        # thiserror error enum; one variant per domain boundary
│   ├── types.rs        # Domain structs, newtypes, enums (no logic)
│   ├── cli.rs          # Clap configuration (noun-verb pattern via clap-noun-verb)
│   ├── chain.rs        # ChainAssembler — content-addressed event chain
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

## Dependency Ecosystem

### chatman-common

Internal utility crate providing shared primitives (error helpers, canonical
serialization, test fixtures). Import it for:

- `chatman_common::canonical_json(value)` — deterministic JSON bytes (sorted keys,
  no whitespace) used as input to BLAKE3 hashing
- `chatman_common::hex_digest(bytes)` — convenience wrapper: `blake3::hash` → 64-char
  lowercase hex string
- `chatman_common::testing::*` — fixture builders and assertion helpers for receipts

Do not re-implement canonical serialization inline. Always go through
`chatman_common::canonical_json` so that hash stability is guaranteed project-wide.

### clap-noun-verb

Provides the `NounVerb` derive macro that generates a `<Noun> <verb>` command tree
from a single enum. Example:

```rust
// src/cli.rs
use clap_noun_verb::NounVerb;

#[derive(NounVerb)]
pub enum Cli {
    /// Emit an event
    Emit(EmitArgs),
    /// Assemble the working chain into a receipt
    Assemble(AssembleArgs),
    /// Verify a sealed receipt
    Verify(VerifyArgs),
}
```

This generates `{{project-name}} emit`, `{{project-name}} assemble`,
`{{project-name}} verify` as top-level subcommands. Each variant maps 1:1 to a
struct in `src/verbs/<verb>.rs`. The `cli.rs` module owns parsing only — no
business logic belongs there.

### linkme (distributed_slice)

Zero-cost distributed slices let downstream crates register handlers without a
central registry. The slice is filled at **link time** — no runtime scanning, no
`inventory` crate, no `ctor` hooks.

**Declare the slice** (in `src/discovery.rs`):

```rust
use linkme::distributed_slice;
use crate::handlers::Handler;

#[distributed_slice]
pub static HANDLERS: [Handler] = [..];
```

**Register a handler** (in any module, including downstream crates):

```rust
use linkme::distributed_slice;
use crate::discovery::HANDLERS;
use crate::handlers::Handler;

#[distributed_slice(HANDLERS)]
pub static BUILD_HANDLER: Handler = Handler::new("build", handle_build_event);

fn handle_build_event(payload: &[u8]) -> anyhow::Result<()> {
    // ...
    Ok(())
}
```

**Iterate at startup** (in `src/handlers.rs`):

```rust
pub fn dispatch(event_type: &str, payload: &[u8]) -> anyhow::Result<()> {
    for handler in crate::discovery::HANDLERS {
        if handler.matches(event_type) {
            return handler.call(payload);
        }
    }
    anyhow::bail!("no handler registered for event type: {event_type}")
}
```

When adding a new event type, add a handler module and a `#[distributed_slice]`
registration — the dispatch loop picks it up automatically.

### blake3

Content-addressed identity for all artifacts. Key properties:

- **Deterministic:** same bytes → same hash, always
- **64-char hex:** `hash.to_hex().to_string()`
- **No timestamps:** ordering is by monotonic `seq`, not wall-clock

```rust
use blake3;

// Hash raw bytes
let digest = blake3::hash(bytes);
let hex = digest.to_hex().to_string();

// Rolling chain hash (fold each event into the running hash)
let mut hasher = blake3::Hasher::new();
for event in &events {
    hasher.update(&chatman_common::canonical_json(event)?);
}
let chain_hash = hasher.finalize().to_hex().to_string();
```

Never hash non-canonical bytes. Always call `chatman_common::canonical_json` first.

---

## Key Patterns

### Seal Pattern

Immutable domain objects are **sealed**: construction is only possible through a
canonical builder path. The private `_seal: ()` field causes struct-literal
construction to fail at compile time with `E0451`:

```rust
// src/types.rs
pub struct Receipt {
    pub format_version: String,
    pub events: Vec<Event>,
    pub chain_hash: String,
    pub profile: String,
    _seal: (),   // private — cannot be named outside this module
}

impl Receipt {
    // Only ChainAssembler (in chain.rs) calls this, and only after validation
    pub(crate) fn seal(
        format_version: String,
        events: Vec<Event>,
        chain_hash: String,
        profile: String,
    ) -> Self {
        Self { format_version, events, chain_hash, profile, _seal: () }
    }
}
```

```rust
// This fails at compile time — E0451:
let r = Receipt { events: vec![], chain_hash: String::new(), _seal: () };

// This works — only path that produces a Receipt:
let r = assembler.finalize()?;
```

Apply the seal pattern to any value that must pass through a validation or hashing
stage before it can be trusted. Users get public read access to fields but cannot
fabricate the type.

### linkme distributed_slice (full example)

See the [linkme section](#linkme-distributed_slice) above. Complete working pattern:

```rust
// src/discovery.rs — declare
#[linkme::distributed_slice]
pub static HANDLERS: [Handler] = [..];

// src/verbs/emit.rs — register
#[linkme::distributed_slice(crate::discovery::HANDLERS)]
static EMIT_HANDLER: Handler = Handler::new("emit", handle_emit);
```

Note: `HANDLERS` must be `[T]` (unsized slice), not `Vec<T>`. The `= [..]`
initializer is required by linkme syntax.

### ChainAssembler (chain.rs)

`ChainAssembler` is the only path to a sealed `Receipt`. Usage:

```rust
use crate::chain::ChainAssembler;

let mut assembler = ChainAssembler::new();

// Append events (order determines seq)
assembler.push(Event {
    seq: 0,
    event_id: "evt-0".into(),
    event_type: "build".into(),
    objects: vec![Object { id: "repo:main".into(), object_type: "git".into() }],
    commitment: blake3_hex_of_payload,
})?;

assembler.push(Event {
    seq: 1,
    event_id: "evt-1".into(),
    event_type: "test".into(),
    objects: vec![],
    commitment: blake3_hex_of_payload,
})?;

// finalize() computes the rolling chain hash and seals the Receipt
let receipt: Receipt = assembler.finalize()?;

// Serialize to disk
let json = serde_json::to_string_pretty(&receipt)?;
std::fs::write("receipt.json", json)?;
```

`push()` validates each event through the admission gates (unique `seq`, well-formed
object IDs, valid commitment digest). `finalize()` computes the rolling BLAKE3 chain
hash over all events in `seq` order and calls `Receipt::seal`.

---

## Feature Phase Architecture

Long-lived features that span multiple releases are gated by **phase feature flags**.
This lets the codebase carry work-in-progress code without shipping it, and lets
integration tests target specific phases.

### Pattern

```toml
# Cargo.toml
[features]
default = []
phase-1 = []                  # core event emission
phase-2 = ["phase-1"]         # assembly + sealing (depends on phase-1)
phase-3 = ["phase-2"]         # full verifier pipeline
otel    = ["opentelemetry"]   # optional observability, not phase-gated
```

```rust
// src/verifier.rs
pub fn run_pipeline(receipt: &Receipt) -> Result<Verdict> {
    // Phase 1: always present
    let decoded = decode(receipt)?;

    #[cfg(feature = "phase-2")]
    let checked = check_format(&decoded)?;

    #[cfg(feature = "phase-3")]
    {
        let integrity = chain_integrity(&checked)?;
        return emit_verdict(integrity);
    }

    #[cfg(not(feature = "phase-3"))]
    Ok(Verdict::Incomplete)
}
```

### Testing against a specific phase

```bash
# Only phase-1 work
cargo test --no-default-features --features phase-1

# Full pipeline
cargo test --features phase-3

# CI gates all phases independently
just ci-all-phases
```

### Rules

- Phase features are **additive and ordered**: `phase-N` always enables `phase-(N-1)`.
- Never gate core types or error types behind a phase flag.
- Remove phase flags in the release commit that ships the feature as stable.

---

## Adding a New Verb (Step by Step)

1. **Create the args struct** in `src/verbs/<verb>.rs`:

   ```rust
   // src/verbs/frobnicate.rs
   use clap::Args;

   #[derive(Args, Debug)]
   pub struct FrobnicateArgs {
       /// Path to the receipt
       pub receipt: std::path::PathBuf,

       /// Output format
       #[arg(long, default_value = "json")]
       pub format: String,
   }

   pub async fn handle_frobnicate(args: FrobnicateArgs) -> anyhow::Result<()> {
       // load receipt, do the work, print output
       Ok(())
   }
   ```

2. **Export the module** in `src/verbs/mod.rs`:

   ```rust
   pub mod frobnicate;
   ```

3. **Add the variant** to the CLI enum in `src/cli.rs`:

   ```rust
   use crate::verbs::frobnicate::FrobnicateArgs;

   #[derive(NounVerb)]
   pub enum Cli {
       // ... existing variants ...
       /// Frobnicate the receipt chain
       Frobnicate(FrobnicateArgs),
   }
   ```

4. **Wire the match arm** in `src/bin/{{project-name}}.rs`:

   ```rust
   match cli {
       Cli::Frobnicate(args) => handle_frobnicate(args).await?,
       // ...
   }
   ```

5. **Add an integration test** in `tests/frobnicate.rs`:

   ```rust
   #[test]
   fn frobnicate_accepts_valid_receipt() {
       // build a receipt via ChainAssembler, call the handler, assert output
   }
   ```

6. **(Optional)** Add `examples/frobnicate.rs` showing a full usage scenario.

7. **Run the full CI gate**: `just ci`

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
| Benchmark | `cargo bench` | Criterion HTML in `target/criterion/` |

### Running Tests

```bash
# All tests
just test

# Single test by name
cargo test <test_name>

# With log output
RUST_LOG=debug cargo test -- --nocapture

# Determinism (single-threaded, required for hash stability tests)
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

This mirrors CI:
1. `cargo fmt --check` — formatting is law
2. `cargo clippy -- -D warnings` — no lint regressions
3. `cargo test` — all tests green
4. `cargo deny check` — license / advisory / duplicate audit
5. `typos` — spell-check source and docs

### Adding a New Domain Type

1. Define the struct/enum in `src/types.rs`.
2. Derive `Debug`, `Clone`, `serde::Serialize`, `serde::Deserialize` unless there is
   a specific reason not to.
3. If the type must be sealed, add a private `_seal: ()` field and expose a
   `pub(crate) fn seal(...)` constructor called only from the canonical builder.
4. Add a unit test in the same file.

### Adding a Plugin Handler

1. Create `src/handlers/<name>.rs`.
2. Implement the handler signature matching `handlers.rs`.
3. Register via `#[linkme::distributed_slice(crate::discovery::HANDLERS)]`.
4. Add a unit test confirming the handler is discovered and dispatches correctly.

### Branching & Commits

- Branch from `main`.
- Conventional commit format: `type(scope): description`
  - `feat`, `fix`, `refactor`, `test`, `docs`, `chore`
- One logical change per commit.
- Add a `CHANGELOG.md` entry under `## [Unreleased]` for user-visible changes.

---

## Troubleshooting

### `E0451` on a sealed type

You are constructing a sealed struct with a struct literal. Use the canonical
builder (`ChainAssembler::finalize`, `Builder::build`, etc.). The private `_seal`
field is intentional — it is the compile-time guarantee.

### `just ci` fails on `cargo deny`

Run `cargo deny check` for the detailed report. Common causes:

- New transitive dependency with a disallowed license — add an `allow` entry to
  `deny.toml` with a justification comment.
- Unmaintained crate advisory — update the dependency or add a `skip` entry with a
  linked issue.

### `typos` flags a domain term

Add it to `typos.toml`:

```toml
[default.extend-words]
myterm = "myterm"   # domain term, not a typo
```

### Hash mismatch at runtime

Canonical serialization order matters. Always call `chatman_common::canonical_json`
before hashing. Verify hash stability with `--test-threads=1`.

### Clippy fires on generated / macro code

Add `#[allow(clippy::<lint>)]` at the item level with a comment. Do not add
crate-level `#![allow(...)]` suppression without a strong reason.

### linkme slice is empty at runtime

Ensure the crate that contains `#[distributed_slice(HANDLERS)]` registrations is
actually linked into the binary. Rust may dead-strip it if the crate is only a dev
dependency or if no symbol from that crate is referenced directly. Add an explicit
`use` or `extern crate` reference in `main.rs` if needed.

---

## Code Conventions

- **No `unwrap`/`expect`/`panic` in library code.** Use `?` and `thiserror`.
- **Public items get rustdoc.** Keep `missing_docs` warning clean.
- **`unsafe_code = "forbid"`.** Relax only when crate semantics require it (linkme,
  WASM bindgen) and document the exception inline.
- **`todo!`/`unimplemented!` are denied.** Stub with a returning `Err(...)` instead.
- **`dbg!` is denied.** Remove debug prints before committing.
- **No wall-clock in receipts.** Ordering is by monotonic `seq` only.

---

## License

MIT OR Apache-2.0
