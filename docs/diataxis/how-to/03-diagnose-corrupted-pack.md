# How to Diagnose a Corrupted Pack

A pack on disk no longer matches the hash recorded in `ggen.lock` — someone
hand-edited a file under a pack directory, a bad merge touched
`ontology.ttl`, or a template was dropped — and you need to find out which
pack, confirm it, and get back to a healthy state.

This guide assumes you already have a working `ggen.toml` + ontology +
pack project; if you don't, do
[Your First Pack](../tutorials/02-your-first-pack.md) first.

## The shortest correct recipe

### 1. Ask `ggen doctor run` first — it never mutates anything

`doctor` computes three independent checks — lockfile/pack drift, orphaned
generated artifacts, and receipt-vs-disk staleness — and ANDs them into one
`healthy` boolean, always computing all three so you see every drift at once
(`crates/ggen/src/verbs/handlers.rs:337-342`). Run it before touching
anything:

```
$ ggen doctor run
```

On a healthy project this returns (captured from a real scratch project,
see below):

```json
{
  "checks": {
    "lockfile_drift": {
      "detail": "ggen.lock matches resolved packs",
      "status": "pass"
    },
    "orphaned_artifacts": {
      "detail": "every receipt output is still produced by a template",
      "orphans": [],
      "status": "pass"
    },
    "receipt_staleness": {
      "detail": "every receipt output matches its recorded hash on disk",
      "stale": [],
      "status": "pass"
    }
  },
  "healthy": true
}
```

### 2. Reproduce the failure

To show the real refusal, a pack's `ontology.ttl` was corrupted in a scratch
project (`/tmp/ggen-howto-scratch`, a copy of
`packs/chicago-tdd-tools-pack` referenced from `ggen.toml` as pack `ctt`) by
appending one comment line — syntactically valid Turtle, so it is not a
parse error, only a content change:

```
$ echo "# corrupted by accident" >> packs/chicago-tdd-tools-pack/ontology.ttl
```

Now `ggen doctor run` fails closed on the first check, `lockfile_drift`:

```
$ ggen doctor run
Error: Command execution failed: doctor found 1 failing check(s): lockfile_drift: validation error: [FM-PACK-008] pack `ctt` (source `path:./packs/chicago-tdd-tools-pack`) content hash mismatch: ggen.lock has `blake3:0f3828f4f0616f52fefe09a91793e5a17eb7d8a396ca9dec6dba94d1cd3b6ee0` but the pack on disk hashes to `blake3:7b7e6651f94ae70135394b3c36cc9542f2ef9f26241b96dc70dd3cf873c2ce14`. Remediation: restore the pack, or delete ggen.lock to intentionally re-lock.
```

`ggen sync run` refuses identically and for the identical reason — `sync`
calls the same `check_lock` function before it reads a single pack ontology
into the graph (`crates/ggen/src/sync.rs:119-121`, calling
`crates/ggen/src/pack.rs:276`):

```
$ ggen sync run
Error: Command execution failed: validation error: [FM-PACK-008] pack `ctt` (source `path:./packs/chicago-tdd-tools-pack`) content hash mismatch: ggen.lock has `blake3:0f3828f4f0616f52fefe09a91793e5a17eb7d8a396ca9dec6dba94d1cd3b6ee0` but the pack on disk hashes to `blake3:7b7e6651f94ae70135394b3c36cc9542f2ef9f26241b96dc70dd3cf873c2ce14`. Remediation: restore the pack, or delete ggen.lock to intentionally re-lock.
```

The hash on each side comes from `content_hash` — a BLAKE3 hash over
`ontology.ttl` plus every template file, sorted by relative path
(`crates/ggen/src/pack.rs:169-202`) — so any byte changed anywhere under the
pack (ontology or template) trips this check, not just the ontology file
used in this recipe.

### 3. Fix it: restore the pack (the common case)

If the change was accidental, undo it — revert the file, `git checkout` it,
or restore from backup — until the pack's bytes match what was locked
again:

```
$ git checkout -- packs/chicago-tdd-tools-pack/ontology.ttl
$ ggen doctor run
```

```json
{
  "checks": {
    "lockfile_drift": {
      "detail": "ggen.lock matches resolved packs",
      "status": "pass"
    },
    "orphaned_artifacts": {
      "detail": "every receipt output is still produced by a template",
      "orphans": [],
      "status": "pass"
    },
    "receipt_staleness": {
      "detail": "every receipt output matches its recorded hash on disk",
      "stale": [],
      "status": "pass"
    }
  },
  "healthy": true
}
```

### 4. Or: the change was intentional — re-lock instead

If you *meant* to change the pack (you edited its ontology or added a
template on purpose), follow the error's own remediation text verbatim:
delete `ggen.lock` and re-sync. `write_lock` regenerates it deterministically
from whatever is on disk right now (`crates/ggen/src/pack.rs:326-329`):

```
$ rm ggen.lock
$ ggen sync run
```

```json
{
  "decisions": {
    "docs/chicago_tdd_tools_boundary.md": "skipped: unchanged: content identical",
    "tests/chicago_tdd_tools_boundary.rs": "skipped: unchanged: content identical"
  },
  "graph_hash_hex": "472fac5f5c287bc9104444655b3a61f836228ff592d09127f714817fb2ac6ed4",
  "packs": {
    "ctt": "9ba309fda3e806d2a9ddd11c1ad10d59942cfbceee52ecbbd2bb8462bfe2cc75"
  },
  "skipped": [
    ["docs/chicago_tdd_tools_boundary.md", "unchanged: content identical"],
    ["tests/chicago_tdd_tools_boundary.rs", "unchanged: content identical"]
  ],
  "written": []
}
```

`ggen.lock` now records the new hash and the next `ggen doctor run` is
healthy again — because there is no longer a mismatch to detect, not
because the check was bypassed.

## Variations and gotchas

- **A missing `ggen.lock` is not an error.** `check_lock` returns `Ok(())`
  immediately if the lockfile doesn't exist yet — first sync always
  succeeds and produces the initial lock (`crates/ggen/src/pack.rs:277-280`).
  Deleting the lock is therefore a safe, first-class remediation, not a
  workaround.
- **A pack absent from the lock is also fine.** `check_lock` only compares
  entries that already have an entry in `doc.packs`
  (`crates/ggen/src/pack.rs:294-295`) — a newly added pack gets locked on
  its first successful sync, it doesn't fail closed on "not yet locked."
- **A malformed `ggen.lock` fails differently — `FM-PACK-009`, not
  `FM-PACK-008`.** If the lockfile itself doesn't parse as TOML or has
  unknown keys, you get "ggen.lock malformed... Remediation: fix or delete
  the lockfile" (`crates/ggen/src/pack.rs:284-293`) — same remediation
  (delete and re-sync) but a different code, so don't pattern-match only on
  `008` when scripting around this.
- **If you corrupt the ontology with genuinely invalid Turtle** (not just a
  changed but valid file), you won't see `FM-PACK-008` at all — the parser
  fails first, deeper into `sync` where the pack's ontology is loaded into
  the graph (`crates/ggen/src/sync.rs:122-134`), and you'll get an
  `FM-GRAPH-002` "turtle load failed" parser error instead. `doctor run`
  itself never reaches that stage, so `doctor` only ever reports
  `FM-PACK-008`/`FM-PACK-009` for pack problems — a syntactically-invalid
  pack ontology surfaces only via `ggen sync run`.
- **`doctor run` reports every failing check, not just the first.** The
  three checks (`lockfile_drift`, `orphaned_artifacts`, `receipt_staleness`)
  are always all computed and ANDed (`crates/ggen/src/verbs/handlers.rs:341,
  458`) — if a pack is corrupted *and* a receipt output is stale, both show
  up in the one error message, so read the whole `failing` list before
  fixing just the first line.
