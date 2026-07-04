# The Template

A ggen template is a plain text file: a leading YAML frontmatter block delimited by `---` lines, followed by a Tera body (`crates/ggen/src/template.rs:1-9`). The frontmatter key set is closed — it is deserialized into a `#[serde(deny_unknown_fields)]` struct, so any key outside the known vocabulary is a hard parse error, not a silently ignored typo (`crates/ggen/src/template.rs:22-26`).

This chapter documents that closed vocabulary field by field, the two Tera extensions the template body can call (`sparql()` and the `snake_case`/`pascal_case` filters), and walks the one worked example shipped in the repository, `crates/ggen/examples/demo-pack/templates/widget.rs.tmpl`, line by line.

## Parsing: two delimiters, one error space

`Template::parse` (`crates/ggen/src/template.rs:82-113`) requires:

1. The file to start with `---` (`crates/ggen/src/template.rs:83-89`). Missing this is error `FM-TPL-001`.
2. That opening `---` to be on its own line — the byte immediately after it must be `\n` (`crates/ggen/src/template.rs:91-93`). Also `FM-TPL-001`.
3. A closing line that is exactly `---` (nothing else on the line) somewhere after the opening delimiter (`crates/ggen/src/template.rs:94-100`, and the scanning helper `split_closing_delimiter` at `crates/ggen/src/template.rs:118-129`). An unterminated block is `FM-TPL-001`.
4. The text between the two delimiters to deserialize as YAML into `Frontmatter` (`crates/ggen/src/template.rs:101-111`). A YAML syntax error, a missing required field (`to`), or an unrecognized key all produce `FM-TPL-002`, and the error message itself enumerates the closed key set: `to, sparql, construct, inject, before, after, at_line, skip_if, unless_exists, force, when, skip_empty` (`crates/ggen/src/template.rs:104-109`).

The unit test `unknown_frontmatter_key_is_err` demonstrates this directly: frontmatter containing a `vars:` key (not in the vocabulary) is parsed and the test asserts the resulting error string contains `FM-TPL-002` (`crates/ggen/src/template.rs:311-316`). There is no warning path and no "extra keys are ignored" fallback — `deny_unknown_fields` on the struct (`crates/ggen/src/template.rs:26`) makes this a compile-time-enforced contract on the deserializer, not a runtime convention.

Everything after the closing `---` line, verbatim, becomes `Template::body` (`crates/ggen/src/template.rs:66-72`) — the Tera source that gets rendered once the frontmatter has been validated.

## The closed Frontmatter vocabulary

All twelve fields live on one struct (`crates/ggen/src/template.rs:27-63`). Every field except `to` is optional and defaults per its type (`#[serde(default)]`).

| Field | Type | Line | Meaning |
|---|---|---|---|
| `to` | `String` | `crates/ggen/src/template.rs:29` | Output path, relative to the project root. The doc comment notes it is "Tera-renderable" — `to` is itself rendered as a Tera string before use, which is what lets a `to:` value containing `{{ }}` drive one output file per SPARQL result row (see below). |
| `sparql` | `BTreeMap<String, String>` | `crates/ggen/src/template.rs:31-32` | Named SPARQL queries available to the template body. Using a `BTreeMap` gives deterministic (sorted-by-key) iteration order — relevant because the sync driver picks "the first named query in map order" as the row source (`crates/ggen/src/sync.rs:186-191`). |
| `construct` | `Option<String>` | `crates/ggen/src/template.rs:34-35` | An optional CONSTRUCT query whose result feeds the template — declared here; validated elsewhere in the pipeline as a CONSTRUCT/DESCRIBE-shaped query (`crates/ggen/src/sync.rs:140`, `:393`). |
| `inject` | `bool` | `crates/ggen/src/template.rs:37-38` | Inject into an existing file instead of creating a new one. |
| `before` | `Option<String>` | `crates/ggen/src/template.rs:40-41` | Inject before the first line containing this marker. |
| `after` | `Option<String>` | `crates/ggen/src/template.rs:43-44` | Inject after the first line containing this marker. |
| `at_line` | `Option<usize>` | `crates/ggen/src/template.rs:46-47` | Inject at this 1-based line number. |
| `skip_if` | `Option<String>` | `crates/ggen/src/template.rs:49-50` | Skip the write when the existing file already contains this substring. |
| `unless_exists` | `bool` | `crates/ggen/src/template.rs:52-53` | Skip the write entirely when the target file already exists. |
| `force` | `bool` | `crates/ggen/src/template.rs:55-56` | Overwrite an existing, differing file instead of failing closed. |
| `when` | `Option<String>` | `crates/ggen/src/template.rs:58-59` | A SPARQL ASK guard: generate only when the graph satisfies it. |
| `skip_empty` | `bool` | `crates/ggen/src/template.rs:61-62` | Skip the write when the rendered body is empty. |

Two of these fields are exercised directly by the sync driver in ways worth citing precisely, since they explain *why* the vocabulary is shaped this way:

- `when` is evaluated as an ASK query against the deterministic graph before any SELECT/CONSTRUCT extraction happens for that template; a false result routes the template into the "skipped" list with a reason rather than aborting the whole sync (`crates/ggen/src/sync.rs:156-165`).
- `skip_empty` is checked after rendering: if the frontmatter sets it and the rendered body is all whitespace, the write is skipped (`crates/ggen/src/sync.rs:281`).

The `to` field's Tera-renderability is what makes per-row generation possible: the driver checks whether the raw `to` string contains `{{` (`crates/ggen/src/sync.rs:193`) to decide between rendering the template once with all rows in scope, or once per row with that row's fields flattened into context (`crates/ggen/src/sync.rs:194-206` vs. `:207-220`).

## The `sparql()` Tera function

`build_tera` (`crates/ggen/src/template.rs:138-151`) constructs a `Tera` instance bound to an `Arc<DeterministicGraph>` and registers:

- `sparql` as a Tera function, at `crates/ggen/src/template.rs:141-147`. It takes one required string argument, `query`, and is called from a template body as `sparql(query="...")`. If the argument is missing or not a string, it errors with `"sparql() requires a string \`query\` argument"` (`crates/ggen/src/template.rs:144-145`). Otherwise it delegates to `sparql_to_value`.
- `snake_case`, a Tera filter, registered at `crates/ggen/src/template.rs:148`.
- `pascal_case`, a Tera filter, registered at `crates/ggen/src/template.rs:149`.

`sparql_to_value` (`crates/ggen/src/template.rs:154-195`) converts the three possible `oxigraph::sparql::QueryResults` shapes into a Tera `Value`:

- `QueryResults::Boolean(b)` (an ASK query) becomes `Value::Bool(b)` directly (`crates/ggen/src/template.rs:156`).
- `QueryResults::Solutions(solutions)` (a SELECT query) becomes a `Value::Array` of `Value::Object` maps, one per solution row, where each bound variable name maps to its rendered term value (`crates/ggen/src/template.rs:157-170`).
- `QueryResults::Graph(triples)` (a CONSTRUCT/DESCRIBE query) becomes a `Value::Array` of `Value::Object` maps with fixed keys `subject`, `predicate`, `object` (`crates/ggen/src/template.rs:171-193`).

Term rendering (`term_value`, `crates/ggen/src/template.rs:199-205`) is intentionally lossy in a specific way: an RDF literal is rendered as its lexical value only (no datatype/language tag), a named node as its bare IRI string, and anything else (blank nodes, quoted triples) via its `Display` (N-Triples-ish) form. This is why a literal like `"widget_id"^^xsd:string` shows up in a template as plain `widget_id`, not as a typed-literal syntax string.

Note that in the `sparql:` frontmatter block, named queries are *not* called through the `sparql()` Tera function at render time by the template author — the sync driver (`crates/ggen/src/sync.rs:182-185`) resolves each named query up front via the same `sparql_to_value` function and injects the results into the render context under that name and under `results` (`crates/ggen/src/sync.rs:244-250`). The `sparql()` Tera function itself is for ad hoc queries written directly in the template body (as in the parser's own test at `crates/ggen/src/template.rs:274`, `q` holding a raw query string passed to `sparql(query=q)`), separate from the `sparql:` frontmatter map.

## The `snake_case` and `pascal_case` filters

Both are plain Tera filters — `fn(&Value, &HashMap<String, Value>) -> tera::Result<Value>` — registered in `build_tera` at `crates/ggen/src/template.rs:148-149`.

`snake_case_filter` (`crates/ggen/src/template.rs:209-233`) walks the input string character by character: `-`, ` `, and `_` all collapse to a single `_` separator (without duplicating one that's already there), an uppercase letter following a lowercase/digit gets a `_` inserted before it and is itself lowercased, and everything else is copied through as-is. The doc comment's examples — `FooBar`, `foo-bar`, `foo bar` all → `foo_bar` (`crates/ggen/src/template.rs:207`) — are exercised by the `snake_and_pascal_filters` test, which asserts `"FooBarBaz" | snake_case` renders as `foo_bar_baz` (`crates/ggen/src/template.rs:325-333`).

`pascal_case_filter` (`crates/ggen/src/template.rs:237-254`) is the inverse-ish transform: `_`, `-`, and ` ` are treated as separators that arm an "uppercase the next character" flag, and every other character is copied through (uppercased if the flag is armed). The same test asserts `"foo_bar-baz qux" | pascal_case` renders as `FooBarBazQux` (`crates/ggen/src/template.rs:329-333`).

Neither filter takes arguments; both error only if the input `Value` isn't a string (`crates/ggen/src/template.rs:210-212`, `:238-240`).

## Worked example: `widget.rs.tmpl`

The full, unmodified contents of `crates/ggen/examples/demo-pack/templates/widget.rs.tmpl`:

```
---
to: src/widget.rs
sparql:
  props: "SELECT ?label WHERE { ?p <http://www.w3.org/2000/01/rdf-schema#domain> <http://example.com/ontology#Widget> ; <http://www.w3.org/2000/01/rdf-schema#label> ?label } ORDER BY ?label"
---
//! Generated by the demo-pack widget template. Do not edit.

/// Widget precipitated from the pack ontology.
pub struct Widget {
{% for row in results %}    pub {{ row.label | snake_case }}: String,
{% endfor %}}
```

Reading this against the frontmatter contract and the sync driver:

- **Frontmatter parse.** `to: src/widget.rs` satisfies the one required field (`crates/ggen/src/template.rs:29`). `sparql:` declares one named query under the key `props` (`crates/ggen/src/template.rs:31-32`). No other frontmatter keys are present, so every optional field (`inject`, `before`, `after`, `at_line`, `skip_if`, `unless_exists`, `force`, `when`, `skip_empty`, `construct`) takes its `#[serde(default)]` value — `false`/`None` — meaning: not an injection, no ASK guard, write unconditionally as a whole-file create.

- **The SELECT query.** `props` selects `?label` for every `?p` that is both `rdfs:domain` of `<http://example.com/ontology#Widget>` and has an `rdfs:label` — i.e., every RDF property declared to belong to the `Widget` class, ordered by label text. This is exactly a SELECT, so at execution time `sparql_to_value` takes the `QueryResults::Solutions` branch (`crates/ggen/src/template.rs:157-170`): each solution row becomes a `Value::Object` with one key, `label`, holding that property's label as a plain string (via `term_value`'s literal branch, `crates/ggen/src/template.rs:201`).

- **Context wiring.** Because `props` is the only (and therefore first, in `BTreeMap` order) named query producing an array, the sync driver's `results` binding is exactly this array of `{label: "..."}` objects (`crates/ggen/src/sync.rs:186-191`, `:244-250`). The template body iterates it as `{% for row in results %}` — it does not need to say `sparql(query="...")` itself because the named `sparql:` frontmatter entry has already been resolved and bound by the driver before rendering.

- **The `to:` field is static, not per-row.** `to: src/widget.rs` contains no `{{`, so `per_row` is `false` (`crates/ggen/src/sync.rs:193`) and the whole body is rendered exactly once, with all rows visible in the `results` array simultaneously (`crates/ggen/src/sync.rs:207-209`) — this is what allows the `{% for %}` loop to emit one struct field per row into a single output file, rather than one file per row.

- **Row-to-output-line transform.** For each row, `row.label` is piped through the `snake_case` filter (`crates/ggen/src/template.rs:207-233`) before being interpolated as the field name. So an RDF label like `"Widget Id"` or `"WidgetId"` becomes the Rust-legal field name `widget_id`; the type is hardcoded in the template text as `String` for every field, not derived from the graph. The net effect: for every ontology property hung off `ex:Widget` via `rdfs:domain`, the rendered `src/widget.rs` gains one line `pub <snake_case(label)>: String,` inside the `Widget` struct body — e.g., a domain triple pairing property `<...#widgetId>` with `rdfs:label "widgetId"` renders as `    pub widget_id: String,`.

- **Static prose passes through untouched.** The two doc-comment lines (`//! Generated by...` and `/// Widget precipitated...`) and the `pub struct Widget {` / closing `}` lines contain no Tera syntax, so they render verbatim into every generation — they are the fixed "shell" the SPARQL-driven field list is spliced into.

This one file therefore demonstrates the full closed vocabulary's minimal useful subset: a static `to`, one named `sparql` SELECT, and a body that uses `results` (populated by the driver from that named query) together with the `snake_case` filter to turn ontology-declared property labels into Rust struct fields — with no frontmatter key present that isn't in the twelve-entry closed set, and no possibility of a typo'd key (say, `outputs:` instead of `to:`) surviving past `Template::parse`'s `FM-TPL-002` check.
