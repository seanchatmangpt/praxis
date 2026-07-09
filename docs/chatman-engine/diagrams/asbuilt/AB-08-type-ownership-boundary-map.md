# AB-08 — Type-Ownership Boundary Map

| Facet | Value |
|---|---|
| Invariant | Each core type has exactly one owning crate; praxis-graphlaw wraps foreign types, never redefines them |
| Information-Loss Risk | Duplicate type definitions would fork semantics silently; single ownership keeps one authoritative shape |
| TPS Purpose | Cellular layout: each crate is a cell owning its parts; no cross-cell part fabrication |
| DfLSS CTQ | Zero foreign-type constructions outside the owning crate (compile-level enforcement) |
| CENG Boundary | Ownership crossings go through wrappers only; direct construction across the boundary is a Refusal/compile error |

```mermaid
flowchart TD
    subgraph compat[wasm4pm-compat]
        RE[ReceiptEnvelope]
        DG[Digest]
        OE[OcelEvent]
        PW[Powl]
        DF[Dfg]
        WN[WfNet]
        CC[ConditionCell]
    end

    subgraph core[wasm4pm-core]
        PT[Pddl8Tape]
    end

    subgraph powl[bcinr-powl]
        VT[v2::PowlTape]
    end

    subgraph powlr[bcinr-powl-receipt]
        OCF[OcelCausalFrame]
    end

    subgraph gl[praxis-graphlaw]
        W[Wrapper layer — wraps only, owns none of the above]
    end

    W -. wraps .-> RE
    W -. wraps .-> DG
    W -. wraps .-> OE
    W -. wraps .-> PW
    W -. wraps .-> DF
    W -. wraps .-> WN
    W -. wraps .-> CC
    W -. wraps .-> PT
    W -. wraps .-> VT
    W -. wraps .-> OCF
```
