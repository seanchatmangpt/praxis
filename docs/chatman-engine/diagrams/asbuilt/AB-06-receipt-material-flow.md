# AB-06 — Receipt Material Flow (9 Hashes in Constitutional Order)

| Facet | Value |
|---|---|
| Invariant | All receipt material sorted/canonical before hashing; BLAKE3 only; hash order is constitutional and fixed |
| Information-Loss Risk | Omitting any of the 9 fields would make that facet unreplayable; the fold order itself is part of the law |
| TPS Purpose | Traceability kanban: every station's output is a card folded into one final ticket |
| DfLSS CTQ | receipt_root byte-identical across replays; any single-field change flips the root |
| CENG Boundary | receipt_root is computed only inside ProcessReceiptEnvelope; no caller may assert or patch it |

```mermaid
flowchart TD
    H1[1 graph_snapshot hash] --> C[blake3_combined fold in constitutional order]
    H2[2 profile hash] --> C
    H3[3 symbol_table hash] --> C
    H4[4 projection hash] --> C
    H5[5 admission_table hash] --> C
    H6[6 route_decision hash] --> C
    H7[7 tape hash] --> C
    H8[8 hook_event hash] --> C
    H9[9 engine_version hash] --> C
    C --> R[receipt_root]
    R --> E[ProcessReceiptEnvelope]
    E --> V[Replay verification AB-07]
```
