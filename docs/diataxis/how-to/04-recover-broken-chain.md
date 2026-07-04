# How to Recover From a Broken Receipt Chain

`ggen receipt history` fails with a non-zero exit naming a broken index in
`.ggen-v2/receipt-log.jsonl`, and you need to know which check failed and
what your real options are.

This guide assumes you already know what the receipt chain is and how
`ggen sync` extends it. If you don't, work through
[Verifying a Receipt Chain](../tutorials/04-verifying-receipts.md) first.

## Step 1: Read the FM-CHAIN code

Every failure from `handle_receipt_history` is an `AppError::fm_chain(code,
msg)` (`/Users/sac/praxis/crates/ggen/src/verbs/handlers.rs:149-248`,
formatting at `/Users/sac/praxis/crates/ggen/src/error.rs:94-97`). The code
tells you which of four things broke:

| Code | Meaning | Source |
|------|---------|--------|
| `FM-CHAIN-005` | Log file missing, or present but every line was blank (`receipts.is_empty()`) | `handlers.rs:152-161`, `handlers.rs:173-182` |
| `FM-CHAIN-006` | One log line is not valid JSON / doesn't parse as a `SyncReceipt` | `handlers.rs:168-170` |
| `FM-CHAIN-007`, "does not chain from genesis" | Record 0's `prev_chain_hash_hex` isn't 64 zero chars | `handlers.rs:184-191` |
| `FM-CHAIN-007`, "payload hash mismatch" | The record's stored `payload` doesn't BLAKE3-hash to its own `payload_hash_hex` | `handlers.rs:193-206` |
| `FM-CHAIN-007`, "chain hash mismatch" | `recompute_chain_hash()` (praxis-core) disagrees with the stored `chain_hash_hex` | `handlers.rs:207-224` |
| `FM-CHAIN-007`, "broken link" | Record `idx`'s `chain_hash_hex` doesn't equal record `idx+1`'s `prev_chain_hash_hex` | `handlers.rs:225-239` |

The message always names the zero-based `idx` of the first broken record —
verification stops at the first failure, so a report at index 3 tells you
nothing about indices 4+ yet.

## Step 2: Reproduce and read the exact failure

Real run, from a scratch project (`/tmp/ggen-chain-demo`, built with
`cargo build -p ggen` from `/Users/sac/praxis` per the tutorial above), after
hand-editing line 2 of a 3-line `.ggen-v2/receipt-log.jsonl` to zero out its
`payload_hash_hex`:

```
$ ggen receipt history
Error: Command execution failed: validation error: [FM-CHAIN-007] history invalid at index 1: payload hash mismatch (stored 0000000000000000000000000000000000000000000000000000000000000000, recomputed 75b92f4fb6713f465b3e194904a610b63e3ffc369a7e70143f785f311e524f1c)
```

Exit code is non-zero (`1`) — confirmed by running the same command with
`; echo "EXIT:$?"` afterward. Same project, this time with the middle line
deleted outright (simulating a truncated/lost append):

```
$ ggen receipt history
Error: Command execution failed: validation error: [FM-CHAIN-007] history invalid at index 0: broken link (record 0 chain_hash_hex 760455f0578b8e32cfeeb64eb7c772006c4da179681998ff5718fd92bb9401e3 != record 1 prev_chain_hash_hex c546787c1c42669fbd7ee5d15ceec73dcb543e524015196034d5a222855bb389)
```

Note the index shifts to 0 here: the check is against *adjacent* records in
the file as read, so removing a line makes the two survivors adjacent and
mismatched, even though neither survivor was itself tampered with.

## Step 3: Choose a remediation

**There is no automated repair.** Searching `handlers.rs`, `sync.rs`, and
`error.rs` in `crates/ggen/src/` for `repair`, `repair_chain`, or `rebuild`
returns nothing — the only chain-related functions in this codebase are
`handle_receipt_verify`, `handle_receipt_history` (detection only,
`handlers.rs:98-134` and `handlers.rs:149-248`), and `write_receipt`
(append-only extension, `/Users/sac/praxis/crates/ggen/src/sync.rs:414-503`).
Your two real options:

### Option A — restore the log from backup or git

If `.ggen-v2/receipt-log.jsonl` is tracked in git or you have a filesystem
backup, restore the last-known-good version and re-run `ggen receipt
history` to confirm:

```
$ git checkout -- .ggen-v2/receipt-log.jsonl   # if tracked
$ ggen receipt history
{
  "head_chain_hash": "58b08f45a7012b19508f11f3b624dbf7a84fd1579135afc3941911b868c06351",
  "records": 3,
  "valid": true
}
```

(Real output above is from restoring the backup copy taken before the
tampering in Step 2.) This is the only option that preserves history —
every prior sync's receipt stays provable.

### Option B — accept the break and start a new chain

If there's no good backup, delete the log and let the next `ggen sync run`
start a fresh genesis. **You must delete both `.ggen-v2/receipt-log.jsonl`
and `.ggen-v2/receipt.json`.** Deleting only the log is a real gotcha:
`write_receipt` reads its `prev_chain_hash_hex` from the surviving
`.ggen-v2/receipt.json`, not from the log (`sync.rs:437-459`), so a fresh
log's first entry inherits a non-zero `prev_chain_hash_hex` and immediately
fails the genesis check:

```
$ rm .ggen-v2/receipt-log.jsonl   # receipt.json NOT removed — wrong
$ ggen sync run && ggen receipt history
Error: Command execution failed: validation error: [FM-CHAIN-007] history invalid at index 0: first record does not chain from genesis (all-zeros prev_chain_hash_hex)
```

Removing both files first gives a clean genesis:

```
$ rm .ggen-v2/receipt.json .ggen-v2/receipt-log.jsonl   # correct
$ ggen sync run
$ ggen receipt history
{
  "head_chain_hash": "760455f0578b8e32cfeeb64eb7c772006c4da179681998ff5718fd92bb9401e3",
  "records": 1,
  "valid": true
}
```

Understand the cost: every receipt before the break is no longer provable
from the new chain. If that history mattered (audit, compliance, dispute
resolution), copy the broken log aside before deleting it rather than
discarding it outright.

## Gotchas

- **A missing log and an empty log both raise `FM-CHAIN-005`** — check
  which by hand (`ls -la .ggen-v2/`, `wc -l`) before assuming the log was
  never created; a genuinely empty file that only contains blank lines
  also trips this code (`handlers.rs:165-166` skips blank lines before the
  emptiness check).
- **`ggen receipt verify` only checks the single current
  `.ggen-v2/receipt.json`**, not the whole log (`handlers.rs:98-134`). A
  clean `receipt verify` does not mean `receipt history` is clean too —
  the log could still have a broken link further back.
- **Fixing a tampered record by re-deriving new hashes is not a supported
  path.** `recompute_chain_hash()` binds `payload_hash_hex` and
  `prev_chain_hash_hex` together (`sync.rs:476-479`); "fixing" a mismatch by
  hand-editing `chain_hash_hex` to match just produces a different,
  equally-detectable break at whichever check you didn't patch.
