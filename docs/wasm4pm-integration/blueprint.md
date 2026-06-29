# wasm4pm & Praxis Integration Blueprint

This document details the architecture, data structures, and execution flow for integrating the `wasm4pm` high-performance process mining engine with the `Praxis` process compliance and verification framework.

## 1. Conformance Dimension: Replaying OCEL 2.0 Logs against PNML Models

### 1.1 Object-Centric Event Logs (OCEL 2.0) Parser
The integration uses the WASM-accelerated OCEL 2.0 parser from `wasm4pm`. The parser loads the JSON-serialized event logs into memory via a Rust-backed parser that handles both traditional flat event attributes and object-centric relationships.

- **Ingress Pipeline**: Event logs (e.g., `anti_llm_cheat_lsp_ocel.json`) are validated at the TypeScript boundary to ensure top-level keys (`eventTypes`, `objectTypes`, `events`, `objects`) exist in either camelCase or snake_case formats.
- **WASM Memory Binding**: The validated JSON string is loaded into the WASM heap using `load_ocel_from_json(ocel_content)`, returning a unique log handle string.
- **Referential Integrity**: The WASM kernel executes `validate_ocel(log_handle)` to ensure that all event-to-object and object-to-object references resolve successfully without dangling identifiers.

### 1.2 PNML Model Parser
Petri Net Markup Language (PNML) models (e.g., `petri_net_lawful_dispatch.pnml`) are parsed by the WASM kernel to construct the Petri net place-transition structure.

- **Marking Resolution**: The parser handles place-level initial markings specified via `<initialMarking><text>value</text></initialMarking>` and top-level `<initialMarking>` maps linking places (`place idref="..."`) to token counts.
- **Arc and Weight Interpretation**: Places, transitions, and arcs are parsed sequentially. Arcs from places to transitions decrease token counts (inputs), while arcs from transitions to places increase token counts (outputs). Read-only or reference arcs are modeled by checking that the place contains the required token without permanently consuming it.

### 1.3 Token Replay and Prefix Conformance Checking
Prefix conformance checking is executed incrementally via `check_prefix_conformance(model_handle, prefix_json)`. It replays trace sequences against the parsed Petri Net.

- **Replay State Machine**:
  - **ALIVE**: The sequence of transitions is fully enabled by the current marking, and a terminal state (such as the final place containing the final token) remains reachable.
  - **FAKE-LIVE**: The sequence is structurally valid, but the net has reached a deadlock or a state from which the final place is unreachable.
  - **BLOCKED**: The trace contains an illegal transition (e.g., taking a transition when its input places lack sufficient tokens).
- **Adjudication**:
  - If a transition fails enablement checks, a violation is recorded at the specific activity index with reason `IllegalTransitionTaken`.
  - If a terminal state becomes unreachable, the trace is flagged with `TerminalStateUnreachable`.

---

## 2. Cryptography Dimension: Compliance Receipt Signing and Verification

### 2.1 ed25519 Cryptographic Keys
To guarantee the authenticity and integrity of compliance audits, all validation and conformance verdicts are sealed in cryptographically signed receipts.

- **Key Generation**: Executing `wpm receipt keygen` produces a deterministic ed25519 public/private key pair.
  - **Private Key (`signing.key`)**: Written in JSON format containing the hex-encoded PKCS#8 private key, restricted with `0600` permissions.
  - **Public Key (`signing.pub`)**: Written in JSON format containing the hex-encoded SPKI public key, distributed to verification nodes.

### 2.2 Receipt Structure
Compliance receipts represent the tamper-proof proof-of-conformance. There are two receipt formats supported:
- **PiReceipt**: Used for tracking individual algorithm run assertions.
- **CommandReceipt**: Used for tracking command-level execution outcomes.

#### CommandReceipt Schema:
```json
{
  "run_id": "9b1deb4d-3b7d-4ba3-9bbf-0b1a23c456d7",
  "command": "conformance",
  "input_hash": "c009c96388366a7c0cede4c06066f36ce87f9c6b276506d1d1fcb295b234868e0",
  "output_hash": "8f87a87b7a7c7c00e12d3c345a987d65b2f34e56c123d4e5f6a7b8c9d0e1f2a3",
  "status": "success",
  "timestamp": "2026-06-29T20:24:34Z",
  "summary": {
    "verdict": "Admitted",
    "conformance_rate": 1.0
  },
  "signature_algorithm": "ed25519",
  "public_key": "a50c82de9531816e87ad638f219bf6ea3c32b5e28a50c82de9531816e87ad638",
  "signature": "3c98ad7f6e5d4c3b2a1a09f8e7d6c5b4a3f2e1d0c9b8a7fa6e5d4c3b2a1a09f8e7d6c5b4a3f2e1d0c9b8a7fa6e5d4c3b2a1a09f8e7d6c5b4a3f2e1d0c9b8a7fa"
}
```

### 2.3 Verification Boundary
Before any change is admitted into production or logged in the audit ledger, `wpm receipt admit` validates the receipt:
1. **Hash Verification**: Re-computes the BLAKE3 hash of the input log and output verdict to ensure they match `input_hash` and `output_hash`.
2. **Signature Verification**: Verifies the ed25519 signature over the combined SHA-256/BLAKE3 digest using the public key associated with the validator.

---

## 3. Autonomic Loop Dimension: MAPE-K Integration with cargo-cicd

The conformance verdicts and compliance receipts feed directly into the `cargo-cicd` autonomic control loop.

```
       +-------------------------------------------------+
       |                    KNOWLEDGE                    |
       |  - Petri net models                             |
       |  - Compliance history                           |
       |  - Key registry                                 |
       +-------------------------------------------------+
           ^                                         |
           |                                         v
   +---------------+   +---------------+     +---------------+   +---------------+
   |    MONITOR    |-->|    ANALYZE    |---->|     PLAN      |-->|    EXECUTE    |
   | - Process logs|   | - Token replay|     | - Determine   |   | - Halt deploy |
   | - Span events |   | - Alignment   |     |   mitigations |   | - Log alert   |
   | - CI triggers |   | - Key validation|   | - Suggest fix |   | - Auto-rollback|
   +---------------+   +---------------+     +---------------+   +---------------+
```

### 3.1 Monitor Phase
The autonomic monitor captures execution events and OTEL spans from the running system. Spans are structured into an OCEL trace prefix and fed directly to the analyzer.

### 3.2 Analyze Phase
The analyzer executes the WASM conformance engine to perform validation and token-based replay.
- If the trace returns `BLOCKED` or `FAKE-LIVE`, the analyzer generates an alert identifying the specific activity that caused the violation, the expected vs. actual token state, and the failure classification.

### 3.3 Plan Phase
The planner maps Andon alerts to corrective actions.
- **Critical Failure**: If the conformance rate falls below the compliance threshold, a plan is created to halt the deployment pipeline and quarantine the commit.
- **Warning**: For minor deviations (such as sparse types or non-critical timing drift), the planner issues warning annotations in the CI logs without stopping execution.

### 3.4 Execute Phase
The execution engine implements the plans generated by the planner:
- Halts the `cargo-cicd` execution.
- Returns non-zero exit status to the build pipeline.
- Automatically rolls back the environment to the last known cryptographically signed compliant state.
