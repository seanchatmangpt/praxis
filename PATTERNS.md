# Praxis House Patterns Guide

The definitive reference for `seanchatmangpt` Rust house style. Hand this to every new developer joining the fleet. Each pattern answers three questions: when to reach for it, what it looks like in code, and what problem it solves.

Sources: empirical survey of 18 repos plus second-wave deep dives into 10 repos. Every pattern here has shipped in at least one production repo in the fleet.

---

## Table of Contents

1. [Seal Pattern](#1-seal-pattern)
2. [Full Typestate: `const trait Witness`](#2-full-typestate-const-trait-witness)
3. [BLAKE3 Content Addressing](#3-blake3-content-addressing)
4. [Rolling Chain Hash](#4-rolling-chain-hash)
5. [CalVer Versioning](#5-calver-versioning)
6. [`linkme` Distributed Slices](#6-linkme-distributed-slices)
7. [Noun-Verb CLI](#7-noun-verb-cli)
8. [Named Refusal Enums](#8-named-refusal-enums)
9. [`BTreeMap` over `HashMap`](#9-btreemap-over-hashmap)
10. [`TestState<Phase>` Compile-Time AAA](#10-teststatephase-compile-time-aaa)
11. [`DocContext` Living Documentation](#11-doccontext-living-documentation)
12. [`TestReceipt` Auditable Test Records](#12-testreceipt-auditable-test-records)
13. [`trybuild` ALIVE Gate](#13-trybuild-alive-gate)
14. [Handle-Based WASM API (`Store<T>`)](#14-handle-based-wasm-api-storet)
15. [`CommonResponse<T>` MCP Pattern](#15-commonresponset-mcp-pattern)
16. [`TimeWindowArgs` Shared CLI Args](#16-timewindowargs-shared-cli-args)
17. [Git-as-Runtime](#17-git-as-runtime)
18. [Workspace Lint Inheritance](#18-workspace-lint-inheritance)
19. [Feature-Phased Architecture](#19-feature-phased-architecture)
20. [Anti-Patterns](#20-anti-patterns)

---

## 1. Seal Pattern

### When to use

Any domain object that must pass through a validation or hashing stage before it is trustworthy. The classic cases are receipts, proofs, and chain-assembled outputs — anything where out-of-thin-air construction would bypass invariants.

### How it looks in code

```rust
// src/types.rs
pub struct Receipt {
    pub format_version: String,
    pub events: Vec<Event>,
    pub chain_hash: String,
    pub profile: String,
    _seal: (),   // private — struct-literal construction fails at compile time (E0451)
}

impl Receipt {
    /// Only ChainAssembler::finalize calls this, and only after full validation.
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

Callers outside the module get full read access to public fields but cannot fabricate a `Receipt`:

```rust
// This fails at compile time — E0451: field `_seal` of struct `Receipt` is private
let r = Receipt { events: vec![], chain_hash: "...".into(), _seal: () };

// This is the only path that produces a Receipt:
let r = assembler.finalize()?;
```

When the type needs to survive `serde` round-trips, skip the field in serialization:

```rust
#[serde(skip)]
_seal: (),
```

### What it prevents

Callers constructing instances that bypass the admission gate. The enforcement is at compile time — no runtime check, no documentation convention, no code review required. `E0451` is the compiler enforcing your invariant.

### Use this when

The type has invariants that must be established by a specific constructor path (hashing, validation, sequencing). The rule of thumb: if there is a `build()`/`finalize()`/`seal()` method and you do not want anyone to bypass it, add `_seal: ()`.

### Do not use this when

The type is a plain data carrier with no construction-time invariants. Adding `_seal: ()` to `struct Point { x: f64, y: f64 }` is noise.

**Fleet examples:** `affidavit` (Receipt), `wasm4pm-compat` (Evidence<T, State, W>), template `src/types.rs`.

---

## 2. Full Typestate: `const trait Witness`

### When to use

When the Seal pattern is not enough — you need to prove at compile time not just that construction went through the right path, but *which authority* signed off on a specific state transition. Used in lifecycle-sensitive objects where multiple parties must validate independently.

### How it looks in code

The basic typestate uses `PhantomData` to track the current phase:

```rust
pub struct Evidence<T, State: EvidenceState, W> {
    inner: T,
    _state: PhantomData<State>,
    _witness: PhantomData<W>,
    _seal: (),   // still present: prevents fabrication even with correct type params
}

// State markers (zero-size types)
pub struct Unverified;
pub struct Verified;
pub struct Committed;

pub trait EvidenceState: sealed::Sealed {}
impl EvidenceState for Unverified {}
impl EvidenceState for Verified {}
impl EvidenceState for Committed {}

// Witness labels (zero-cost authority tokens)
pub struct HumanReviewer;
pub struct AutomatedSuite;
```

State transitions are only available from specific implementors:

```rust
impl<T, W> Evidence<T, Unverified, W> {
    /// Only callable by the verifier module — not pub.
    pub(crate) fn verify(self) -> Evidence<T, Verified, W> {
        Evidence { inner: self.inner, _state: PhantomData, _witness: PhantomData, _seal: () }
    }
}

impl<T, W> Evidence<T, Verified, W> {
    /// Commit requires a human-review witness token.
    pub(crate) fn commit(self) -> Evidence<T, Committed, HumanReviewer> {
        Evidence { inner: self.inner, _state: PhantomData, _witness: PhantomData, _seal: () }
    }
}
```

A function that requires committed evidence cannot accidentally receive unverified evidence:

```rust
// This signature is enforced by the type system — Unverified never satisfies it.
fn publish(e: Evidence<Payload, Committed, HumanReviewer>) { ... }
```

### What it prevents

State confusion: calling a "publish" step with evidence that has not been verified, or calling "commit" without a human-review witness. Unlike runtime guards (`if !self.verified { panic! }`), the type system eliminates the entire class of invalid transitions — the erroneous code does not compile.

### Use this when

Objects have a well-defined lifecycle (Unverified → Verified → Committed → Published) and multiple distinct authorities must be distinguished. The "exactly-N-features discipline" from `wasm4pm-compat` applies here: keep the number of state types small (three to five) or the API surface explodes.

### Do not use this when

The lifecycle has only one meaningful state, or the phases are purely informational. Use the simple `_seal: ()` pattern instead.

**Fleet examples:** `wasm4pm-compat` (Evidence<T, State, W> with 444 compile-fail fixtures), `bcinr` (OCEL event lifecycle).

---

## 3. BLAKE3 Content Addressing

### When to use

Whenever you need a stable, portable identity for an artifact, receipt, event, or message. Use BLAKE3 as the identity — not UUIDs, not wall-clock timestamps, not database auto-increment IDs.

### How it looks in code

The canonical type is `Blake3Hash`, a thin newtype over the 64-character lowercase hex string:

```rust
// src/types.rs
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Blake3Hash(pub String);

impl Blake3Hash {
    /// Hash raw bytes and return the hex digest. Primary constructor.
    pub fn content_address(bytes: &[u8]) -> Self {
        Blake3Hash(blake3::hash(bytes).to_hex().to_string())
    }

    /// Reconstruct from a known hex string (deserialization only).
    pub fn from_hex(hex: impl Into<String>) -> Self {
        Blake3Hash(hex.into())
    }

    pub fn as_hex(&self) -> &str { &self.0 }
}
```

Always hash **canonical bytes**, not raw struct bytes or pretty-printed JSON:

```rust
// canonical_bytes() sorts JSON keys, removes whitespace — deterministic across all platforms
let bytes = canonical_bytes(&my_event)?;
let hash = Blake3Hash::content_address(&bytes);
```

The `canonical_bytes` function (defined in `types.rs`, re-exported from `chatman_common`):

```rust
pub fn canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
    let v = serde_json::to_value(value)?;
    let sorted = sort_value(v);        // recursively sorts object keys via BTreeMap
    serde_json::to_vec(&sorted)        // no whitespace
}
```

### What it prevents

- Identity collisions that UUIDs cannot rule out
- Non-reproducible identifiers that embed wall-clock time
- Byte-order or field-order sensitivity in hashed payloads (canonical serialization guards this)
- Forgery: the hash of tampered bytes will not match the original hash

### Use this when

Creating receipts, committing artifacts, building audit trails, or deduplicating any content. The 64-char hex string is human-diffable and JSON-safe.

### Do not use this when

Ordering is the only concern (use `seq: u64`). BLAKE3 is not a sequence number.

**Fleet examples:** `affidavit`, `lsp-max`, `bcinr`, `wasm4pm`, `wasm4pm-compat`, `gitvan` (JavaScript BLAKE3 via `@noble/hashes`), template `src/types.rs`.

---

## 4. Rolling Chain Hash

### When to use

Event streams, audit logs, or any append-only sequence where tampering with any entry should be detectable by verifying the final hash — without storing a hash per entry. The chain hash is the BLAKE3 equivalent of a Merkle-path authentication.

### How it looks in code

The genesis seed is project-scoped to prevent cross-project chain splicing:

```rust
// src/chain.rs
const GENESIS_SEED_STR: &str = concat!("{{project-name}}-v", env!("CARGO_PKG_VERSION"), "-genesis");
pub const GENESIS_SEED: &[u8] = GENESIS_SEED_STR.as_bytes();

fn genesis_hash() -> Blake3Hash {
    Blake3Hash::from_hex(blake3::hash(GENESIS_SEED).to_hex().to_string())
}

// Fold rule: blake3(prev_hex_bytes || event_bytes)
fn fold(prev: &Blake3Hash, event_bytes: &[u8]) -> Blake3Hash {
    let mut buf = Vec::with_capacity(prev.as_hex().len() + event_bytes.len());
    buf.extend_from_slice(prev.as_hex().as_bytes());
    buf.extend_from_slice(event_bytes);
    Blake3Hash::from_hex(blake3::hash(&buf).to_hex().to_string())
}
```

The `ChainAssembler` holds the running hash so `finalize()` is O(1):

```rust
pub struct ChainAssembler {
    running: Blake3Hash,
}

impl ChainAssembler {
    pub fn new() -> Self {
        ChainAssembler { running: genesis_hash() }
    }

    pub fn append(&mut self, event_bytes: &[u8]) -> Blake3Hash {
        self.running = fold(&self.running, event_bytes);
        self.running.clone()
    }

    /// Consume the assembler and return the final chain hash.
    pub fn finalize(self) -> String {
        self.running.into()
    }
}
```

Verification recomputes from scratch — used by the verifier, not by assembler:

```rust
pub fn recompute_chain(events: &[impl AsRef<[u8]>]) -> String {
    let mut acc = genesis_hash();
    for e in events {
        acc = fold(&acc, e.as_ref());
    }
    acc.into()
}
```

Usage:

```rust
let mut asm = ChainAssembler::new();
for event in &events {
    let bytes = canonical_bytes(event)?;
    asm.append(&bytes);
}
let receipt = Receipt::seal(asm.finalize(), events);
```

### What it prevents

Undetected tampering of any event in the sequence. Deleting, reordering, or modifying any event changes every subsequent hash and breaks verification. The genesis seed prevents splicing events from a different project's chain.

### Use this when

Building audit trails, OCEL event logs, receipts with sequences, or any append-only log where integrity verification is required.

### Do not use this when

The set is unordered or events are independent. Use per-event `content_address` hashes instead.

**Fleet examples:** `affidavit` (ChainAssembler → Receipt), `lsp-max`, `bcinr` (OCEL receipts), template `src/chain.rs`.

---

## 5. CalVer Versioning

### When to use

All repos in the fleet. The only exception is prototypes or published crates that must obey semantic versioning for external consumers.

### How it looks in code

Version format: `YY.M.patch` where `YY` is the two-digit year, `M` is the month (no leading zero), and `patch` increments within the month starting at zero.

```toml
# Cargo.toml
[package]
version = "26.6.0"   # June 2026, first release of the month
```

Subsequent releases within June:

```
26.6.0   # first
26.6.1   # second
26.6.2   # hotfix
```

In a workspace, set version once at the root and inherit everywhere:

```toml
# workspace Cargo.toml
[workspace.package]
version = "26.6.0"

# member crate Cargo.toml
[package]
version.workspace = true
```

The CI release workflow derives the tag from this scheme: pushing `v26.6.0` triggers the release job.

### What it prevents

Debates about major/minor/patch semantics for internal tooling. CalVer communicates recency — `26.6.0` vs `26.1.3` conveys immediate temporal context without semantic promises.

### Use this when

Internal tooling, binaries, or any crate where "how recent is this?" matters more than semantic API stability guarantees.

### Do not use this when

Publishing a library to crates.io that external consumers version-pin with SemVer constraints. Use SemVer in that narrow case and note the exception in `CLAUDE.md`.

**Fleet examples:** `affidavit` (26.6.17), `clap-noun-verb` (26.6.14), `lsp-max` (26.6.18), `bcinr` (26.6.x), `ggen` (26.6.DD — note: fleet prefers `.patch` over `.DD` since `.DD` cannot exceed one release per day).

---

## 6. `linkme` Distributed Slices

### When to use

Plugin systems, verb registries, event-handler dispatch, or any situation where you want downstream crates to register items without a central registry. Zero runtime scanning, zero `ctor` hooks, zero `inventory` crate.

### How it looks in code

Declare the slice once in `src/discovery.rs`:

```rust
use linkme::distributed_slice;
use crate::handlers::Handler;

#[distributed_slice]
pub static HANDLERS: [Handler] = [..];
```

Register a handler from any module — including in downstream crates:

```rust
use linkme::distributed_slice;
use crate::discovery::HANDLERS;

#[distributed_slice(HANDLERS)]
pub static BUILD_HANDLER: Handler = Handler::new("build", handle_build_event);

fn handle_build_event(payload: &[u8]) -> anyhow::Result<()> {
    // ...
    Ok(())
}
```

Iterate at startup in `src/handlers.rs`:

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

Important: `HANDLERS` must be `[T]` (unsized slice), not `Vec<T>`. The `= [..]` initializer is required by the `linkme` syntax.

If the linker dead-strips registrations because no symbol from that crate is referenced directly, add an explicit reference in `main.rs`:

```rust
// Force the linker to include the crate's registrations.
#[allow(unused_imports)]
use my_plugin_crate as _;
```

### What it prevents

Central registries that require modification every time a new handler or verb is added. With distributed slices, adding a new verb is entirely local to the new module — the dispatch loop picks it up at link time with no central change.

### Use this when

Multiple handlers need to register for the same dispatch point, especially across crate boundaries. The noun-verb CLI (pattern 7) uses this internally.

### Do not use this when

There are only two or three handlers and they will never be extended. A `match` statement is clearer. Also: distributed slices require `linkme`, which uses `unsafe` internally — this means the crate must relax `unsafe_code` from `forbid` to `warn` and document why.

**Fleet examples:** `affidavit`, `clap-noun-verb`, `clnrm`, `mac-artifact-cleaner`. Template `src/discovery.rs`.

---

## 7. Noun-Verb CLI

### When to use

Any binary with more than one subcommand. The pattern enforces that `cli.rs` owns parsing only, and business logic lives in `src/verbs/<verb>.rs`.

### How it looks in code

The CLI enum via `clap-noun-verb`:

```rust
// src/cli.rs
use clap_noun_verb::NounVerb;
use crate::verbs::{emit::EmitArgs, assemble::AssembleArgs, verify::VerifyArgs};

#[derive(NounVerb)]
pub enum Cli {
    /// Emit an event to the working chain
    Emit(EmitArgs),
    /// Assemble the working chain into a sealed receipt
    Assemble(AssembleArgs),
    /// Verify a sealed receipt
    Verify(VerifyArgs),
}
```

Each verb in `src/verbs/<verb>.rs`:

```rust
// src/verbs/emit.rs
use clap::Args;

#[derive(Args, Debug)]
pub struct EmitArgs {
    #[arg(long)]
    pub event_type: String,
    #[arg(long, default_value = "json")]
    pub format: String,
}

pub async fn handle_emit(args: EmitArgs) -> anyhow::Result<()> {
    // business logic here, not in cli.rs
    Ok(())
}
```

Wire in the binary entry point:

```rust
// src/bin/myapp.rs
let cli = Cli::parse();
match cli {
    Cli::Emit(args)     => handle_emit(args).await?,
    Cli::Assemble(args) => handle_assemble(args).await?,
    Cli::Verify(args)   => handle_verify(args).await?,
}
```

Step-by-step checklist for adding a new verb:

1. Create `src/verbs/<verb>.rs` with `<Verb>Args` and `handle_<verb>`.
2. Export in `src/verbs/mod.rs`.
3. Add the variant to the `Cli` enum in `src/cli.rs`.
4. Wire the match arm in `src/bin/<name>.rs`.
5. Add an integration test in `tests/<verb>.rs`.

### What it prevents

God-module `main.rs` files where argument parsing and business logic are interleaved. With the noun-verb pattern, `cli.rs` is a stable interface; verbs change independently.

### Use this when

Building CLIs with multiple subcommands. The pattern also pairs with `linkme` distributed slices for fully declarative verb registration.

### Do not use this when

The binary has exactly one action (run it directly) or is a pure library with no CLI surface.

**Fleet examples:** `affidavit`, `cargo-cicd`, `mac-artifact-cleaner`, `clap-noun-verb` (the crate itself). Template `src/cli.rs` + `src/verbs/`.

---

## 8. Named Refusal Enums

### When to use

Anywhere an operation can fail. Never use `String` as an error type. Named enum variants are the unit of failure taxonomy.

### How it looks in code

```rust
// src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("receipt file not found: {path}")]
    ReceiptNotFound { path: std::path::PathBuf },

    #[error("chain hash mismatch: expected {expected}, got {actual}")]
    ChainMismatch { expected: String, actual: String },

    #[error("event sequence gap: expected seq {expected}, got {got}")]
    SequenceGap { expected: u64, got: u64 },

    #[error("malformed commitment digest: {0}")]
    MalformedDigest(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialize(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
```

In library code, return `Error` variants directly — never `anyhow::Error` from library functions:

```rust
pub fn verify_chain(receipt: &Receipt) -> Result<()> {
    let computed = recompute_chain(&receipt.events);
    if computed != receipt.chain_hash {
        return Err(Error::ChainMismatch {
            expected: receipt.chain_hash.clone(),
            actual: computed,
        });
    }
    Ok(())
}
```

In binary/CLI code, `anyhow` is acceptable for context chaining:

```rust
// src/verbs/verify.rs
pub async fn handle_verify(args: VerifyArgs) -> anyhow::Result<()> {
    let receipt = load_receipt(&args.path)
        .with_context(|| format!("loading receipt from {}", args.path.display()))?;
    verify_chain(&receipt)
        .with_context(|| "chain verification failed")?;
    Ok(())
}
```

### What it prevents

Stringly-typed errors that cannot be matched, tested, or evolved. When an error is `String`, callers must parse the message to understand the failure kind. When it is an enum variant, callers can `match` on it, tests can assert `assert_fail!(result, Error::ChainMismatch { .. })`, and adding context to a variant is non-breaking.

### Use this when

Writing any function that can fail. Library functions must use the named enum. Binary / CLI glue may use `anyhow` for convenience chaining.

### Do not use this when

There is genuinely only one failure mode and the error never needs to be pattern-matched (rare). Even then, prefer a single-variant enum over `String`.

**Fleet examples:** Every repo in the fleet. `thiserror` 2 is the house dep (`affidavit`, `ggen`, `lsp-max`, `cargo-cicd`, `dteam`). Template `src/error.rs`.

---

## 9. `BTreeMap` over `HashMap`

### When to use

Any map whose iteration order appears in output, is serialized, or is hashed. In WASM contexts: always. In any code that produces content-addressed output: always.

### How it looks in code

```rust
use std::collections::BTreeMap;

// In WASM-exposed types — hash randomization is enabled by default in Rust
// and makes HashMap non-deterministic across runs
pub fn aggregate_events(events: &[Event]) -> BTreeMap<String, Vec<Event>> {
    let mut by_type: BTreeMap<String, Vec<Event>> = BTreeMap::new();
    for event in events {
        by_type
            .entry(event.event_type.clone())
            .or_default()
            .push(event.clone());
    }
    by_type   // iteration order: lexicographic by key, always
}
```

The `sort_value` function in `canonical_bytes` uses `BTreeMap` internally to sort JSON object keys:

```rust
fn sort_value(value: serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    match value {
        Value::Object(map) => {
            let sorted: BTreeMap<String, Value> =
                map.into_iter().map(|(k, v)| (k, sort_value(v))).collect();
            // ... rebuild as serde_json::Map in sorted order
        }
        // ...
    }
}
```

### What it prevents

Non-reproducible output. `HashMap` iteration order is randomized by default in Rust (using a random seed per process start). Serializing a `HashMap` produces different JSON bytes on different runs — identical logical content produces different BLAKE3 hashes, breaking content addressing and all downstream verification.

### Use this when

- Any map serialized to JSON that will be hashed
- WASM guest functions (pure functions must be deterministic)
- Test fixtures and golden-file comparisons
- Any output checked by `assert_eq!` in a way that depends on ordering

### Do not use this when

The map is used purely for in-memory lookups with no serialization or output, and performance on large maps matters more than order. `HashMap` is O(1) amortized; `BTreeMap` is O(log n).

**Fleet examples:** `wasm4pm` (BTreeMap throughout WASM-exposed API), template `src/types.rs` (`sort_value`), `canonical_bytes`.

---

## 10. `TestState<Phase>` Compile-Time AAA

### When to use

Tests that have a strict Arrange → Act → Assert structure and where accidentally skipping or reordering phases would cause false positives. The pattern makes phase skipping a compile error.

### How it looks in code

The `TestState<Phase>` type and its transitions live in `chatman_common::testkit`:

```rust
// Arrange phase
pub struct Arrange;
// Act phase
pub struct Act;
// Assert phase
pub struct Assert;

pub struct TestState<Phase> {
    _phase: PhantomData<Phase>,
}

impl TestState<Arrange> {
    pub fn new() -> Self { TestState { _phase: PhantomData } }
    pub fn act(self) -> TestState<Act> { TestState { _phase: PhantomData } }
}

impl TestState<Act> {
    pub fn assert(self) -> TestState<Assert> { TestState { _phase: PhantomData } }
}
```

Usage in a test:

```rust
#[test]
fn verify_rejects_tampered_receipt() {
    // Arrange
    let state = TestState::new();
    let mut asm = ChainAssembler::new();
    asm.append(b"event-1");
    let chain_hash = asm.finalize();
    let state = state.act();

    // Act
    let result = verify_chain_hash("tampered-event", &chain_hash);
    let state = state.assert();

    // Assert — state is now TestState<Assert>; calling state.act() here is a compile error
    let _ = state;
    assert!(result.is_err(), "tampered event must not verify");
}
```

Trying to jump from Arrange to Assert (skipping Act) does not compile because `TestState<Arrange>` has no `.assert()` method.

### What it prevents

Tests that accidentally assert before acting, or act-then-arrange patterns that produce confusing results. The type system enforces the test contract — the same mechanism used in production code (pattern 2) applied to test structure.

### Use this when

Integration tests with non-trivial setup, tests where acting before arranging would silently pass, or team conventions that require documented test structure.

### Do not use this when

Simple unit tests with a single `assert_eq!` — the overhead is not worth it there.

**Fleet examples:** `chicago-tdd-tools` (original source), `chatman_common::testkit` (integrated implementation). See `crates/chatman-common/src/testkit.rs`.

---

## 11. `DocContext` Living Documentation

### When to use

When test output should become committed documentation. Tests that validate behavior should simultaneously produce a Markdown artifact describing that behavior. This is the "tests as the source of documentation truth" pattern.

### How it looks in code

`DocContext` lives in `chatman_common::testkit` behind the `living-docs` feature:

```rust
#[cfg(test)]
#[cfg(feature = "living-docs")]
mod docs_tests {
    use chatman_common::testkit::{DocContext, doc_assert};

    #[test]
    fn document_chain_verification() {
        let mut ctx = DocContext::for_test("chain_verification");
        ctx.say_section("Chain Integrity Verification");
        ctx.say("The rolling chain hash detects any tampering with the event sequence.");

        // Setup
        let mut asm = ChainAssembler::new();
        asm.append(b"event-1");
        asm.append(b"event-2");
        let chain_hash = asm.finalize();

        ctx.say_code("rust", "let chain_hash = assembler.finalize();");

        // The doc_assert! macro asserts AND documents — assertion failure prevents doc emission
        doc_assert!(ctx, "honest chain verifies", verify_chain(&chain_hash).is_ok());
        doc_assert!(ctx, "tampered chain fails", verify_chain("corrupted").is_err());

        ctx.say_table(
            &["Property", "Value"],
            &[
                &["Hash length", "64 chars"],
                &["Algorithm", "BLAKE3"],
                &["Key sort", "Lexicographic"],
            ],
        );

        ctx.finish().unwrap();
        // Writes docs/test/chain_verification.md with a BLAKE3 footer hash
    }
}
```

The `doc_assert!` macro is atomic: if the assertion fails, the test panics and the documentation line is never written. A passing test always produces accurate documentation.

`finish()` writes `docs/test/<name>.md` relative to the crate root and appends a BLAKE3 chain hash of the rendered content — so the doc file is itself content-addressed.

Update docs by running tests with the `living-docs` feature:

```bash
# Regenerate all living docs
UPDATE_GOLDEN=1 cargo test --features living-docs -- --test-threads=1

# Verify docs are up to date (CI mode)
cargo test --features living-docs -- --test-threads=1
```

### What it prevents

Documentation that drifts from the implementation. Because documentation is generated by tests that must pass, stale docs are impossible — a failing test means no doc is emitted, and a change in behavior forces a doc update.

### Use this when

Documenting protocol invariants, API behavior, or any property that the test suite already verifies. The pattern shines in audit-trail repos where the docs serve as evidence artifacts.

### Do not use this when

Documentation is mostly prose narrative with no machine-verifiable claims. Plain Markdown is appropriate there.

**Fleet examples:** `dtr` (Java original, `sayAndAssert` pattern), `chatman_common::testkit` (Rust integration). See `crates/chatman-common/src/testkit.rs` lines 429–625.

---

## 12. `TestReceipt` Auditable Test Records

### When to use

Any test suite where auditability matters — compliance tools, pipeline correctness proofs, or repos that need to demonstrate that a specific version of the test suite passed on a specific platform.

### How it looks in code

```rust
use chatman_common::testkit::TestReceipt;

#[test]
fn audit_chain_integrity() {
    // capture() runs the closure, catches panics, and records the result
    let receipt = TestReceipt::capture("chain_integrity", || {
        let mut asm = ChainAssembler::new();
        asm.append(b"a");
        asm.append(b"b");
        let h1 = asm.finalize();

        let h2 = recompute_chain(&[b"a", b"b"]);
        assert_eq!(h1, h2);
    });

    assert!(receipt.passed);
    assert!(receipt.duration_ms < 100, "hash should be fast");

    // With "living-docs" feature: chain_hash is populated
    // receipt.chain_hash = Some("a3f...") — BLAKE3 of (name|passed|duration|os)
    println!("receipt: {:?}", receipt);
}
```

The `EnvironmentFingerprint` captures OS, Rust version, target architecture, and timestamp — all the context needed to reproduce the result:

```rust
pub struct EnvironmentFingerprint {
    pub os: String,
    pub rust_version: &'static str,   // from CARGO_PKG_RUST_VERSION
    pub target: &'static str,         // std::env::consts::ARCH
    pub timestamp_unix: i64,
}
```

### What it prevents

Test results that exist only in CI logs. `TestReceipt` is a first-class value that can be serialized, committed, or emitted to an audit ledger. The chain hash links the receipt to its content — tampering with the record changes the hash.

### Use this when

Building compliance tooling, creating provenance trails for CI runs, or any context where "this test passed" needs to be a signed artifact rather than a CI badge.

### Do not use this when

Tests are purely exploratory or the audit overhead adds no value. Normal `#[test]` functions need no `TestReceipt`.

**Fleet examples:** `chicago-tdd-tools` (original), `affidavit` (OCEL event receipts), `chatman_common::testkit`.

---

## 13. `trybuild` ALIVE Gate

### When to use

After implementing the Seal pattern or typestate typesystem (patterns 1 and 2). Compile-fail tests verify that the type system rejects exactly what it should reject — they are tests for your tests.

### How it looks in code

Add `trybuild` to `[dev-dependencies]`:

```toml
[dev-dependencies]
trybuild = "1"
```

Create fixture files that should fail to compile:

```rust
// tests/compile_fail/seal_bypass.rs
fn main() {
    // This must NOT compile — _seal is private
    let r = my_crate::Receipt {
        events: vec![],
        chain_hash: "abc".into(),
        _seal: (),   // E0451: field `_seal` of struct `Receipt` is private
    };
}
```

And fixture files that should compile successfully:

```rust
// tests/compile_pass/builder_path.rs
fn main() {
    // This MUST compile — the canonical builder path
    let mut asm = my_crate::ChainAssembler::new();
    asm.append(b"event");
    let _chain_hash = asm.finalize();
}
```

The integration test that runs them:

```rust
// tests/type_system.rs
#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}

#[test]
fn compile_pass_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile_pass/*.rs");
}
```

Run the full `trybuild` suite as the "ALIVE gate" to confirm the type system is working:

```just
# justfile
alive-gate:
    cargo test --test type_system
```

Add this to the CI job list so the ALIVE gate runs on every PR.

### What it prevents

Regressions in type-system guarantees. Without compile-fail tests, a refactor might accidentally make `_seal` public or remove the typestate constraint — and no runtime test would catch it. `trybuild` makes type-system contracts as testable as runtime behavior.

### Use this when

You have implemented the Seal pattern, typestate lifecycle, or any compile-time constraint that needs ongoing verification. The gold standard: one compile-fail fixture per rejected construct you claim to enforce.

### Do not use this when

There are no type-system invariants to protect. Adding `trybuild` with no fixtures is dead weight.

**Fleet examples:** `wasm4pm-compat` (444 compile-fail + 413 compile-pass fixtures — the most rigorous use in the fleet).

---

## 14. Handle-Based WASM API (`Store<T>`)

### When to use

Any WASM crate that exposes Rust objects to JavaScript. Never pass raw pointers across the WASM boundary — pass opaque string handles that identify objects held in a Rust-side store.

### How it looks in code

The `Store<T>` holds objects by string handle on the Rust side:

```rust
// src/store.rs
use std::collections::BTreeMap;  // BTreeMap: deterministic iteration order
use wasm_bindgen::prelude::*;

thread_local! {
    static STORE: std::cell::RefCell<Store<ChainAssembler>> =
        std::cell::RefCell::new(Store::new());
}

pub struct Store<T> {
    inner: BTreeMap<String, T>,
    counter: u64,
}

impl<T> Store<T> {
    pub fn new() -> Self {
        Store { inner: BTreeMap::new(), counter: 0 }
    }

    pub fn insert(&mut self, value: T) -> String {
        let handle = format!("handle-{}", self.counter);
        self.counter += 1;
        self.inner.insert(handle.clone(), value);
        handle
    }

    pub fn get(&self, handle: &str) -> Option<&T> {
        self.inner.get(handle)
    }

    pub fn get_mut(&mut self, handle: &str) -> Option<&mut T> {
        self.inner.get_mut(handle)
    }

    pub fn remove(&mut self, handle: &str) -> Option<T> {
        self.inner.remove(handle)
    }
}
```

WASM-bindgen entry points return and accept handles, never raw objects:

```rust
#[wasm_bindgen]
pub fn create_assembler() -> String {
    STORE.with(|s| s.borrow_mut().insert(ChainAssembler::new()))
}

#[wasm_bindgen]
pub fn append_event(handle: &str, event_json: &str) -> Result<(), JsValue> {
    STORE.with(|s| {
        let mut store = s.borrow_mut();
        let asm = store.get_mut(handle)
            .ok_or_else(|| JsValue::from_str("handle not found"))?;
        let bytes = event_json.as_bytes();
        asm.append(bytes);
        Ok(())
    })
}

#[wasm_bindgen]
pub fn finalize_assembler(handle: String) -> String {
    STORE.with(|s| {
        let mut store = s.borrow_mut();
        store.remove(&handle)
            .map(|asm| asm.finalize())
            .unwrap_or_default()
    })
}
```

JavaScript usage:

```javascript
const handle = create_assembler();
append_event(handle, JSON.stringify({ seq: 0, event_type: "build" }));
const chainHash = finalize_assembler(handle);
```

### What it prevents

Memory safety violations from mismanaged lifetimes across the WASM boundary. Raw pointers passed to JavaScript can dangle if the Rust-side object is dropped before JS is done. Handles are just strings — the Rust store owns the memory, and the JavaScript side never touches it directly.

### Use this when

Building WASM crates that expose stateful Rust objects to JavaScript. Also: `console_error_panic_hook` for human-readable panics in the browser; `panic = "abort"` + `opt-level = "s"` + `strip = false` in the release profile.

### Do not use this when

The WASM module is stateless (pure functions only). In that case, passing serialized JSON in and out is sufficient.

**Fleet examples:** `wasm4pm` (five deployment profiles, handle-based API throughout), `pm4wasm`, `miniml`.

---

## 15. `CommonResponse<T>` MCP Pattern

### When to use

All MCP tool implementations. Every tool response must include `passed: bool` so CI gate tools can pattern-match the result without parsing tool-specific output, and `result_hash` so the response is content-addressed.

### How it looks in code

```rust
// src/shared_args.rs
use blake3;
use serde::{Deserialize, Serialize};

/// Standard response envelope for all MCP tool calls.
#[derive(Debug, Serialize, Deserialize)]
pub struct CommonResponse<T: Serialize> {
    /// Whether the tool operation succeeded. Always present, even on error.
    /// CI gates check this field; tool-specific fields are secondary.
    pub passed: bool,
    /// Human-readable status message.
    pub message: String,
    /// Tool-specific payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// BLAKE3 hex digest of the canonical JSON bytes of `data` (when present).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_hash: Option<String>,
}

impl<T: Serialize> CommonResponse<T> {
    pub fn ok(message: impl Into<String>, data: T) -> Self {
        let json = serde_json::to_vec(&data).unwrap_or_default();
        let result_hash = Some(blake3::hash(&json).to_hex().to_string());
        CommonResponse {
            passed: true,
            message: message.into(),
            data: Some(data),
            result_hash,
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        CommonResponse {
            passed: false,
            message: message.into(),
            data: None,
            result_hash: None,
        }
    }
}
```

Tool implementation:

```rust
use rmcp::tool;
use crate::shared_args::{CommonResponse, TimeWindowArgs};

#[derive(Default)]
pub struct AnalysisTool;

#[tool(description = "Analyze event patterns in the given time window")]
impl AnalysisTool {
    async fn analyze(&self, #[tool(aggr)] args: TimeWindowArgs) -> CommonResponse<Vec<String>> {
        match run_analysis(args.hours, args.limit).await {
            Ok(results) => CommonResponse::ok("analysis complete", results),
            Err(e) => CommonResponse::err(format!("analysis failed: {e}")),
        }
    }
}
```

### What it prevents

Heterogeneous tool responses that each require custom parsing. When every tool wraps its output in `CommonResponse`, a CI gate can check `response.passed` without knowing which tool it called. The `result_hash` enables downstream verification: "the analysis I acted on had this exact content."

### Use this when

Building any MCP server in the fleet. The `passed: bool` field is mandatory — even on errors, include it as `false` rather than omitting the field.

### Do not use this when

Building non-MCP APIs. This pattern is specific to the MCP tool-call contract.

**Fleet examples:** `ggen-mcp` (Rust, `rmcp`), `pm4py-mcp` (Python, FastMCP — has `passed` but not `result_hash`; the Rust template adds it).

---

## 16. `TimeWindowArgs` Shared CLI Args

### When to use

Any verb or MCP tool that operates over a time window. Rather than re-declaring `--hours` and `--limit` in every verb, compose from `TimeWindowArgs`.

### How it looks in code

```rust
// src/shared_args.rs (or chatman_common::cli)
use clap::Args;
use serde::Serialize;

/// Shared time-window parameters for any tool or verb that filters by recency.
#[derive(Args, Debug, Clone, Serialize)]
pub struct TimeWindowArgs {
    /// Number of hours to look back
    #[arg(long, default_value = "24")]
    pub hours: u32,
    /// Maximum number of results to return
    #[arg(long, default_value = "1000")]
    pub limit: usize,
}
```

Compose into a verb's args with `#[command(flatten)]`:

```rust
#[derive(Args, Debug)]
pub struct AnalyzeArgs {
    #[command(flatten)]
    pub window: TimeWindowArgs,
    #[arg(long, default_value = "json")]
    pub format: String,
}
```

Or compose into an MCP tool with `#[tool(aggr)]`:

```rust
async fn analyze(&self, #[tool(aggr)] args: TimeWindowArgs) -> CommonResponse<Vec<String>> {
    run_analysis(args.hours, args.limit).await
}
```

### What it prevents

Inconsistent `--hours` / `--limit` defaults across verbs. Without a shared struct, one verb defaults to 24 hours and another to 72, or one spells it `--limit` and another `--max-results`.

### Use this when

More than one verb or tool has a time-window concept. Even if only one verb uses it today, using `TimeWindowArgs` signals that it is part of the house interface contract.

### Do not use this when

The time concept in the verb is meaningfully different (e.g., a duration in seconds, not a lookback in hours). In that case, define a different named struct.

**Fleet examples:** `pm4py-mcp` (Python original), `ggen-mcp` (Rust MCP server). Template `template-mcp/src/shared_args.rs`.

---

## 17. Git-as-Runtime

### When to use

Distributed coordination and audit ledger requirements. When multiple processes need to coordinate writes without a shared database, and when audit trails need to be tamper-evident and append-only.

### How it looks in code

**Distributed CAS lock via `git update-ref`:**

```rust
use std::process::Command;

/// Acquire a distributed lock by atomically creating a git ref.
/// Returns `Ok(())` if the lock was acquired, `Err` if another process holds it.
pub fn acquire_lock(lock_name: &str, value: &str) -> anyhow::Result<()> {
    let status = Command::new("git")
        .args(["update-ref", &format!("refs/locks/{lock_name}"), value, ""])
        .status()?;
    if !status.success() {
        anyhow::bail!("lock {lock_name} is held by another process");
    }
    Ok(())
}

pub fn release_lock(lock_name: &str) -> anyhow::Result<()> {
    let status = Command::new("git")
        .args(["update-ref", "-d", &format!("refs/locks/{lock_name}")])
        .status()?;
    if !status.success() {
        anyhow::bail!("failed to release lock {lock_name}");
    }
    Ok(())
}
```

**Immutable audit ledger via `git notes`:**

```rust
/// Append an NDJSON receipt to the git notes audit ledger.
/// Notes are append-only at the object level; the ledger grows monotonically.
pub fn append_receipt_note(commit_sha: &str, receipt_json: &str) -> anyhow::Result<()> {
    let status = Command::new("git")
        .args([
            "notes",
            "--ref=receipts",
            "append",
            "-m",
            receipt_json,
            commit_sha,
        ])
        .status()?;
    if !status.success() {
        anyhow::bail!("failed to append receipt to git notes");
    }
    Ok(())
}
```

In `justfile`:

```just
# Append the latest receipt to the git notes audit ledger
receipt-commit:
    git notes --ref=receipts append -m "$(cat receipt.json)" HEAD
```

### What it prevents

External state stores for coordination and audit. Git's object store is already a content-addressed, append-only, distributed database. `git update-ref` provides atomic compare-and-swap; `git notes` provides an append-only ledger. The ledger survives without a separate database process.

### Use this when

Coordinating writes across multiple concurrent processes in the same repository, or building audit trails that must be committed alongside the code they describe.

### Do not use this when

Coordination is across machines that do not share a git repository, or the audit volume is too high for git's object model (millions of notes per day).

**Fleet examples:** `gitvan` (JavaScript implementation, full `git update-ref` lock + `git notes` NDJSON ledger with 90-day/365-day retention tiers).

---

## 18. Workspace Lint Inheritance

### When to use

Every workspace. The `[workspace.lints]` block in the root `Cargo.toml` is the single source of truth for lint configuration. Member crates opt in with `lints.workspace = true`. Never pass `-D warnings` on the command line — wire it into `[lints]`.

### How it looks in code

Root `Cargo.toml`:

```toml
[workspace.lints.rust]
unsafe_code = "forbid"        # relax to "warn" for linkme/WASM/proc-macro crates
missing_docs = "warn"         # "allow" in bins; "warn" in libs
unexpected_cfgs = "warn"

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
# Lib crates deny these; bins warn
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
# Always deny
todo = "deny"
unimplemented = "deny"
exit = "deny"
dbg_macro = "deny"
# Practical allows
multiple_crate_versions = "allow"
```

Member crate `Cargo.toml` (no lint block needed — just opt in):

```toml
[lints]
workspace = true
```

If a member crate needs to override a specific lint (e.g., the `linkme` crate itself needs to use `unsafe`), override at the crate level with a justification comment:

```toml
# member-crate/Cargo.toml — linkme requires unsafe internally
[lints.rust]
unsafe_code = "warn"   # overrides workspace "forbid": linkme distributed_slice uses transmute
```

### What it prevents

The `-D warnings` pattern (passing lint flags on the command line) does not compose — different CI steps may run different lint levels, and adding a new step that omits `-D warnings` silently degrades quality. `[workspace.lints]` is checked by `cargo check` and `cargo build`, not just in CI.

The alternative anti-pattern (putting RUSTFLAGS in `.cargo/config.toml`) is worse: it is invisible in lint reports and silences all diagnostics including safety-critical ones. See pattern 20.

### Use this when

Any project with more than one crate, and also single-crate projects. The `[lints]` block is cheaper to set up than any CI lint step and runs during normal development.

### Do not use this when

There is a genuine reason to allow a lint fleet-wide (rare). Add it as `"allow"` with a comment, not as an omission.

**Fleet examples:** `ggen`, `clap-noun-verb` (both have `[workspace.lints]`). The other 15 repos in the fleet are being migrated to this pattern.

---

## 19. Feature-Phased Architecture

### When to use

Long-lived features that span multiple releases. Phase features let the codebase carry work-in-progress without shipping it, and let integration tests target specific phases.

### How it looks in code

```toml
# Cargo.toml
[features]
default = []
phase-1 = []               # core event emission
phase-2 = ["phase-1"]      # assembly + sealing (depends on phase-1)
phase-3 = ["phase-2"]      # full verifier pipeline
otel    = ["opentelemetry"] # optional observability, not phase-gated
```

Phase features are additive and ordered — `phase-N` always enables `phase-(N-1)`:

```rust
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

Testing against a specific phase:

```bash
cargo test --no-default-features --features phase-1
cargo test --features phase-3       # full pipeline
just ci-all-phases                  # CI gates each phase independently
```

Three rules:
1. Phase features are additive and ordered.
2. Never gate core types or error types behind a phase flag.
3. Remove phase flags in the release commit that ships the feature as stable.

### What it prevents

All-or-nothing feature flags that force partial implementations to hide behind dead code. Phase features make the progression of a long-running feature visible in the codebase and testable at each stage.

### Use this when

A feature requires multiple PRs to land fully. Single-PR features do not need phases — use a feature flag only if the code needs to be committed but not yet shipped.

### Do not use this when

The feature is small enough to land in one PR, or the phase structure would require more than five phases (a sign the feature needs to be decomposed differently).

**Fleet examples:** `affidavit` (phase-1 / phase-2 / phase-3 feature progression), template `CLAUDE.md` "Feature Phase Architecture" section.

---

## 20. Anti-Patterns

The following patterns appear in the fleet and cause concrete harm. Each has a CHECKLIST entry for remediation.

---

### ANTI-1: `.cargo/config.toml` RUSTFLAGS Lint Suppression

**Source repo:** `wasm4pm`

**What it looks like:**

```toml
# .cargo/config.toml — DO NOT DO THIS
[build]
rustflags = ["-A", "clippy::all"]
```

**Why it is harmful:** Invisible to CI lint reports. Silences all clippy diagnostics fleet-wide, including safety-critical ones. Propagates across all crates in the workspace. Reviewers cannot see which lints are suppressed because the file is easy to overlook.

**The correct approach:** Use `[workspace.lints]` with explicit `"allow"` entries and justification comments (pattern 18).

**CHECKLIST:** `[H] Verify .cargo/config.toml does not contain RUSTFLAGS that suppress lints. Move any necessary allows to [workspace.lints.clippy] with justification comments.`

---

### ANTI-2: Nightly Without Documented Justification

**Source repos:** `wasm4pm`, `clnrm`, `ggen`, `chicago-tdd-tools`, `affidavit` (unpinned), `bcinr`, `dteam`, `mac-artifact-cleaner`

**What it looks like:**

```toml
# rust-toolchain.toml — problematic if there is no documented need
[toolchain]
channel = "nightly"
```

**Why it is harmful:** Nightly is unversioned at the channel level. A new nightly can break CI silently. The majority of nightly uses in the fleet are for `rustfmt` options that exist only on nightly — but `cargo fmt` is the only step that needs nightly; everything else (check, test, clippy) can use stable.

**The correct approach:** Pin stable for all jobs. If nightly is genuinely required (e.g., `#![feature(generic_const_exprs)]`), document exactly which feature and why stable is insufficient:

```toml
# rust-toolchain.toml
[toolchain]
channel = "1.82.0"   # stable; pinned to prevent new-lint-breaks-CI churn
components = ["rustfmt", "clippy"]
```

```markdown
<!-- CLAUDE.md -->
## Nightly Note
`cargo fmt` in this repo uses nightly-only `imports_granularity = "Module"`.
Run: `rustup run nightly cargo fmt`
All other tasks (`cargo test`, `cargo clippy`, `cargo build`) use stable.
```

**CHECKLIST:** `[H] If rust-toolchain.toml pins nightly, document exactly which nightly features are required and why stable is insufficient. Consider using stable for all jobs except fmt.`

---

### ANTI-3: `strip = true` in WASM Release Profile

**Source:** Template `Cargo.toml` (fixed in praxis), `pm4wasm`

**What it looks like:**

```toml
# Cargo.toml — DO NOT inherit this for WASM targets
[profile.release]
strip = true   # corrupts WASM binaries
```

**Why it is harmful:** `strip = true` passes `--strip-all` to the native toolchain linker. For WASM targets, stripping must go through `wasm-opt`, not the Rust compiler. The result is a non-loadable `.wasm` binary with no useful error message at build time.

**The correct approach:** Remove `strip = true` from the base release profile. For WASM crates, set it explicitly to `false`:

```toml
# workspace Cargo.toml
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
# strip removed — wasm-opt handles size optimization for WASM, linker for native

# WASM member crate Cargo.toml
[profile.release.package."my-wasm-crate"]
opt-level = "s"
strip = false   # explicit: prevent workspace inheritance and document intent
```

**CHECKLIST:** `[AUTO] Verify [profile.release] does NOT contain strip = true when the workspace includes WASM targets. Use wasm-opt for WASM size reduction.`

---

### ANTI-4: `String` Error Types

**Source repos:** Multiple early-stage repos in the fleet

**What it looks like:**

```rust
// DO NOT return String as an error
pub fn verify(path: &Path) -> Result<(), String> {
    Err(format!("verification failed: {}", path.display()))
}
```

**Why it is harmful:** `String` errors cannot be matched by callers, cannot be tested with `assert_fail!(result, MyError::Variant)`, and cannot be evolved without breaking callers. Every downstream `match` must parse the error message text.

**The correct approach:** Use named refusal enums (pattern 8).

**CHECKLIST:** `[H] Grep for -> Result<_, String> and -> std::result::Result<_, String> in library code. Replace with named error enum variants.`

---

### ANTI-5: No `[workspace.lints]` in Multi-Crate Workspace

**Source repos:** `bcinr` (12 crates), `wasm4pm` (nightly), `clnrm` (partial), `ggen` (200+ `#[allow]` overrides)

**What it looks like:** Each member crate sets its own `[lints]` section (or omits it entirely), causing inconsistency across the workspace.

**Why it is harmful:** Lint posture becomes a property of each crate individually, not the workspace. Auditing lint coverage requires reading every `Cargo.toml`. New crates silently inherit no lints.

**The correct approach:** Pattern 18 — one `[workspace.lints]` block, all members inherit.

**CHECKLIST:** `[AUTO] Add [workspace.lints] to root Cargo.toml. Add lints.workspace = true to all member Cargo.toml files. Verify cargo check --workspace compiles cleanly.`

---

## Quick Decision Table

| Situation | Pattern |
|---|---|
| Type must pass validation before use | Seal (`_seal: ()`) — Pattern 1 |
| Multiple parties must authorize state transitions | Typestate + Witness — Pattern 2 |
| Need a stable identity for an artifact | BLAKE3 content address — Pattern 3 |
| Detecting tampering in an event sequence | Rolling chain hash — Pattern 4 |
| Version a binary/internal tool | CalVer `YY.M.patch` — Pattern 5 |
| Add a handler without modifying a central registry | `linkme` distributed slice — Pattern 6 |
| CLI with multiple subcommands | Noun-verb + `clap-noun-verb` — Pattern 7 |
| Function can fail in multiple ways | Named `thiserror` enum — Pattern 8 |
| Map appears in serialized or hashed output | `BTreeMap` — Pattern 9 |
| Test must enforce Arrange→Act→Assert order | `TestState<Phase>` — Pattern 10 |
| Tests should generate committed documentation | `DocContext` — Pattern 11 |
| Test result needs to be an auditable artifact | `TestReceipt` — Pattern 12 |
| Type-system constraint needs regression protection | `trybuild` ALIVE gate — Pattern 13 |
| Exposing Rust objects to JavaScript over WASM | Handle-based `Store<T>` — Pattern 14 |
| MCP tool result | `CommonResponse<T>` with `passed` + `result_hash` — Pattern 15 |
| CLI arg for time-window lookback | `TimeWindowArgs` — Pattern 16 |
| Distributed lock or append-only audit trail | Git-as-runtime — Pattern 17 |
| Lint configuration in a workspace | `[workspace.lints]` inheritance — Pattern 18 |
| Long-running feature across multiple PRs | Feature-phased architecture — Pattern 19 |
| Seeing `.cargo/config.toml` with RUSTFLAGS | Anti-pattern 20.1 — remove it |
| Seeing `channel = "nightly"` | Anti-pattern 20.2 — document or migrate |
| WASM target with `strip = true` | Anti-pattern 20.3 — remove `strip` |

---

## Key Files in the Template

| File | Pattern |
|---|---|
| `template/src/types.rs` | Blake3Hash, canonical_bytes, Seal pattern comment |
| `template/src/chain.rs` | ChainAssembler, rolling hash, genesis seed |
| `template/src/cli.rs` | Noun-verb CLI, NounVerb derive |
| `template/src/discovery.rs` | `linkme` distributed slice |
| `template/src/error.rs` | Named refusal enum, `thiserror` 2 |
| `crates/chatman-common/src/testkit.rs` | TestState, TestReceipt, DocContext, assert_fail!, performance_test!, doc_assert! |
| `template/Cargo.toml` | [workspace.lints], CalVer version, profile.release |
| `template/CLAUDE.md` | Feature-phased architecture, verb step-by-step |

---

*Derived from 10-agent survey of 18 repos (`survey/00-SYNTHESIS.md`) and 10-repo second-wave deep dive (`survey/01-SECOND-WAVE.md`). Every pattern cited here has shipped in at least one fleet repo.*
