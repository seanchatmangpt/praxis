# AB-07 — Replay Verification Sequence (Fail-Fast Per-Field)

| Facet | Value |
|---|---|
| Invariant | Deterministic under fixed inputs: a fresh in-memory re-run of S1–S5 must reproduce every hash byte-identically |
| Information-Loss Risk | A whole-root-only check would hide which stage diverged; per-field ReplayMismatch names the first bad field |
| TPS Purpose | Jidoka: the verifier stops at the first defective field instead of passing it down the line |
| DfLSS CTQ | First divergent field identified in every failed replay; zero false-pass replays |
| CENG Boundary | Replay runs fully in memory against the same envelope; it never actuates and never mutates state |

```mermaid
sequenceDiagram
    participant V as Verifier
    participant E as ProcessReceiptEnvelope
    participant P as Fresh in-memory pipeline S1-S5

    V->>E: load claimed receipt (9 hashes + receipt_root)
    V->>P: re-run S1 fetch_snapshot
    P-->>V: graph_snapshot'
    V->>V: compare field 1, mismatch -> Refusal ReplayMismatch(graph_snapshot)
    V->>P: re-run S2 OWL closure
    P-->>V: profile', symbol_table'
    V->>V: compare fields 2-3 fail-fast
    V->>P: re-run S3 PDDL plan
    P-->>V: projection', admission_table', route_decision', tape'
    V->>V: compare fields 4-7 fail-fast
    V->>P: re-run S4-S5 admission + hooks
    P-->>V: hook_event'
    V->>V: compare field 8, then engine_version field 9
    V->>V: fold 9 hashes -> receipt_root'
    V-->>E: receipt_root' == receipt_root -> verified
```
