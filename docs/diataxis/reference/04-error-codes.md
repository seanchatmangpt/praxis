# Error Code Reference

All `crates/ggen` errors are variants of `AppError` (`crates/ggen/src/error.rs:8-26`). Every failure-mode (FM) code is embedded as a `[FM-<FAMILY>-<NNN>]` prefix inside the `Validation`, `Graph`, or `Config` variant's string payload via one of eight typed constructors defined at `crates/ggen/src/error.rs:86-128`. There is no central registry/enum of codes; codes exist only as the `u16` argument passed at each call site.

## `AppError` variants

| Variant | Definition | Underlying data | Produced by |
|---|---|---|---|
| `Io` | `crates/ggen/src/error.rs:9-10` | `#[from] std::io::Error` | Any `?` on a raw I/O call (no FM code attached) |
| `Serde` | `crates/ggen/src/error.rs:12-13` | `#[from] serde_json::Error` | Any `?` on raw JSON (de)serialization (no FM code attached) |
| `Validation` | `crates/ggen/src/error.rs:16-17` | `String` | `AppError::validation`, `fm_cli`, `fm_chain`, `fm_tpl`, `fm_write`, `fm_pack`, `fm_watch` |
| `Graph` | `crates/ggen/src/error.rs:20-21` | `String` | `AppError::fm_graph` |
| `Config` | `crates/ggen/src/error.rs:24-25` | `String` | `AppError::fm_config` |

## FM code families

| Family | Constructor | Definition | Message format | Wraps `AppError` variant |
|---|---|---|---|---|
| `FM-CLI-*` | `fm_cli(code, msg)` | `crates/ggen/src/error.rs:90-92` | `[FM-CLI-{code:03}] {msg}` | `Validation` |
| `FM-CHAIN-*` | `fm_chain(code, msg)` | `crates/ggen/src/error.rs:95-97` | `[FM-CHAIN-{code:03}] {msg}` | `Validation` |
| `FM-GRAPH-*` | `fm_graph(code, msg)` | `crates/ggen/src/error.rs:100-102` | `[FM-GRAPH-{code:03}] {msg}` | `Graph` |
| `FM-TPL-*` | `fm_tpl(code, msg)` | `crates/ggen/src/error.rs:105-107` | `[FM-TPL-{code:03}] {msg}` | `Validation` |
| `FM-WRITE-*` | `fm_write(code, msg)` | `crates/ggen/src/error.rs:110-112` | `[FM-WRITE-{code:03}] {msg}` | `Validation` |
| `FM-PACK-*` | `fm_pack(code, msg)` | `crates/ggen/src/error.rs:115-117` | `[FM-PACK-{code:03}] {msg}` | `Validation` |
| `FM-CONFIG-*` | `fm_config(code, msg)` | `crates/ggen/src/error.rs:120-122` | `[FM-CONFIG-{code:03}] {msg}` | `Config` |
| `FM-WATCH-*` | `fm_watch(code, msg)` | `crates/ggen/src/error.rs:125-127` | `[FM-WATCH-{code:03}] {msg}` | `Validation` |

The code is formatted `{code:03}` (zero-padded to 3 digits), e.g. `fm_graph(1, ..)` renders `[FM-GRAPH-001]`.

## FM-CLI-*

| Code | Status | Triggering condition | Call site |
|---|---|---|---|
| `FM-CLI-001` | **Dead in practice** | `CliValidator::validate_run_args` returns this when `parallel == true && jobs == 0` | `crates/ggen/src/error.rs:67-76` (definition), inside `impl CliValidator for DefaultCliValidator` |

`FM-CLI-001` is the only code in this family. `grep -rn "AppError::fm_cli(" crates/ggen/src/` outside `error.rs` returns no results — no CLI command construction, verb handler, or argument-parsing path in `crates/ggen/src` calls `CliValidator::validate_run_args` or reads `CLI_VALIDATOR` (`crates/ggen/src/error.rs:80`). The trait, its default impl, and the static singleton are defined but never invoked from any real command path. The only place `fm_cli` is exercised at all is the crate's own unit tests (`crates/ggen/src/error.rs:219`, `crates/ggen/src/error.rs:252`).

## FM-CHAIN-*

Receipt chain construction/integrity failures, produced by `crate::sync::write_receipt` and `crate::verbs::handlers::handle_receipt_history`.

| Code | Triggering condition | Call site |
|---|---|---|
| `FM-CHAIN-001` | **Dead in production code.** No call site outside a unit test. | `crates/ggen/src/error.rs:227` (`validation_chain_tests::multiple_errors_joined`) |
| `FM-CHAIN-002` | `ReceiptRecord::recompute_chain_hash()` fails while computing the chain hash for a new sync receipt | `crates/ggen/src/sync.rs:478` |
| `FM-CHAIN-003` | Previous `.ggen-v2/receipt.json` exists but fails to deserialize (malformed JSON) | `crates/ggen/src/sync.rs:441` |
| `FM-CHAIN-003` | Previous `.ggen-v2/receipt.json` exists but a non-`NotFound` I/O error occurs reading it | `crates/ggen/src/sync.rs:454` |
| `FM-CHAIN-004` | Append of a new line to `.ggen-v2/receipt-log.jsonl` fails (open-for-append or write) | `crates/ggen/src/sync.rs:497` |
| `FM-CHAIN-005` | `.ggen-v2/receipt-log.jsonl` unreadable during `ggen receipt history` | `crates/ggen/src/verbs/handlers.rs:153` |
| `FM-CHAIN-005` | `.ggen-v2/receipt-log.jsonl` reads as empty (zero non-blank lines) during `ggen receipt history` | `crates/ggen/src/verbs/handlers.rs:174` |
| `FM-CHAIN-006` | One line of `.ggen-v2/receipt-log.jsonl` fails to deserialize as a `SyncReceipt` | `crates/ggen/src/verbs/handlers.rs:169` |
| `FM-CHAIN-007` | First logged receipt's `prev_chain_hash_hex` is not the genesis all-zeros value | `crates/ggen/src/verbs/handlers.rs:186` |
| `FM-CHAIN-007` | A receipt's recomputed payload hash does not match its stored `payload_hash_hex` | `crates/ggen/src/verbs/handlers.rs:198` |
| `FM-CHAIN-007` | `ReceiptRecord::recompute_chain_hash()` fails during `ggen receipt history` verification | `crates/ggen/src/verbs/handlers.rs:209` |
| `FM-CHAIN-007` | A receipt's recomputed chain hash does not match its stored `chain_hash_hex` | `crates/ggen/src/verbs/handlers.rs:216` |
| `FM-CHAIN-007` | Adjacent-record link check fails: record `idx`'s `chain_hash_hex` does not equal record `idx+1`'s `prev_chain_hash_hex` | `crates/ggen/src/verbs/handlers.rs:228` |

Note: `FM-CHAIN-007` is reused across five distinct verification failures inside `handle_receipt_history` (genesis check, payload-hash check, recompute failure, chain-hash mismatch, broken adjacent link); each call site's message text distinguishes the specific failure (`"history invalid at index {idx}: ..."`).

## FM-GRAPH-*

RDF store, canonicalization, or delta failures, produced by `DeterministicGraph` and `Delta` (`crates/ggen/src/graph.rs`) and by `crate::sync::insert_construct`.

| Code | Triggering condition | Call site |
|---|---|---|
| `FM-GRAPH-001` | `oxigraph::store::Store::new()` fails during `DeterministicGraph::new()` | `crates/ggen/src/graph.rs:38` |
| `FM-GRAPH-002` | `store.len()` fails before loading Turtle in `insert_turtle` | `crates/ggen/src/graph.rs:51` |
| `FM-GRAPH-002` | `store.load_from_slice(RdfFormat::Turtle, ..)` fails (Turtle syntax error) | `crates/ggen/src/graph.rs:54` |
| `FM-GRAPH-002` | `store.len()` fails after loading Turtle in `insert_turtle` | `crates/ggen/src/graph.rs:58` |
| `FM-GRAPH-003` | `SparqlEvaluator::parse_query` fails (SPARQL syntax error) | `crates/ggen/src/graph.rs:69` |
| `FM-GRAPH-003` | Query evaluation (`.execute()`) fails after successful parse | `crates/ggen/src/graph.rs:72` |
| `FM-GRAPH-004` | `store.iter()` collection into `Vec<Quad>` fails in `all_quads` | `crates/ggen/src/graph.rs:83` |
| `FM-GRAPH-005` | `BlankNode::new(format!("c14n{idx}"))` is rejected while assigning canonical blank-node labels | `crates/ggen/src/graph.rs:326` |
| `FM-GRAPH-006` | `Delta::apply`: a deletion's canonical N-Quads string has no matching quad currently in the graph | `crates/ggen/src/graph.rs:144` |
| `FM-GRAPH-006` | `Delta::apply`: `store.remove(..)` fails removing a matched quad | `crates/ggen/src/graph.rs:150` |
| `FM-GRAPH-006` | `Delta::apply`: `store.load_from_slice(RdfFormat::NQuads, ..)` fails inserting delta additions | `crates/ggen/src/graph.rs:166` |
| `FM-GRAPH-007` | `insert_construct`: the `construct:` frontmatter query's result is not `QueryResults::Graph` (not a CONSTRUCT/DESCRIBE query) | `crates/ggen/src/sync.rs:391` |
| `FM-GRAPH-007` | `insert_construct`: iterating the CONSTRUCT query's triples fails mid-stream | `crates/ggen/src/sync.rs:400` |

Doc comments at `crates/ggen/src/graph.rs:35`, `:46`, `:65`, `:78`, `:93-94`, `:134-135` state the intended code for each public method; all match the call sites above.

## FM-TPL-*

Template frontmatter parse or render failures, produced by `Template::parse` (`crates/ggen/src/template.rs`), `sparql_to_value`, `crate::sync` render/discovery helpers, and `crate::lint::lint_template`.

| Code | Triggering condition | Call site |
|---|---|---|
| `FM-TPL-001` | `Template::parse`: content does not start with the `---` prefix | `crates/ggen/src/template.rs:83-89` |
| `FM-TPL-001` | `Template::parse`: opening `---` is not followed by a newline (not on its own line) | `crates/ggen/src/template.rs:91-93` |
| `FM-TPL-001` | `Template::parse`: no closing `---` line found (unterminated frontmatter) | `crates/ggen/src/template.rs:94-100` |
| `FM-TPL-002` | `Template::parse`: `serde_yaml::from_str` fails to deserialize the frontmatter YAML (includes any unknown key — closed key set) | `crates/ggen/src/template.rs:101-111` |
| `FM-TPL-003` | `sparql_to_value`: iterating a SELECT solution fails | `crates/ggen/src/template.rs:160-162` |
| `FM-TPL-003` | `sparql_to_value`: iterating a CONSTRUCT/DESCRIBE triple fails | `crates/ggen/src/template.rs:174-176` |
| `FM-TPL-003` | `lint_template`: template body consumes `{{ var }}` not projected by any frontmatter SPARQL SELECT | `crates/ggen/src/lint.rs:261-270` |
| `FM-TPL-004` | `sync`: a `when:` frontmatter query's result is not `QueryResults::Boolean` (not an ASK query) | `crates/ggen/src/sync.rs:169-176` |
| `FM-TPL-004` | `lint_template`: `to:` path consumes `{{ var }}` not projected by any frontmatter SPARQL SELECT | `crates/ggen/src/lint.rs:275-284` |
| `FM-TPL-005` | `sync::render_str`: `tera.render_str` fails rendering a template body or `to:` string | `crates/ggen/src/sync.rs:261-265` |
| `FM-TPL-005` | `lint_template`: `construct:` frontmatter is an identity CONSTRUCT (CONSTRUCT pattern equals WHERE pattern; no-op enrichment) | `crates/ggen/src/lint.rs:292-301` |
| `FM-TPL-006` | `sync::parse_template_file`: `Template::parse` fails for a discovered `*.tmpl` file | `crates/ggen/src/sync.rs:361` |

Note: `FM-TPL-003`, `FM-TPL-004`, and `FM-TPL-005` are each reused for two distinct conditions — one in the runtime `sync`/`template` path, one in the static `lint_template` checker (`crates/ggen/src/lint.rs`). The doc comment at `crates/ggen/src/template.rs:77-81` documents only the `template.rs`-local `FM-TPL-001`/`FM-TPL-002` meanings; it predates the `lint.rs` reuse of `003`/`004`/`005` and does not mention it.

## FM-WRITE-*

File-write planning or application failures, produced by `crate::write` (`plan_write`, `resolve_target`, `inject_into`, `find_marker_line`).

| Code | Triggering condition | Call site |
|---|---|---|
| `FM-WRITE-001` | `resolve_target`: `root.canonicalize()` fails | `crates/ggen/src/write.rs:122-127` |
| `FM-WRITE-002` | `resolve_target`: `rel_to` is an absolute path | `crates/ggen/src/write.rs:129-133` |
| `FM-WRITE-002` | `resolve_target`: `rel_to` contains a non-`Normal`/`CurDir` path component (traversal) | `crates/ggen/src/write.rs:135-146` |
| `FM-WRITE-002` | `resolve_target`: nearest existing ancestor of the target fails to canonicalize | `crates/ggen/src/write.rs:162-164` |
| `FM-WRITE-002` | `resolve_target`: resolved target path escapes the canonicalized project root | `crates/ggen/src/write.rs:165-173` |
| `FM-WRITE-003` | `plan_write`: `frontmatter.inject = true` but the target file does not exist | `crates/ggen/src/write.rs:79-89` |
| `FM-WRITE-004` | `inject_into`: `at_line` is `0` or greater than `lines.len() + 1` (out of range) | `crates/ggen/src/write.rs:197-206` |
| `FM-WRITE-004` | `find_marker_line`: `before:`/`after:` marker string not found in the target file's lines | `crates/ggen/src/write.rs:222-234` |
| `FM-WRITE-005` | `plan_write`: target exists with content differing from the rendered body, `force` is not set (refuses silent clobber) | `crates/ggen/src/write.rs:108-115` |

## FM-PACK-*

Pack resolution, hashing, or lockfile failures, produced by `crate::pack` (`resolve`, `resolve_path_pack`, `content_hash`, `check_lock`) and `crate::sync::sync`.

| Code | Triggering condition | Call site |
|---|---|---|
| `FM-PACK-001` | `resolve_path_pack`: pack directory does not exist under `config_root` | `crates/ggen/src/pack.rs:90-97` |
| `FM-PACK-002` | `resolve_path_pack`: `pack.toml` unreadable at the pack root | `crates/ggen/src/pack.rs:101-110` |
| `FM-PACK-003` | `resolve_path_pack`: `pack.toml` fails to parse as `PackToml` (invalid TOML or unknown keys) | `crates/ggen/src/pack.rs:111-120` |
| `FM-PACK-004` | `resolve_path_pack`: `ontology.ttl` missing at the pack root | `crates/ggen/src/pack.rs:123-131` |
| `FM-PACK-004` | `sync`: a resolved pack's `ontology_path` fails to read (`std::fs::read_to_string`) | `crates/ggen/src/sync.rs:123-132` |
| `FM-PACK-005` | `resolve_path_pack`: zero `*.tmpl` files found under the pack's `templates/` directory | `crates/ggen/src/pack.rs:145-153` |
| `FM-PACK-006` | `content_hash`: a pack file (ontology or template) becomes unreadable while computing its BLAKE3 content hash | `crates/ggen/src/pack.rs:187-197` |
| `FM-PACK-007` | `resolve`: a `[packs]` entry is `PackRef::Git` (git packs are not implemented) | `crates/ggen/src/pack.rs:68-77` |
| `FM-PACK-008` | `check_lock`: a pack's recomputed content hash differs from the hash recorded in `ggen.lock` | `crates/ggen/src/pack.rs:296-305` (block starts `:294`, error at `:297`) |
| `FM-PACK-009` | `check_lock`: `ggen.lock` exists but is unreadable | `crates/ggen/src/pack.rs:282` |
| `FM-PACK-009` | `check_lock`: `ggen.lock` exists but fails to parse as `LockDoc` (malformed TOML or unknown keys) | `crates/ggen/src/pack.rs:284-293` |

The doc comment at `crates/ggen/src/pack.rs:56-63` lists the intended codes for `resolve`/`resolve_path_pack` (001–005, 007); `crates/ggen/src/pack.rs:174-176` documents `006` for `content_hash`; `crates/ggen/src/pack.rs:273-275` documents `008`/`009` for `check_lock`. All match the call sites above.

## FM-CONFIG-*

`ggen.toml` loading or schema failures, produced by `GgenConfig::load`/`from_toml_str` (`crates/ggen/src/config.rs`) and `crate::sync` ontology/templates-dir resolution.

| Code | Triggering condition | Call site |
|---|---|---|
| `FM-CONFIG-001` | `GgenConfig::load`: `star_toml::Error::FileNotFound` — `ggen.toml` does not exist at the given path | `crates/ggen/src/config.rs:84-90` |
| `FM-CONFIG-001` | `GgenConfig::load`: `star_toml::Error::Io` — `ggen.toml` exists but cannot be read (permissions, etc.) | `crates/ggen/src/config.rs:91-94` |
| `FM-CONFIG-002` | `GgenConfig::load`: any other `star_toml::Error` variant (TOML syntax error or unknown key) | `crates/ggen/src/config.rs:95-101` |
| `FM-CONFIG-002` | `GgenConfig::from_toml_str`: `star_toml::from_str` fails (TOML syntax error or unknown key) | `crates/ggen/src/config.rs:111-116` |
| `FM-CONFIG-003` | `sync`: the file at `[ontology].source` (relative to `root`) is unreadable | `crates/ggen/src/sync.rs:104-112` |
| `FM-CONFIG-004` | `sync::collect_tmpl_paths`: the `[templates].dir` directory is unreadable (`std::fs::read_dir` fails) | `crates/ggen/src/sync.rs:366-374` |

The doc comment at `crates/ggen/src/config.rs:79-81` documents `001`/`002` for `GgenConfig::load`; `crates/ggen/src/sync.rs:96-99` refers generically to "FM-* codes on `crate::config`" for the `sync` function without enumerating `003`/`004` individually.

**Live confirmation** — running `ggen sync run` from an empty directory (no `ggen.toml`) on the built binary (`cargo build -p ggen`, from `/Users/sac/praxis`) produces:

```
$ /Users/sac/praxis/target/debug/ggen sync run
Error: Command execution failed: config error: [FM-CONFIG-001] ggen.toml not found at `/private/tmp/ggen-err-demo/ggen.toml`. Remediation: create the manifest or fix the path.
```

Captured 2026-07-04 from a scratch directory (`/tmp/ggen-err-demo`), not a repo file.

## FM-WATCH-*

Filesystem watch setup or initial-sync failures, produced by `crate::watch::watch_loop`.

| Code | Triggering condition | Call site |
|---|---|---|
| `FM-WATCH-001` | `watch_loop`: the up-front synchronous `sync(root, ..)` call fails | `crates/ggen/src/watch.rs:54-55` |
| `FM-WATCH-002` | `watch_loop`: `notify_debouncer_full::new_debouncer` fails to construct the filesystem watcher | `crates/ggen/src/watch.rs:64-66` |
| `FM-WATCH-002` | `watch_loop`: `debouncer.watch(root, RecursiveMode::Recursive)` fails to start watching `root` | `crates/ggen/src/watch.rs:67-69` |

The doc comment at `crates/ggen/src/watch.rs:43-45` documents both codes for `watch`/`watch_loop`; matches the call sites above.

## Summary: code counts by family

| Family | Distinct codes defined | Codes with a real (non-test) call site | Dead codes |
|---|---|---|---|
| `FM-CLI` | 1 (`001`) | 0 | `001` — constructor exists, `CliValidator` is never invoked outside `error.rs`'s own default impl and unit tests |
| `FM-CHAIN` | 7 (`001`–`007`) | 6 (`002`–`007`) | `001` — only reachable from a unit test (`crates/ggen/src/error.rs:227`) |
| `FM-GRAPH` | 7 (`001`–`007`) | 7 | none |
| `FM-TPL` | 6 (`001`–`006`) | 6 | none |
| `FM-WRITE` | 5 (`001`–`005`) | 5 | none |
| `FM-PACK` | 9 (`001`–`009`) | 9 | none |
| `FM-CONFIG` | 4 (`001`–`004`) | 4 | none |
| `FM-WATCH` | 2 (`001`–`002`) | 2 | none |

Method: for each of the 8 typed constructors, `grep -rn "AppError::fm_<family>(" crates/ggen/src/` was run and every match outside `crates/ggen/src/error.rs` itself was read at its reported line to confirm the numeric code literal and surrounding condition (see individual call-site tables above). `FM-CLI-001` and `FM-CHAIN-001` were the only two constructor invocations found solely inside `crates/ggen/src/error.rs` (either the default trait impl, which nothing calls, or `#[cfg(test)]` unit tests), and are marked dead in production code above accordingly.
