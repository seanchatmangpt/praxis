# What ggen Is

ggen is a Rust crate (`praxis/crates/ggen`) that generates files from an RDF
ontology by running one pipeline: **Resolve → Enrich → Extract → Render →
Write**. The crate's own slogan, stated in its module-level doc comment, is
`A = μ(O)` — artifacts are a function of an ontology (`crates/ggen/src/sync.rs:1`).
This chapter describes the crate as it exists: its module layout, the exact
pipeline implementation, the types that carry a sync's result, and a real
invocation with real output.

## Module structure

`crates/ggen/src/lib.rs` is short and enumerates the crate's entire public
surface. Reading it top to bottom tells you what ggen is made of:

```rust
pub mod chain;
pub mod cli;
pub mod config;
pub mod error;
pub mod graph;
pub mod lint;
pub mod pack;
pub mod sync;
pub mod template;
pub mod types;
pub mod write;

#[cfg(feature = "otel")]
pub mod telemetry;

#[cfg(feature = "lsp")]
pub mod lsp;

#[cfg(feature = "discovery")]
pub mod discovery;

pub mod repl;
pub mod verbs;
pub mod watch;
```

(`crates/ggen/src/lib.rs:5-28`)

Unpacking that list by role:

- `config` parses and validates `ggen.toml` (`crates/ggen/src/config.rs:1-8`,
  `:17-28`). The root type is `GgenConfig`, with `[project]`, `[ontology]`,
  `[packs]`, and `[templates]` tables (`crates/ggen/src/config.rs:17-28`), all
  marked `#[serde(deny_unknown_fields)]` so an unrecognized key is a hard
  parse error rather than a silently ignored typo.
- `graph` wraps the RDF store (`DeterministicGraph`, used throughout
  `sync.rs`, e.g. `crates/ggen/src/sync.rs:113-114`).
- `template` parses `*.tmpl` files into a `Frontmatter` + body pair; the
  `Frontmatter` struct's fields — `to`, `sparql`, `construct`, `inject`,
  `before`/`after`/`at_line`, `skip_if`, `unless_exists`, `force`, and the
  `when` ASK guard — are declared at `crates/ggen/src/template.rs:27-57`.
- `write` implements the injection/overwrite semantics that `sync.rs` calls
  through `plan_write` (`crates/ggen/src/sync.rs:37`, invoked at
  `crates/ggen/src/sync.rs:305`).
- `pack` resolves and locks external template/ontology bundles referenced
  from `[packs]` (called from `sync` at `crates/ggen/src/sync.rs:119-121`).
- `sync` is the pipeline itself — the subject of this chapter.
- `chain`, `error`, `types` supply the receipt-chaining primitives
  (`ReceiptRecord`, `AppError`, `Blake3Hash`/`ObjectRef`/`ProfileId` and the
  `Evidence`/`Admit` typestate markers re-exported at
  `crates/ggen/src/lib.rs:31`).
- `cli` and `verbs` wire the pipeline to a `clap`-based command line;
  `repl` and `watch` are interactive/incremental front ends over the same
  `sync` function; `telemetry`, `lsp`, and `discovery` are feature-gated
  (`crates/ggen/src/lib.rs:17-24`) and not compiled by default.

Nothing here is a framework abstraction over "codegen in general" — every
module maps to one stage or one supporting concern of the same five-stage
pipeline.

## The five-stage pipeline, stage by stage

The module doc comment at the top of `sync.rs` names all five stages in one
sentence:

> "The five-stage sync pipeline: Resolve → Enrich → Extract → Render →
> Write." (`crates/ggen/src/sync.rs:1`)

The public entry point is `pub fn sync(root: &Path, opts: SyncOptions) ->
Result<SyncReport>` (`crates/ggen/src/sync.rs:100`). Each stage below is
cited at the line the source code itself marks it with a `// ── Stage N
──` comment, plus the concrete operations performed there.

**Stage 1 — Resolve** (`crates/ggen/src/sync.rs:101`, comment `// ── Stage 1:
Resolve`). Loads `ggen.toml` via `GgenConfig::load` (`sync.rs:102`), reads the
ontology file named by `[ontology].source` (`sync.rs:103-112`), constructs a
fresh `DeterministicGraph` and inserts the ontology's Turtle
(`sync.rs:113-114`), resolves any `[packs]` entries and checks their content
hashes against `ggen.lock` (`sync.rs:119-121`), unions each pack's ontology
into the same graph (`sync.rs:122-134`), and discovers every `*.tmpl` file —
project templates plus pack templates — via `discover_templates`
(`sync.rs:136`, defined at `sync.rs:329-341`).

**Stage 2 — Enrich** (`crates/ggen/src/sync.rs:138`, comment `// ── Stage 2:
Enrich (single pass; see module docs) ──`). For every template that declares
a `construct:` frontmatter key, `insert_construct` runs that SPARQL
`CONSTRUCT`/`DESCRIBE` query and inserts the resulting triples back into the
graph (`sync.rs:139-143`, `insert_construct` defined at `sync.rs:387-409`).
The module doc is explicit that this is a **single pass**, not a fixed-point
iteration: "constructs that depend on other constructs' output require a
second `sync` run" (`crates/ggen/src/sync.rs:6-8`). After enrichment, the
graph's deterministic state hash is computed once (`sync.rs:145`) and the
graph is wrapped in an `Arc` for the read-only stages that follow
(`sync.rs:146`).

**Stages 3-5 — Extract, Render, Write**, run together per template inside one
loop (`crates/ggen/src/sync.rs:148`, comment `// ── Stages 3–5: Extract,
Render, Write per template ──`, loop at `sync.rs:154-221`):

- *Extract* (`sync.rs:155-191`): evaluates the `when:` ASK guard if present —
  `Boolean(false)` records a skip and moves to the next template
  (`sync.rs:156-178`) — then runs every named query in `sparql:` via
  `sparql_to_value` and collects them into a `named` map (`sync.rs:182-185`);
  the driving row set is the first named query whose result is an array
  (`sync.rs:186-191`).
- *Render* (`sync.rs:193-220`): if the `to:` path contains `{{`, the template
  is rendered once per row (`sync.rs:194-206`) via Tera (`build_tera`,
  imported at `sync.rs:36`); otherwise it renders once against the shared
  context (`sync.rs:207-220`). `render_str` maps Tera failures to an
  `FM-TPL-005` error (`sync.rs:255-267`).
- *Write* (`sync.rs:271-320`, function `apply`): `skip_empty` frontmatter
  short-circuits an empty rendered body (`sync.rs:281-286`); a `dry_run`
  classifies the write without touching disk, comparing against existing
  content byte-for-byte (`sync.rs:287-304`); otherwise `plan_write` from
  `crate::write` performs the actual write/injection and returns
  `Written`/`Injected`/`Skipped(reason)`, each recorded into `decisions` and
  either `written` or `skipped` (`sync.rs:305-319`).

After the loop, `sync` assembles pack content hashes from the lock entries
(`sync.rs:223-231`) into the final `SyncReport` (`sync.rs:233`), and — unless
`opts.dry_run` is set — writes a chained receipt via `write_receipt`
(`sync.rs:235-236`, function defined at `sync.rs:414-503`) and updates
`ggen.lock` if any packs were resolved (`sync.rs:237-239`).

## `SyncOptions` and `SyncReport`

`SyncOptions` is the sole input to a sync run beyond the project root:

```rust
#[derive(Debug, Clone, Copy, Default)]
pub struct SyncOptions {
    /// Compute outcomes without writing any file (and without a receipt).
    pub dry_run: bool,
}
```

(`crates/ggen/src/sync.rs:40-44`)

One field. `dry_run: true` makes `sync` a pure read of the current state —
no file write, no receipt (the `if !opts.dry_run` guard at `sync.rs:235`
covers both).

`SyncReport` is the return value:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct SyncReport {
    pub written: Vec<PathBuf>,
    pub skipped: Vec<(PathBuf, String)>,
    pub graph_hash_hex: String,
    pub decisions: BTreeMap<String, String>,
    pub packs: BTreeMap<String, String>,
}
```

(`crates/ggen/src/sync.rs:46-60`)

`written` and `skipped` are the two possible outcomes per output path (with a
reason string for skips); `graph_hash_hex` is the BLAKE3 hex of the
post-Enrich canonical graph state (`sync.rs:53-54`, computed at
`sync.rs:145`); `decisions` is a root-relative-path → decision-string map
covering every template output, including the exact reason text
(`"skipped: unchanged: content identical"`, `"written"`, `"injected"`, etc. —
see `apply`, `sync.rs:271-320`); `packs` maps each resolved pack's name to
its BLAKE3 content hash (`sync.rs:58-59`, populated at `sync.rs:223-231`).

A non-dry-run sync additionally produces a `SyncReceipt` on disk at
`.ggen-v2/receipt.json` (`RECEIPT_REL_PATH`, `crates/ggen/src/sync.rs:63`)
and appends one line to `.ggen-v2/receipt-log.jsonl`
(`RECEIPT_LOG_REL_PATH`, `sync.rs:67`). `SyncReceipt` wraps a praxis-core
`ReceiptRecord` chained over a `ReceiptPayload` (`sync.rs:70-91`); the
payload holds `graph_hash`, per-output BLAKE3 hashes (`outputs`), pack
hashes, and the same `decisions` map (`sync.rs:79-91`). The module doc notes
that `ts_ns` is fixed to `0` in every receipt — no wall clock is read on this
path — because this crate "forbids wall clocks" and instead builds the
record directly and chains it with `ReceiptRecord::recompute_chain_hash`
(`crates/ggen/src/sync.rs:14-21`, chaining logic at `sync.rs:461-479`).

## Why this shape: μ as a composed morphism

`docs/ggen-theory.md` situates this five-function pipeline inside a formal
algebra before any of `praxis/crates/ggen` was written. Its central claim
(§1, "Algebra: μ as a composed morphism in a category of graphs") is that the
five stages are literally function composition: `μ = μ₅ ∘ μ₄ ∘ μ₃ ∘ μ₂ ∘ μ₁ :
Ont → Artifact`, where `μ₁` is load, `μ₂` is inference (required to behave as
a closure operator — extensive, monotone, idempotent), `μ₃` is generation (a
pure function of the graph, with no wall clock in the render path), `μ₄` is
validation (partitioning the ontology space into a feasible region), and
`μ₅` is emit-plus-receipt. The document is explicit that this isn't
decoration: idempotence of `μ₂` is the reason "run inference twice" must be
a no-op, and purity of the receipt hash `H = BLAKE3(canonicalize(O))` is the
reason a receipt can prove what produced it (§1.2). Matched against the code
above: `sync.rs`'s Stage 1 is `μ₁`, Stage 2's single-pass `insert_construct`
loop is `μ₂` (with the module doc's own caveat that it is not yet iterated
to a fixed point — i.e., not yet a full closure operator by that theory's own
idempotence test), Extract/Render together are `μ₃`, the `when:` ASK guard
is the closest current analogue to `μ₄`'s feasibility check, and `write_receipt`'s
zero-`ts_ns`, chain-recomputed record is the concrete implementation of
`μ₅`'s receipt requirement (§1.2-§1.3).

## Running `ggen sync`: one real invocation

`ggen sync` is the single command that runs the whole pipeline. The CLI
exposes it as `ggen sync run`, wired to `handle_sync_run`, which resolves the
project root, and — when not watching — calls `sync(&root, SyncOptions {
dry_run })` directly and serializes the `SyncReport`
(`crates/ggen/src/verbs/handlers.rs:36-44`).

To show real behavior rather than describe it, the following was executed
against a minimal project built for this chapter (`/tmp/ggenbook-demo`),
using the debug binary built from this checkout (`cargo build -p ggen`,
producing `target/debug/ggen`).

`ggen.toml`:

```toml
[project]
name = "ggenbook-demo"

[ontology]
source = "ontology.ttl"

[templates]
dir = "templates"
```

`ontology.ttl`:

```turtle
@prefix ex: <http://example.org/> .
ex:Widget a ex:Component ;
    ex:name "Widget" .
ex:Gadget a ex:Component ;
    ex:name "Gadget" .
```

`templates/component.tmpl`:

```
---
to: "src/{{ row.name }}.rs"
sparql:
  components: |
    SELECT ?name WHERE {
      ?c a <http://example.org/Component> ;
         <http://example.org/name> ?name .
    } ORDER BY ?name
---
// Generated component: {{ row.name }}
pub struct {{ row.name }};
```

Invocation and actual stdout:

```
$ ggen sync run
{
  "decisions": {
    "src/Gadget.rs": "written",
    "src/Widget.rs": "written"
  },
  "graph_hash_hex": "4fe7463bfa222108e967c3cb4ad6143a3ebacc3fefb7d034f498267cfd85aea7",
  "packs": {},
  "skipped": [],
  "written": [
    "src/Gadget.rs",
    "src/Widget.rs"
  ]
}
```

The files it wrote, exactly as they landed on disk:

```
$ cat src/Widget.rs
// Generated component: Widget
pub struct Widget;

$ cat src/Gadget.rs
// Generated component: Gadget
pub struct Gadget;
```

And the receipt written alongside them at `.ggen-v2/receipt.json`:

```json
{
  "record": {
    "version": 1,
    "instruction_id": 0,
    "activity_idx": 0,
    "activity": "ggen.sync",
    "node_kind": 0,
    "ts_ns": 0,
    "payload_hash_hex": "559e94279a18990ba6d90d82e325c9b42bb61fcca9f6473a08db9eca60ad042f",
    "prev_chain_hash_hex": "0000000000000000000000000000000000000000000000000000000000000000",
    "chain_hash_hex": "0ef6dfe8342cf2caf329bb5a10b5a57bb4475501cf95a00c6466ad3e22254397",
    "andon": "Green",
    "obligation_count": 0,
    "object_ids": [
      "law:559e94279a18990b"
    ]
  },
  "payload": {
    "graph_hash": "4fe7463bfa222108e967c3cb4ad6143a3ebacc3fefb7d034f498267cfd85aea7",
    "outputs": {
      "src/Gadget.rs": "d8692d4e83447eb1f191cb9e8ad425196eb57d0d492135bfac7d7c8ff1d954ef",
      "src/Widget.rs": "90ffce3118f2f3afa68a86db8d901001759b3917b75bed652bd8d5ba8479ef10"
    },
    "packs": {},
    "decisions": {
      "src/Gadget.rs": "written",
      "src/Widget.rs": "written"
    }
  }
}
```

In practice this is what "`ggen sync` is the single command" means: one
invocation walked the ontology, ran the template's SPARQL `SELECT`, rendered
one output file per matched row, wrote both files, and left a chained,
zero-wall-clock receipt proving exactly which graph hash and which output
hashes that run produced — matching, line for line, the five-stage,
receipt-producing pipeline described above.

`cargo build -p ggen` from `/Users/sac/praxis` succeeds and produces
`target/debug/ggen`. `cargo test -p ggen` runs 14 separate test binaries
(the lib plus 13 integration/doctest targets); on a clean run every binary
reports `test result: ok`, for 182 passing tests total and 0 failures. One
of those binaries, `watch::tests::watch_loop_resyncs_on_template_change`,
polls a filesystem watcher against a 5-second wall-clock timeout and has
been observed to fail intermittently under load — a re-run with no other
change passed, so this is a timing-sensitive test, not a broken feature.
