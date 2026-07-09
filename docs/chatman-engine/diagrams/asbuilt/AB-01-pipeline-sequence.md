# AB-01 — Pipeline Sequence (S1–S6, As Built)

| Facet | Value |
|---|---|
| Invariant | Receipts are computed (BLAKE3) from canonical material, never asserted; no wall clock in S1–S6 |
| Information-Loss Risk | Stage outputs not bound into receipt_root would be unauditable; every stage hash is folded into S6 |
| TPS Purpose | One-piece flow: each envelope passes all six stations in order, no rework loops |
| DfLSS CTQ | Byte-identical receipt_root for identical InvocationEnvelope inputs |
| CENG Boundary | actuate() accepts only AdmittedTransition; nothing bypasses S4 admission |

```mermaid
sequenceDiagram
    participant Caller
    participant S1 as S1 fetch_snapshot
    participant S2 as S2 OWL closure
    participant S3 as S3 PDDL plan (bcinr-pddl)
    participant S4 as S4 POWL trace admission
    participant S5 as S5 Knowledge hooks
    participant S6 as S6 ProcessReceiptEnvelope
    participant Act as actuate

    Caller->>S1: InvocationEnvelope
    S1->>S1: RDFC-1.0 canonical hash (graph_snapshot)
    S1->>S2: canonical graph
    S2->>S2: OWL RL closure to fixpoint
    S2->>S3: closed graph + profile hash
    S3->>S3: bcinr-pddl plan synthesis (tape)
    S3->>S4: candidate trace
    S4->>S4: 3 legality layers (pattern, mask, causal)
    S4->>S5: admitted trace
    S5->>S5: hooks emit BoundaryRequests (sealed)
    S5->>S6: hook_event material
    S6->>S6: 9 hashes folded -> receipt_root (BLAKE3)
    S6->>Act: AdmittedTransition + receipt
    Act-->>Caller: actuation result + ProcessReceiptEnvelope
```
