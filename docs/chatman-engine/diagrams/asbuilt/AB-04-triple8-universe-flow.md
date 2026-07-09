# AB-04 — Triple8 Universe Flow (256-Term Fence)

| Facet | Value |
|---|---|
| Invariant | Closed vocabulary: the Triple8 universe is exactly 256 terms; term 257 and unknown terms are refused by name |
| Information-Loss Risk | Projection into 8-bit space is lossy by construction; the projection hash binds it to the source graph |
| TPS Purpose | Poka-yoke: the 256-term fence makes an out-of-universe term physically unrepresentable downstream |
| DfLSS CTQ | Zero silent truncation — every over-fence input yields UniverseOverflow, never a wrapped index |
| CENG Boundary | Only fenced, projected terms enter the [Admission8;256] table; raw IRIs never cross this line |

```mermaid
flowchart TD
    G[Canonical graph terms] --> INT[Intern into symbol table]
    INT --> CHK{Term count <= 256?}
    CHK -- "term 257 arrives" --> OV[Refusal UniverseOverflow]
    CHK -- yes --> KNOWN{Term in closed universe?}
    KNOWN -- no --> UNK[Refusal UnknownTerm]
    KNOWN -- yes --> PROJ[Project to 8-bit Triple8 ids]
    PROJ --> PH[projection hash BLAKE3 over sorted mapping]
    PH --> BIND[Bind projection hash into receipt material]
    PROJ --> TBL["[Admission8;256] table construction"]
    BIND --> RCPT[receipt_root fold at S6]
    TBL --> ADM[Admission mask evaluation AB-05]
```
