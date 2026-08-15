//! Deterministic SPARQL CONSTRUCT projection.
//!
//! `TripleStore::query` historically evaluated a CONSTRUCT WHERE clause and
//! returned the template-variable bindings, but it did not instantiate the
//! template into an RDF graph.  That is sufficient for SELECT-style query
//! inspection, but not for Praxis' CONSTRUCT doctrine: reversible graph
//! manufacture must produce a candidate graph before any planning or DO
//! boundary is crossed.
//!
//! This module keeps CONSTRUCT non-actuating: it returns a canonical,
//! deduplicated `Vec<Triple>` and never mutates the source `TripleStore`.
//! Blank-node templates are refused until the caller supplies an explicit
//! identity law; silently minting process-local blank-node identities would
//! make cross-run receipts unstable.

use std::collections::BTreeMap;

use spargebra::Query;

use crate::sparql::evaluate_plan_and_debug;
use crate::triples::Triple;
use crate::{plan_query_or_refuse, TripleStore};

fn instantiate_component(
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
    /// Execute a SPARQL `CONSTRUCT` as a reversible graph projection.
    ///
    /// The returned graph is sorted by decoded triple text and deduplicated.
    /// The source store is not mutated. A template triple containing an
    /// unbound variable is omitted, matching SPARQL CONSTRUCT semantics.
    /// Anonymous blank-node templates are deliberately refused because
    /// deterministic identity is a prerequisite for receipted manufacture.
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

        // Use the same governed planner/evaluator as TripleStore::query.  For
        // CONSTRUCT, GraphLaw's plan projects the variables referenced by the
        // template; that is exactly the binding set required for template
        // instantiation.
        let plan = plan_query_or_refuse(&query, &self.triple_index)?;
        let rows: Vec<Vec<crate::sparql::Binding>> =
            evaluate_plan_and_debug(&plan, &self.triple_index).collect();

        let mut canonical = BTreeMap::<String, Triple>::new();
        for row in &rows {
            for pattern in template {
                let Some(subject) = instantiate_component(pattern.subject.to_string(), row)? else {
                    continue;
                };
                let Some(predicate) = instantiate_component(pattern.predicate.to_string(), row)? else {
                    continue;
                };
                let Some(object) = instantiate_component(pattern.object.to_string(), row)? else {
                    continue;
                };

                let triple = Triple::from(subject, predicate, object);
                canonical.insert(TripleStore::decode_triple(&triple), triple);
            }
        }

        Ok(canonical.into_values().collect())
    }
}
