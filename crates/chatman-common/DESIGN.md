# chatman-common Design Guide

`chatman-common` is the house-standard shared library for all seanchatmangpt Rust projects. It provides:

- **Error handling** — Unified `Error` type with context and FM codes
- **Content addressing** — BLAKE3-based deterministic hashing
- **Serialization** — Canonical JSON for hash stability
- **CLI utilities** — Bootstrap helpers and configuration
- **Testing** — Fixtures, assertion helpers, test receipts
- **Signatures** — ed25519 signing and verification
- **Telemetry** — Structured logging with tracing
- **Git integration** — Audit ledgers and lock management

All features are optional via Cargo features. Use only what you need.

---

## **Core Concepts**

### Content Addressing

Every artifact in the pipeline is identified by its **BLAKE3 content hash** (64-char lowercase hex):

```rust
use chatman_common::canonical_json;
use blake3;

let data = serde_json::json!({
    "name": "event",
    "timestamp": "2026-06-23T10:00:00Z"
});

// Canonical serialization: sorted keys, no whitespace
let bytes = canonical_json(&data)?;

// Content address
let hash = blake3::hash(&bytes).to_hex().to_string();
// "abc123def456..." (64 chars)
```

**Key property:** Same data → same hash, always. Different data → different hash, always.

### Canonical JSON

Standard JSON serialization **does not** guarantee consistent hashes:
- Key order is unspecified
- Whitespace varies
- Number representations differ

**chatman-common's solution:** `canonical_json(value)` returns deterministic bytes:
- Keys sorted lexicographically
- No whitespace
- Consistent number encoding
- UTF-8 encoded

```rust
use chatman_common::canonical_json;

let obj = serde_json::json!({"z": 1, "a": 2});
let bytes = canonical_json(&obj)?;
// Always: `{"a":2,"z":1}` (sorted keys, compact)
// Hash is reproducible across runs, languages, and machines
```

### Rolling Chain Hash

For sequences of events, compute a **rolling chain hash** where each event's hash depends on all previous events:

```rust
use chatman_common::chain::fold_event;

let event1 = canonical_json(&event1)?;
let event2 = canonical_json(&event2)?;

// Hash accumulates
let hash0 = "0000000000000000000000000000000000000000000000000000000000000000";  // Genesis
let hash1 = fold_event(&hash0, &event1);  // hash(hash0 + event1)
let hash2 = fold_event(&hash1, &event2);  // hash(hash1 + event2)

// Final chain hash represents all events in order
assert_ne!(hash1, hash2);
```

This creates **tamper-evident chains**: changing any earlier event propagates the change to all subsequent hashes.

---

## **Module Reference**

### `error.rs` — Unified Error Type

Provides a single `Error` enum that all crates share:

```rust
use chatman_common::Error;

// Create errors
let e1 = Error::msg("something went wrong");
let e2 = Error::msg(format!("failed to parse: {}", input));

// With context (from thiserror)
#[error("io failed: {0}")]
pub struct MyError(String);

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::msg(format!("io: {}", e))
    }
}

// Use in functions
fn my_function() -> Result<i32> {
    std::fs::read_to_string("file.txt")?;  // auto-converts via From
    Ok(42)
}
```

**Features:**
- No dependencies (only std)
- Composable with thiserror
- Pretty Display output

---

### `provenance.rs` — Content Addressing

Core hashing utilities (requires `provenance` feature):

```rust
use chatman_common::chain::{fold_event, genesis_seed};
use blake3;

// Start with genesis seed
let genesis = genesis_seed("my-service");

// Build rolling chain
let event1 = b"first event";
let hash1 = fold_event(&genesis, event1);

let event2 = b"second event";
let hash2 = fold_event(&hash1, event2);

// Final hash is tamper-evident
println!("Chain hash: {}", hash2);  // 64-char hex
```

**Functions:**
- `fold_event(prev_hash: &str, event_bytes: &[u8]) -> String` — Compute next hash in chain
- `genesis_seed(service: &str) -> String` — Bootstrap initial hash (for consistency)

---

### `chain.rs` — Rolling Hashes (deprecated, use `provenance`)

Legacy module for rolling BLAKE3 chains. Use `provenance::fold_event` instead.

```rust
use chatman_common::chain::RollingChain;

let mut chain = RollingChain::new("my-service");
chain.push(b"event 1");
chain.push(b"event 2");
let final_hash = chain.finalize();
```

---

### `signed_receipt.rs` — Cryptographic Signing

Sign hashes with ed25519 for non-repudiable audit trails (requires `signed-receipts` feature):

#### Generating a Key Pair

```rust
use chatman_common::signed_receipt::KeyPair;

let kp = KeyPair::generate();
println!("Signing key: {}", kp.signing_key_hex());    // 64 hex chars (secret)
println!("Verifying key: {}", kp.verifying_key_hex()); // 64 hex chars (public)

// Save signing key securely (e.g., in PRAXIS_SIGNING_KEY env var)
// Distribute verifying key to auditors
```

#### Signing a Receipt

```rust
use chatman_common::signed_receipt::{sign, sign_with_env_key, SignedReceipt};

let chain_hash = "abc123def456...";  // 64-char BLAKE3 hash

// Option 1: Sign with explicit key
let signed = sign(&chain_hash, &signing_key_hex)?;

// Option 2: Sign from environment (PRAXIS_SIGNING_KEY or PRAXIS_SIGNING_KEY_FILE)
let signed = sign_with_env_key(&chain_hash)?;

// SignedReceipt includes:
// - The original hash
// - The signature (ed25519)
// - Timestamp (ISO 8601)

let json = serde_json::to_string_pretty(&signed)?;
std::fs::write("receipt.json", json)?;
```

#### Verifying a Signature

```rust
use chatman_common::signed_receipt::verify;

let json = std::fs::read_to_string("receipt.json")?;
let signed: SignedReceipt = serde_json::from_str(&json)?;

// Verify with the verifying key
let is_valid = verify(&signed, &verifying_key_hex)?;
assert!(is_valid, "signature verification failed");
```

#### Environment Variables

| Var | Format | Precedence |
|-----|--------|-----------|
| `PRAXIS_SIGNING_KEY` | 64 hex chars | 1 (first tried) |
| `PRAXIS_SIGNING_KEY_FILE` | Path to file | 2 (if key env not set) |

**Example:**
```bash
export PRAXIS_SIGNING_KEY=abc123def456...
# OR
export PRAXIS_SIGNING_KEY_FILE=~/.praxis/signing-key
```

---

### `cli.rs` — CLI Bootstrap

Command-line utilities (requires `cli` feature):

```rust
use chatman_common::cli::{GlobalArgs, init_tracing};
use clap::Parser;

#[derive(Parser)]
#[command(name = "my-app")]
struct Args {
    #[command(flatten)]
    global: GlobalArgs,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize structured logging
    init_tracing(&args.global.service_name)?;
    
    // Your app logic
    Ok(())
}
```

**Provided:**
- `GlobalArgs` — Common CLI flags (log level, service name, etc.)
- `init_tracing(service: &str)` — Bootstrap tracing subscriber

---

### `telemetry.rs` — Structured Logging

Tracing integration (requires `telemetry` feature):

```rust
use chatman_common::cli::init_tracing;
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing("my-service")?;
    
    info!(event = "startup", "service started");
    
    match risky_operation().await {
        Ok(result) => info!(result = ?result, "operation succeeded"),
        Err(e) => error!(error = %e, "operation failed"),
    }
    
    Ok(())
}
```

Control log level with `RUST_LOG` env var:
```bash
RUST_LOG=debug cargo run
RUST_LOG=info cargo run
RUST_LOG=my_module=trace cargo run
```

---

### `testkit.rs` — Test Fixtures and Helpers

Testing utilities (requires `testkit` feature):

#### Test Receipts

```rust
use chatman_common::testkit::TestReceipt;

#[test]
fn my_integration_test() {
    let receipt = TestReceipt::capture("my_operation", || {
        // Your test code here
        Ok(42)
    }).unwrap();
    
    // receipt includes:
    // - operation name
    // - execution time
    // - result status
    // - optional signature
    
    println!("{}", serde_json::to_string_pretty(&receipt).unwrap());
}
```

#### Golden File Assertions

```rust
use chatman_common::testkit::assert_golden;

#[test]
fn output_format_is_stable() {
    let output = generate_report();
    
    // First run: creates `tests/golden/report.json`
    // Subsequent runs: compares against stored golden file
    assert_golden("golden/report", &output);
}
```

#### Deterministic Test Utilities

```rust
use chatman_common::testkit::{TestUuid, TestTimestamp};

#[test]
fn deterministic_ids() {
    // Predictable UUID for testing
    let id = TestUuid::fixed("test-event");
    assert_eq!(id.to_string(), "...fixed value...");
    
    // Predictable timestamp
    let ts = TestTimestamp::fixed(1234567890);
    assert_eq!(ts.to_rfc3339(), "2009-02-13T23:31:30Z");
}
```

---

### `git_runtime.rs` — Git Integration

Git-based audit ledgers and distributed locking (requires `git-runtime` feature):

#### Audit Ledger

```rust
use chatman_common::git_runtime::GitAuditLedger;

let ledger = GitAuditLedger::open(".")?;

// Append an audit event
ledger.append(
    "refs/audit/my-event",
    "audit entry content"
)?;

// Read all events
let events = ledger.read_all()?;
for event in events {
    println!("{}", event);
}
```

#### Distributed Lock

```rust
use chatman_common::git_runtime::GitLock;

let lock = GitLock::acquire(".")?;
// Lock acquired: ref created in git

// Critical section
println!("Resource locked");

// Lock automatically released on drop
drop(lock);
```

---

## **Feature Flags**

Enable features in `Cargo.toml`:

```toml
[dependencies]
chatman-common = { version = "26.6", features = ["provenance", "testkit"] }
```

| Feature | Purpose | Adds |
|---------|---------|------|
| `serde` | JSON serialization | serde 1.0 |
| `provenance` | BLAKE3 hashing | blake3 1.8 |
| `cli` | CLI bootstrap | clap, tracing |
| `telemetry` | Structured logging | tracing 0.1, tracing-subscriber |
| `testkit` | Test fixtures | — |
| `signed-receipts` | ed25519 signatures | ed25519-dalek 2.0, rand |
| `git-runtime` | Git integration | — (uses system git) |
| `full` | All features | all above |

**Minimal setup:**
```toml
chatman-common = "26.6"  # No default features, add as needed
```

**Full setup:**
```toml
chatman-common = { version = "26.6", features = ["full"] }
```

---

## **Common Patterns**

### Pattern: Content-Addressed Artifacts

```rust
use chatman_common::{canonical_json, chain::fold_event};
use blake3;

pub struct Artifact {
    pub id: String,  // 64-char BLAKE3 hash
    pub data: serde_json::Value,
}

impl Artifact {
    pub fn new(data: serde_json::Value) -> Result<Self> {
        let bytes = canonical_json(&data)?;
        let id = blake3::hash(&bytes).to_hex().to_string();
        Ok(Self { id, data })
    }
}
```

### Pattern: Tamper-Evident Chains

```rust
use chatman_common::chain::fold_event;

pub struct Chain {
    events: Vec<serde_json::Value>,
    current_hash: String,
}

impl Chain {
    pub fn new() -> Self {
        Self {
            events: vec![],
            current_hash: "0000000000000000000000000000000000000000000000000000000000000000",
        }
    }
    
    pub fn append(&mut self, event: serde_json::Value) -> Result<String> {
        let bytes = chatman_common::canonical_json(&event)?;
        self.current_hash = fold_event(&self.current_hash, &bytes);
        self.events.push(event);
        Ok(self.current_hash.clone())
    }
    
    pub fn finalize(self) -> String {
        self.current_hash
    }
}
```

### Pattern: Signed Audit Trail

```rust
use chatman_common::signed_receipt::sign_with_env_key;

pub async fn seal_audit_trail(events: &[String]) -> Result<SignedAuditTrail> {
    // Build chain hash
    let mut chain = Chain::new();
    for event in events {
        let data = serde_json::json!({"event": event});
        chain.append(data)?;
    }
    let chain_hash = chain.finalize();
    
    // Sign the chain
    let signed = sign_with_env_key(&chain_hash)?;
    
    Ok(SignedAuditTrail {
        chain_hash,
        signed_receipt: signed,
    })
}
```

---

## **Error Handling**

### Creating Errors

```rust
use chatman_common::Error;

// Simple message
let e1 = Error::msg("operation failed");

// With context
let e2 = Error::msg(format!("failed to parse {}: {}", path, reason));

// From another error
let json_str = r#"{"invalid": json}"#;
match serde_json::from_str::<Value>(json_str) {
    Ok(val) => { /* ... */ }
    Err(e) => return Err(Error::msg(format!("JSON: {}", e))),
}
```

### Propagating Errors

```rust
use chatman_common::Error;

fn operation() -> Result<i32> {
    let content = std::fs::read_to_string("file.txt")?;  // ? converts to Error
    let value: i32 = content.parse()?;  // ? converts ParseIntError
    Ok(value)
}
```

### With thiserror

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("parse: {0}")]
    Parse(String),
}

// Converts to chatman_common::Error automatically
impl From<MyError> for chatman_common::Error {
    fn from(e: MyError) -> Self {
        chatman_common::Error::msg(e.to_string())
    }
}
```

---

## **Testing with chatman-common**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chatman_common::testkit::{TestReceipt, assert_golden};

    #[test]
    fn my_algorithm_is_deterministic() {
        let receipt = TestReceipt::capture("algorithm", || {
            let result = my_algorithm();
            Ok(result)
        }).unwrap();
        
        assert_golden("algorithm_result", &receipt);
    }

    #[test]
    fn hash_is_stable() {
        use chatman_common::canonical_json;
        
        let data = serde_json::json!({"z": 1, "a": 2});
        let bytes1 = canonical_json(&data).unwrap();
        let bytes2 = canonical_json(&data).unwrap();
        
        assert_eq!(bytes1, bytes2, "canonical JSON should be deterministic");
    }
}
```

---

## **Versioning**

`chatman-common` follows **CalVer**: `YY.M.patch`
- `26.6.0` = June 2026, patch 0
- `26.6.1` = June 2026, patch 1

Consult `CHANGELOG.md` for breaking changes between versions.

---

## **Contributing**

To add a new module to `chatman-common`:

1. Create `src/new_module.rs` with functionality
2. Add a feature gate (optional): `new-feature = []` in `Cargo.toml`
3. Export in `src/lib.rs`: `pub mod new_module;`
4. Add rustdoc and tests
5. Update this `DESIGN.md` with examples
6. Submit PR with justification for why this belongs in house-common

Additions should be **broadly useful** across multiple fleet repos, not single-project utilities.

---

## **Support**

- **Documentation:** This file (`DESIGN.md`)
- **Examples:** See tests in each module
- **API docs:** `cargo doc --open`
- **Issues:** File in Praxis repo

