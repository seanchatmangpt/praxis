# Packs

A pack is a bundle of an ontology and templates that a project pulls in
alongside its own `ontology.ttl` and `templates/`. `crates/ggen/src/pack.rs`
implements the whole surface: resolving `[packs]` entries from `ggen.toml`,
hashing a pack's content deterministically, and locking that hash into
`ggen.lock` so a pack cannot silently drift under a project between syncs.
The module doc comment states the shape directly:

> "A pack is a directory containing `pack.toml`, `ontology.ttl`, and a
> `templates/` directory of `*.tmpl` files. Packs are declared in
> `ggen.toml` under `[packs]` and resolved fail-closed: a missing pack
> directory, missing manifest, missing ontology, unknown manifest keys, or
> an empty template set all refuse by name with an `FM-PACK-*` code."
> (`crates/ggen/src/pack.rs:3-7`)

## The `Pack` type and `PackRef`

`Pack` is the resolved, ready-to-sync representation of one pack:

```rust
pub struct Pack {
    pub name: String,
    pub version: String,
    pub description: String,
    pub root: PathBuf,
    pub ontology_path: PathBuf,
    pub template_paths: Vec<PathBuf>,
}
```

(`crates/ggen/src/pack.rs:20-34`) — `name` is the `[packs]` key from
`ggen.toml`, not the manifest's own `name` field (see below); `version` and
`description` come from the pack's `pack.toml`; `root` is the resolved
absolute directory; `template_paths` is the sorted list of `*.tmpl` files
found under `templates/`.

A pack is declared in `ggen.toml` as a `PackRef`, an untagged enum with two
variants:

```rust
pub enum PackRef {
    Path { path: PathBuf },
    Git { git: String, version: String },
}
```

(`crates/ggen/src/config.rs:50-61`) — `{ path = "…" }` for a local
filesystem pack, `{ git = "…", version = "…" }` for a remote one. Only
`Path` is implemented today; a `Git` ref makes `resolve` return immediately
with `FM-PACK-007` naming the pack, the git URL, and the version, and
telling the caller to vendor the pack locally instead
(`crates/ggen/src/pack.rs:64-84`, specifically the `PackRef::Git` match arm
at `:68-77`).

## `resolve`

`pub fn resolve(config: &GgenConfig, config_root: &Path) -> Result<Vec<Pack>>`
(`crates/ggen/src/pack.rs:64`) iterates `config.packs` — a `BTreeMap`, so
iteration order is always alphabetical by pack name regardless of the order
packs were declared in `ggen.toml` — and resolves each `PackRef::Path` entry
via `resolve_path_pack` (`:78-80`). `resolve_path_pack`
(`crates/ggen/src/pack.rs:87-167`) performs five checks in order, each
fail-closed with its own `FM-PACK-*` code documented on `resolve` itself
(`:57-63`):

1. **`FM-PACK-001`** — the pack directory (`config_root.join(path)`) must
   exist (`:88-98`).
2. **`FM-PACK-002`** — `pack.toml` must be present and readable
   (`:100-110`).
3. **`FM-PACK-003`** — `pack.toml` must parse as valid TOML matching the
   closed `PackToml { pack: PackMeta }` schema, which is
   `#[serde(deny_unknown_fields)]` on both the outer struct and `PackMeta`
   (`:36-50`, error at `:111-120`). `PackMeta` has exactly three fields:
   `name`, `version`, `description` (`:46-50`).
4. **`FM-PACK-004`** — `ontology.ttl` must exist directly under the pack
   root (`:122-132`).
5. **`FM-PACK-005`** — `templates/` must contain at least one `*.tmpl` file;
   template paths are collected, filtered by the `.tmpl` extension, and
   sorted (`:134-154`).

One deliberate detail: the manifest's own `pack.name` field is read and then
explicitly discarded — `let _ = &manifest.pack.name;` (`:158`) — with the
comment "The `[packs]` key in `ggen.toml` is the authoritative resolution
name; the manifest's own `name` is informational" (`:156-157`). The
resolved `Pack.name` is always the `[packs]` key, not whatever string
`pack.toml` happens to say. This means a project's `ggen.toml` can name a
pack anything it wants regardless of what the pack calls itself internally.

## `content_hash`

`pub fn content_hash(pack: &Pack) -> Result<[u8; 32]>` (`:177`) computes a
deterministic BLAKE3 hash over the pack's `ontology.ttl` plus every template
file, as sorted `(relative_path, bytes)` pairs (`:169-172`). The
implementation builds one `Vec` of `(String, PathBuf)` — the ontology path
first, then each template path (`:178-182`) — sorts that vector by the
relative-path string (`:183`), then feeds a single `blake3::Hasher` the
relative-path bytes followed by the file's bytes, for each entry in sorted
order (`:185-200`). Sorting by relative path before hashing is what makes
the hash independent of filesystem read-directory order. If a file becomes
unreadable between resolution and hashing (e.g. deleted mid-sync), the read
fails closed with `FM-PACK-006` naming the pack and the specific file
(`:186-197`).

## `lock_entries` and `LockEntry`

A `LockEntry` (`:212-220`) is `{ name, source, content_hash }`: `source` is
the pack reference as written in `ggen.toml`, textually prefixed —
`source_string` (`:239-244`) renders `PackRef::Path` as `path:<path>` and
`PackRef::Git` as `git:<url>@<version>` — and `content_hash` is
`format!("blake3:{}", hex)` of the pack's BLAKE3 digest.

`pub fn lock_entries(config: &GgenConfig, packs: &[Pack]) -> Result<Vec<LockEntry>>`
(`:250`) builds one `LockEntry` per resolved `Pack`, looking up that pack's
`source_string` from `config.packs` and calling `content_hash` for the
digest (`:251-265`). This is called from the sync pipeline right after
`resolve` — `crates/ggen/src/sync.rs:119-120` — before any template is
rendered.

## `check_lock`

`pub fn check_lock(root: &Path, entries: &[LockEntry]) -> Result<()>`
(`:276`) is the drift guard. If `ggen.lock` does not exist at the project
root, it returns `Ok(())` immediately — a missing lockfile is fine, meaning
this is the first sync (`:277-280`). Otherwise it reads and parses the
lockfile against a closed `LockDoc { packs: BTreeMap<String, LockDocEntry> }`
schema (`:222-235`), failing closed with `FM-PACK-009` if the file is
unreadable or doesn't parse (`:281-293`). For each currently-resolved
`LockEntry`, if that pack name is *already* present in the parsed lockfile
and its `content_hash` differs from what's on disk right now, `check_lock`
returns `FM-PACK-008` naming the pack, its source, the locked hash, and the
hash actually on disk, with the remediation "restore the pack, or delete
ggen.lock to intentionally re-lock" (`:294-309`). Packs present on disk but
absent from an existing lock are not an error — the comment at `:271`
states they "get locked on the next successful sync." This is called at
`crates/ggen/src/sync.rs:121`, immediately after `lock_entries`, and before
any ontology from a pack is even inserted into the graph — the whole sync
refuses before any pack content is used if the lock doesn't match.

## `write_lock`

`pub fn write_lock(root: &Path, entries: &[LockEntry]) -> Result<()>`
(`:326`) writes `ggen.lock` sorted by pack name, prefixed with the comment
line `# ggen.lock — generated by `ggen sync`. Do not edit.` (`:330`), and
one `[packs.<name>]` block per entry with `source` and `content_hash`
(`:331-337`). It is idempotent by content, not by timestamp: it first reads
whatever `ggen.lock` currently contains and, if the newly generated text is
byte-identical, returns without writing anything, leaving the file's mtime
untouched (`:338-343`). The doc comment explains why this matters:
`ggen.lock` lives at the project root — not under an ignored directory like
`.ggen-v2` — so `--watch` mode, which re-runs the pipeline on every
debounced filesystem change under the project root, would retrigger itself
forever on its own lockfile write if the write weren't conditional
(`:313-321`). Two unit tests in the same file exercise exactly this:
`write_lock_is_idempotent_when_content_is_unchanged` asserts the file's
mtime is unchanged across two writes of identical entries
(`crates/ggen/src/pack.rs:375-393`), and
`write_lock_rewrites_when_content_changes` asserts the file *does* change
when a second pack is added (`:396-404`).

`write_lock` is called from `sync` right after the receipt is written, and
only if `lock_entries` is non-empty (`crates/ggen/src/sync.rs:236-239`) — a
project with no `[packs]` never gets a `ggen.lock` at all.

## The eight real packs

`/Users/sac/praxis/packs/` (mirrored under this repo's own `packs/`)
currently ships eight packs, each with its own `pack.toml`, `ontology.ttl`,
and `templates/` directory. Every `[pack]` block below was read directly
from the corresponding `pack.toml` on disk.

**`chicago-tdd-tools-pack`** (v0.1.0) — "Generates CliHarness-based
Chicago-style CLI boundary tests (`#[test]` fns crossing a real binary
boundary) from `ctt:CliBoundaryTest` individuals." This pack turns ontology
individuals describing a CLI boundary test into real Rust `#[test]`
functions that drive a compiled binary through `chicago_tdd_tools`'s
`CliHarness`, rather than testing in-process.

**`clap-noun-verb-pack`** (v0.1.0) — "Generates clap-noun-verb route
skeletons (noun-verb `#[verb]` fns calling handler stubs) from `cnv:Command`
individuals in the pack ontology." Given `cnv:Command` individuals, it
emits the noun-verb command-routing skeleton (verb functions delegating to
handler stubs) rather than requiring that boilerplate to be hand-written
per command.

**`lsp-max-pack`** (v0.1.0) — "Generates lsp-max RulePackServer
`rules/*.toml` regex rule-pack files from `LintRule` individuals in the pack
ontology." Each `LintRule` individual becomes one `rules/*.toml` file
consumable by lsp-max's `RulePackServer`.

**`praxis-core-pack`** (v0.1.0) — "praxis-core pack: generates the
refusal-taxonomy reference table (Rust const table + markdown doc) from
`RefusalScenario` individuals mirroring `praxis_core::refusal`." Its
ontology (`packs/praxis-core-pack/ontology.ttl`, 79 lines) encodes refusal
scenarios; its two templates
(`packs/praxis-core-pack/templates/refusal_taxonomy_rs.tmpl` and
`refusal_taxonomy_md.tmpl`) render both a Rust `const` table and a Markdown
doc from the same SPARQL-queried rows, so the two artifacts cannot drift
apart from each other.

**`star-toml-pack`** (v0.1.0) — "Generates a `star_toml` config-admission
module: `deny_unknown_fields` serde structs per `ConfigSection`, loaded via
`star_toml::load_file`, plus per-section admission docs." Each
`ConfigSection` individual becomes one closed (`deny_unknown_fields`) serde
struct plus matching documentation, keeping struct and docs generated from
one ontology source rather than maintained separately by hand.

**`wasm4pm-algorithms-pack`** (v0.1.0) — "Typed Rust catalog + reference doc
for the wasm4pm process-intelligence ALGORITHM surface (catalog/caller
surface only; all analysis stays in wasm4pm)." Its description is explicit
about the boundary this project enforces elsewhere in this book (see the
Process Intelligence Boundary note): it only generates a typed catalog and
caller surface, never analysis logic itself.

**`wasm4pm-cognition-pack`** (v0.1.0) — "wasm4pm cognition breed catalog and
typed dispatch-surface skeleton over the stable 6-verb ABI
(`cognition_show/run/verify/replay`, `system_build/verify`);
catalog/caller surface only — evidence and analysis stay in wasm4pm." Same
boundary discipline as the algorithms pack, scoped to the cognition-breed
6-verb ABI instead.

**`wasm4pm-compat-pack`** (v0.1.0) — "OCEL event-type emission enum and
emit helper stubs targeting `wasm4pm-compat` (emission surface only; all
analysis stays in `wasm4pm-compat`)." Generates the emission-side enum and
helper stubs a caller uses to emit OCEL events; the pack does not generate
anything that consumes or analyzes those events.

## `cross_pack_matrix.rs`: combinatorial proofs over all eight packs

`crates/ggen/tests/cross_pack_matrix.rs` is a Chicago TDD integration test
file — real filesystem (`TempDir`), real oxigraph, real subprocess via
`CliHarness`, no mocks (`crates/ggen/tests/cross_pack_matrix.rs:1-5`) — that
proves properties of the pack system across combinations of the eight real
packs listed above, not just one pack in isolation.

`all_eight_framework_packs_exist_on_disk` (`:81-93`) is the ground-truth
check: it asserts every one of the eight packs named in the `PACKS` const
(`:18-27`) exists on disk with `pack.toml`, `ontology.ttl`, and at least one
`*.tmpl` file under `templates/` — the same shape `resolve_path_pack`
enforces at runtime, checked independently here so the rest of the matrix
has solid ground to build on.

`mega_project_all_packs_sync` (`:100-192`) scaffolds one consumer project
that references all eight packs simultaneously
(`scaffold_multi_pack_project`, `:53-70`) and runs the real `ggen` binary's
`sync run` against it. It asserts: the sync exits 0; every pack's
"distinctive output" file (a file only that pack's ontology/templates could
produce, e.g. `src/w4pm_algorithms_catalog.rs` for
`wasm4pm-algorithms-pack`) is present (`:112-118`); `ggen.lock` lists all
eight packs in alphabetical order, each with a `blake3:` hash, and nothing
else (`:120-133`); the sync receipt's `payload.packs` map contains all
eight pack names and nothing more (`:135-144`); both `ggen receipt verify`
and `ggen doctor run` exit 0 against the resulting project (`:146-158`);
and running `sync run` a *second* time is fully idempotent — same
`graph_hash`, same `outputs`, same `packs` map in the receipt, every
decision in the second receipt starts with `"skipped:"`, the second
receipt's `prev_chain_hash_hex` chains onto the first receipt's
`chain_hash_hex`, and `ggen.lock` is byte-identical across both runs
(`:160-192`).

`pairwise_pack_matrix_syncs` (`:199-241`) is the full pairwise matrix:
C(8,2) = 28 pairs, each scaffolded into its own fresh `TempDir` project
containing exactly those two packs, driven through the library `sync()`
call directly rather than a subprocess (for speed, per the comment at
`:194-198`). For every pair it asserts: `sync` returns `Ok`; both packs'
distinctive outputs are present; `ggen.lock` contains exactly those two
pack names (and no others); and the sync report's `packs` map has exactly
two entries. The test collects failures across all 28 pairs into one `Vec`
and asserts it's empty at the end (`:201`, `:240`), so a single run reports
every failing pair rather than stopping at the first. The comment at
`:198` states the purpose: catching "pair-specific ontology prefix or
output-path collisions invisible to single-pack tests" — i.e., two packs
that each work fine alone but collide when unioned into the same graph or
write to the same output path.

`ontology_union_and_declaration_order_are_canonical` (`:248-284`) proves
two properties. First, that the mega-project's graph hash (all eight
ontologies unioned) differs from every single-pack project's graph hash —
i.e., the union actually merged something and isn't silently collapsing to
one pack's graph (`:251-267`). Second, that declaring `[packs]` in reverse
alphabetical order in `ggen.toml` produces a byte-identical receipt
payload to the normal alphabetical order — proving the `BTreeMap` backing
`GgenConfig.packs` canonicalizes declaration order so textual order in
`ggen.toml` never leaks into the deterministic output (`:269-283`).

`corrupting_one_pack_post_lock_fails_closed_naming_only_that_pack`
(`:291-351`) is the sabotage test for `check_lock`. It runs the mega-project
sync once (locking all eight packs), then appends a real (still valid)
Turtle triple to `praxis-core-pack`'s `ontology.ttl` directly in the
temp-dir copy, changing that one pack's content hash
(`:302-308`). The next `sync run` must fail with `FM-PACK-008`, the stderr
must name `praxis-core-pack`, and the stderr must *not* name any of the
other seven packs (`:310-327`) — proving `check_lock`'s error only
implicates the pack that actually drifted. It also asserts the refused
sync didn't rewrite any output (`receipt verify` still passes, `:329-336`)
and that `doctor run` exits nonzero naming `lockfile_drift` (`:338-350`).

## `ggen.lock` format, with a real example

`ggen.lock` is written by `write_lock` in the shape described above: a
"do not edit" comment line, then one `[packs.<name>]` block per locked pack
with `source` and `content_hash` keys, sorted by pack name.

To see the exact bytes `write_lock` produces, this chapter builds a
throwaway one-pack project — `[project] name = "consumer"`, an empty
`ontology.ttl`, and a single `[packs.praxis-core-pack]` entry pointing at
this repo's real `packs/praxis-core-pack` — and runs the real `ggen`
binary's `sync run` against it. The resulting `ggen.lock`, quoted verbatim:

```toml
# ggen.lock — generated by `ggen sync`. Do not edit.

[packs.praxis-core-pack]
source = "path:../praxis-core-pack"
content_hash = "blake3:468c3ab7ebcc15a4ad1ae4ceda433e5b4f6653d2bc433d77175e8ef273abeb35"
```

The sync also emitted the two files `praxis-core-pack`'s templates produce
(`src/praxis_core_refusal_table.rs`, `docs/praxis_core_refusal_taxonomy.md`)
and printed a JSON report whose `packs` map carries the exact same hex
digest as the lock's `content_hash` (minus the `blake3:` prefix) —
`"praxis-core-pack": "468c3ab7ebcc15a4ad1ae4ceda433e5b4f6653d2bc433d77175e8ef273abeb35"`.

The `source` value is exactly `source_string`'s output for a
`PackRef::Path` (`path:` prefix plus the path as written in `ggen.toml`,
`crates/ggen/src/pack.rs:241`), and `content_hash` is
`blake3:` followed by the 64-character lowercase hex digest of
`content_hash(pack)` (`crates/ggen/src/pack.rs:262`) — a hash over
`packs/praxis-core-pack/ontology.ttl` and its two templates
(`refusal_taxonomy_rs.tmpl`, `refusal_taxonomy_md.tmpl`), as sorted
`(relative_path, bytes)` pairs. If any of those three files changes by even
one byte, this hash changes, and any project with a previously-locked
`ggen.lock` referencing `praxis-core-pack` will refuse to sync with
`FM-PACK-008` until the lock is deleted or the pack is restored — exactly
the behavior `corrupting_one_pack_post_lock_fails_closed_naming_only_that_pack`
proves above.

`crates/ggen/tests/pack_e2e.rs` exercises the same lock invariant against
the crate's own `examples/demo-pack`/`examples/demo-project` pair,
asserting the lock contains `[packs.widget]`, `source = "path:../demo-pack"`,
and a `content_hash` line matching the `blake3` hash returned in the sync
report's `packs` map (`crates/ggen/tests/pack_e2e.rs:60-66`) — the same
shape as the `praxis-core-pack` example above, just with a different pack.
