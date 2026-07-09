# PROJ-417: Surface Status::HashMismatch in verify_replay
**Title:** Emit `Status::HashMismatch` from a full-pipeline replay comparison in the WASM bridge
**Type:** Feature / Debt
**Target:** `/Users/sac/praxis` (module: `crates/praxis-graphlaw-wasm/src/`)
**Status:** OPEN

## Description
`Status::HashMismatch` (`crates/praxis-graphlaw-wasm/src/dto.rs:30-37`) is documented as
reserved and is not constructible from current logic: `verify_replay`
(`crates/praxis-graphlaw-wasm/src/core.rs:621-649`) only re-hashes the parse artifact
(preprocessed Turtle → `TripleStore` → content string), ignoring its `_profile_ttl`,
`_shacl_shapes`, `_shex_schema`, and `_shex_shape_map` parameters. It can therefore only
emit `Admitted` or `ReplayMismatch`; the materialization/validation stages are never
re-executed, so a post-materialization hash divergence is undetectable.

## Implementation Spec
1. Extend `verify_replay` to re-run the full validation pipeline (parse → profile →
   materialization → SHACL/ShEx validation) on the second pass, using the currently
   ignored parameters.
2. Recompute the canonical graph hash after materialization and compare it against the
   first run's post-materialization hash; emit `Status::HashMismatch` when the canonical
   graph hashes diverge (distinct from `ReplayMismatch`, which covers execution-result
   divergence).
3. Remove the "Reserved / not yet constructible" caveat from the `HashMismatch` doc
   comment in `dto.rs` once constructible.
4. Preserve the documented O(2 * cost_of_validation) bound — replay is exactly one extra
   full pass, no more.

## Acceptance Criteria
- [ ] `verify_replay` performs a full-pipeline second run, not just a parse re-hash.
- [ ] `Status::HashMismatch` is constructible and returned on canonical-hash divergence,
      with an end-to-end test triggering it.
- [ ] Happy path still emits `Admitted` with byte-identical hashes across runs.
- [ ] No panics, unwraps, or silent defaults in new code.
