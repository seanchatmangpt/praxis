//! Hygen-style template parsing and Tera environment construction.
//!
//! A template is a leading `--- yaml ---` frontmatter block followed by a
//! Tera body. The frontmatter key set is closed ([`Frontmatter`] uses
//! `deny_unknown_fields`), so any unrecognized key is a hard error.
//!
//! [`build_tera`] produces a Tera environment with a `sparql(query="…")`
//! function bound to a [`DeterministicGraph`], plus `snake_case` and
//! `pascal_case` filters.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use oxigraph::model::Term;
use oxigraph::sparql::QueryResults;
use serde::Deserialize;
use tera::{Tera, Value};

use crate::error::{AppError, Result};
use crate::graph::DeterministicGraph;

/// Closed frontmatter key set for a ggen template (Hygen semantics).
///
/// Unknown keys are rejected at parse time (`deny_unknown_fields`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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
                     when, skip_empty)."
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
///   ASK → bool; SELECT → array of `{var: value}` objects; CONSTRUCT /
///   DESCRIBE → array of `{subject, predicate, object}` objects.
/// - `snake_case` and `pascal_case` filters.
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
    tera.register_filter("snake_case", snake_case_filter);
    tera.register_filter("pascal_case", pascal_case_filter);
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
                row.insert(
                    "subject".to_string(),
                    Value::String(triple.subject.to_string()),
                );
                row.insert(
                    "predicate".to_string(),
                    Value::String(triple.predicate.to_string()),
                );
                row.insert(
                    "object".to_string(),
                    Value::String(term_value(&triple.object)),
                );
                rows.push(Value::Object(row));
            }
            Ok(Value::Array(rows))
        }
    }
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
    let s = value
        .as_str()
        .ok_or_else(|| tera::Error::msg("snake_case filter requires a string"))?;
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
    let s = value
        .as_str()
        .ok_or_else(|| tera::Error::msg("pascal_case filter requires a string"))?;
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
}
