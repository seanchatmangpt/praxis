//! Hygen-style template parsing and Tera environment construction.
//!
//! A template is a leading `--- yaml ---` frontmatter block followed by a
//! Tera body. The frontmatter key set is closed ([`Frontmatter`] uses
//! `deny_unknown_fields`), so any unrecognized key is a hard error.
//!
//! [`build_tera`] produces a Tera environment with a `sparql(query="…")`
//! function bound to a [`DeterministicGraph`], plus `snake_case` and
//! `pascal_case` filters.

use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use oxigraph::{model::Term, sparql::QueryResults};
use schemars::JsonSchema;
use serde::Deserialize;
use tera::{Tera, Value};

use crate::{
    error::{AppError, Result},
    graph::DeterministicGraph,
};

/// Closed frontmatter key set for a ggen template (Hygen semantics).
///
/// Unknown keys are rejected at parse time (`deny_unknown_fields`).
///
/// `#[derive(JsonSchema)]` is load-bearing: it lets
/// `tests/frontmatter_schema_match.rs` compare this struct's *actual* field
/// set (via `schemars::schema_for!`) against `schema/frontmatter-schema.ttl`,
/// instead of a hand-maintained mirror list that could itself drift.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Frontmatter {
    /// Output path relative to the project root (Tera-renderable).
    pub to: String,
    /// Named SPARQL queries available to the template body.
    #[serde(default)]
    pub sparql: BTreeMap<String, String>,
    /// Optional CONSTRUCT query whose result feeds the template.
    #[serde(default)]
    pub construct: Option<String>,
    /// Inject into an existing file instead of creating a new one.
    #[serde(default)]
    pub inject: bool,
    /// Inject before the first line containing this marker.
    #[serde(default)]
    pub before: Option<String>,
    /// Inject after the first line containing this marker.
    #[serde(default)]
    pub after: Option<String>,
    /// Inject at this 1-based line number.
    #[serde(default)]
    pub at_line: Option<usize>,
    /// Skip the write when the existing file already contains this substring.
    #[serde(default)]
    pub skip_if: Option<String>,
    /// Skip the write entirely when the target file already exists.
    #[serde(default)]
    pub unless_exists: bool,
    /// Overwrite an existing, differing file instead of failing closed.
    #[serde(default)]
    pub force: bool,
    /// SPARQL ASK guard: generate only when the graph satisfies it.
    #[serde(default)]
    pub when: Option<String>,
    /// Skip the write when the rendered body is empty.
    #[serde(default)]
    pub skip_empty: bool,
    /// Load the Tera body from this path instead (relative to the template
    /// file's own directory); frontmatter fields still come from this file.
    #[serde(default)]
    pub from: Option<String>,
    /// Shell command run before the write decision. Refused (not executed)
    /// if it matches [`crate::shell_safety::check_shell_command_safe`]'s
    /// denylist. Runs with the project root as its working directory.
    #[serde(default, alias = "sh")]
    pub sh_before: Option<String>,
    /// Shell command run after a successful `Written`/`Injected` outcome
    /// (never after `Skipped`). Same denylist and working directory as
    /// `sh_before`.
    #[serde(default)]
    pub sh_after: Option<String>,
    /// Before overwriting an existing file (`force` or `inject`), copy it to
    /// `<path>.bak` first.
    #[serde(default)]
    pub backup: bool,
    /// SHACL shape file paths (relative to the project root) declared as
    /// governing this output. **Existence-checked only** — no SHACL engine
    /// runs in this crate yet, so listed shapes are not evaluated against
    /// the rendered output; see `docs/v26.7.4/GGEN_TOML_SCHEMA_MAPPING.md`.
    #[serde(default)]
    pub shape: Vec<String>,
    /// When `true`, the sync pipeline renders this template's body twice
    /// with identical inputs and refuses if the bytes differ (a real,
    /// enforced determinism assertion, not a declared-but-unchecked claim).
    #[serde(default)]
    pub determinism: Option<bool>,
    /// Freeze policy for this output once written; see [`FreezePolicy`].
    /// Defaults to `never` (no freeze behavior) when absent.
    #[serde(default)]
    pub freeze_policy: Option<FreezePolicy>,
    /// Directory (relative to the project root) storing per-output BLAKE3
    /// checksums for `freeze_policy: checksum`. Required when that policy
    /// is set; ignored otherwise.
    #[serde(default)]
    pub freeze_slots_dir: Option<String>,
}

/// Freeze policy for a frontmatter's output, once it has been written once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FreezePolicy {
    /// Never skip on freeze grounds; the normal write decision table applies
    /// unchanged. Equivalent to omitting `freeze_policy` entirely.
    Never,
    /// Once the target exists, always skip regeneration — a one-time
    /// scaffold that is never touched again by `ggen sync`.
    Always,
    /// Skip regeneration only when the target's on-disk content no longer
    /// matches the checksum ggen recorded the last time it wrote this file
    /// (i.e. a human has edited it since); otherwise proceed normally and
    /// record the new checksum. Requires `freeze_slots_dir`.
    Checksum,
}

/// A parsed template: validated frontmatter plus the raw Tera body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    /// Parsed and validated frontmatter block.
    pub frontmatter: Frontmatter,
    /// Tera template body (everything after the closing `---`).
    pub body: String,
}

impl Template {
    /// Parse a template file: a leading `---` YAML `---` block, then the body.
    ///
    /// # Errors
    /// - `[FM-TPL-001]` when the leading frontmatter block is missing or
    ///   unterminated.
    /// - `[FM-TPL-002]` when the YAML fails to deserialize, including any
    ///   unknown frontmatter key (closed key set, fail closed).
    pub fn parse(content: &str) -> Result<Self> {
        let rest = content.strip_prefix("---").ok_or_else(|| {
            AppError::fm_tpl(
                1,
                "template must start with a `---` frontmatter block. \
                 Remediation: begin the file with `---`, YAML keys, `---`.",
            )
        })?;
        // The opening delimiter must be its own line.
        let rest = rest.strip_prefix('\n').ok_or_else(|| {
            AppError::fm_tpl(1, "`---` frontmatter delimiter must be on its own line")
        })?;
        let (yaml, body) = split_closing_delimiter(rest).ok_or_else(|| {
            AppError::fm_tpl(
                1,
                "unterminated frontmatter: no closing `---` line found. \
                 Remediation: close the YAML block with a `---` line.",
            )
        })?;
        let frontmatter: Frontmatter = serde_yaml::from_str(yaml).map_err(|e| {
            AppError::fm_tpl(
                2,
                format!(
                    "frontmatter rejected: {e}. \
                     Remediation: use only the closed key set (to, sparql, construct, \
                     inject, before, after, at_line, skip_if, unless_exists, force, \
                     when, skip_empty, from, sh_before, sh_after, backup, shape, \
                     determinism, freeze_policy, freeze_slots_dir)."
                ),
            )
        })?;
        Ok(Self { frontmatter, body: body.to_string() })
    }
}

/// Split `rest` at the first line that is exactly `---`, returning
/// `(yaml, body)`. The body excludes the delimiter line itself.
fn split_closing_delimiter(rest: &str) -> Option<(&str, &str)> {
    let mut offset = 0usize;
    for line in rest.split_inclusive('\n') {
        if line.trim_end_matches(['\r', '\n']) == "---" {
            let yaml = &rest[..offset];
            let body = &rest[offset + line.len()..];
            return Some((yaml, body));
        }
        offset += line.len();
    }
    None
}

/// Build a Tera environment bound to `graph`.
///
/// Registers:
/// - `sparql(query="…")` — executes the query against the graph.
///   ASK → bool; SELECT → array of `{var: value}` objects (bare variable
///   names, e.g. `name`, never `?name` — see
///   `sparql_row_keys_are_bare_not_question_mark_prefixed` for why that's a
///   pinned invariant, not an accident); CONSTRUCT / DESCRIBE → array of
///   `{subject, predicate, object}` objects.
/// - `local(iri="…")` — the local name/fragment of an IRI (after the last
///   `#` or `/`).
/// - `sparql_first(rows=…)` — the first row of a SELECT result array, or
///   `null` if empty.
/// - `sparql_values(rows=…, column="…")` — the array of one column's values
///   across every row.
/// - `sparql_empty(rows=…)` — `true` if the array is empty.
/// - `sparql_count(rows=…)` — the number of rows.
/// - `snake_case`, `pascal_case`, `camel_case`, `kebab_case`,
///   `shouty_snake_case`, `title_case`, `pluralize`, `singularize` filters.
#[must_use]
pub fn build_tera(graph: Arc<DeterministicGraph>) -> Tera {
    let mut tera = Tera::default();
    tera.register_function("sparql", move |args: &HashMap<String, Value>| {
        let query = args
            .get("query")
            .and_then(Value::as_str)
            .ok_or_else(|| tera::Error::msg("sparql() requires a string `query` argument"))?;
        sparql_to_value(&graph, query).map_err(|e| tera::Error::msg(e.to_string()))
    });
    tera.register_function("local", local_fn);
    tera.register_function("sparql_first", sparql_first_fn);
    tera.register_function("sparql_values", sparql_values_fn);
    tera.register_function("sparql_empty", sparql_empty_fn);
    tera.register_function("sparql_count", sparql_count_fn);
    tera.register_filter("snake_case", snake_case_filter);
    tera.register_filter("pascal_case", pascal_case_filter);
    tera.register_filter("camel_case", camel_case_filter);
    tera.register_filter("kebab_case", kebab_case_filter);
    tera.register_filter("shouty_snake_case", shouty_snake_case_filter);
    tera.register_filter("title_case", title_case_filter);
    tera.register_filter("pluralize", pluralize_filter);
    tera.register_filter("singularize", singularize_filter);
    tera
}

/// Execute `query` and convert the results into a Tera [`Value`].
pub(crate) fn sparql_to_value(graph: &DeterministicGraph, query: &str) -> Result<Value> {
    match graph.query(query)? {
        QueryResults::Boolean(b) => Ok(Value::Bool(b)),
        QueryResults::Solutions(solutions) => {
            let mut rows = Vec::new();
            for solution in solutions {
                let solution = solution.map_err(|e| {
                    AppError::fm_tpl(3, format!("SELECT solution iteration failed: {e}"))
                })?;
                let mut row = tera::Map::new();
                for (var, term) in solution.iter() {
                    row.insert(var.as_str().to_string(), Value::String(term_value(term)));
                }
                rows.push(Value::Object(row));
            }
            Ok(Value::Array(rows))
        }
        QueryResults::Graph(triples) => {
            let mut rows = Vec::new();
            for triple in triples {
                let triple = triple.map_err(|e| {
                    AppError::fm_tpl(3, format!("CONSTRUCT triple iteration failed: {e}"))
                })?;
                let mut row = tera::Map::new();
                row.insert("subject".to_string(), Value::String(triple.subject.to_string()));
                row.insert("predicate".to_string(), Value::String(triple.predicate.to_string()));
                row.insert("object".to_string(), Value::String(term_value(&triple.object)));
                rows.push(Value::Object(row));
            }
            Ok(Value::Array(rows))
        }
    }
}

/// `local(iri="…")` — the local name/fragment of an IRI: the substring
/// after the last `#`, or after the last `/` if there is no `#`, or the
/// whole string if neither separator is present.
fn local_fn(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let iri = args
        .get("iri")
        .and_then(Value::as_str)
        .ok_or_else(|| tera::Error::msg("local() requires a string `iri` argument"))?;
    let local = iri
        .rsplit_once('#')
        .map(|(_, frag)| frag)
        .or_else(|| iri.rsplit_once('/').map(|(_, seg)| seg))
        .unwrap_or(iri);
    Ok(Value::String(local.to_string()))
}

/// Extract the `rows` argument (a SELECT result array) from `args`, erroring
/// with `fn_name` in the message if it's missing or not an array.
fn rows_arg<'a>(args: &'a HashMap<String, Value>, fn_name: &str) -> tera::Result<&'a Vec<Value>> {
    args.get("rows")
        .and_then(Value::as_array)
        .ok_or_else(|| tera::Error::msg(format!("{fn_name}() requires an array `rows` argument")))
}

/// `sparql_first(rows=…)` — the first row, or `null` if `rows` is empty.
fn sparql_first_fn(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let rows = rows_arg(args, "sparql_first")?;
    Ok(rows.first().cloned().unwrap_or(Value::Null))
}

/// `sparql_values(rows=…, column="…")` — the array of `column`'s value
/// across every row (missing/non-object rows contribute `null`).
fn sparql_values_fn(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let rows = rows_arg(args, "sparql_values")?;
    let column = args
        .get("column")
        .and_then(Value::as_str)
        .ok_or_else(|| tera::Error::msg("sparql_values() requires a string `column` argument"))?;
    let values: Vec<Value> = rows
        .iter()
        .map(|row| row.as_object().and_then(|o| o.get(column)).cloned().unwrap_or(Value::Null))
        .collect();
    Ok(Value::Array(values))
}

/// `sparql_empty(rows=…)` — `true` if `rows` has no elements.
fn sparql_empty_fn(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let rows = rows_arg(args, "sparql_empty")?;
    Ok(Value::Bool(rows.is_empty()))
}

/// `sparql_count(rows=…)` — the number of rows.
fn sparql_count_fn(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let rows = rows_arg(args, "sparql_count")?;
    Ok(Value::Number(rows.len().into()))
}

/// Render a term as a plain value string: literals as their lexical form,
/// IRIs as the bare IRI, blank nodes / quoted triples in their N-Triples form.
fn term_value(term: &Term) -> String {
    match term {
        Term::Literal(lit) => lit.value().to_string(),
        Term::NamedNode(n) => n.as_str().to_string(),
        other => other.to_string(),
    }
}

/// `snake_case` filter: `FooBar`, `foo-bar`, `foo bar` → `foo_bar`.
#[allow(clippy::unnecessary_wraps)]
fn snake_case_filter(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let s =
        value.as_str().ok_or_else(|| tera::Error::msg("snake_case filter requires a string"))?;
    let mut out = String::with_capacity(s.len() + 4);
    let mut prev_lower = false;
    for ch in s.chars() {
        if ch == '-' || ch == ' ' || ch == '_' {
            if !out.ends_with('_') && !out.is_empty() {
                out.push('_');
            }
            prev_lower = false;
        } else if ch.is_uppercase() {
            if prev_lower && !out.ends_with('_') {
                out.push('_');
            }
            out.extend(ch.to_lowercase());
            prev_lower = false;
        } else {
            out.push(ch);
            prev_lower = ch.is_lowercase() || ch.is_ascii_digit();
        }
    }
    Ok(Value::String(out))
}

/// `pascal_case` filter: `foo_bar`, `foo-bar`, `foo bar` → `FooBar`.
#[allow(clippy::unnecessary_wraps)]
fn pascal_case_filter(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let s =
        value.as_str().ok_or_else(|| tera::Error::msg("pascal_case filter requires a string"))?;
    let mut out = String::with_capacity(s.len());
    let mut upper_next = true;
    for ch in s.chars() {
        if ch == '_' || ch == '-' || ch == ' ' {
            upper_next = true;
        } else if upper_next {
            out.extend(ch.to_uppercase());
            upper_next = false;
        } else {
            out.push(ch);
        }
    }
    Ok(Value::String(out))
}

/// Split `s` into lowercase words on `_`/`-`/` ` separators and
/// lower-to-upper case boundaries (shared by the case/inflection filters
/// below, so each filter only decides how to rejoin the words).
fn split_words(s: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut prev_lower = false;
    for ch in s.chars() {
        if ch == '_' || ch == '-' || ch == ' ' {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            prev_lower = false;
        } else if ch.is_uppercase() && prev_lower {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            current.extend(ch.to_lowercase());
            prev_lower = false;
        } else {
            current.extend(ch.to_lowercase());
            prev_lower = ch.is_lowercase() || ch.is_ascii_digit();
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

/// `camel_case` filter: `foo_bar`, `FooBar`, `foo-bar` → `fooBar`
/// (lowerCamelCase — first word lowercase, subsequent words capitalized, no
/// separators).
#[allow(clippy::unnecessary_wraps)]
fn camel_case_filter(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let s =
        value.as_str().ok_or_else(|| tera::Error::msg("camel_case filter requires a string"))?;
    let words = split_words(s);
    let mut out = String::with_capacity(s.len());
    for (i, word) in words.iter().enumerate() {
        if i == 0 {
            out.push_str(word);
        } else {
            let mut chars = word.chars();
            if let Some(first) = chars.next() {
                out.extend(first.to_uppercase());
                out.push_str(chars.as_str());
            }
        }
    }
    Ok(Value::String(out))
}

/// `kebab_case` filter: `foo_bar`, `FooBar`, `foo bar` → `foo-bar`.
#[allow(clippy::unnecessary_wraps)]
fn kebab_case_filter(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let s =
        value.as_str().ok_or_else(|| tera::Error::msg("kebab_case filter requires a string"))?;
    Ok(Value::String(split_words(s).join("-")))
}

/// `shouty_snake_case` filter: `foo_bar`, `FooBar`, `foo-bar` → `FOO_BAR`.
#[allow(clippy::unnecessary_wraps)]
fn shouty_snake_case_filter(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let s = value
        .as_str()
        .ok_or_else(|| tera::Error::msg("shouty_snake_case filter requires a string"))?;
    Ok(Value::String(split_words(s).join("_").to_uppercase()))
}

/// `title_case` filter: `foo_bar`, `FooBar`, `foo-bar` → `Foo Bar`.
#[allow(clippy::unnecessary_wraps)]
fn title_case_filter(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let s =
        value.as_str().ok_or_else(|| tera::Error::msg("title_case filter requires a string"))?;
    let titled: Vec<String> = split_words(s)
        .into_iter()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => word,
            }
        })
        .collect();
    Ok(Value::String(titled.join(" ")))
}

/// `pluralize` filter: naive English pluralization (`bus`→`buses`,
/// `city`→`cities`, `cat`→`cats`). Operates on the whole input string
/// as-is (no word-splitting) since it's meant for a single noun, not a
/// multi-word identifier.
#[allow(clippy::unnecessary_wraps)]
fn pluralize_filter(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let s = value.as_str().ok_or_else(|| tera::Error::msg("pluralize filter requires a string"))?;
    Ok(Value::String(pluralize_word(s)))
}

fn pluralize_word(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }
    let lower = s.to_ascii_lowercase();
    if lower.ends_with('y')
        && !lower.ends_with("ay")
        && !lower.ends_with("ey")
        && !lower.ends_with("oy")
        && !lower.ends_with("uy")
    {
        format!("{}ies", &s[..s.len() - 1])
    } else if lower.ends_with('s')
        || lower.ends_with('x')
        || lower.ends_with('z')
        || lower.ends_with("ch")
        || lower.ends_with("sh")
    {
        format!("{s}es")
    } else {
        format!("{s}s")
    }
}

/// `singularize` filter: reverse of [`pluralize_word`]'s heuristic
/// (`buses`→`bus`, `cities`→`city`, `cats`→`cat`). Best-effort, not a
/// dictionary — irregular plurals (`people`, `children`) are unaffected.
#[allow(clippy::unnecessary_wraps)]
fn singularize_filter(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let s =
        value.as_str().ok_or_else(|| tera::Error::msg("singularize filter requires a string"))?;
    let lower = s.to_ascii_lowercase();
    let out = if lower.ends_with("ies") && s.len() > 3 {
        format!("{}y", &s[..s.len() - 3])
    } else if lower.ends_with("ches") || lower.ends_with("shes") {
        s[..s.len() - 2].to_string()
    } else if (lower.ends_with("ses") || lower.ends_with("xes") || lower.ends_with("zes"))
        && s.len() > 3
    {
        s[..s.len() - 2].to_string()
    } else if lower.ends_with('s') && !lower.ends_with("ss") {
        s[..s.len() - 1].to_string()
    } else {
        s.to_string()
    };
    Ok(Value::String(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TTL: &str = r#"
        @prefix ex: <http://example.org/> .
        ex:alice ex:name "alice_smith" .
        ex:bob   ex:name "bob_jones" .
    "#;

    fn graph() -> Arc<DeterministicGraph> {
        let g = DeterministicGraph::new().expect("graph");
        g.insert_turtle(TTL).expect("ttl");
        Arc::new(g)
    }

    #[test]
    fn sparql_row_keys_are_bare_not_question_mark_prefixed() {
        // Direct regression guard for a confirmed bug in ~/ggen's rendering
        // engine (ggen-core/src/pipeline.rs builds row keys with
        // `var.to_string()`, whose `Display` impl is `write!(f, "?{name}")`,
        // forcing template authors to write `row["?name"]`). This crate's
        // `sparql_to_value` must keep using `Variable::as_str()` (bare name,
        // no `?`), so `row.name` works directly. If this test ever fails
        // after touching `sparql_to_value`, that's this exact regression.
        let value = sparql_to_value(
            &graph(),
            "SELECT ?name WHERE { ?s <http://example.org/name> ?name } ORDER BY ?name",
        )
        .expect("query");
        let rows = value.as_array().expect("array");
        let first = rows[0].as_object().expect("object");
        assert!(first.contains_key("name"), "row must have bare key `name`: {first:?}");
        assert!(
            !first.contains_key("?name"),
            "row must NOT have `?`-prefixed key `?name`: {first:?}"
        );
    }

    #[test]
    fn parse_and_render_with_sparql_against_graph() {
        let content = "---\nto: out.rs\nsparql:\n  people: SELECT ?name WHERE { ?s <http://example.org/name> ?name } ORDER BY ?name\n---\n{% for row in sparql(query=q) %}{{ row.name | pascal_case }};{% endfor %}";
        let tpl = Template::parse(content).expect("parse");
        assert_eq!(tpl.frontmatter.to, "out.rs");
        let q = tpl.frontmatter.sparql.get("people").expect("query").clone();

        let mut tera = build_tera(graph());
        let mut ctx = tera::Context::new();
        ctx.insert("q", &q);
        let rendered = tera.render_str(&tpl.body, &ctx).expect("render");
        assert_eq!(rendered, "AliceSmith;BobJones;");
    }

    #[test]
    fn ask_query_returns_bool() {
        let tera = build_tera(graph());
        let mut t = tera;
        let rendered = t
            .render_str(
                "{% if sparql(query=\"ASK { ?s ?p ?o }\") %}yes{% else %}no{% endif %}",
                &tera::Context::new(),
            )
            .expect("render");
        assert_eq!(rendered, "yes");
    }

    #[test]
    fn construct_returns_triples() {
        let mut tera = build_tera(graph());
        let rendered = tera
            .render_str(
                "{% set ts = sparql(query=\"CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }\") %}{{ ts | length }}",
                &tera::Context::new(),
            )
            .expect("render");
        assert_eq!(rendered, "2");
    }

    #[test]
    fn unknown_frontmatter_key_is_err() {
        let content = "---\nto: out.rs\nvars:\n  x: 1\n---\nbody";
        let err = Template::parse(content).expect_err("must reject `vars:`");
        assert!(err.to_string().contains("FM-TPL-002"), "{err}");
    }

    #[test]
    fn missing_frontmatter_is_err() {
        let err = Template::parse("no frontmatter here").expect_err("must reject");
        assert!(err.to_string().contains("FM-TPL-001"), "{err}");
    }

    #[test]
    fn snake_and_pascal_filters() {
        let mut tera = build_tera(graph());
        let rendered = tera
            .render_str(
                "{{ \"FooBarBaz\" | snake_case }} {{ \"foo_bar-baz qux\" | pascal_case }}",
                &tera::Context::new(),
            )
            .expect("render");
        assert_eq!(rendered, "foo_bar_baz FooBarBazQux");
    }

    #[test]
    fn camel_kebab_shouty_title_filters() {
        let mut tera = build_tera(graph());
        let rendered = tera
            .render_str(
                "{{ \"foo_bar-baz\" | camel_case }} \
                 {{ \"FooBarBaz\" | kebab_case }} \
                 {{ \"foo-bar baz\" | shouty_snake_case }} \
                 {{ \"foo_bar-baz\" | title_case }}",
                &tera::Context::new(),
            )
            .expect("render");
        assert_eq!(rendered, "fooBarBaz foo-bar-baz FOO_BAR_BAZ Foo Bar Baz");
    }

    #[test]
    fn pluralize_and_singularize_filters() {
        let mut tera = build_tera(graph());
        let rendered = tera
            .render_str(
                "{{ \"cat\" | pluralize }} {{ \"city\" | pluralize }} {{ \"bus\" | pluralize }} \
                 {{ \"cats\" | singularize }} {{ \"cities\" | singularize }} {{ \"buses\" | singularize }}",
                &tera::Context::new(),
            )
            .expect("render");
        assert_eq!(rendered, "cats cities buses cat city bus");
    }

    #[test]
    fn local_function_strips_namespace() {
        let mut tera = build_tera(graph());
        let rendered = tera
            .render_str(
                "{{ local(iri=\"http://example.org/name\") }} \
                 {{ local(iri=\"http://example.org/ns#Widget\") }} \
                 {{ local(iri=\"plain\") }}",
                &tera::Context::new(),
            )
            .expect("render");
        assert_eq!(rendered, "name Widget plain");
    }

    #[test]
    fn sparql_first_values_empty_count_functions() {
        let mut tera = build_tera(graph());
        let rendered = tera
            .render_str(
                "{% set rows = sparql(query=\"SELECT ?name WHERE { ?s <http://example.org/name> ?name } ORDER BY ?name\") %}\
                 {% set first = sparql_first(rows=rows) %}\
                 {{ first.name }};\
                 {{ sparql_values(rows=rows, column=\"name\") | join(sep=\",\") }};\
                 {{ sparql_count(rows=rows) }};\
                 {{ sparql_empty(rows=rows) }};\
                 {{ sparql_empty(rows=[]) }}",
                &tera::Context::new(),
            )
            .expect("render");
        assert_eq!(rendered, "alice_smith;alice_smith,bob_jones;2;false;true");
    }

    #[test]
    fn sparql_first_on_empty_rows_is_null() {
        let mut tera = build_tera(graph());
        let rendered =
            tera.render_str("{{ sparql_first(rows=[]) }}", &tera::Context::new()).expect("render");
        assert_eq!(rendered, "", "Tera renders a Value::Null function result as empty output");
    }
}
