# Absolute Law

This rule refines `AGENTS.md`; it may not weaken it.

## Preserve

Preserve the strongest live statement, architecture, user intent, generated
source-of-truth, and receipt chain before changing anything. Unfamiliar state
is not invalid state. Existing mechanisms have standing until their fence is
understood and an exact replacement is verified.

## Fence

Before deletion, replacement, or refutation, identify:

- the exact objects and boundaries;
- the invariant the mechanism protects;
- the consumers and generated surfaces it feeds;
- the admission and refusal rules;
- the actuation path;
- the receipt/replay behavior;
- the known exclusions.

Adjacency, resemblance, and file-name proximity are not equivalence.

## Calculus

Use:

```text
O   = partial or stale observation
O*  = admitted, aligned, grounded, bounded observation
A   = μ(O*)
R   = receipt(A)
```

The operational state machine is:

```text
parse → route → admit/refuse → diagnose/repair → actuate → receipt → replay/hook
```

A lawful transformation must preserve the governing invariants or state the
intentional break explicitly. No wrapper may convert refusal into success.

## Exclusions

Every claim names what it does not prove. Keep checkpoint evidence separate
from crown closure. Keep local execution separate from packed, installed,
published, or deployed execution. Keep registry presence separate from runtime
reachability. Keep semantic replay separate from cryptographic receipt-chain
verification.

## Falsifier

Every material claim must name an executable falsifier. Preferred falsifiers:

- tamper a disposable receipt or artifact and verify rejection;
- mutate a critical implementation edge and verify the owning test fails;
- move the branch head and verify exact-head finalization refuses;
- reorder deterministic input and verify canonical output remains stable;
- remove a required boundary and verify typed refusal, never false success.

A test without a plausible failure mode is evidence theater.

## Extension

Extend only after the preserved mechanism is understood, the new path is
bounded, and equivalence or intentional non-equivalence is demonstrated.
Generated outputs are extended through admitted sources and generators, never
through hand edits.

## Operationalization

Use the standing lattice exactly:

- `PARTIAL_ALIVE`
- `ALIVE`
- `BLOCKED`
- `BUILD_BROKEN`
- `UNKNOWN`
- `UNSUPPORTED`

`ALIVE` requires observed execution and verified receipts in the current
session. `UNKNOWN` is not admitted. `UNSUPPORTED` is not refused.

Verification expands only after the narrow owning boundary passes:

```text
unit → integration → e2e → chaos/tamper → stress → benchmark → verifier report
```

The hard invariant is **zero unreceipted actuation**.
