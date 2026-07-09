# PROJ-412: Unify `praxis-graphlaw` with `bcinr-pddl` and `bcinr-powl`
**Title:** Add and Configure Chatman Engine Dependency Chain
**Type:** Chore / Integration
**Target:** `/Users/sac/praxis/Cargo.toml`
**Status:** IN PROGRESS (workflow wf_255e0807)

> Note: path and dependency details in this ticket are superseded by
> `docs/chatman-engine/DEFINITION_OF_DONE.md`.

## Description
To manufacture graph dialect artifacts, `praxis-graphlaw` must route execution through `bcinr-pddl` (for RDF-structured planning) and `bcinr-powl` (for lawful trace admission) using the shared workspace's `oxigraph` backend.

## Implementation Spec
1. **File Mod:** add the following absolute-path dependencies (no workspace `[patch]`
   entries — build against upstream `oxigraph`):
   ```toml
   bcinr-pddl = { path = "/Users/sac/bcinr/crates/bcinr-pddl" }
   bcinr-powl = { path = "/Users/sac/bcinr/crates/bcinr-powl" }
   bcinr-powl-receipt = { path = "/Users/sac/bcinr/crates/bcinr-powl-receipt" }
   oxigraph = "0.5.9"
   ```
   * *Note:* `blake3 = "1"` is already present for receipt generation.
2. **No local patches:** `oxigraph 0.5.9` comes from upstream (crates.io); there are no
   `oxigraph-local`/`oxrdf-patched` workspace patches to unify against.
3. **Type facts:** `Pddl8Tape` is defined in `wasm4pm-compat`/`wasm4pm-core` and
   re-exported by `bcinr-pddl`. `wasm4pm-compat` has no `ProcessReceipt` type — receipt
   material is `ReceiptEnvelope`/`Digest`/`blake3_hex`; wrap those.
4. **Toolchain:** `nightly-2026-06-22` (pinned in `rust-toolchain.toml`).

## Acceptance Criteria
- [ ] `cargo check` builds cleanly with exactly one `oxrdf =0.3.3` and upstream
      `oxigraph 0.5.9` (rdf-12 ON); no duplicate `wasm4pm-compat` instances.
