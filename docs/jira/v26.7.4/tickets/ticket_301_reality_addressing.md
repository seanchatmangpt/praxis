# Ticket: Reality Addressing Layer — CLOSED

## Title
Bind admitted referents to public-ontology coordinates (PROJ-301) — **STATUS: CLOSED**

## Description
The v26.7.4 vision names a three-way address split: content address (exact bytes), reality
address (admitted real-world referent), receipt address (action standing). Content and receipt
addresses already existed in this crate. Reality address did not. This ticket closed that gap
with `crates/praxis-synthesis/src/reality.rs`:

- `RealityAddressRecord::bind(triples, subject)` reads three PUBLIC ontology predicates
  directly off the already-admitted graph — `OWL-Time`'s `inXSDDateTimeStamp`, `GeoSPARQL`'s
  `asWKT`, `PROV-O`'s `wasAttributedTo` — no new private time/space/provenance vocabulary was
  invented.
- A subject with all three anchors absent is refused (`Refusal::RealityAddressIllFormed`), not
  silently returned as a valid-looking empty record — an unanchored subject is bare graph
  content, not a reality address.
- `reality_hash()` computes a content address over the record's own canonical rendering
  (never asserted), matching this crate's universal hashing doctrine.
- Wired into `lib.rs`: `pub mod reality;`, public re-exports, and a new `Refusal` variant
  appended at the end of the enum (a safe, non-conflicting append point while a separate
  adversarial-repair pass was concurrently editing `delta.rs`/`quarantine.rs`/`hooks.rs`).

## Acceptance Criteria
- [x] `RealityAddressRecord` with private fields (constructor-witnessed, consistent with the
  existing `GraphDelta`/`Reference`/`AdmittedEvent` hardening pattern).
- [x] Binds to exactly the three named public predicates; no praxis-private time/space/
  provenance vocabulary added.
- [x] Refuses zero-anchor subjects by name.
- [x] `reality_hash()` is computed, stable under re-binding, and differs for differing anchor
  sets.
- [x] 4 tests, all passing.

## Dependencies
None.

## Verification Mechanism
1. `cargo test -p praxis-synthesis --lib reality::` — 4/4 passing (confirmed).
2. `cargo test -p praxis-synthesis` — full suite unaffected by the addition.
3. Read `crates/praxis-synthesis/src/reality.rs` directly to confirm no wall-clock, no
   external HTTP/network call, and no dependency added to `Cargo.toml`.
