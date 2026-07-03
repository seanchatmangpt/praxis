# Ticket: Confirm/Extend the Receipt Chain as the Temporal-Memory Layer (figex/KGC-4D Claim)

## Title
Audit `firing.rs` against the figex/KGC-4D event-sourcing + snapshot + replay claim (PROJ-202)

## Description
The figex/KGC-4D material claims event-sourced design files, temporal RDF, Git-backed
snapshots, wormhole time-travel, cryptographic integrity, and quality gates. Praxis-synthesis's
outer firing chain (`crates/praxis-synthesis/src/firing.rs`) already folds
`event_hash -> admission_hash -> handler_hash -> hook_hash -> history_hash -> inner chains ->
outcome_hash`, and `replay_firing` re-derives every stage plus binds every embedded payload to
its claimed hash. This already covers event history, cryptographic integrity, and replay.

The one genuine open question: figex additionally claims durable, Git-backed **snapshots** of
intermediate state ("wormhole time travel"). Praxis's `Reference` (the admitted base state) is
in-memory only — reproducibility instead comes from replaying base TTL + delta documents
through the deterministic pipeline, which is a different (and arguably sufficient) mechanism:
you don't need to store every intermediate graph state if you can deterministically
re-derive any of them from the genesis TTL plus the ordered delta log. This ticket makes that
tradeoff explicit and produces a written ADOPT/DEFER verdict rather than silently building a
snapshot mechanism nobody asked for.

## Acceptance Criteria
- A written verdict is added to `docs/v26.7.3/RECEIPTS_REPLAY_VERIFY.md` (new subsection,
  e.g. "Snapshot vs. replay-from-genesis") stating explicitly: praxis chooses replay-from-
  genesis-plus-deltas over disk snapshots, and why (determinism + content-addressing already
  gives reproducibility without needing to persist intermediate graph states).
- If the verdict is DEFER (the expected outcome), no new code is required.
- If a genuine need for snapshotting surfaces (e.g. replaying a very long delta history becomes
  a real performance problem), that becomes a separate, concretely-scoped follow-up ticket with
  a benchmark showing the problem — not folded into this one.
- No new "KGC-4D" or "temporal RDF" vocabulary is introduced into the crate; existing
  `epoch: u64` (logical, not wall-clock) remains the only temporal dimension.

## Dependencies
PROJ-201 (scoping precedent for how this project handles "adopt external theory vs. keep
existing simpler mechanism" decisions).

## Verification Mechanism
1. `cargo test -p praxis-synthesis --test firing_chain --test repair_loop` — green, unchanged.
2. `grep -n "Snapshot vs" docs/v26.7.3/RECEIPTS_REPLAY_VERIFY.md` — verdict section present.
3. Manual read confirming no snapshot-to-disk code was added unless the verdict was ADOPT.
