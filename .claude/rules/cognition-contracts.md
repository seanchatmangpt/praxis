# Cognition Contracts

This rule governs cognition breeds, planners, hooks, process-mining algorithms,
projection layers, composed pipelines, and their CLI/API/WASM bridges.

## 1. Separate observation, proposal, authority, and actuation

A cognition component may observe and propose without authority to actuate.
Authority is explicit, scoped, admitted, and receipted.

```text
Observation → Admission → Breed execution → Proposal → Authority check →
Projection/actuation → Receipt → Replay
```

A witness observes only. A proposal is visible but inert. A projection is not
a source of authority. No breed may self-elevate its permits.

## 2. Breed closure matrix

A breed or algorithm may be called `ALIVE` only when the exact claimed surface
has all applicable rows:

| Row | Required evidence |
|---|---|
| Identity | Stable registry ID and version |
| Admission | Positive admitted input and typed refused input |
| Reachability | Dispatcher and public surface reach the implementation |
| Boundary | Real CLI/API/WASM/subprocess/state transition executes |
| Determinism | Byte-stable trace under fixed input and seed |
| Authority | Per-breed permit and projection-authority tests |
| State | Multi-run state does not leak or silently alias |
| Composition | Downstream handoff preserves obligations and refusal semantics |
| Invariant | At least one property/invariant test |
| Receipt | Recomputable digest over canonical inputs and outputs |
| Replay | Replay succeeds; tampering is rejected |
| Exclusions | Unsupported surfaces and missing closure are named |

Registry enumeration alone is not closure. A passing happy path is not global
behavior evidence.

## 3. Composition laws

Composed pipelines must satisfy:

1. ordered breed execution is explicit and deterministic;
2. each breed receives admitted input, not ambient mutable state;
3. duplicate observations are typed refusals or explicitly idempotent;
4. obligations are handed off without loss or unauthorized weakening;
5. proposal visibility does not imply downstream authority;
6. a breed failure remains typed at the composition boundary;
7. an empty pipeline has explicit semantics;
8. projections remain stable across composition;
9. multi-run state is isolated or intentionally persistent and receipted;
10. defaults are admitted configuration, not hidden compatibility shims.

Remove default shims when the public composition contract becomes explicit.
Compatibility behavior may not obscure missing required fields.

## 4. Process-mining algorithm closure

For discovery, conformance, OCEL, social-network, and related algorithms, prove:

- registry and dispatcher reachability;
- exact algorithm identity and parameters;
- positive fixture from admitted source data;
- typed malformed/adversarial fixture;
- a domain invariant or published-value check where applicable;
- host and WASM behavior parity where both are claimed;
- CLI bridge exit code and refusal mapping;
- stable serialization and receipt digest;
- no panic, silent fallback, fabricated model, or forced closure;
- a witness for formal claims such as soundness, fitness, or deadlock freedom.

A model object existing in memory does not prove it was discovered from the
claimed event log. A Boolean `true` is not a formal witness.

## 5. Generated cognition surfaces

Generated registries, IDs, paper pointers, anti-cheat fixtures, bridge tables,
and capability matrices are projections. Change their admitted ontology or
specification source, run the generator, run it again for idempotence, and
verify the receipt.

Hand-flipping standing, registry state, or generated IDs is evidence
falsification.

## 6. Anti-cheating and proof teeth

Primary evidence paths may not use mocks, fabricated traces, synthetic OCEL,
fake receipts, hardcoded successful outputs, missing-fixture skips, or test-only
authority implementations.

For every crown claim, perform a disposable tamper or mutation check. The
verifier must reject the corrupted artifact or broken edge before the valid
state is restored.

## 7. Status discipline

- `PARTIAL_ALIVE`: a real subset executes; missing rows are named.
- `ALIVE`: every row required by the exact claim executes and receipts verify.
- `BLOCKED`: the next required boundary is known but unreachable.
- `BUILD_BROKEN`: exact tree obtained; owner build/verifier fails.
- `UNKNOWN`: target evidence not observed.
- `UNSUPPORTED`: intentionally not implemented by contract.

Per-breed `ALIVE` does not promote a composed pipeline. A composed pipeline does
not promote every alternate projection or deployment target.
