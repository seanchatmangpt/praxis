# 20. Packet Diagram Family

This file contains the Packet diagram family for the Chatman Engine, structured across the 8 projection lenses.

Fallback rendering for Mermaid compatibility.

---

## Lens 1: Semantic Authority

Diagram ID: PACKET-L1
Diagram family: Packet
Projection lens: Semantic Authority
Architectural invariant preserved: RDF/Oxigraph is the single semantic source of truth. Shadow copies of RDF data are strictly prohibited.
Information-loss risk if omitted: Ambiguity in the binary layout of RDF update payload packets, leading to deserialization bugs.
TPS visual-control purpose: Prevents transport defects by explicitly mapping byte positions.
DfLSS CTQ protected: Zero semantic shadow copies.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Details the byte layout of the RDF Update payload packet.

```mermaid
flowchart TD
    subgraph PacketLayout ["RDF Update Payload Binary Layout"]
        direction TB
        B0_3["Bytes 0-3: Magic Bytes (0x52444654)"]
        B4_7["Bytes 4-7: Graph Size (Triple Count)"]
        B8_39["Bytes 8-39: BLAKE3 Graph Hash (32 Bytes)"]
        B40_N["Bytes 40+: RDF N-Quads Payload Data"]
    end
    B0_3 --> B4_7
    B4_7 --> B8_39
    B8_39 --> B40_N
```

---

## Lens 2: Routing Constitution

Diagram ID: PACKET-L2
Diagram family: Packet
Projection lens: Routing Constitution
Architectural invariant preserved: Least-expressive-power routing; hot/warm/cold path isolation. N3 is disabled by default.
Information-loss risk if omitted: Overlapping headers in routing packets leading to misrouted execution paths.
TPS visual-control purpose: Prevents routing classification errors (scrap).
DfLSS CTQ protected: Safe isolation of cold-path N3 execution.
CENG ticket or boundary constrained: CENG-411 (design-only, implementation blocked).
Why this diagram is non-redundant: Outlines routing packet headers.

```mermaid
flowchart TD
    subgraph PacketLayout ["Query Routing Header Binary Layout"]
        direction TB
        B0["Byte 0: Path ID (0x01 = Hot, 0x02 = Warm, 0x03 = Cold)"]
        B1["Byte 1: Quarantine Flag (0x00 = Clear, 0x01 = Quarantined)"]
        B2_3["Bytes 2-3: Constraints Count (u16)"]
        B4_11["Bytes 4-11: Profile Gate Mask (64-bit mask)"]
    end
    B0 --> B1
    B1 --> B2_3
    B2_3 --> B4_11
```

---

## Lens 3: Type Kernel Ownership

Diagram ID: PACKET-L3
Diagram family: Packet
Projection lens: Type Kernel Ownership
Architectural invariant preserved: Canonical type ownership across wasm4pm-compat, wasm4pm-cognition, bcinr-pddl, bcinr-powl, and praxis-graphlaw.
Information-loss risk if omitted: Untracked module source identifiers in binary type structures.
TPS visual-control purpose: Prevents cross-module type mapping duplication.
DfLSS CTQ protected: Zero duplicate type classes.
CENG ticket or boundary constrained: CENG-412 (design-only, implementation blocked).
Why this diagram is non-redundant: Outlines the binary structure of type identifiers.

```mermaid
flowchart TD
    subgraph PacketLayout ["Type Identifier Binary Layout"]
        direction TB
        B0["Byte 0: Module Owner (0x01=compat, 0x02=cognition, 0x03=pddl, 0x04=law)"]
        B1["Byte 1: Type Family Identifier (e.g. Breed, Schema, Rule)"]
        B2_5["Bytes 2-5: Type Registry Offset (u32)"]
        B6_N["Bytes 6+: Type Metadata Payload (Closed Vocab)"]
    end
    B0 --> B1
    B1 --> B2_5
    B2_5 --> B6_N
```

---

## Lens 4: Transition Lifecycle

Diagram ID: PACKET-L4
Diagram family: Packet
Projection lens: Transition Lifecycle
Architectural invariant preserved: Every transition must pass through candidate invocation, validation, planning, execution, receipting, and replay.
Information-loss risk if omitted: Sending transitions with missing lifecycle stage tags or invalid receipt offsets.
TPS visual-control purpose: Eliminates transaction lifecycle ordering defects.
DfLSS CTQ protected: Guaranteed transaction replay validation under fixed seed.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Outlines the binary layout of state transition packets.

```mermaid
flowchart TD
    subgraph PacketLayout ["Transition Packet Binary Layout"]
        direction TB
        B0_3["Bytes 0-3: Magic Bytes (0x5452414E)"]
        B4["Byte 4: Stage ID (0x01=Invoc, 0x02=Val, 0x03=Act, 0x04=Replay)"]
        B5_36["Bytes 5-36: BLAKE3 Receipt Hash (32 Bytes)"]
        B37_44["Bytes 37-44: OWL-Time Literal Timestamp (u64)"]
    end
    B0_3 --> B4
    B4 --> B5_36
    B5_36 --> B37_44
```

---

## Lens 5: Event / Hook / Actuation

Diagram ID: PACKET-L5
Diagram family: Packet
Projection lens: Event / Hook / Actuation
Architectural invariant preserved: Hooks cannot actuate without receipts; no unreceipted actuation.
Information-loss risk if omitted: Loss of receipt signature tracking in external event packets.
TPS visual-control purpose: Poka-Yoke check validating receipt fields prior to ingestion.
DfLSS CTQ protected: Zero unreceipted execution events.
CENG ticket or boundary constrained: CENG-416A-F (design-only, implementation blocked).
Why this diagram is non-redundant: Details the layout of incoming event payloads and signatures.

```mermaid
flowchart TD
    subgraph PacketLayout ["OCEL Event Payload Binary Layout"]
        direction TB
        B0_3["Bytes 0-3: OCEL Event ID (u32)"]
        B4_7["Bytes 4-7: Ingestion Timestamp (u32)"]
        B8_11["Bytes 8-11: Knowledge Hook ID (u32)"]
        B12_43["Bytes 12-43: BLAKE3 Receipt Signature (32 Bytes)"]
    end
    B0_3 --> B4_7
    B4_7 --> B8_11
    B8_11 --> B12_43
```

---

## Lens 6: Performance / 8-Constraint Hot Path

Diagram ID: PACKET-L6
Diagram family: Packet
Projection lens: Performance / 8-Constraint Hot Path
Architectural invariant preserved: Maximum of 8 constraints checked in parallel via RDFTriple8 and ConditionCell<BITS>.
Information-loss risk if omitted: Exceeding the 64-bit alignment size of the hot-path ConditionCell.
TPS visual-control purpose: Andon check verifying hot-path bit alignments.
DfLSS CTQ protected: Latency bound of hot path operations.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Outlines the hot-path ConditionCell binary layout.

```mermaid
flowchart TD
    subgraph PacketLayout ["ConditionCell Binary Layout"]
        direction TB
        B0["Byte 0: Constraints Size (Max 8, ConditionCell<BITS> size)"]
        B1_8["Bytes 1-8: Byte Mask (64-bit Condition Cell)"]
        B9_10["Bytes 9-10: State Admission Table Offset (u16)"]
        B11_12["Bytes 11-12: Latency SLA Upper Bound (u16)"]
    end
    B0 --> B1_8
    B1_8 --> B9_10
    B9_10 --> B11_12
```

---

## Lens 7: Refusal / Risk / Governance

Diagram ID: PACKET-L7
Diagram family: Packet
Projection lens: Refusal / Risk / Governance
Architectural invariant preserved: Every failure is a typed Refusal; N3 quarantine rules are strictly enforced.
Information-loss risk if omitted: Untyped exception code or unsigned governance overrides.
TPS visual-control purpose: Exposes safety and refusal fields to prevent security escapes.
DfLSS CTQ protected: No panic or silent fallbacks.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Outlines the binary structure of Refusal packets.

```mermaid
flowchart TD
    subgraph PacketLayout ["Refusal Payload Binary Layout"]
        direction TB
        B0["Byte 0: Refusal Category (0x01=Schema, 0x02=Auth, 0x03=Quarantine)"]
        B1["Byte 1: Quarantine Status (0x00=Normal, 0x01=Enforced)"]
        B2_3["Bytes 2-3: Refusal Error Code (u16)"]
        B4_35["Bytes 4-35: CENG Board Approval Signature Hash (32 Bytes)"]
    end
    B0 --> B1
    B1 --> B2_3
    B2_3 --> B4_35
```

---

## Lens 8: TPS / DfLSS / Continuous Improvement

Diagram ID: PACKET-L8
Diagram family: Packet
Projection lens: TPS / DfLSS / Continuous Improvement
Architectural invariant preserved: WIP reduction, continuous process improvement loops, and visual waste elimination.
Information-loss risk if omitted: Missing measurement data fields in Kaizen tracking payloads.
TPS visual-control purpose: Structures Kaizen metrics for performance telemetry.
DfLSS CTQ protected: Flow efficiency and defect rate minimization.
CENG ticket or boundary constrained: CENG-410-FINAL (in progress).
Why this diagram is non-redundant: Details the telemetry packet structure for Kaizen metrics.

```mermaid
flowchart TD
    subgraph PacketLayout ["Kaizen Metrics Telemetry Binary Layout"]
        direction TB
        B0_3["Bytes 0-3: Kaizen Iteration ID (u32)"]
        B4_7["Bytes 4-7: Core WIP Limit (u32)"]
        B8_11["Bytes 8-11: Avg Latency Score (u32, microseconds)"]
        B12_15["Bytes 12-15: Defect Rate Score (u32, ppm)"]
    end
    B0_3 --> B4_7
    B4_7 --> B8_11
    B8_11 --> B12_15
```
