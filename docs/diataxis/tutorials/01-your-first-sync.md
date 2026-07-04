# Your First Sync

In this tutorial you will build the smallest possible `ggen` project from
scratch: a manifest, a 3-triple ontology, and one template that queries that
ontology with SPARQL and loops over the results with Tera. Then you will run
the real pipeline and inspect everything it produced, including its
cryptographic receipt.

You need a built `ggen` binary and an empty scratch directory. Nothing else.

## Step 1: Build the `ggen` binary

From the repository root, build the CLI crate:

```bash
cargo build -p ggen
```

This produces the binary at `target/debug/ggen` (crate `crates/ggen`,
binary name `ggen`, confirmed by the `[[bin]]` table naming the other two
binaries `dod` and `mcp_server` in `crates/ggen/Cargo.toml:1-8` — the crate's
default binary, matching the package name, is the one you just built).

## Step 2: Create a scratch project directory

Never build a demo project inside the repository. Create an isolated
directory outside it:

```bash
mkdir -p /tmp/ggen-tutorial-demo/scratch/templates
cd /tmp/ggen-tutorial-demo/scratch
```

## Step 3: Write the manifest

Create a file named `ggen.toml` with exactly this content:

```toml
[project]
name = "hello-ggen"

[ontology]
source = "ontology.ttl"

[templates]
dir = "templates"
```

Every key here is required. `GgenConfig` in `crates/ggen/src/config.rs:19-29`
defines the manifest as a `[project]` table (name only,
`crates/ggen/src/config.rs:34-37`), an `[ontology]` table pointing at your
Turtle source file (`crates/ggen/src/config.rs:42-48`), an optional
`[packs]` map you are not using yet (`crates/ggen/src/config.rs:26`), and a
`[templates]` table naming the directory `ggen` scans for template files
(`crates/ggen/src/config.rs:71-74`). Every table in the file uses
`#[serde(deny_unknown_fields)]` (`crates/ggen/src/config.rs:18`, `:33`,
`:41`, `:70`), so a typo'd key is a hard parse error, not a silently
ignored one.

## Step 4: Write the ontology

Create a file named `ontology.ttl` with exactly this content:

```turtle
@prefix ex: <http://example.org/> .

ex:alice ex:name "Alice" .
ex:bob   ex:name "Bob" .
ex:carol ex:name "Carol" .
```

This is three triples: three subjects, each with one `ex:name` literal.
`ggen` loads this file into an in-memory RDF store as plain Turtle
(`DeterministicGraph::insert_turtle`, `crates/ggen/src/graph.rs:44-56`).

## Step 5: Write the template

Create a file named `templates/greeters.tmpl` with exactly this content:

```
---
to: greeters.txt
sparql:
  people: SELECT ?name WHERE { ?s <http://example.org/name> ?name } ORDER BY ?name
---
{% for row in people %}Hello, {{ row.name }}!
{% endfor %}
```

A `.tmpl` file is a YAML frontmatter block delimited by `---` lines,
followed by a Tera body (`Template::parse`, `crates/ggen/src/template.rs:82-113`).
The frontmatter key set is closed — `to`, `sparql`, `construct`, `inject`,
`before`, `after`, `at_line`, `skip_if`, `unless_exists`, `force`, `when`,
`skip_empty` are the only allowed keys (`crates/ggen/src/template.rs:27-63`);
an unrecognized key is rejected at parse time. Here you use only two:
`to` (the output path, `crates/ggen/src/template.rs:29`) and a named
`sparql:` query called `people` (`crates/ggen/src/template.rs:31-32`).

Inside the body, `people` is available as an array of row objects because
`build_tera` registers a `sparql(query="…")` Tera function whose SELECT
results become an array of `{var: value}` objects, one per solution row
(`crates/ggen/src/template.rs:138-151` and `:157-170`) — the pipeline
evaluates every named `sparql:` query and inserts the result directly into
the template context under its key (`crates/ggen/src/sync.rs:184-187`), so
`{% for row in people %}` iterates those rows and `row.name` reads the
`?name` binding from each one.

## Step 6: Run the sync

From `/tmp/ggen-tutorial-demo/scratch`, run:

```bash
/path/to/your/checkout/target/debug/ggen sync run
```

You will see JSON output like this (captured from a real run):

```json
{
  "decisions": {
    "greeters.txt": "written"
  },
  "graph_hash_hex": "e9c523723d4429529607fbef6908f379c7ae732ffd41c70e2b7746ce9b3ac7ea",
  "packs": {},
  "skipped": [],
  "written": [
    "greeters.txt"
  ]
}
```

`sync run` is the five-stage pipeline — resolve, enrich, extract, render,
write — implemented by `sync()` in `crates/ggen/src/sync.rs:106-215` and
exposed as `ggen sync run` via the `#[clap_noun_verb_macros::verb("run",
"sync")]` route in `crates/ggen/src/verbs/sync.rs:11-13`, which calls
`handle_sync_run` in `crates/ggen/src/verbs/handlers.rs:36-44`. Your
template had no `construct:` query, so enrich did nothing; the `when:`
guard was absent, so extract always proceeded; the named `people` SELECT
ran against the graph and rendered the Tera body; then the write stage
created `greeters.txt` because it didn't exist yet — reported as
`"written"` (`crates/ggen/src/sync.rs:265-267`).

## Step 7: Inspect the generated file

```bash
cat greeters.txt
```

Real output:

```
Hello, Alice!
Hello, Bob!
Hello, Carol!
```

Three names, alphabetically ordered — exactly what the `ORDER BY ?name`
clause in your SPARQL query guarantees.

## Step 8: Confirm the receipt exists

```bash
ls .ggen-v2/
cat .ggen-v2/receipt.json
```

Real output:

```
receipt-log.jsonl
receipt.json
```

```json
{
  "record": {
    "version": 1,
    "instruction_id": 0,
    "activity_idx": 0,
    "activity": "ggen.sync",
    "node_kind": 0,
    "ts_ns": 0,
    "payload_hash_hex": "173c6e4baea9c32fac169342c624cb0adfe7bccf567cc10f57e3602a328ddf43",
    "prev_chain_hash_hex": "0000000000000000000000000000000000000000000000000000000000000000",
    "chain_hash_hex": "d8043f1ccd02d214476c15e3c4f4e216590c173b2d2995c6ec94f3448e5d2110",
    "andon": "Green",
    "obligation_count": 0,
    "object_ids": [
      "law:173c6e4baea9c32f"
    ]
  },
  "payload": {
    "graph_hash": "e9c523723d4429529607fbef6908f379c7ae732ffd41c70e2b7746ce9b3ac7ea",
    "outputs": {
      "greeters.txt": "ba26b1c188731d895c7c468d25378cef95a358f2984d988ee4a010ae619f27fa"
    },
    "packs": {},
    "decisions": {
      "greeters.txt": "written"
    }
  }
}
```

`ggen` writes this file to `<root>/.ggen-v2/receipt.json` after every
non-dry-run sync (`RECEIPT_REL_PATH`, `crates/ggen/src/sync.rs:64-66`, and
the write call at `crates/ggen/src/sync.rs:210`), and appends one line to
`.ggen-v2/receipt-log.jsonl` for every sync you ever run
(`RECEIPT_LOG_REL_PATH`, `crates/ggen/src/sync.rs:68-70`). The `payload`
holds the BLAKE3 hash of the post-enrich graph state plus a BLAKE3 hash of
every output file's bytes (`ReceiptPayload`, `crates/ggen/src/sync.rs:79-90`);
the `record` chains that payload's hash cryptographically, `Green` for a
clean run with no obligations.

## What you built

You built the smallest complete `ggen` project: a manifest declaring where
your ontology and templates live, a 3-triple RDF ontology, and one template
that binds a named SPARQL `SELECT` into a Tera loop. You ran the real
five-stage sync pipeline with `ggen sync run`, watched it generate
`greeters.txt` from your ontology data, and confirmed it recorded a signed,
chained receipt of that run at `.ggen-v2/receipt.json`.

From here, see the how-to guide for adding a second template and a
`construct:` query to enrich your ontology before it's queried.
