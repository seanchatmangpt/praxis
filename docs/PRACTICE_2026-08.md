# Praxis capability convergence — 2026-08

Praxis now treats reconciliation as an executable law boundary rather than an autonomic loop with ambient repair authority.

## Runtime correspondence

```text
raw O
  -> admission
O*
  -> DfCM SELECT (preserve all bounded edges; prefer reversible lawful edges)
Selection
  -> CONSTRUCT
PreparedReconciliation / ConstructedIntent
  -> fresh O* check
  -> exact construct-bound authority admission
BRCE DO
  -> atomic BLAKE3 ActuationReceipt
replay
  -> fresh O* observation
ReconcileCheckpoint (PARTIAL_ALIVE)
```

The checkpoint is deliberately `PARTIAL_ALIVE`: one successful repair edge is not proof that the whole subject is closed.

## Techniques made executable

- **Chatman Equation:** `A = μ(O*)`; raw observations cannot reach actuation.
- **DfCM / combinatorial maximalism:** every bounded candidate edge is retained, including excluded topology, before deterministic selection.
- **SELECT / CONSTRUCT / DO:** `prepare_reconciliation` manufactures the exact intent without DO; `execute_prepared` rechecks freshness and admits construct-bound authority before actuation.
- **BRCE:** missing, mismatched, or insufficient authority refuses before the actuator is called; success requires a recomputable receipt and replay match.
- **Evidence lattice:** observed/admitted/executed/changed/verified/inferred/refused/blocked/unsupported are independent facts; standing is scoped separately.
- **Gall checkpoints:** the reconciler executes one bounded checkpoint at a time rather than hiding an unbounded autonomic loop.
- **Typed refusal:** expected failure is represented by stable refusal codes with salvage metadata.
- **BLAKE3 identity:** O*, selection, construct, receipt, and checkpoint have content-derived identities.
- **Deterministic replay:** the DO receipt carries a replay key and must reproduce the exact post-state identity.
- **Public ontology projection:** `ontology/praxis-reconciler.ttl` maps observations and receipts to PROV-O, authority to ODRL, logical time to OWL-Time, and Praxis phase vocabulary to SKOS concepts.
- **Anti-theater testing:** tests prove missing authority causes zero calls, construct mismatch causes zero calls, prepared-object mutation and stale observation cause zero calls, valid authority produces one receipt/replay, and receipt mutation is rejected.

## Ecosystem boundary

Praxis owns planning/reconciliation/action execution. `bcinr` remains the deterministic planning substrate. `mfact` remains the proof/factory layer. `ggen` renders projections. None of those layers receives ambient DO authority through this crate; they may only supply observations, candidate operators, proofs, or constructed intents.

The custom ontology is a semantic projection of the Rust types, not an alternate execution surface. Runtime authority remains in the BRCE boundary.

## Capability ingress contract

The reconciler is intentionally protocol-agnostic. CLI, API, MCP, A2A, OCEL/process-intelligence, gym/planner, and ggen/reconstitution surfaces may project into the same four bounded object classes: `Observation`, `RepairOperator`, `ConstructedIntent`, and `AuthorityGrant`. Protocol adapters do not inherit authority merely because they can construct one of those objects.

This lets newer ecosystem techniques compose without introducing a second execution constitution:

- reconstitution and ontology compilers can manufacture `O`/`O*` projections;
- planners, gyms, ERRC/TRIZ, and DfCM search can manufacture candidate topology;
- GraphLaw/Lean/mfact-style proof systems can strengthen admission evidence;
- OCEL v2 and process-mining surfaces can corroborate execution and replay;
- CLI/API/MCP/A2A remain transports around the same admission and BRCE boundary.

Those are composition seams, not blanket `ALIVE` claims. Each adapter still needs exact-subject execution evidence before its own standing can be promoted.
