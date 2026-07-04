# Receipts

Every non-dry-run `sync` produces a receipt: a chained, tamper-evident record of what the pipeline did. This chapter reads the receipt type, the chaining function it reuses from `praxis-core`, and the three commands that check it (`receipt verify`, `receipt history`, `doctor run`), then works through a real example — two syncs, a verify, a tamper, and the refusal — with pasted command output.

## What gets hashed

`sync` (`crates/ggen/src/sync.rs:100-242`) runs the five-stage pipeline (Resolve → Enrich → Extract → Render → Write) and, when `opts.dry_run` is `false`, calls `write_receipt` at the end (`crates/ggen/src/sync.rs:235-239`).

`write_receipt` (`crates/ggen/src/sync.rs:414-503`) builds a `ReceiptPayload`:

```rust
pub struct ReceiptPayload {
    pub graph_hash: String,
    pub outputs: BTreeMap<String, String>,
    pub packs: BTreeMap<String, String>,
    pub decisions: BTreeMap<String, String>,
}
```
(`crates/ggen/src/sync.rs:79-91`)

- `graph_hash` is the BLAKE3 hex of the post-Enrich canonical graph state (`crates/ggen/src/sync.rs:145`).
- `outputs` is filled by re-reading, from disk, every path that has a decision entry — i.e. every file written this run or found unchanged — and BLAKE3-hashing its bytes (`crates/ggen/src/sync.rs:419-425`). Because this reads the *actual bytes on disk* rather than the in-memory rendered string, a no-op re-sync (nothing changed, nothing rewritten) still produces the identical payload, which matters for the chain below.
- `packs` is the pack name → content-hash map from the resolved lockfile entries (`crates/ggen/src/sync.rs:223-231`).
- `decisions` is the root-relative path → outcome string (`"written"`, `"injected"`, or `"skipped: <reason>"`) collected during the Write stage (`crates/ggen/src/sync.rs:152, 161-164, 283, 295, 299, 307, 311, 315`).

The payload is serialized to canonical JSON via `serde_json::to_vec` and BLAKE3-hashed to get `payload_hash_hex` (`crates/ggen/src/sync.rs:432-433`).

## Chaining onto praxis-core

`sync.rs` does not invent its own chain-hash construction. It reuses `praxis_core::receipt_record::ReceiptRecord` (`crates/ggen/src/sync.rs:28`), whose fields are documented at `crates/praxis-core/src/receipt_record.rs:27-69`. The on-disk `SyncReceipt` wraps this record plus the payload:

```rust
pub struct SyncReceipt {
    pub record: ReceiptRecord,
    pub payload: ReceiptPayload,
}
```
(`crates/ggen/src/sync.rs:70-76`)

`write_receipt` reads the *previous* `.ggen-v2/receipt.json`, if any, to get `prev_chain_hash_hex`:

- If a prior receipt exists and parses, `prev_chain_hash_hex` is that receipt's `record.chain_hash_hex` (`crates/ggen/src/sync.rs:438-451`).
- If no prior receipt exists (`ErrorKind::NotFound`), the chain starts from genesis: 64 `'0'` characters (`crates/ggen/src/sync.rs:452`).
- Any other read error, or a parse failure on an existing file, is `[FM-CHAIN-003]` and aborts the sync (`crates/ggen/src/sync.rs:441-458`) — a malformed previous receipt is never silently overwritten.

It then constructs a `ReceiptRecord` directly (`crates/ggen/src/sync.rs:461-475`):

```rust
let mut record = ReceiptRecord {
    version: RECEIPT_RECORD_VERSION,
    instruction_id: 0,
    activity_idx: 0,
    activity: Some("ggen.sync".to_string()),
    node_kind: 0,
    ts_ns: 0,
    duration_ms: None,
    object_ids: vec![format!("law:{}", &payload_hash_hex[..16])],
    payload_hash_hex,
    prev_chain_hash_hex,
    chain_hash_hex: String::new(),
    andon: Andon::Green,
    obligation_count: 0,
};
```

`ts_ns` is fixed to `0` rather than the wall clock, because this crate forbids wall-clock reads and praxis-core's live emission path only falls back to the clock when `ts_ns` is `None` (`crates/ggen/src/sync.rs:15-21`).

`chain_hash_hex` is filled in by calling `record.recompute_chain_hash()` (`crates/ggen/src/sync.rs:476-479`), which is defined in `praxis-core`:

```rust
pub fn recompute_chain_hash(&self) -> Result<[u8; 32], CoreError> {
    let payload_hash = self.payload_hash()?;
    let prev_chain_hash = self.prev_chain_hash()?;
    let meta = self.receipt_meta();
    let frame = build_admission_frame(&payload_hash, &prev_chain_hash, &meta, self.ts_ns);
    Ok(chain_from_frame(&prev_chain_hash, &frame))
}
```
(`crates/praxis-core/src/receipt_record.rs:121-127`)

The doc comment on this function is explicit about why it is safe to call twice, once at write time and again at verify time: it uses "the exact same `build_admission_frame`/`chain_from_frame` construction... `LawObject::receipt`/`receipt_with_record` use at emission time — so this can never silently diverge from the live emission path" (`crates/praxis-core/src/receipt_record.rs:114-120`). `payload_hash()` and `prev_chain_hash()` decode the two hex fields into raw 32-byte arrays via `decode_hex32`, which is a hard error (`CoreError::HexDecodeFailed`) on non-hex input or on any length other than 32 bytes (`crates/praxis-core/src/receipt_record.rs:71-93`) — this is what a `recompute_chain_hash()` test in the same file exercises directly (`crates/praxis-core/src/receipt_record.rs:170-182`).

## Writing both files

`write_receipt` writes two things (`crates/ggen/src/sync.rs:481-501`):

1. `<root>/.ggen-v2/receipt.json` (`RECEIPT_REL_PATH`, `crates/ggen/src/sync.rs:63`) — the *current* head, overwritten each sync, pretty-printed.
2. `<root>/.ggen-v2/receipt-log.jsonl` (`RECEIPT_LOG_REL_PATH`, `crates/ggen/src/sync.rs:67`) — append-only, one compact JSON line per non-dry-run sync, opened with `.create(true).append(true)` (`crates/ggen/src/sync.rs:488-495`). A failed append is `[FM-CHAIN-004]` (`crates/ggen/src/sync.rs:496-501`).

`receipt.json` is always exactly the *head* of the chain — the same object as the last line of `receipt-log.jsonl` — because both are written from the same in-memory `receipt` value in the same call (`crates/ggen/src/sync.rs:481, 485, 489`).

## The three checks

### `receipt verify`

`handle_receipt_verify` (`crates/ggen/src/verbs/handlers.rs:98-134`) reads only `receipt.json` — the current head — and performs two checks:

1. **Payload binding**: re-serialize `receipt.payload` to canonical JSON, BLAKE3-hash it, and compare against `receipt.record.payload_hash_hex`. A mismatch is a hard error naming both the stored and recomputed hash (`crates/ggen/src/verbs/handlers.rs:107-115`).
2. **Chain integrity**: call `receipt.record.recompute_chain_hash()` and compare against `receipt.record.chain_hash_hex` (`crates/ggen/src/verbs/handlers.rs:117-125`).

Note there is no separate `[FM-CHAIN-*]` code wrapping these two failures in `handle_receipt_verify` — they're `NounVerbError::execution_error` strings built directly (`crates/ggen/src/verbs/handlers.rs:23-25, 111, 121`). The doc comment is explicit that failure here always means non-zero exit, "never a cheerful `valid: false`" (`crates/ggen/src/verbs/handlers.rs:94-97`).

### `receipt history`

`handle_receipt_history` (`crates/ggen/src/verbs/handlers.rs:149-248`) reads the *entire* `receipt-log.jsonl` and checks every record, not just the head. Per its doc comment (`crates/ggen/src/verbs/handlers.rs:136-148`), for every logged receipt:

1. the stored payload must hash to that record's `payload_hash_hex`,
2. the record's `chain_hash_hex` must match a praxis-core recompute, and
3. each record's `chain_hash_hex` must equal the *next* record's `prev_chain_hash_hex`.

The first record must chain from genesis (all-zeros).

Concretely, the checks and their exact `[FM-CHAIN-*]` codes:

| Code | Where | Trigger |
|---|---|---|
| `FM-CHAIN-005` | `handlers.rs:152-161`, `173-181` | `receipt-log.jsonl` unreadable, or present but empty (0 valid lines) |
| `FM-CHAIN-006` | `handlers.rs:168-170` | one JSONL line fails to deserialize as a `SyncReceipt` |
| `FM-CHAIN-007` (genesis) | `handlers.rs:186-190` | index 0's `prev_chain_hash_hex` is not 64 `'0'` characters |
| `FM-CHAIN-007` (payload) | `handlers.rs:198-206` | some index's stored payload doesn't hash to its own `payload_hash_hex` |
| `FM-CHAIN-007` (chain recompute) | `handlers.rs:208-224` | some index's `recompute_chain_hash()` doesn't match its own `chain_hash_hex` |
| `FM-CHAIN-007` (broken link) | `handlers.rs:226-238` | index *i*'s `chain_hash_hex` doesn't equal index *i+1*'s `prev_chain_hash_hex` |

All four "index-scoped" failures share code `007` but differ in message text (`"history invalid at index {idx}: ..."`), naming the specific check that failed and both the stored and recomputed values where applicable. Every branch fails closed: there is no partial-success return, and the function returns after the *first* index where any of the three checks fails (`crates/ggen/src/verbs/handlers.rs:193-240`).

Separately, `write_receipt` itself uses `FM-CHAIN-002` for a chain-hash *computation* failure at write time (`crates/ggen/src/sync.rs:476-478`) and `FM-CHAIN-003` for an unreadable or malformed *previous* receipt (`crates/ggen/src/sync.rs:441-458`) — both are sync-time, not verify-time, failures.

### `doctor run`

`handle_doctor` (`crates/ggen/src/verbs/handlers.rs:353-493`) is a broader, non-invasive health check that uses the receipt as one of three independent inputs. Unlike `receipt verify`/`receipt history`, an absent receipt is not an error here — "a project that has never been synced... is healthy by definition for the artifact/staleness checks" (`crates/ggen/src/verbs/handlers.rs:344-346`). The three checks, always all computed and ANDed into `healthy` (`crates/ggen/src/verbs/handlers.rs:458`):

1. **`lockfile_drift`** — `crate::pack::check_lock` against the resolved packs (`crates/ggen/src/verbs/handlers.rs:360-371`).
2. **`orphaned_artifacts`** — every path in `receipt.payload.outputs` must still be producible by some template today, matched either exactly (static `to:`) or via literal-chunk matching for templated (`{{ }}`) targets (`crates/ggen/src/verbs/handlers.rs:250-335`, `393-397`).
3. **`receipt_staleness`** — every `receipt.payload.outputs` entry is re-hashed from disk; a missing file or hash mismatch is reported per-path (`crates/ggen/src/verbs/handlers.rs:401-455`).

`doctor` does not call `recompute_chain_hash()` at all — it only trusts the recorded output hashes for the staleness check, and does not itself verify the chain. Running `receipt verify`/`receipt history` first is what proves the receipt itself hasn't been tampered with; `doctor` then asks whether the *filesystem* still matches what that (trusted) receipt claims.

## Worked example

The commands below were run against `ggen` built at `/Users/sac/praxis/target/debug/ggen`, in a fresh temp project (`/tmp/ggen-receipt-demo`) with this `ggen.toml`:

```toml
[project]
name = "receipt-demo"

[ontology]
source = "ontology/domain.ttl"

[templates]
dir = "templates"
```

`ontology/domain.ttl`:

```turtle
@prefix ex: <http://example.org/> .
ex:Widget a ex:Thing ;
  ex:name "Sprocket" .
```

`templates/hello.tmpl`:

```
---
to: "out/hello.txt"
sparql:
  things: "SELECT ?name WHERE { ?s <http://example.org/name> ?name }"
---
Hello, {{ things.0.name }}!
```

### First sync

```
$ ggen --format json-pretty sync run
{
  "decisions": {
    "out/hello.txt": "written"
  },
  "graph_hash_hex": "94c8778ec9a4919025594b8a5242d3eee04e4fcb91f86a8587846666d0e89638",
  "packs": {},
  "skipped": [],
  "written": [
    "out/hello.txt"
  ]
}
```

`.ggen-v2/receipt.json` after this run:

```json
{
  "record": {
    "version": 1,
    "instruction_id": 0,
    "activity_idx": 0,
    "activity": "ggen.sync",
    "node_kind": 0,
    "ts_ns": 0,
    "payload_hash_hex": "adbbd0b440806b2609cae60332f92c95c7de2b29066ad02f6f30e372add31ff0",
    "prev_chain_hash_hex": "0000000000000000000000000000000000000000000000000000000000000000",
    "chain_hash_hex": "698d78bf45d83155dce00af09a6152e0153efe04f49122a0297dcfb625611ff4",
    "andon": "Green",
    "obligation_count": 0,
    "object_ids": [
      "law:adbbd0b440806b26"
    ]
  },
  "payload": {
    "graph_hash": "94c8778ec9a4919025594b8a5242d3eee04e4fcb91f86a8587846666d0e89638",
    "outputs": {
      "out/hello.txt": "f4b87997f8c4541475f6252e486a5369bac9e595e67a908bc626770b0b2f7f94"
    },
    "packs": {},
    "decisions": {
      "out/hello.txt": "written"
    }
  }
}
```

Note `prev_chain_hash_hex` is genesis (all zeros), exactly as `crates/ggen/src/sync.rs:452` computes for a first-ever sync.

### Second sync (no-op)

```
$ ggen --format json-pretty sync run
{
  "decisions": {
    "out/hello.txt": "skipped: unchanged: content identical"
  },
  "graph_hash_hex": "94c8778ec9a4919025594b8a5242d3eee04e4fcb91f86a8587846666d0e89638",
  "packs": {},
  "skipped": [
    [
      "out/hello.txt",
      "unchanged: content identical"
    ]
  ],
  "written": []
}
```

`.ggen-v2/receipt.json` is now the second record:

```json
{
  "record": {
    "version": 1,
    "instruction_id": 0,
    "activity_idx": 0,
    "activity": "ggen.sync",
    "node_kind": 0,
    "ts_ns": 0,
    "payload_hash_hex": "8ce785a147acbfb4ffbd2954220c4f3239de7d4670856de5672900bb6ee54e86",
    "prev_chain_hash_hex": "698d78bf45d83155dce00af09a6152e0153efe04f49122a0297dcfb625611ff4",
    "chain_hash_hex": "da2683d9de6e448d961833e5143b04e23865dd9fc12e7f8bbf8cb5e127ffe2fc",
    "andon": "Green",
    "obligation_count": 0,
    "object_ids": [
      "law:8ce785a147acbfb4"
    ]
  },
  "payload": {
    "graph_hash": "94c8778ec9a4919025594b8a5242d3eee04e4fcb91f86a8587846666d0e89638",
    "outputs": {
      "out/hello.txt": "f4b87997f8c4541475f6252e486a5369bac9e595e67a908bc626770b0b2f7f94"
    },
    "packs": {},
    "decisions": {
      "out/hello.txt": "skipped: unchanged: content identical"
    }
  }
}
```

`prev_chain_hash_hex` here is exactly the first record's `chain_hash_hex` (`698d78bf...11ff4`) — the link `write_receipt` reads at `crates/ggen/src/sync.rs:450`. Note that even though nothing was written this run (the file's content was unchanged), the payload still differs from the first sync's (`decisions` records `"skipped: unchanged..."` instead of `"written"`), so `payload_hash_hex` and `chain_hash_hex` are both new values — the receipt records the *decision made*, not merely a written/not-written boolean.

`.ggen-v2/receipt-log.jsonl` now holds two lines, one per sync:

```
$ wc -l .ggen-v2/receipt-log.jsonl
       2 .ggen-v2/receipt-log.jsonl
```

### Verifying the untampered chain

```
$ ggen --format json-pretty receipt verify
{
  "chain_hash": "da2683d9de6e448d961833e5143b04e23865dd9fc12e7f8bbf8cb5e127ffe2fc",
  "graph_hash": "94c8778ec9a4919025594b8a5242d3eee04e4fcb91f86a8587846666d0e89638",
  "outputs": 1,
  "payload_hash": "8ce785a147acbfb4ffbd2954220c4f3239de7d4670856de5672900bb6ee54e86",
  "valid": true
}

$ ggen --format json-pretty receipt history
{
  "head_chain_hash": "da2683d9de6e448d961833e5143b04e23865dd9fc12e7f8bbf8cb5e127ffe2fc",
  "records": 2,
  "valid": true
}
```

`head_chain_hash` matches the head `receipt.json`'s `chain_hash_hex`, and `records: 2` confirms both log lines were checked.

### Tampering `receipt.json`

Editing `receipt.json` in place — changing the recorded output hash for `out/hello.txt` to `ffff...` without updating `payload_hash_hex` or `chain_hash_hex` — and re-running `receipt verify`:

```
$ ggen receipt verify
Error: Command execution failed: receipt invalid: payload hash mismatch (stored 8ce785a147acbfb4ffbd2954220c4f3239de7d4670856de5672900bb6ee54e86, recomputed 8f5f3875fd646147616489ce14d1feed9f5350cba493abca5dd8c7e480afd7c9)
```

(exit status 1.)

This is the payload-binding check at `crates/ggen/src/verbs/handlers.rs:107-115`: the tampered `outputs` map re-hashes to a different value (`8f5f3875...`) than what's stored in `record.payload_hash_hex` (`8ce785a1...`), so verification fails before the chain-hash check even runs.

### Tampering `receipt-log.jsonl`

Making the identical edit to the second line of `receipt-log.jsonl` (the same field, `out/hello.txt`'s recorded hash, set to `ffff...`) and re-running `receipt history`:

```
$ ggen receipt history
Error: Command execution failed: validation error: [FM-CHAIN-007] history invalid at index 1: payload hash mismatch (stored 8ce785a147acbfb4ffbd2954220c4f3239de7d4670856de5672900bb6ee54e86, recomputed 8f5f3875fd646147616489ce14d1feed9f5350cba493abca5dd8c7e480afd7c9)
```

This is the per-index payload check at `crates/ggen/src/verbs/handlers.rs:194-206`, firing for index `1` (the second, zero-based, log line) with the exact `[FM-CHAIN-007]` code from `AppError::fm_chain(7, ...)`. The recomputed hash (`8f5f3875...`) is identical to the one `receipt verify` produced above, because both commands hash the same tampered payload bytes the same way.

In both cases the failure is a hard, non-zero-exit error — never a `"valid": false` JSON body — matching the fail-closed convention documented at `crates/ggen/src/verbs/handlers.rs:94-97` and `crates/ggen/src/verbs/handlers.rs:145-148`.
