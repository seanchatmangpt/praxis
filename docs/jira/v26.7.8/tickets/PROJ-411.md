# PROJ-411: Define CE-ABI Boundary and Typed Refusals
**Title:** Implement `CE-ABI` Envelopes and Typed Refusals for Engine Boundaries
**Type:** Feature / Architecture
**Target:** `/Users/sac/praxis` (module: `src/chatman/`)
**Status:** IN PROGRESS (workflow wf_255e0807)

> Note: path details in this ticket are superseded by
> `docs/chatman-engine/DEFINITION_OF_DONE.md`.

## Description
The Chatman Engine requires a deterministic boundary (CE-ABI) between the semantic graph, planners, hooks, and execution traces. This prevents the recreation of intermediate representations. Every failure across this boundary must be a typed `Refusal` (Invariant 1). 

## Implementation Spec
1. **New File:** `/Users/sac/praxis/src/chatman/abi.rs` (approx. 60 LOC)
   * Define `InvocationId(String)`, `GraphSnapshotId(String)`, `ProfileId(String)`, `OperatorId(String)`.
   * Define `InputHandles { nodes: Vec<String>, events: Vec<String>, plan_steps: Vec<String> }`.
   * Define `InvocationEnvelope { invocation_id, snapshot_id, profile_id, operator_id, input_handles }`.
   * Define `Receipt { blake3_hash: String, canon_nquads: String }`.
   * Define `AdmittedTransition { receipt: Receipt }`.
   * Define `#[derive(Debug, thiserror::Error)] pub enum Refusal` with explicit variants:
     * `#[error("Validation failed: {0}")] ValidationFailed(String)`
     * `#[error("Plan infeasible: {0}")] PlanInfeasible(String)`
     * `#[error("Trace unlawful: {0}")] TraceUnlawful(String)`
     * `#[error("Hook unpermitted: {0}")] HookUnpermitted(String)`
     * `#[error("Missing receipt: {0}")] MissingReceipt(String)`
     * `#[error("Snapshot not found: {0}")] SnapshotNotFound(String)`

2. **File Mod:** register `pub mod abi;` in the `src/chatman/` module root (not in a
   top-level `src/lib.rs` of `praxis-graphlaw`).

## Acceptance Criteria
- [ ] No panics, unwraps, or silent defaults anywhere in `abi.rs`.
- [ ] All structs implement `Clone, PartialEq, Eq, Hash, Serialize, Deserialize`.
