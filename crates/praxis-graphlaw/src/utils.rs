use std::collections::BTreeMap;

use spargebra::Query;

use crate::sparql::evaluate_plan_and_debug;
use crate::{plan_query_or_refuse, Encoder, Rule, Triple, TripleStore};

pub struct Utils;

impl Utils {
    pub fn decode_triple(triple: &Triple) -> String {
        let s = Encoder::decode(&triple.s.to_encoded()).unwrap();
        let p = Encoder::decode(&triple.p.to_encoded()).unwrap();
        let o = Encoder::decode(&triple.o.to_encoded()).unwrap();
        format!("{} {} {}", s, p, o)
    }
    pub fn decode_rule(rule: &Rule) -> String {
        let body = rule
            .body
            .iter()
            .map(|lit| {
                let s = Self::decode_triple(&lit.pattern);
                if lit.negated {
                    format!("not {{{}}},", s)
                } else {
                    s + ","
                }
            })
            .collect::<String>();
        let head = Self::decode_triple(&rule.head);
        format!("{{{}}}=>{{{}}}", body, head)
    }
    /// Strip a `^^<datatype>` suffix (if present) and the surrounding quotes from a
    /// decoded literal's lexical form -- `"10"^^<http://.../integer>` and `"10"` both
    /// become `10`.
    ///
    /// Previously only stripped quotes when a `^^` suffix was present; an untagged
    /// literal (RDF's own default for a bare quoted value, e.g. `"10"` with no
    /// datatype) fell through the `else` branch unchanged, still wrapped in quote
    /// characters. `Encoder::decode` (encoding.rs) always renders a literal's value as
    /// `"value"`, quoted, regardless of whether it carries a datatype -- so every
    /// caller of this function that immediately does `.parse::<f64>()` on the result
    /// (this crate's SUM/MIN/MAX/AVG accumulators, sparql/accumulators.rs) received a
    /// quote-wrapped string for any untyped numeric literal, which `f64::from_str`
    /// rejects outright, silently falling back to 0 via each caller's own
    /// `.unwrap_or(0.0)`. Confirmed via direct tracing this session: a SUM over three
    /// untagged literal triples computed 0 instead of 6 before this fix.
    pub fn remove_literal_tags(literal: &str) -> String {
        let lexical = literal.split("^^").next().unwrap_or(literal);
        if lexical.len() >= 2 && lexical.starts_with('"') && lexical.ends_with('"') {
            lexical[1..lexical.len() - 1].to_string()
        } else {
            lexical.to_string()
        }
    }
}

fn instantiate_construct_component(
    encoded_pattern: String,
    row: &[crate::sparql::Binding],
) -> Result<Option<String>, String> {
    if encoded_pattern.starts_with("_:") || encoded_pattern.starts_with("[]") {
        return Err(
            "SPARQL CONSTRUCT blank node template refused: provide an explicit deterministic identity law"
                .to_string(),
        );
    }

    if let Some(variable) = encoded_pattern.strip_prefix('?') {
        return Ok(row
            .iter()
            .find(|binding| binding.var == variable)
            .map(|binding| binding.val.clone()));
    }

    Ok(Some(encoded_pattern))
}

impl TripleStore {
    /// Execute SPARQL `CONSTRUCT` as a reversible graph projection.
    ///
    /// `query()` historically returned the template-variable bindings for a
    /// CONSTRUCT but did not instantiate the template. This method closes
    /// that semantic gap without granting the query any mutation or DO
    /// authority: it returns a canonical, deduplicated candidate graph and
    /// leaves the source law state unchanged.
    ///
    /// A template triple containing an unbound variable is omitted, matching
    /// SPARQL CONSTRUCT semantics. Anonymous blank-node templates are refused
    /// because deterministic identity is a prerequisite for receipts/replay.
    #[allow(deprecated)]
    pub fn construct(&self, query_str: &str) -> Result<Vec<Triple>, String> {
        let query = Query::parse(query_str, None)
            .map_err(|err| format!("Unable to parse CONSTRUCT query: {}", err))?;

        let template = match &query {
            Query::Construct { template, .. } => template,
            _ => {
                return Err(
                    "TripleStore::construct requires a SPARQL CONSTRUCT query".to_string(),
                )
            }
        };

        let plan = plan_query_or_refuse(&query, &self.triple_index)?;
        let rows: Vec<Vec<crate::sparql::Binding>> =
            evaluate_plan_and_debug(&plan, &self.triple_index).collect();

        let mut canonical = BTreeMap::<String, Triple>::new();
        for row in &rows {
            for pattern in template {
                let Some(subject) =
                    instantiate_construct_component(pattern.subject.to_string(), row)?
                else {
                    continue;
                };
                let Some(predicate) =
                    instantiate_construct_component(pattern.predicate.to_string(), row)?
                else {
                    continue;
                };
                let Some(object) = instantiate_construct_component(pattern.object.to_string(), row)?
                else {
                    continue;
                };

                let triple = Triple::from(subject, predicate, object);
                canonical.insert(TripleStore::decode_triple(&triple), triple);
            }
        }

        Ok(canonical.into_values().collect())
    }
}

#[cfg(test)]
mod test {
    use crate::utils::Utils;

    #[test]
    fn test_remove_literal_tages() {
        let literal = "\"10\"^^<http://www.w3.org/2001/XMLSchema#integer>";
        let expected = "10".to_string();
        assert_eq!(expected, Utils::remove_literal_tags(literal));
    }

    // Regression: an untagged (no `^^datatype`) literal previously passed through
    // unchanged, still wrapped in quotes -- silently unparseable as a number by any
    // caller that immediately does `.parse::<f64>()` (this crate's SUM/MIN/MAX/AVG
    // accumulators), which is exactly the shape `Encoder::decode` produces for a plain
    // literal with no datatype.
    #[test]
    fn remove_literal_tags_strips_quotes_from_untagged_literal() {
        assert_eq!(Utils::remove_literal_tags("\"10\""), "10");
        assert_eq!(Utils::remove_literal_tags("\"3.5\""), "3.5");
    }

    #[test]
    fn remove_literal_tags_leaves_non_quoted_input_unchanged() {
        // Not every caller passes a quoted literal (e.g. an already-bare IRI string);
        // this function must not strip characters from input it doesn't recognize as
        // a quoted lexical form.
        assert_eq!(
            Utils::remove_literal_tags("http://example.org/foo"),
            "http://example.org/foo"
        );
    }
}
