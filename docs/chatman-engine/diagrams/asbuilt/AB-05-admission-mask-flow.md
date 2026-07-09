# AB-05 — Admission Mask Flow (Branchless [Admission8;256] Lookup)

| Facet | Value |
|---|---|
| Invariant | Admission is pure bit arithmetic over a fixed table — O(1) per transition, no data-dependent branches |
| Information-Loss Risk | Mask semantics compress legality to bits; the admission_table hash preserves the full table for audit |
| TPS Purpose | Standard work: one fixed lookup procedure for every transition, identical cycle time |
| DfLSS CTQ | required and forbidden masks both checked on every transition; no partial admission |
| CENG Boundary | The table is built once from the fenced universe (AB-04); admission never consults the raw graph |

```mermaid
flowchart TD
    ST[state_mask u8] --> AND1[state_mask AND required_mask]
    REQ[required_mask from Admission8 entry] --> AND1
    AND1 --> C1{"== required_mask?"}
    C1 -- no --> RF1[Refusal RequiredMaskUnmet]

    ST --> AND2[state_mask AND forbidden_mask]
    FOR[forbidden_mask from Admission8 entry] --> AND2
    AND2 --> C2{"== 0?"}
    C2 -- no --> RF2[Refusal ForbiddenMaskHit]

    T8[Triple8 id 0..255] --> LUT["[Admission8;256] branchless index"]
    LUT --> REQ
    LUT --> FOR

    C1 -- yes --> J[join]
    C2 -- yes --> J
    J --> OK[AdmittedTransition candidate to S4 legality layers]
```
