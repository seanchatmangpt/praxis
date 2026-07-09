# PROJ-414: Implement BLAKE3 Receipt Evidence Generation
**Title:** Generate Proof-of-Consequence Receipts for Admitted Transitions
**Type:** Feature
**Target:** `/Users/sac/praxis/src/chatman/engine.rs`
**Status:** IN PROGRESS (workflow wf_255e0807)

> Note: path details in this ticket are superseded by
> `docs/chatman-engine/DEFINITION_OF_DONE.md`.

## Description
To fulfill the invariant "No unreceipted actuation", the final step of `admit_transition` must freeze the graph state into canonical N-Quads and emit a BLAKE3 receipt.

## Implementation Spec
1. **File Mod:** `/Users/sac/praxis/src/chatman/engine.rs` (inside `impl ChatmanEngine`)
   * Implement `fn generate_receipt(&self, graph_name: &NamedNode) -> Result<Receipt, Refusal>`.
   * Execute an ordered SPARQL query over the specific graph snapshot: `SELECT ?s ?p ?o WHERE { GRAPH <graph_name> { ?s ?p ?o } } ORDER BY ?s ?p ?o`.
   * Format the `QueryResults::Solutions` into a canonical N-Quads `String`.
   * Generate the hash using `blake3::hash(canon_nquads.as_bytes()).to_hex().to_string()`.
   * Return the fully populated `Receipt`.

## Acceptance Criteria
- [ ] **Invariant Check:** Zero usage of `SystemTime`, `Instant::now()`, or `chrono` within the receipt generation path (No wall clock in receipt paths).
- [ ] The generated hash is fully deterministic across identical graph inputs.
