### Design: BLAKE3 Receipt Generation ($\mu_5$) and Verification for the Praxis Workflow

The design guarantees a deterministic cryptographic custody chain binding project state to successful validation checks without mock surfaces.

#### 1. $\mu_5$ Receipt Generation (Emit Stage)
Upon successful validation ($\mu_4$ checks: `cargo test`, `cargo fmt --check`, and structural conformance), `praxis-guard` emits a deterministic cryptographic receipt.

**Deterministic Aggregate BLAKE3 Digest:**
To ensure causal consistency, the digest must strictly and identically reflect the source tree state:
- **File Discovery:** Recursively walk `src/` and essential configs (`Cargo.toml`, `deny.toml`), explicitly rejecting volatile output directories (`target/`, `.git/`).
- **Stable Ordering:** Sort all discovered files strictly by their relative paths.
- **Hashing Process:** Use a sequential BLAKE3 mechanism (e.g., a `RollingChain`). For each file in the sorted list:
  1. Hash the relative path string (e.g., `"src/main.rs"`).
  2. Hash the raw byte content of the file.
- **Result:** A single, deterministic `source_digest` acting as the content-addressable identity of the validated source state.

**Custody Chain & Signatures:**
- **Key Binding:** An Ed25519 keypair binds the execution to the state.
- **Signing:** The `SigningKey` signs the execution context payload (e.g., `project_name || timestamp || source_digest`).
- **Output:** The final receipt is saved as `receipt.json`.

**Data Structure:**
```json
{
  "data": {
    "project_name": "...",
    "timestamp": "2026-06-23T...",
    "source_digest": "<BLAKE3_HEX>",
    "public_key": "<ED25519_PUBKEY_HEX>"
  },
  "signature": "<ED25519_SIGNATURE_HEX>"
}
```

#### 2. Verification Mechanism
Verification asserts that a given project folder strictly adheres to the claims of a previously issued compliance receipt.

- **Recomputation:** Re-run the exact file discovery, sorting, and BLAKE3 hashing pipeline to derive the local `source_digest`.
- **Integrity Check:** Assert the local digest perfectly matches `receipt.data.source_digest`. A mismatch instantly invalidates the custody chain (proving drift or tampering).
- **Signature Validation:** Extract the Ed25519 `public_key` and verify the `signature` against the receipt data.
- **Result:** A green pass guarantees the current tree is causally consistent and identical to the tree that originally passed the rigorous $\mu_4$ validation gates.

#### 3. Alignment with `AGENTS.md` verification laws
- **No Synthetic Telemetry/Mocks:** The receipt generation requires reading physical bytes off the disk after real boundaries (`cargo test`) are crossed.
- **Multi-surface Corroboration:** The receipt guarantees execution (cargo results), state (BLAKE3 hash of files), and causality (Ed25519 signature binding them).
