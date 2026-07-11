# Chapter 4: Cryptographic Evidence and the Verification Substrate

## 4.1 Introduction

In large-scale agentic execution systems, standard organizational trust mechanisms—such as human code reviews and implicit audit logs—fail to provide rigid guarantees against logic drift and state hallucinations. The Praxis architecture replaces implicit trust with deterministic mathematical verification, enforcing an unyielding invariant: *receipts are computed, never asserted*. 

By grounding execution state in canonical N-Quads and replacing wall-clock time with explicit causal structures, the verification substrate ensures that all state transitions are immutably bound by continuous cryptographic digests.

## 4.2 Workflow Evidence and Causal Manifest Generation

The foundation of the verification substrate lies in its workflow evidence generation, designed to capture workflow state progression through deterministic causal manifolds. When an orchestrated plan executes across a `PowlTape`, the system eschews raw text logging in favor of generating byte-aligned `OcelCausalFrame` objects (128-byte structures). These frames form the structural backbone of an `OcelCausalReceipt`.

To achieve deterministic tamper-evidence without allocation overhead, the system leverages rolling BLAKE3 digests. The causal sequence is maintained via a strict cryptographic recurrence:

`chain_hash(t+1) = BLAKE3(chain_hash(t) || frame_bytes)`

This streaming hash architecture ensures continuous temporal monotonicity (`receipt_i.causal_time < receipt_i+1.causal_time`) and contiguous state transitions (`receipt_i.post_state == receipt_i+1.pre_state`). The resulting execution manifest binds temporal logic execution to an immutable footprint that external verifiers can recompute algorithmically.

## 4.3 HookReceipt Chains and Deterministic Delta Hashing

For knowledge base mutations driven by subgraph deltas, the engine employs a dedicated structure known as the `HookReceipt`. Whenever a SPARQL CONSTRUCT query evaluation triggers a graph mutation, it yields a receipt encapsulating four core properties: the `hook_name`, `idempotency_key`, serialized `delta_quads`, and a BLAKE3 `delta_hash`.

Within the Chatman Engine's `praxis-graphlaw` orchestrator, delta processing is stripped of any non-deterministic inputs. Randomness and system wall-clock primitives are banned in logic paths; instead, the system strictly relies on graph OWL-Time literals. 

When serializing delta quads for addition or deletion, the engine computes a discrete BLAKE3 hash for every generated triple. For instance:

```rust
let triple_str = format!("{} {} {}", s, p, o);
let hash = blake3::hash(triple_str.as_bytes()).to_hex().to_string();
let bn_id = if is_addition { format!("_:add_{}", hash) } else { format!("_:del_{}", hash) };
```

This binds N-Quads tracking directly to blank node identifiers matching the hash of the payload (`_:add_<hash>`). Aggregating these exact cryptographic mutations yields the overarching `delta_hash`. The system achieves reproducible byte-identical outputs—across all targets—given fixed inputs. Consequently, the compilation of consecutive `HookReceipt`s acts as a distributed, mathematically rigorous ledger of all active business logic state changes.

## 4.4 Immutable Provenance: From Root to Edge

The overarching goal of the cryptographic substrate is to provide an unbroken chain of custody, ensuring that every low-level operational artifact maps securely to an authorized planning artifact. This mapping leverages both the Praxis ontology (`powl2`) and the W3C PROV-O standard (`prov`).

### 4.4.1 Root Custody via `powl2:derivedFrom`
At the highest echelon of the plan ontology, every executable model or tape is restricted to having *exactly one* provenance root. Praxis mandates that the root model node carries a single `powl2:derivedFrom` triple pointing to the composed plan source IRI. If this structural invariant is violated—if an agent attempts to inject multiple unverified root sources—the execution pipeline is structurally blocked and refused.

### 4.4.2 Edge Traceability via `prov:wasDerivedFrom`
While the root relies on `powl2:derivedFrom`, fine-grained step executions project their provenance downward utilizing the `prov:wasDerivedFrom` edge. Every leaf activity, workflow token, and generated output structurally asserts its derivation from its parent causal step. 

Using formal reasoning techniques—evidenced by both Datafrog Datalog closures and Lean theorem proving models (`con_provo.lean`)—the system computes the transitive closure of all `prov:wasDerivedFrom` relationships. This verifies that every leaf node traces flawlessly back to the singular `powl2:derivedFrom` root target. No artifact can "hallucinate" its way into the evidence manifest; if a step lacks a verifiable cryptographic link to the authorized source, the transition receipt fails validation.

## 4.5 Hash-Resistant Categorical Pullbacks
The Zero Unreceipted Actuation law is not merely a logging convention; it is implemented as a **Hash-Resistant Categorical Pullback**. The `HookReceipt` structure mathematically guarantees that the orchestrator cannot dispatch an actuation without a corresponding cryptographic hash resolving to the origin blank node. This replaces strict Galois Adjunctions with a cryptographically enforced bijection, preventing any "orphaned" side effects in the category of execution.

## 4.6 OCEL Geometry and Topological Bounds
By generating `OcelCausalFrame` objects sequentially, the engine physically constructs a 1-dimensional workflow graph. We map this to **OCEL Geometry**: because the graph is strictly 1-dimensional, its second Betti number ($b_2$) is mathematically forced to zero. This topology guarantees that 2-dimensional execution failures—such as cyclic dependency deadlocks or unresolvable race conditions—are mathematically impossible in the projected POWL execution trace.

By enforcing these constraints at both the compile-time type layer and the runtime reasoning layer, Praxis ensures that executing agents cannot spoof outcomes, bypass gates, or assert facts without mathematically sound cryptographic evidence.
