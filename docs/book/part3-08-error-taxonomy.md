# Error Taxonomy

All errors in `ggen` funnel through a single type, `AppError`, defined in
`crates/ggen/src/error.rs:8-26`. There is no per-module error enum and no
`anyhow`-style opaque error box in this crate — every fallible function
returns the crate-local alias `pub type Result<T> = std::result::Result<T, AppError>`
(`crates/ggen/src/error.rs:36`).

## The `AppError` enum

`AppError` has five variants (`crates/ggen/src/error.rs:8-26`):

| Variant | Definition | Meaning |
|---|---|---|
| `Io(#[from] std::io::Error)` | `error.rs:10` | Wraps any `std::io::Error` via `?` (automatic `From` conversion). |
| `Serde(#[from] serde_json::Error)` | `error.rs:13` | Wraps any `serde_json::Error` via `?`. |
| `Validation(String)` | `error.rs:17` | Argument/precondition failures. The doc comment at `error.rs:15` states the message "MUST include FM identifier and remediation text" — this is a convention enforced by the `fm_*` constructors below, not by the type system. |
| `Graph(String)` | `error.rs:21` | RDF store, canonicalization, or delta failures. Doc comment at `error.rs:19` specifies messages "carry an FM-GRAPH-* code". |
| `Config(String)` | `error.rs:25` | `ggen.toml` load/parse failures. Doc comment at `error.rs:23` specifies messages "carry an FM-CONFIG-* code". |

Each variant derives its `Display` message from a `#[error("...")]` attribute
via `thiserror::Error` (`error.rs:7`, `error.rs:9,12,16,20,24`). Note that
`Validation` is the catch-all destination: five of the eight `fm_*`
constructors below (`fm_cli`, `fm_chain`, `fm_tpl`, `fm_write`, `fm_watch`)
construct `Self::Validation(...)`, while only `fm_graph` builds `Self::Graph`
and only `fm_config` builds `Self::Config` (`error.rs:90-127`). The enum
shape is coarser than the FM-code taxonomy it carries; the FM code lives
inside the formatted string, not as a separate enum discriminant.

## The FM-code constructors

Section `crates/ggen/src/error.rs:82-128` defines eight typed constructors,
each embedding a fixed `[FM-<FAMILY>-<code>]` prefix (zero-padded to three
digits via `{code:03}`) ahead of the caller-supplied message. All eight share
the same shape: `pub fn fm_x(code: u16, msg: impl Into<String>) -> Self`.

### `fm_cli` — `error.rs:90-92`

```rust
pub fn fm_cli(code: u16, msg: impl Into<String>) -> Self {
    Self::Validation(format!("[FM-CLI-{code:03}] {}", msg.into()))
}
```

Documented at `error.rs:87-89` as covering "CLI argument or invocation
violation." A grep across `crates/ggen/src/` for `AppError::fm_cli(` outside
`error.rs` itself turns up nothing — the only call sites are inside
`error.rs`'s own doc-example (`error.rs:145,147`) and its `#[cfg(test)]`
module (`error.rs:219,238,246,252`). The one concrete, still-real use of the
`[FM-CLI-001]` code is not via the `fm_cli` constructor at all: it is
hand-written inside `DefaultCliValidator::validate_run_args`
(`error.rs:66-77`), which calls `AppError::validation(...)` directly and
embeds the literal string `"[FM-CLI-001] --parallel requires --jobs > 0."`
(`error.rs:70-73`). So `fm_cli` the function is currently unused in
production code, but the `FM-CLI` family it names is live via that sibling
path.

### `fm_chain` — `error.rs:95-97`

Covers "receipt chain construction or integrity violation"
(`error.rs:94`). Real call sites are all in `crates/ggen/src/sync.rs`:

- `sync.rs:441-448`: raised when the previous sync receipt at
  `.ggen-v2/receipt.json` exists but fails to parse as JSON — code `3`,
  message "previous receipt `{path}` malformed: {e}. Remediation: verify or
  remove the stale receipt."
- `sync.rs:454-457`: same code `3`, raised instead when the previous receipt
  file exists but is unreadable for a reason other than "not found"
  (`std::io::ErrorKind::NotFound` is handled separately at `sync.rs:452` by
  treating a missing receipt as the all-zeros genesis chain hash).
- `sync.rs:478`: code `2`, raised when receipt-chain hash computation itself
  fails: `AppError::fm_chain(2, format!("receipt chain computation failed: {e}"))`.
- `sync.rs:497` (also in the receipt-write path) and
  `verbs/handlers.rs:153` (wrapped in `exec_err(...)`) are two further call
  sites in the same family.

Concretely: `AppError::fm_chain(3, ...)` is raised at `sync.rs:441` when
`.ggen-v2/receipt.json` from a prior sync exists on disk but its content is
not valid `SyncReceipt` JSON.

### `fm_graph` — `error.rs:100-102`

Covers "RDF store, canonicalization, or delta violation" (`error.rs:99`).
Real call sites are in `crates/ggen/src/graph.rs`:

- `graph.rs:37-39`: `DeterministicGraph::new()` raises
  `AppError::fm_graph(1, format!("failed to create in-memory store: {e}"))`
  when the underlying `oxigraph::store::Store::new()` call fails.
- `graph.rs:51,54,58`: code `2`, raised on store-length lookup failure and
  on Turtle parse/load failure during graph population.
- `graph.rs:69`: code `3`, raised on SPARQL parse failure.

Concretely: `AppError::fm_graph(1, ...)` is raised at `graph.rs:38` when
`Store::new()` — oxigraph's in-memory RDF store constructor — itself
returns an error, which `DeterministicGraph::new()`'s own doc comment
(`graph.rs:34-35`) calls out as the `[FM-GRAPH-001]` case.

### `fm_tpl` — `error.rs:105-107`

Covers "template frontmatter parse or render violation" (`error.rs:104`).
Call sites span `lint.rs` (three, at `lint.rs:261,275,292`) and `sync.rs`
(at least two, `sync.rs:169,262`). Concretely: `AppError::fm_tpl(5, ...)` is
raised at `sync.rs:262` inside `render_str`, when `tera::Tera::render_str`
fails — the message is `format!("render failed for {}: {e}", tpl_path.display())`
(`sync.rs:263-266`).

### `fm_write` — `error.rs:110-112`

Covers "file-write planning or application violation" (`error.rs:109`). All
call sites are in `crates/ggen/src/write.rs` (`write.rs:81,108,123,130,139`).
Concretely: `AppError::fm_write(3, ...)` is raised at `write.rs:80-87` when a
template frontmatter sets `inject: true` but the injection target file does
not already exist on disk — `existing.ok_or_else(...)` converts the missing
`Option<String>` into this error, with remediation text "create the file
first or drop `inject: true`" (`write.rs:83-85`).

### `fm_pack` — `error.rs:115-117`

Covers "pack resolution, hashing, or lockfile violation" (`error.rs:114`).
Call sites appear in `sync.rs:124` and `pack.rs:69,90,102,112,286,297`.
Concretely: `AppError::fm_pack(8, ...)` is raised at `pack.rs:297-304` when
a pack recorded in `ggen.lock` has a `content_hash` that no longer matches
the hash of the pack's actual on-disk content — i.e., a locked pack's
content hash has drifted from the lockfile's recorded value. The message
includes both hashes and instructs the operator to "restore the pack, or
delete ggen.lock to intentionally re-lock" (`pack.rs:298-303`). A sibling
call one code lower, `AppError::fm_pack(9, ...)` at `pack.rs:285-292`, fires
when `ggen.lock` itself fails to parse.

### `fm_config` — `error.rs:120-122`

Covers "ggen.toml loading or schema violation" (`error.rs:119`). Call sites
are in `GgenConfig::load` in `crates/ggen/src/config.rs:78-100` and in
`sync.rs:105`. Concretely: `AppError::fm_config(1, ...)` is raised at
`config.rs:84-89` when `star_toml::load_file` returns
`star_toml::Error::FileNotFound(p)` — i.e. `ggen.toml` does not exist at the
given path. The doc comment on `load` (`config.rs:82-83`) states this
maps to `[FM-CONFIG-001]`, and a syntax/unknown-key error instead maps to
`[FM-CONFIG-002]` via the `other => AppError::fm_config(2, ...)` arm
(`config.rs:95-100`).

### `fm_watch` — `error.rs:125-127`

Covers "filesystem watch setup or initial-sync violation" (`error.rs:124`).
All three call sites are in `crates/ggen/src/watch.rs:55,64-66,69`.
Concretely: `AppError::fm_watch(1, ...)` is raised at `watch.rs:54-55`
inside `watch_loop` when the initial `sync(root, SyncOptions { dry_run })`
call — run once before the filesystem watcher is even installed — returns
an error. Code `2` (`watch.rs:64-66` and `watch.rs:69`) instead covers
failure to construct or arm the underlying `notify_debouncer_mini`
debouncer.

## `ValidationChain`: aggregating multiple failures

`crates/ggen/src/error.rs:155-199` defines `ValidationChain`, a small
accumulator used where the caller wants every validation failure reported at
once rather than stopping at the first `?`. It holds a `Vec<String>`
(`error.rs:156`) and exposes:

- `check(&mut self, result: Result<()>) -> &mut Self` (`error.rs:166-171`) —
  records the `Display` string of an `Err`, ignores `Ok(())`.
- `require(&mut self, condition: bool, error: AppError) -> &mut Self`
  (`error.rs:174-179`) — records `error.to_string()` only when `condition`
  is `false`.
- `finish(self) -> Result<()>` (`error.rs:182-188`) — returns `Ok(())` if no
  errors were recorded, else `Err(AppError::Validation(joined))` where
  `joined` is every recorded message joined with `"; "` (`error.rs:186`).
- `has_errors(&self) -> bool` and `error_count(&self) -> usize`
  (`error.rs:191-198`).

The doc-example at `error.rs:141-154` and the `#[cfg(test)]` module at
`error.rs:207-256` exercise this directly: `fn multiple_errors_joined`
(`error.rs:224-233`) checks one `fm_chain` failure and one `fm_graph`
failure through the same chain and asserts both `"FM-CHAIN-001"` and
`"FM-GRAPH-003"` appear in the single joined error string
(`error.rs:230-232`).

## Correction of record

An earlier, now-deleted doc set for this project described a diagnostic
code scheme of `E0001`, `E0105`, and `E0203`, glossed as "Cycle in
Ontology," "Surrogate Bypass Attempt," and "Temporal Order Violation"
respectively. A repository-wide search of `crates/ggen/src/` for any of
these three code strings, or for any of those three phrases, returns zero
matches — they do not appear in `error.rs`, nor anywhere else in the crate.
There is no `E00xx`-style code scheme in this codebase at all.

The actual, verifiable fail-closed error scheme is the FM-code family
documented above: eight `fm_*` constructors on `AppError`
(`crates/ggen/src/error.rs:86-128`), each embedding a `[FM-<FAMILY>-<NNN>]`
prefix — `FM-CLI`, `FM-CHAIN`, `FM-GRAPH`, `FM-TPL`, `FM-WRITE`, `FM-PACK`,
`FM-CONFIG`, `FM-WATCH` — inside an `AppError::Validation`, `AppError::Graph`,
or `AppError::Config` variant. That is the only diagnostic-code taxonomy
that exists in this crate; readers should disregard any prior reference to
`E0001`/`E0105`/`E0203` as fictional.
