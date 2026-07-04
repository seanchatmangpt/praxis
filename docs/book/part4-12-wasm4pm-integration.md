# wasm4pm Integration

This chapter documents the integration architecture between `wasm4pm` (a WASM-accelerated process-mining engine) and Praxis's compliance/verification tooling, as specified in `docs/wasm4pm-integration/blueprint.md`. It then connects that architecture to two real ggen packs built against the wasm4pm ontology: `packs/wasm4pm-algorithms-pack` and `packs/wasm4pm-cognition-pack`.

The blueprint describes three integration dimensions: conformance checking (replaying event logs against process models), cryptographic receipt signing, and an autonomic MAPE-K control loop feeding off both. Source: `docs/wasm4pm-integration/blueprint.md:1-111`.

## 1. Conformance: replaying OCEL 2.0 logs against PNML models

The blueprint specifies a WASM-accelerated OCEL 2.0 (Object-Centric Event Log) parser that loads JSON-serialized event logs into a Rust-backed in-memory structure supporting both flat event attributes and object-centric relationships (`docs/wasm4pm-integration/blueprint.md:7-8`).

The described ingress pipeline has three steps:

1. **Boundary validation** — event log JSON (e.g. `anti_llm_cheat_lsp_ocel.json`) is checked at the TypeScript boundary for the presence of top-level keys `eventTypes`, `objectTypes`, `events`, `objects` in either camelCase or snake_case (`docs/wasm4pm-integration/blueprint.md:10`).
2. **WASM memory binding** — the validated JSON is loaded into the WASM heap via `load_ocel_from_json(ocel_content)`, which returns a log handle string (`docs/wasm4pm-integration/blueprint.md:11`).
3. **Referential integrity** — `validate_ocel(log_handle)` checks that every event-to-object and object-to-object reference resolves, with no dangling identifiers (`docs/wasm4pm-integration/blueprint.md:12`).

Process models are supplied as PNML (Petri Net Markup Language), e.g. `petri_net_lawful_dispatch.pnml` (`docs/wasm4pm-integration/blueprint.md:15`). The parser resolves initial markings from both place-level `<initialMarking><text>value</text></initialMarking>` elements and top-level `<initialMarking>` maps keyed by `place idref`, and interprets arcs directionally: place-to-transition arcs decrement token counts (inputs), transition-to-place arcs increment them (outputs); read-only/reference arcs check for token presence without consuming it (`docs/wasm4pm-integration/blueprint.md:17-18`).

Conformance is checked incrementally via `check_prefix_conformance(model_handle, prefix_json)`, which replays a trace prefix against the parsed net and classifies the outcome into one of three states (`docs/wasm4pm-integration/blueprint.md:21-26`):

| State | Meaning |
|---|---|
| `ALIVE` | The transition sequence is fully enabled by the current marking and a terminal state remains reachable. |
| `FAKE-LIVE` | The sequence is structurally valid but the net has deadlocked or the final place is no longer reachable. |
| `BLOCKED` | The trace attempts an illegal transition (insufficient tokens in an input place). |

Violations are adjudicated with a specific reason code attached to the offending activity index: `IllegalTransitionTaken` for a blocked transition, `TerminalStateUnreachable` when the terminal marking becomes unreachable (`docs/wasm4pm-integration/blueprint.md:27-29`).

## 2. Cryptography: receipt signing and verification

Every validation/conformance verdict is sealed into a signed receipt using ed25519 (`docs/wasm4pm-integration/blueprint.md:33-36`). The blueprint describes `wpm receipt keygen` as producing a deterministic ed25519 key pair: a private key file (`signing.key`, hex-encoded PKCS#8, `0600` permissions) and a public key file (`signing.pub`, hex-encoded SPKI) distributed to verification nodes (`docs/wasm4pm-integration/blueprint.md:38-40`).

Two receipt shapes are defined: `PiReceipt` for individual algorithm-run assertions, and `CommandReceipt` for command-level execution outcomes (`docs/wasm4pm-integration/blueprint.md:43-45`). The `CommandReceipt` schema shown in the blueprint carries a `run_id`, the `command` name, BLAKE3 `input_hash`/`output_hash`, a `status`, `timestamp`, a `summary` (verdict + conformance rate), and an ed25519 `signature_algorithm`/`public_key`/`signature` triple (`docs/wasm4pm-integration/blueprint.md:47-64`).

The blueprint's verification boundary, `wpm receipt admit`, performs two checks before a receipt is admitted into the audit ledger (`docs/wasm4pm-integration/blueprint.md:66-69`):

1. **Hash verification** — recompute the BLAKE3 hash of the input log and output verdict and compare against the receipt's `input_hash`/`output_hash`.
2. **Signature verification** — verify the ed25519 signature over the combined digest using the validator's registered public key.

## 3. Autonomic loop: MAPE-K with cargo-cicd

Conformance verdicts and receipts feed a MAPE-K (Monitor-Analyze-Plan-Execute over shared Knowledge) loop integrated with `cargo-cicd` (`docs/wasm4pm-integration/blueprint.md:73-92`):

- **Monitor** captures execution events and OTEL spans from the running system and structures them into an OCEL trace prefix (`docs/wasm4pm-integration/blueprint.md:94-95`).
- **Analyze** runs the WASM conformance engine (token-based replay) against that prefix; a `BLOCKED` or `FAKE-LIVE` result produces an alert identifying the offending activity, expected vs. actual token state, and failure classification (`docs/wasm4pm-integration/blueprint.md:97-99`).
- **Plan** maps alerts to corrective actions: a conformance rate below threshold triggers a plan to halt the deployment pipeline and quarantine the commit; minor deviations produce non-blocking warning annotations in CI logs (`docs/wasm4pm-integration/blueprint.md:101-104`).
- **Execute** carries out the plan: halts `cargo-cicd`, returns a non-zero exit status to the build pipeline, and can automatically roll back to the last cryptographically signed compliant state (`docs/wasm4pm-integration/blueprint.md:106-110`).

The knowledge base shared across all four phases holds the Petri net models, compliance history, and key registry (`docs/wasm4pm-integration/blueprint.md:78-83`).

## 4. From blueprint to working packs

The blueprint above is a design document for conformance checking, receipt signing, and the MAPE-K loop — it does not itself constitute ggen-generated code. Two ggen packs built this session turn the wasm4pm side of that story (the algorithm and cognition-breed surfaces the blueprint's Analyze phase would call into) into typed, generated Rust catalogs, respecting the process-intelligence boundary: ggen packs only catalog and dispatch against wasm4pm's surface — the actual discovery, conformance, and fitness computation stay in `wasm4pm`/`wasm4pm-compat`.

**`wasm4pm-algorithms-pack`** (`/Users/sac/praxis/packs/wasm4pm-algorithms-pack/pack.toml:1-4`) is described in its own `pack.toml` as a "Typed Rust catalog + reference doc for the wasm4pm process-intelligence ALGORITHM surface (catalog/caller surface only; all analysis stays in wasm4pm)". Its ontology, `packs/wasm4pm-algorithms-pack/ontology.ttl`, declares the `pi:ProcessIntelligenceAlgorithm` class (`packs/wasm4pm-algorithms-pack/ontology.ttl:25`) and bundles all **60** instances of it, per the file's own header comment (`packs/wasm4pm-algorithms-pack/ontology.ttl:12`) — confirmed by a direct count of `a pi:ProcessIntelligenceAlgorithm` declarations in the file (60 matches).

**`wasm4pm-cognition-pack`** (`/Users/sac/praxis/packs/wasm4pm-cognition-pack/pack.toml:1-4`) is described as a "wasm4pm cognition breed catalog and typed dispatch-surface skeleton over the stable 6-verb ABI (cognition_show/run/verify/replay, system_build/verify); catalog/caller surface only — evidence and analysis stay in wasm4pm". Its ontology, `packs/wasm4pm-cognition-pack/ontology.ttl`, declares the `compat:CognitionBreed` class (`packs/wasm4pm-cognition-pack/ontology.ttl:18`) and carries **55** instances of it (direct count of `a compat:CognitionBreed` declarations in the file).

Together these two packs give ggen a generated, typed caller surface — 60 algorithm entries and 55 cognition-breed entries — that a MAPE-K Analyze phase (section 3 above) or a conformance-checking client (section 1) could dispatch through, while leaving all actual discovery, replay, and fitness/precision computation inside `wasm4pm`/`wasm4pm-compat`, matching the process-intelligence boundary already enforced elsewhere in this codebase.
