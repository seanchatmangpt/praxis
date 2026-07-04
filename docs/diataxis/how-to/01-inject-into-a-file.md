# How to Inject Into an Existing File

You have a hand-written file — a `mod.rs`, a router table, a registry list —
and you want `ggen` to append a generated line into it next to a marker
comment, without touching the rest of the file, and without duplicating the
line on the next `ggen sync run`.

This guide assumes you already have a working `ggen.toml` + ontology +
template project; if you don't, do
[Your First Sync](../tutorials/01-your-first-sync.md) first.

## The shortest correct recipe

### 1. Put a marker comment in the target file

The file must already exist — inject never creates it
(`crates/ggen/src/write.rs:79-92`, error `FM-WRITE-003` below). Put a stable
comment where you want generated content to land:

```rust
// existing module file, hand-written
// GGEN:INJECT-MODULES
pub mod existing;
```

(`/tmp/ggen-howto-inject/scratch/src/mod.rs`, written by hand before any
`ggen` run.)

### 2. Write a template with `inject`, `after`, and `skip_if`

```
---
to: src/mod.rs
inject: true
after: "// GGEN:INJECT-MODULES"
skip_if: "pub mod generated;"
sparql:
  people: SELECT ?name WHERE { ?s <http://example.org/name> ?name } ORDER BY ?name
---
pub mod generated; // {{ people | length }} people in graph
```

(`/tmp/ggen-howto-inject/scratch/templates/inject_module.tmpl`)

Three frontmatter fields do the work, all defined on `Frontmatter` in
`crates/ggen/src/template.rs:27-63`:

- `inject: true` (`crates/ggen/src/template.rs:37-38`) — write into the
  existing file at `to:` instead of creating/overwriting it.
- `after: "..."` (`crates/ggen/src/template.rs:42-44`) — insert the rendered
  body on the line right after the first line containing this substring.
  (`before:` and `at_line:` are the other two positioning fields,
  `crates/ggen/src/template.rs:39-41` and `:45-47` — pick one.)
- `skip_if: "..."` (`crates/ggen/src/template.rs:48-50`) — if the *existing*
  file already contains this substring, skip the write entirely. This is
  what makes injection idempotent: put a substring in `skip_if` that only
  exists once your generated line has already landed (here, the literal
  `pub mod generated;` your own body emits).

The decision order that makes this safe is documented in
`crates/ggen/src/write.rs:9-16`: `skip_if` is checked *before* `inject` runs,
so a second sync short-circuits to `Skipped` instead of re-inserting.

### 3. Run sync the first time — it injects

```bash
$ /path/to/checkout/target/debug/ggen sync run
```

Real output:

```json
{
  "decisions": {
    "src/mod.rs": "injected"
  },
  "graph_hash_hex": "fa993868c3aca3f5354201c0fc488b0682cc89735ec9b10dbebe2288b24bbb0b",
  "packs": {},
  "skipped": [],
  "written": [
    "src/mod.rs"
  ]
}
```

File before:

```
// existing module file, hand-written
// GGEN:INJECT-MODULES
pub mod existing;
```

File after (real content read back from disk):

```
// existing module file, hand-written
// GGEN:INJECT-MODULES
pub mod generated; // 2 people in graph
pub mod existing;
```

The rendered body landed on the line immediately after the marker line
(`inject_into`, `crates/ggen/src/write.rs:189-219`: an `after:` marker
resolves to `find_marker_line(...) + 1`, `crates/ggen/src/write.rs:196`),
pushing `pub mod existing;` down by one line. `pub mod existing;` itself
was never touched.

### 4. Run sync again — it's a no-op

```bash
$ /path/to/checkout/target/debug/ggen sync run
```

Real output:

```json
{
  "decisions": {
    "src/mod.rs": "skipped: skip_if: existing file already contains \"pub mod generated;\""
  },
  "graph_hash_hex": "fa993868c3aca3f5354201c0fc488b0682cc89735ec9b10dbebe2288b24bbb0b",
  "packs": {},
  "skipped": [
    [
      "src/mod.rs",
      "skip_if: existing file already contains \"pub mod generated;\""
    ]
  ],
  "written": []
}
```

`src/mod.rs` on disk is byte-identical to step 3's output — no duplicate
`pub mod generated;` line. That's the whole recipe: marker comment +
`inject: true` + `after:` + a `skip_if:` substring that only appears once
the injected line is already there.

## Gotchas

- **`FM-WRITE-003` — target file doesn't exist.** Inject only ever modifies
  an existing file (`crates/ggen/src/write.rs:80-89`); it will not create
  `to:` for you. Real error from a run where `src/nope.rs` was never
  created:

  ```
  Error: Command execution failed: validation error: [FM-WRITE-003] inject
  target /private/tmp/.../src/nope.rs does not exist. Remediation: create
  the file first or drop `inject: true`.
  ```

  Fix: create the target file (by hand, or with a prior non-inject
  template run) before the injecting template runs.

- **`FM-WRITE-004` — marker not found.** If `after:` (or `before:`)
  doesn't match any line in the target file, the write fails closed rather
  than silently appending to the end (`find_marker_line`,
  `crates/ggen/src/write.rs:222-235`). Real error from a run where the
  marker comment was missing from the target file:

  ```
  Error: Command execution failed: validation error: [FM-WRITE-004] inject
  `after:` marker "// GGEN:INJECT-MODULES" not found in target file.
  Remediation: add the marker line or fix the frontmatter.
  ```

  Fix: add the exact marker substring to the target file, or correct the
  typo in `after:`/`before:`.

- **No `skip_if` means no idempotency.** Without `skip_if`, a second sync
  re-runs the same `after:`/`before:`/`at_line:` insertion and duplicates
  the injected line every time — `inject_into` has no memory of what it
  already did (`crates/ggen/src/write.rs:189-219` unconditionally splices).
  Always pair `inject: true` with a `skip_if:` substring drawn from your
  own rendered body.

- **Omitting `before`/`after`/`at_line` entirely appends to end-of-file**
  (`crates/ggen/src/write.rs:209-211`, the final `else` arm) — useful for
  simple append-only lists, but combine it with `skip_if` for the same
  reason as above.

- **`at_line` is 1-based** and errors (`FM-WRITE-004`) if it's `0` or more
  than one past the last line (`crates/ggen/src/write.rs:197-207`) — it's
  the position *before* which the body is inserted, so `at_line: 1` puts
  your content before the current first line.
