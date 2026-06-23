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
├── tests/
│   ├── compile_tests.rs             # trybuild ALIVE gate
│   └── compile/
│       ├── pass/                    # compile-pass fixtures
│       └── fail/                    # compile-fail fixtures + .stderr snapshots
├── benches/            # Criterion benchmarks
├── examples/           # Runnable demonstrations (`cargo run --example`)
├── ontology/
│   └── domain.ttl      # RDF domain ontology — source of truth for ggen
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

### Full Typestate Pattern (`State<S>`)

When a domain object must progress through distinct phases (e.g., Pending →
Verified → Rejected) and incorrect phase usage should be a *compile-time* error,
use the full typestate pattern with a private `Sealed` supertrait:

```rust
// src/types.rs

mod sealed {
    mod private {
        pub trait Sealed {}
    }

    // This trait is NOT re-exported from the crate root.
    // Downstream code cannot implement it.
    pub trait Sealed: private::Sealed {}

    pub struct Pending;
    impl Sealed for Pending {}
    impl private::Sealed for Pending {}

    pub struct Verified;
    impl Sealed for Verified {}
    impl private::Sealed for Verified {}
}

pub use sealed::{Pending, Sealed, Verified};

pub struct State<S: Sealed> {
    _state: std::marker::PhantomData<S>,
}

impl State<Pending> {
    pub fn new() -> Self { State { _state: std::marker::PhantomData } }
    pub fn transition(self) -> State<Verified> {
        State { _state: std::marker::PhantomData }
    }
}
```

Usage — the type system enforces the state machine:

```rust
use crate::types::{State, Pending, Verified};

fn verify(s: State<Pending>) -> State<Verified> { s.transition() }

let pending: State<Pending> = State::new();
let verified: State<Verified> = verify(pending);

// Compile error — E0308: State<Verified> ≠ State<Pending>:
// verify(verified);

// Compile error — E0599: no method `transition` on State<Verified>:
// verified.transition();

// Compile error — E0277: MySpy does not implement sealed::private::Sealed:
// impl Sealed for MySpy {}
```

The double-module trick (`mod sealed { mod private { ... } }`) is the key: the
inner `private::Sealed` supertrait is completely unnameable outside `types`, so
nobody can add a new state marker or forge a `State<T>` for any `T` not defined
here.  This is the "full typestate" variant documented as INN-1 in the second-wave
survey (`wasm4pm-compat`).

**When to use full typestate vs. basic seal:**

| Situation | Use |
|---|---|
| One valid construction path, no phases | Basic `_seal: ()` |
| Multiple lifecycle phases, transitions must be enforced | Full `State<S: Sealed>` |
| External crates must never add new states | Full typestate (private supertrait) |

### Named Refusal Enums

Domain errors must be **named enums**, never bare `String`s or `anyhow::Error::msg`
calls.  Named variants make error handling exhaustive and let tests use `assert_eq!`:

```rust
// WRONG — loses structure at the call site:
return Err(anyhow::Error::msg("seq out of order"));

// RIGHT — caller can match on the variant, tests can assert_eq!:
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ReceiptRefusal {
    #[error("seq out of order: got {got}, expected {expected}")]
    SeqOutOfOrder { got: u32, expected: u32 },

    #[error("empty or blank event_id")]
    EmptyEventId,
}

return Err(ReceiptRefusal::SeqOutOfOrder { got: seq, expected: next });
```

```rust
// In tests — no string parsing needed:
assert_eq!(admit(5, 0), Err(ReceiptRefusal::SeqOutOfOrder { got: 5, expected: 0 }));
```

Every domain boundary gets its own `*Refusal` or `*Error` enum.  `AppError` in
`error.rs` wraps these for the top-level binary; library code returns the domain
enum directly.

### Compile-Time Ontology KEY Uniqueness (`assert_unique_ids`)

Every verb/class declared in `ontology/domain.ttl` must have a unique ID.
Duplicates are caught at **compile time** — not at test time, not at runtime —
using a `const fn` check:

```rust
// src/types.rs

pub const fn assert_unique_ids(ids: &[&str]) -> () {
    // O(n²) byte-by-byte comparison — runs only during compilation
    // Panics at compile time if any two IDs are identical
}

pub const ONTOLOGY_VERB_IDS: &[&str] = &[
    "dom:build",
    "dom:test",
    "dom:deploy",
    "dom:verify",
    // Add new IDs here when adding new verbs to domain.ttl
];

// This line causes a build failure if any ID appears twice:
const _: () = assert_unique_ids(ONTOLOGY_VERB_IDS);
```

When you add a new verb to `domain.ttl`, add its `dom:verbId` to
`ONTOLOGY_VERB_IDS` and re-run `cargo check`.  A duplicate is caught immediately:

```
error[E0080]: evaluation of constant value failed
  --> src/types.rs:419:5
   |
   = note: duplicate ontology ID detected — every ID must be globally unique
```

### trybuild ALIVE Gate

The `tests/compile_tests.rs` suite uses [`trybuild`](https://crates.io/crates/trybuild)
to verify that the type system rejects exactly what it should.  It is called the
"ALIVE" gate because if any fixture starts compiling when it should fail (or vice
versa), the gate turns red.

```bash
# Run the ALIVE gate:
cargo test --test compile_tests

# Regenerate .stderr snapshots after a Rust version upgrade:
TRYBUILD=overwrite cargo test --test compile_tests
```

**Fixture layout:**

```
tests/
├── compile_tests.rs           # Registers all fixtures with trybuild
└── compile/
    ├── pass/                  # Must compile with no errors
    │   ├── seal_via_builder.rs
    │   ├── typestate_transition.rs
    │   ├── refusal_eq.rs
    │   └── unique_ids_ok.rs
    └── fail/                  # Must produce the error in the .stderr file
        ├── seal_forgery.rs    + seal_forgery.stderr
        ├── typestate_wrong_state.rs + typestate_wrong_state.stderr
        └── sealed_impl_forgery.rs  + sealed_impl_forgery.stderr
```

**Adding a new compile-fail fixture:**

1. Create `tests/compile/fail/<name>.rs` with code that should not compile.
2. Run `TRYBUILD=overwrite cargo test --test compile_tests` to capture the
   expected `.stderr` snapshot.
3. Commit both the `.rs` and `.stderr` files.
4. Add `t.compile_fail("tests/compile/fail/<name>.rs");` to `compile_tests.rs`.

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
4. If the type has lifecycle phases, use `State<S: Sealed>` (full typestate).
5. If the type represents a domain rejection, define a named `*Refusal` enum
   with `#[derive(Debug, PartialEq, Eq, thiserror::Error)]` — never use bare strings.
6. Add a unit test in the same file.

### Adding a New Ontology Verb

1. Add a `dom:MyVerb` class to `ontology/domain.ttl` with `dom:verbId "dom:my-verb"`.
2. Add `"dom:my-verb"` to `ONTOLOGY_VERB_IDS` in `src/types.rs`.
3. Run `cargo check` — the `const _: () = assert_unique_ids(...)` binding will
   catch any duplicate at compile time.
4. Run `ggen sync` to regenerate the CLI verb stub in `src/verbs/`.

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

### `E0277` — `Sealed` trait not satisfied

You are trying to implement `Sealed` for a type not defined in `types.rs`, or
trying to call a function that requires `S: Sealed` with a type parameter that
is not one of `Pending`, `Verified`, or `Rejected`.  This is the non-forgery
guarantee of the full typestate pattern.  Only the state markers defined inside
`types::sealed` can satisfy the bound.

### `E0599` — method not found on `State<S>`

You are calling a lifecycle method (e.g., `transition()`) on a `State` value
that is in the wrong phase.  `transition()` is only defined on `State<Pending>`.
Check the state machine diagram in the `State` docs.

### `E0080` — duplicate ontology ID at compile time

You have added a duplicate entry to `ONTOLOGY_VERB_IDS` in `src/types.rs` (or
the ontology has two verbs with the same `dom:verbId`).  Every ID must be
globally unique.  Fix: remove or rename the duplicate, then `cargo check`.

### `trybuild` tests fail after Rust upgrade

The `.stderr` snapshot files in `tests/compile/fail/` contain the exact compiler
output from the toolchain version that generated them.  After upgrading
`rust-toolchain.toml`, regenerate all snapshots:

```bash
TRYBUILD=overwrite cargo test --test compile_tests
git add tests/compile/fail/*.stderr
git commit -m "chore: update trybuild snapshots for Rust <version>"
```

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

## Provenance Patterns

### BLAKE3 Chain Receipts

Every artifact that flows through the pipeline is content-addressed with BLAKE3:

```rust
use chatman_common::chain::{genesis_seed, fold_event, RollingChain};

// Build a rolling chain over a sequence of events
let mut chain = RollingChain::new("my-service");
chain.push(b"event-0");
chain.push(b"event-1");
let chain_hash = chain.finalize();  // 64-char lowercase hex
```

The rolling chain is tamper-evident: any change to an earlier event propagates to
every subsequent hash, making tampering detectable without out-of-band signatures.

### Signed Receipts (opt-in, feature `signed-receipts`)

For non-repudiable audit trails, enable the `signed-receipts` feature to wrap a
BLAKE3 chain hash with an ed25519 digital signature:

```toml
# Cargo.toml
chatman-common = { version = "...", features = ["provenance", "signed-receipts"] }
```

**Generating a key pair:**

```bash
# Generate a new ed25519 key pair (run once, store securely)
just keygen
# Output:
#   SIGNING KEY (secret): <64 hex chars>  ← store in PRAXIS_SIGNING_KEY
#   VERIFYING KEY (public): <64 hex chars> ← distribute to verifiers
```

Or in Rust:

```rust
use chatman_common::signed_receipt::KeyPair;

let kp = KeyPair::generate();
println!("SIGNING KEY: {}", kp.signing_key_hex());
println!("VERIFYING KEY: {}", kp.verifying_key_hex());
```

**Signing a chain hash:**

```rust
use chatman_common::signed_receipt::{sign, sign_with_env_key};

// Sign using a key from the environment (PRAXIS_SIGNING_KEY or PRAXIS_SIGNING_KEY_FILE)
let signed = sign_with_env_key(&chain_hash)?;

// Or sign with an explicit key
let signed = sign(&chain_hash, &signing_key_hex)?;

// Serialize to JSON for storage / transmission
let json = serde_json::to_string_pretty(&signed)?;
```

**Verifying a signed receipt:**

```rust
use chatman_common::signed_receipt::{SignedReceipt, verify};

let signed: SignedReceipt = serde_json::from_str(&json)?;
let is_valid = verify(&signed, &verifying_key_hex)?;
assert!(is_valid, "receipt signature verification failed");
```

**Attaching a signature to `TestReceipt`:**

```rust
use chatman_common::testkit::TestReceipt;

// When signed-receipts feature is active, sign() attaches a SignedReceipt
// using the key in PRAXIS_SIGNING_KEY (silently skips if key not set)
let receipt = TestReceipt::capture("integration_test", || { /* ... */ })
    .sign();

// Or sign with an explicit key
let kp = chatman_common::signed_receipt::KeyPair::generate();
let receipt = TestReceipt::record("my_test", true, 10)
    .sign_with(&kp.signing_key_hex())?;
```

**Key storage conventions:**

| Priority | Source | Format |
|----------|--------|--------|
| 1 | `PRAXIS_SIGNING_KEY` env var | 64 lowercase hex chars |
| 2 | File at `PRAXIS_SIGNING_KEY_FILE` env var | 64 lowercase hex chars, may have trailing newline |

The signing key is the **secret** 32-byte ed25519 seed. The verifying key is the
**public** key distributed to verifiers. Never commit the signing key to source
control; store it in a secrets manager or CI secret.

**Signing an existing receipt file:**

```bash
PRAXIS_SIGNING_KEY=<hex> just receipt-sign path/to/receipt.json
```

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
