# Schema Validation (`schema/`)

The [`schema/`](file:///Users/sac/praxis/schema/) directory stores schema rules and data schemas used to validate transaction payloads, receipt ledgers, and configurations.

## Purpose

While the `ontology/` directory declares semantic hierarchies, the `schema/` directory enforces structure. It ensures that payload parameters conform to expected formats before they are passed to the planner or admission gate.

## Key Files

- **`manifest.json`**: Structure validation for the compile-time manifests.
- **`receipt_schema.json`**: JSON Schema defining key fields in transaction records (`payload_hash_hex`, `prev_chain_hash_hex`, etc.).
- **`configuration.json`**: Defines configuration limits (e.g., active signing keys, path filters).

## Enforcement points

Schemas are evaluated at three main stages:
1. **Ingress (Admission)**: paylaod structures are checked against schemas.
2. **Replay (Validation)**: verified receipts are parsed and evaluated to confirm structural conformity.
3. **Synchronization**: configuration files are cross-checked before committing local changes.
