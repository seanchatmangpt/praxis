# lean-lake

Lake-built Lean 4 project (`Praxis.lean`, `Praxis/Corpus/`, `Praxis/Mathlib/`,
`Praxis/Milestone/V26711/`) with a working `lakefile.lean` and `lean-toolchain`. Unlike
`tools/paper-factory/lean-pilot/` (bare draft `.lean` files, never built), this directory's
files that are reachable from `Praxis.lean`'s import graph are actually compiled by
`lake build` and produce `.olean` artifacts. See `README_mathlib_migration_receipts.md`
for the detailed status-vocabulary definitions and the 2026-07-12 correction record.

## Terminology note

`mathlib_migration_receipts.jsonl` is a self-reported status log from formalization
attempts (`label`/`status`/`attempts`/`last_error` fields, no hash, no timestamp), not a
cryptographic receipt; do not cite "verified" status here as proof without independently
re-running `lake build`. For a real BLAKE3 hash-chained receipt, use `crates/praxis-lean`'s
`praxis-l4 verify` command (schema v2, `VerificationReceipt` in
`crates/praxis-lean/src/receipt.rs`), which this file predates and which the crate reads
via a read-only `LegacyReceiptV1` compatibility shim, not as its native format.
