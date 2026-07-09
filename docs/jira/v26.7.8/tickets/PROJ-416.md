# PROJ-416: Wire Pattern-4 Canonical Renders to BLAKE3 Receipt Hashing
**Title:** Connect Pattern-4 canonicalization outputs to a production receipt consumer (or feature-gate them)
**Type:** Feature / Debt
**Target:** `/Users/sac/praxis` (modules: `crates/praxis-graphlaw/src/shacl/equivalence.rs` and related)
**Status:** OPEN

## Description
The Pattern-4 canonicalization surface is exported but has no production consumer — the
canonical renders are computed and then dropped instead of feeding BLAKE3 receipt hashing:

- `canonicalize_equivalences` (`crates/praxis-graphlaw/src/shacl/equivalence.rs`)
- `ClosureMatrix::render_canonical`
- `SubclassClosure.dense_to_global`
- `TripleIndexSnapshot`
- `QueryResultIter`

Doc comments on these items now state "no hash consumer exists yet (PROJ-416)". Exported
dead surface violates the principle that receipt material is computed for a consumer, not
asserted; it also risks bit-rot since nothing exercises the canonical rendering in
production paths.

## Implementation Spec
Choose one of two resolutions:

1. **Wire into the receipt path (preferred):** feed the Pattern-4 canonical renders into
   the BLAKE3 receipt pipeline (canonical N-Quads order, sorted before hashing per
   Invariant 2). Receipts derived from `ClosureMatrix::render_canonical` /
   `canonicalize_equivalences` must be byte-identical across runs.
2. **Feature-gate:** put the Pattern-4 canonical surface behind a `pattern4-canonical`
   Cargo feature so the unconsumed exports are opt-in rather than dead default surface.

Either way, update the "no hash consumer exists yet (PROJ-416)" doc comments to reflect
the resolution.

## Acceptance Criteria
- [ ] Either a production BLAKE3 receipt consumer exists for the five listed items, or
      they are gated behind `pattern4-canonical`.
- [ ] If wired: determinism test shows byte-identical receipt hashes across repeated runs.
- [ ] If gated: default build compiles without the surface; feature build compiles with it.
- [ ] Doc comments no longer claim an absent consumer without qualification.
