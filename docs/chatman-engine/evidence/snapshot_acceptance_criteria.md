# Snapshot Acceptance Criteria — chatman_s1_receipt_shape

Defined before inspecting `.snap.new`, so the review is a checklist match, not a judgment call
made after seeing the content.

A `.snap.new` is admissible (→ `cargo insta accept`) only if ALL of the following hold:

1. Receipt shape is stable — the same top-level field set as the other 3 committed `.snap`
   baselines in the same test file (structural consistency).
2. All 9 digest slots are present (per the "9-hash BLAKE3 receipt envelope" / "constitutional
   order" design — cross-check against `EngineProcessReceipt` in
   `crates/praxis-graphlaw/src/chatman/engine.rs`: 9 digests + `receipt_root` +
   `canon_nquads`. Note: `abi.rs`'s `Receipt` is the compat envelope type, not this struct).
3. Digest names and order match the doctrine documented in `EngineProcessReceipt`'s field
   doc comments (`engine.rs`) — no undocumented reordering.
4. No volatile timestamp, random value, or absolute filesystem path appears anywhere in the
   snapshot body.
5. No `HashMap`-iteration-order leak — every collection in the snapshot must be in a
   documented sorted/canonical order.
6. No missing receipt field relative to the `Receipt` struct's actual field list in source.
7. No new semantic field appears that isn't already reviewed/expected — a `.snap.new` that
   introduces a field never discussed in this session's work is a BLOCKED finding, not an
   auto-accept, and must be reported instead of accepted.

If ALL 7 hold: accept the baseline **inside praxis** at
`crates/praxis-graphlaw/tests/snapshots/` (the test binds
`Settings::set_snapshot_path` to that directory — never accept into chicago-tdd-tools),
then re-run the specific test by name to confirm pass, then the full chatman suite.

Resolution 2026-07-09: all 7 held; baseline accepted at
`crates/praxis-graphlaw/tests/snapshots/chicago_tdd_tools__testing__snapshot__chatman_s1_receipt_shape.snap`;
`chatman_snapshot_semantics` passes 3/3. The stale `.snap.new` in chicago-tdd-tools was
left untouched (out of this repo's scope).

If ANY fail: do not accept. Either the producing code has a real bug (fix `src/chatman/`, not
the snapshot) or this is a genuine ambiguity requiring a BLOCKED report with file:line.
