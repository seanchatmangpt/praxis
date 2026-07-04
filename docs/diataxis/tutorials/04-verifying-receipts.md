# Verifying a Receipt Chain

In this tutorial you build a tiny ontology-driven project, run `ggen sync` three
times with small edits in between, and inspect the cryptographic receipt chain
that each sync appends to. Then you tamper with one line of the receipt log
by hand and watch `ggen receipt history` refuse it by name.

You will need a working `ggen` binary. If you haven't built one yet, run this
from the `praxis` repository root:

```bash
cargo build -p ggen --bin ggen
```

The binary will be at `target/debug/ggen`. The rest of this tutorial calls it
by that path; assign it to a shell variable so the commands stay short:

```bash
GGEN=/absolute/path/to/praxis/target/debug/ggen
```

## Step 1: Create a scratch project

Work outside the `praxis` repository so you never touch real project files.
Create a fresh directory and move into it:

```bash
mkdir -p /tmp/ggen-tutorial-04/templates
cd /tmp/ggen-tutorial-04
```

## Step 2: Write a minimal `ggen.toml`

Create the project's configuration file:

```bash
cat > ggen.toml << 'EOF'
[project]
name = "demo"

[ontology]
source = "ontology.ttl"

[templates]
dir = "templates"
EOF
```

## Step 3: Write the first version of the ontology

Create `ontology.ttl` with a single fact:

```bash
cat > ontology.ttl << 'EOF'
@prefix ex: <http://example.org/> .
ex:alice ex:name "alice" .
EOF
```

## Step 4: Write a template

Create a template that queries every `ex:name` and lists them, one per line:

```bash
cat > templates/one.tmpl << 'EOF'
---
to: out/names.txt
force: true
sparql:
  people: SELECT ?name WHERE { ?s <http://example.org/name> ?name } ORDER BY ?name
---
{% for row in results %}{{ row.name }}
{% endfor %}
EOF
```

## Step 5: Run the first sync

```bash
$GGEN sync run
```

You will see JSON output naming the file that was written:

```json
{
  "decisions": {
    "out/names.txt": "written"
  },
  "graph_hash_hex": "93d2b7656a65080a995cb0aba7f94092d4e2caa27d403c317a1e679847836b07",
  "packs": {},
  "skipped": [],
  "written": [
    "out/names.txt"
  ]
}
```

Check the generated file:

```bash
cat out/names.txt
```

```
alice
```

Every real (non-dry-run) sync also writes two files under `.ggen-v2/`:
`receipt.json` (the latest receipt) and `receipt-log.jsonl` (every receipt
ever produced, one JSON object per line). This is implemented in
`handle_sync_run` at `crates/ggen/src/verbs/handlers.rs:36`, which calls
`sync(&root, SyncOptions { dry_run })` — the pipeline that appends to the
receipt log on every non-dry-run invocation.

## Step 6: Edit the ontology and sync again

Add a second fact:

```bash
cat > ontology.ttl << 'EOF'
@prefix ex: <http://example.org/> .
ex:alice ex:name "alice" .
ex:bob ex:name "bob" .
EOF
```

Sync again:

```bash
$GGEN sync run
```

```json
{
  "decisions": {
    "out/names.txt": "written"
  },
  "graph_hash_hex": "e1366487492a100f7fa3c5e6ae7913ecb11f542f93e1fa537a7aba2814fcb9d7",
  "packs": {},
  "skipped": [],
  "written": [
    "out/names.txt"
  ]
}
```

```bash
cat out/names.txt
```

```
alice
bob
```

Notice the `graph_hash_hex` changed — the ontology content is different, so
its hash is different.

## Step 7: Edit the ontology a third time and sync again

```bash
cat > ontology.ttl << 'EOF'
@prefix ex: <http://example.org/> .
ex:alice ex:name "alice" .
ex:bob ex:name "bob" .
ex:carol ex:name "carol" .
EOF
$GGEN sync run
```

```json
{
  "decisions": {
    "out/names.txt": "written"
  },
  "graph_hash_hex": "78da961e2480c4fd14a0528beae1e419f507e85a0e0d0dc138469089b394808d",
  "packs": {},
  "skipped": [],
  "written": [
    "out/names.txt"
  ]
}
```

```bash
cat out/names.txt
```

```
alice
bob
carol
```

You have now run three syncs, so `.ggen-v2/receipt-log.jsonl` holds three
receipt records, each one chained to the previous.

## Step 8: Verify the latest receipt

Run:

```bash
$GGEN receipt verify
```

```json
{
  "chain_hash": "5809cb08ee37c5bf79c3229e51d5fad9d1b8732a6770fac3fc30d7502f0e5504",
  "graph_hash": "78da961e2480c4fd14a0528beae1e419f507e85a0e0d0dc138469089b394808d",
  "outputs": 1,
  "payload_hash": "b2e7a7d04bb18a0e58db4b5e98c951bfee96c8228f59ee857ff5185f478ed887",
  "valid": true
}
```

This command reads `.ggen-v2/receipt.json`, recomputes the payload hash and
the chain hash, and reports `"valid": true` only if both recomputed values
match what is stored on disk. This is `handle_receipt_verify` at
`crates/ggen/src/verbs/handlers.rs:98`: it hashes the stored `payload` with
BLAKE3 and compares it against `record.payload_hash_hex`
(`crates/ggen/src/verbs/handlers.rs:108-115`), then calls
`receipt.record.recompute_chain_hash()` and compares that against
`record.chain_hash_hex` (`crates/ggen/src/verbs/handlers.rs:118-125`).

## Step 9: Verify the whole chain with `receipt history`

Run:

```bash
$GGEN receipt history
```

```json
{
  "head_chain_hash": "5809cb08ee37c5bf79c3229e51d5fad9d1b8732a6770fac3fc30d7502f0e5504",
  "records": 3,
  "valid": true
}
```

`records: 3` confirms all three of your syncs are present in the log.
`handle_receipt_history` (`crates/ggen/src/verbs/handlers.rs:149`) reads every
line of `.ggen-v2/receipt-log.jsonl`, and for each record checks three things
in order: the payload hash (`crates/ggen/src/verbs/handlers.rs:195-206`), the
chain hash recompute (`crates/ggen/src/verbs/handlers.rs:208-224`), and that
each record's `chain_hash_hex` equals the *next* record's
`prev_chain_hash_hex` (`crates/ggen/src/verbs/handlers.rs:226-239`) — the
adjacency check that makes this a chain, not just a list. It also requires
the very first record's `prev_chain_hash_hex` to be 64 zeros, the genesis
value (`crates/ggen/src/verbs/handlers.rs:184-191`).

## Step 10: Tamper with the middle receipt

Now break the chain on purpose. Open `.ggen-v2/receipt-log.jsonl` and look at
line 2 (the second sync's receipt — index 1, zero-based):

```bash
sed -n '2p' .ggen-v2/receipt-log.jsonl
```

Change the `payload.graph_hash` field inside that line to 64 `f` characters,
without touching any of the `record.*` hash fields, then write the file back:

```bash
python3 -c "
import json
with open('.ggen-v2/receipt-log.jsonl') as f:
    lines = [l for l in f.read().splitlines() if l.strip()]
mid = json.loads(lines[1])
mid['payload']['graph_hash'] = 'f' * 64
lines[1] = json.dumps(mid)
with open('.ggen-v2/receipt-log.jsonl', 'w') as f:
    f.write('\n'.join(lines) + '\n')
"
```

This simulates someone editing the payload without recomputing its hash —
exactly the kind of tamper the chain exists to catch.

## Step 11: Run `receipt history` again and watch it fail closed

```bash
$GGEN receipt history
```

```
Error: Command execution failed: validation error: [FM-CHAIN-007] history invalid at index 1: payload hash mismatch (stored 00670f128a9ad5335a314ed56b999c14fc7ac50a095418b7b6ba4c7db1683aad, recomputed 033c6ce6c29f563e35300215779a4860ee1e9d98ba3dc5faa1aed3a9abf2240d)
```

Check the exit code:

```bash
echo $?
```

```
1
```

The command names the exact broken record (`index 1` — the receipt you
edited), the exact failed check (`payload hash mismatch`), and both the
stored and recomputed hashes, so you can see precisely how they diverge. It
never reports a soft `"valid": false`; a broken chain is a hard, non-zero
exit. This is the `[FM-CHAIN-007]` branch of the payload-hash check at
`crates/ggen/src/verbs/handlers.rs:197-206`, constructed via
`AppError::fm_chain(7, ...)` (`crates/ggen/src/error.rs:94-96`, which formats
the code as `[FM-CHAIN-{code:03}]`). Had you instead deleted the whole log
file, or left it empty, you would see `[FM-CHAIN-005]` instead — the
"log unreadable" and "log empty" branches at
`crates/ggen/src/verbs/handlers.rs:152-161` and
`crates/ggen/src/verbs/handlers.rs:173-182`.

## What you built

You built a scratch ontology-driven project, ran three syncs that each
extended a genesis-rooted BLAKE3 receipt chain in `.ggen-v2/receipt-log.jsonl`,
confirmed the chain with `ggen receipt verify` and `ggen receipt history`, and
then proved the chain's tamper-evidence by hand-editing one receipt and
watching `ggen receipt history` name the exact broken index and fail with a
non-zero exit.

For the day-to-day version of this workflow — checking receipts as part of
your normal edit/sync loop rather than as a from-scratch exercise — see the
How-To guide on verifying a project's sync receipts.
