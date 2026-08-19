# PROJ-814: Extract shared receipt-root fold helper into `chatman-common`

**Status**: OPEN — see audit note. The independent audit confirmed the implementer's own
finding: a real discrepancy exists between `engine.rs`'s and `otel_receipt.rs`'s byte-layout
conventions that blocks extracting a single shared `fold_digest_root` helper as originally
scoped. The implementer correctly stopped and touched no files rather than force a helper over
a mismatched convention; the audit found this stop-and-report decision correct, not the
extraction itself complete. No shared helper exists yet — this ticket remains open pending a
reconciled design for the two conventions (or an explicit decision to keep them separate).
**Dependencies**: PROJ-811

## Scope

From the chatman cross-analysis: the tagged-ordered-digest-fold convention is implemented twice,
independently:

- **`crates/praxis-graphlaw/src/chatman/engine.rs:242,263`** — `EngineProcessReceipt`,
  `receipt_root`, the canonical 9-digest fold for the S1–S6 admission pipeline.
- **`crates/cng/src/otel_receipt.rs:34-36,234`** — a disclosed, deliberate reimplementation of
  the same fold convention, because `cng` does not include `praxis-graphlaw` in its default
  build surface.

Both are receipt-root fold *conventions* over the same shape (ordered digest list → BLAKE3
fold), not competing receipt *shapes* — this is duplicated mechanism, not duplicated meaning,
which is what makes it the lowest-risk unification target identified in the chatman review.

## Proposed change

1. Add a standalone function to `crates/chatman-common/src/provenance.rs`, e.g.:
   ```rust
   /// Folds an ordered sequence of tagged digests into a single BLAKE3 root hash.
   /// Order is significant — callers must sort/tag digests deterministically before calling.
   pub fn fold_digest_root(tagged_digests: &[(&str, [u8; 32])]) -> [u8; 32]
   ```
   matching the actual fold semantics used by `engine.rs::receipt_root` (verify the exact
   tag/order/separator convention by reading `engine.rs:242-263` in full before writing the
   helper — do not guess the byte layout).
2. `chatman::engine.rs` wraps/re-exports it rather than keeping its own copy.
3. `cng::otel_receipt.rs:34-36` imports it instead of reimplementing.

## Why this is autonomous-safe

- `chatman-common` has zero dependents inside `praxis-graphlaw` today (confirmed in the prior
  chatman survey) — no crate-graph cycle risk from `praxis-graphlaw` depending on it.
- `chatman-common` has no `praxis-graphlaw` dependency, so `cng` → `chatman-common` is a new,
  one-directional edge — safe to add.
- Purely additive at the type level (new function, existing call sites updated to call it) —
  reversible in one commit per this repo's FIX FORWARD ONLY / git-reversibility discipline.

## Explicitly out of scope for this ticket

Do **not** attempt to unify the four different *Receipt shapes* found across the ecosystem
(`chatman::abi::Receipt`, `chatman-common::SignedReceipt`/`TempReceipt`/`TestReceipt`,
`multifractal-workflow::f25::Receipt`) — the chatman review flagged that as needing human
sign-off (they may be structurally different for good reason: production/signed/test-fixture/
replay are different problems). This ticket only extracts the shared low-level fold primitive
both `EngineProcessReceipt` and `otel_receipt`'s reimplementation build on top of.

## Verification plan

```
just praxis-graphlaw-check
just fmt-check
just test-changed
```
Plus a determinism check per this repo's invariant #5: run the affected receipt-generating test
5× consecutively and confirm byte-identical output, per
`.claude/rules/rust-agi-core-team.md` §1.
