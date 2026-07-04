# Self-Hosting

Praxis generates part of its own CLI. The `ggen` crate's noun-verb command
routing is not hand-written Rust — it is a SPARQL projection of an RDF
ontology, rendered through Tera templates by `ggen sync`, the same tool the
generated code belongs to. This chapter walks through that loop precisely:
the ontology class and instances, the generation rules that query them, the
templates that render the query results, and the generated files that
result — including the comment header that tells a human never to hand-edit
them.

## The ontology: `praxis:CliCommand`

The class definition lives at `schema/praxis.ttl:43-45`:

```turtle
praxis:CliCommand a rdfs:Class ;
    rdfs:label "CliCommand" ;
    rdfs:comment "A noun-verb CLI command definition." .
```

Five properties describe a `CliCommand` instance:

- `praxis:noun` (`schema/praxis.ttl:134-137`) — the CLI noun, `xsd:string`, domain `praxis:CliCommand`.
- `praxis:verb` (`schema/praxis.ttl:139-142`) — the CLI verb, `xsd:string`, domain `praxis:CliCommand`.
- `praxis:handler` (`schema/praxis.ttl:144-147`) — the name of the hand-written Rust function the generated route dispatches to.
- `praxis:inCrate` (`schema/praxis.ttl:149-153`) — "Which workspace crate a CLI command's noun-verb route is generated into," range `praxis:RustCrate`.
- `praxis:flag` (`schema/praxis.ttl:155-159`) — "An optional boolean CLI flag (snake_case name) exposed by the command and passed to its handler."

The `ggen` crate itself is a `praxis:RustCrate` instance at
`schema/praxis.ttl:255-257`, named `"ggen"` and described as "SPARQL-in-Tera
code generation: ggen sync as the single verb." Every `CmdGgen*` instance
below points at this crate via `praxis:inCrate`.

### The five `CmdGgen*` instances

`schema/praxis.ttl:259-294` defines five `praxis:CliCommand` instances, all
with `praxis:inCrate praxis:CrateGgen`:

| Instance | Lines | noun | verb | handler | flags |
|---|---|---|---|---|---|
| `CmdGgenSyncRun` | `259-266` | `sync` | `run` | `handle_sync_run` | `dry_run`, `watch` |
| `CmdGgenGraphValidate` | `268-273` | `graph` | `validate` | `handle_graph_validate` | none |
| `CmdGgenReceiptVerify` | `275-280` | `receipt` | `verify` | `handle_receipt_verify` | none |
| `CmdGgenReceiptHistory` | `282-287` | `receipt` | `history` | `handle_receipt_history` | none |
| `CmdGgenDoctorRun` | `289-294` | `doctor` | `run` | `handle_doctor` | none |

Each instance also carries an `rdfs:comment` describing its behavior — for
example `CmdGgenSyncRun`'s comment at `schema/praxis.ttl:266`: "Run the
five-stage generation pipeline: resolve, enrich, extract, render, write.
--watch re-runs the pipeline on filesystem changes." That comment string is
not decorative; it is queried out and rendered as the doc comment on the
generated function (see below).

Note that `receipt` is a single noun with two verbs (`verify` and
`history`), both defined as separate `CliCommand` instances that share the
same `inCrate` and `noun` but differ in `verb` and `handler`
(`schema/praxis.ttl:275-287`).

## The generation rules: `ggen.toml`

Five `[[generation.rules]]` blocks in `ggen.toml` consume these ontology
instances, one per output file. The section header at
`ggen.toml:174-177` states the intent directly: "Route files are pure
projections of the ontology (Overwrite); hand logic lives in
`crates/ggen/src/verbs/handlers.rs`."

**`crate-ggen-verbs-mod`** (`ggen.toml:179-194`) queries every distinct
noun in the crate and writes `crates/ggen/src/verbs/mod.rs`:

```sparql
PREFIX praxis: <http://seanchatmangpt.github.io/praxis/schema#>
SELECT DISTINCT ?noun
WHERE {
  ?cmd a praxis:CliCommand ;
       praxis:inCrate ?c ;
       praxis:noun ?noun .
  ?c praxis:name "ggen" .
}
ORDER BY ?noun
```

It renders through `templates/crates/ggen/verbs_mod.rs.tera` with
`mode = "Overwrite"` (`ggen.toml:194`).

**`crate-ggen-verbs-sync`, `-graph`, `-receipt`, `-doctor`**
(`ggen.toml:196-224`, `226-254`, `256-284`, `286-314`) each run the same
query shape filtered to one noun, e.g. the `sync` rule's `FILTER(?noun =
"sync")` at `ggen.toml:214`. Each selects `?noun ?verb ?handler ?comment`
plus a `GROUP_CONCAT`-aggregated `?flags` column (comma-separated), grouped
by `?noun ?verb ?handler ?comment` and ordered by `?verb`
(`ggen.toml:219-220`). Each rule renders through the same template,
`templates/crates/ggen/verbs_noun.rs.tera` (`ggen.toml:222`, `252`, `282`,
`312`), and writes to a different `output_file`: `crates/ggen/src/verbs/sync.rs`
(`ggen.toml:223`), `graph.rs` (`ggen.toml:253`), `receipt.rs`
(`ggen.toml:283`), `doctor.rs` (`ggen.toml:313`) — all with
`mode = "Overwrite"`.

Because `receipt` has two `CliCommand` instances (`verify` and `history`)
sharing one noun, the `crate-ggen-verbs-receipt` rule's `GROUP BY ?noun
?verb ?handler ?comment` (`ggen.toml:279`) yields two result rows, and the
template (below) loops over both to emit two `#[verb(...)]` functions in
the same `receipt.rs`.

## The templates

`templates/crates/ggen/verbs_mod.rs.tera` (15 lines, read in full) declares
the module tree:

```tera
//! Noun-verb command modules for the `ggen` binary.
//!
//! GENERATED by `ggen sync` from `schema/praxis.ttl` (praxis:CliCommand
//! instances with `praxis:inCrate praxis:CrateGgen`). Do not edit by hand —
//! add a `CliCommand` instance to the ontology instead.
//!
//! Each file's stem is a CLI noun; each `#[verb]` fn inside is a verb under
//! that noun.

/// Hand-written handler implementations (not generated).
pub mod handlers;

{% for row in results -%}
pub mod {{ row.noun }};
{% endfor -%}
```

(`templates/crates/ggen/verbs_mod.rs.tera:1-15`). It loops over the
`?noun` rows from the mod-list query and emits one `pub mod <noun>;` line
per distinct noun, plus a hand-maintained `pub mod handlers;` line
(line 11) that is not generated — it exists in the template source itself,
not derived from a query row.

`templates/crates/ggen/verbs_noun.rs.tera` (15 lines, read in full)
renders one function per `CliCommand` row:

```tera
//! `{{ results.0.noun }}` noun — routes GENERATED by `ggen sync` from
//! `schema/praxis.ttl` (`praxis:CliCommand` instances). Do not edit by hand:
//! routes are a pure projection of the ontology (mode = Overwrite); logic
//! lives in `crate::verbs::handlers`.

use clap_noun_verb::Result;

{% for row in results %}
{% set flags = row.flags | default(value="") | split(pat=",") %}
/// {{ row.comment | default(value="Generated route.") }}
#[clap_noun_verb_macros::verb("{{ row.verb }}", "{{ row.noun }}")]
fn {{ row.noun }}_{{ row.verb }}({% for f in flags %}{% if f %}{{ f }}: bool{% if not loop.last %}, {% endif %}{% endif %}{% endfor %}) -> Result<serde_json::Value> {
    crate::verbs::handlers::{{ row.handler }}({% for f in flags %}{% if f %}{{ f }}{% if not loop.last %}, {% endif %}{% endif %}{% endfor %})
}
{% endfor %}
```

(`templates/crates/ggen/verbs_noun.rs.tera:1-15`). Each row's `?flags`
column (a comma-joined string from `GROUP_CONCAT`) is split back into a
list on line 9, filtered for non-empty entries, and used twice: once as
`bool` parameters in the function signature (line 12) and once as the
plain identifiers forwarded into the handler call (line 13). This is why
`CmdGgenSyncRun`'s two flags (`dry_run`, `watch`, `schema/praxis.ttl:264-265`)
turn into `fn sync_run(dry_run: bool, watch: bool)` calling
`handle_sync_run(dry_run, watch)` — visible in the generated output below —
while the flagless commands (`graph validate`, `receipt verify`, `receipt
history`, `doctor run`) generate zero-argument functions.

## The generated output

Reading the actual generated file confirms the template's contract. The
first four lines of `crates/ggen/src/verbs/sync.rs` are:

```
1	//! `sync` noun — routes GENERATED by `ggen sync` from
2	//! `schema/praxis.ttl` (`praxis:CliCommand` instances). Do not edit by hand:
3	//! routes are a pure projection of the ontology (mode = Overwrite); logic
4	//! lives in `crate::verbs::handlers`.
```

(`crates/ggen/src/verbs/sync.rs:1-4`). That header is the load-bearing
sentence in this whole chapter: **do not edit by hand**. Line 2 names the
source of truth (`schema/praxis.ttl`), line 3 names the mechanism (pure
projection, `mode = "Overwrite"`), and line 4 names where actual logic
belongs instead (`crate::verbs::handlers`) — which is exactly the one
`pub mod` in `verbs_mod.rs.tera` that is *not* templated from a query row.

The body of the same file matches the ontology and query exactly:

```
9	/// Run the five-stage generation pipeline: resolve, enrich, extract, render, write. --watch re-runs the pipeline on filesystem changes.
10	#[clap_noun_verb_macros::verb("run", "sync")]
11	fn sync_run(dry_run: bool, watch: bool) -> Result<serde_json::Value> {
12	    crate::verbs::handlers::handle_sync_run(dry_run, watch)
13	}
```

(`crates/ggen/src/verbs/sync.rs:9-13`). The doc comment on line 9 is the
`rdfs:comment` literal from `schema/praxis.ttl:266`, word for word. The
function name `sync_run` is `{{ row.noun }}_{{ row.verb }}` with `noun =
"sync"`, `verb = "run"` (`schema/praxis.ttl:261-263`). The parameter list
and call-site arguments are the two `praxis:flag` values
(`schema/praxis.ttl:264-265`) rendered as `bool` parameters and forwarded
verbatim to `handle_sync_run`, the `praxis:handler` literal
(`schema/praxis.ttl:263`).

Because `mode = "Overwrite"` for every one of these five rules
(`ggen.toml:194`, `224`, `254`, `284`, `314`), every `ggen sync` run
replaces the file's contents completely rather than skipping it — unlike
`mode = "Create"`, which this project's root `CLAUDE.md` reserves for
bootstrap scaffolds meant to be hand-completed after first generation.
These verb files carry no such hand-completed logic; they are pure routing
shims, so full overwrite is correct and expected on every sync.

## The dogfood loop

Putting the pieces together, the loop for changing the `ggen` CLI's surface
is:

1. **Edit `schema/praxis.ttl`** — add or modify a `praxis:CliCommand`
   instance with `praxis:inCrate praxis:CrateGgen`, a `praxis:noun`, a
   `praxis:verb`, a `praxis:handler` naming an existing (or soon-to-exist)
   function in `crates/ggen/src/verbs/handlers.rs`, optional `praxis:flag`
   values, and an optional `rdfs:comment`.
2. **Edit `ggen.toml`** if the noun is new — an entirely new noun needs its
   own `[[generation.rules]]` block (mirroring the four `crate-ggen-verbs-*`
   blocks at `ggen.toml:196-314`) with a `FILTER(?noun = "...")` clause and
   an `output_file` under `crates/ggen/src/verbs/`. An existing noun with a
   new verb needs no new rule — the existing rule's query and `GROUP BY`
   already pick up any row matching that noun.
3. **Edit the `.tera` templates** only if the shape of the generated code
   itself needs to change (e.g., a new column consumed by the template) —
   day-to-day command additions do not touch
   `templates/crates/ggen/verbs_noun.rs.tera` or `verbs_mod.rs.tera` at all.
4. **Run `ggen sync`** from the project root. The binary invoked here is
   the already-installed one on `PATH` — at the time of writing,
   `ggen --version` reports `ggen 26.7.2` — not a from-source rebuild of
   the very crate being regenerated. It reads `ggen.toml`, executes each
   rule's SPARQL query against the loaded `schema/praxis.ttl` graph, renders
   the matching Tera template with the query's result rows bound to
   `results`, and overwrites the target files under
   `crates/ggen/src/verbs/`.
5. **Never hand-edit** `crates/ggen/src/verbs/{mod,sync,graph,receipt,doctor}.rs`.
   Any change made directly to these files is silently discarded on the
   next sync (`mode = "Overwrite"`) and, more importantly, is undocumented
   in the one place — the ontology — that every other tool and template in
   this project treats as the source of truth. The generated header
   comment exists precisely to make this a self-enforcing rule rather than
   a policy someone has to remember: opening `sync.rs` immediately tells
   you where to make the change instead.

This is what "self-hosting" means concretely here: the tool that generates
Rust code from RDF ontologies uses that same mechanism to generate its own
command-line routing, and the boundary between "ontology-derived" and
"hand-written" is drawn at the module level — `handlers.rs` is hand-written
and never regenerated; every other file under `crates/ggen/src/verbs/` is
regenerated in full on every `ggen sync` and never hand-edited.
