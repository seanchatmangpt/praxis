//! Cross-graph gluing — lawful composition of workflow graph fragments.
//!
//! ## Doctrine
//!
//! A workflow may be authored as several TTL documents that overlap on shared
//! IRIs. The glue law: on the overlap the constituents must AGREE. For every
//! functional `wf:` predicate (at most one value per subject in the
//! single-graph shape rules), all constituents asserting a value for the same
//! subject must assert the *same* value; disagreement is a typed
//! [`Refusal::GlueConflict`] naming subject, predicate, and every conflicting
//! value. Non-functional predicates (`wf:init`/`goal`/`pre`/`add`/`del`,
//! `rdf:type`, foreign namespaces) merge as set union.
//!
//! The composition is a sorted-set union of ground triples, so it is
//! associative, commutative, and idempotent by construction — the cocycle
//! condition is trivial and the receipt is merge-order free. The merged
//! canonical form is itself a lawful document: it feeds the untouched
//! single-document pipeline (`parse_ttl` → … → [`crate::execute_workflow`]),
//! so gluing weakens no downstream shape law.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

#[allow(deprecated)]
use crate::graph::execute_workflow;
use crate::graph::{
    canonical_form, graph_hash, parse_ttl, Object, Triple, WorkflowReceipt, MAX_TRIPLES,
    MAX_TTL_BYTES, WF_NS,
};
use crate::Refusal;

/// Functional `wf:` predicate locals: at most one value per subject across
/// the whole composition (the single-graph cardinality rules, lifted to
/// gluing).
pub const FUNCTIONAL_WF_LOCALS: [&str; 17] = [
    "budget",
    "name",
    "params",
    "cost",
    "predicate",
    "kind",
    "a",
    "b",
    "k",
    "arg0",
    "arg1",
    "arg2",
    "arg3",
    "arg4",
    "arg5",
    "arg6",
    "arg7",
];

/// A lawful composition of constituent graphs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComposedGraph {
    /// Constituent `graph_hash`es, byte-sorted (order of composition is
    /// erased — the cocycle condition demands the receipt be merge-order
    /// free).
    pub sections: Vec<String>,
    /// Merged, deduplicated ground triples, in canonical (sorted) order.
    pub triples: Vec<Triple>,
    /// `graph_hash` of the merged canonical form.
    pub merged_graph_hash: String,
    /// The merged canonical form itself — valid input to `parse_ttl`, so the
    /// merged graph executes through the untouched single-document pipeline.
    pub canonical_ttl: String,
}

/// The composed receipt: constituent sections + merged hash + the ordinary
/// derived-chain workflow receipt of the merged graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComposedWorkflowReceipt {
    /// Constituent `graph_hash`es (byte-sorted).
    pub sections: Vec<String>,
    /// The merged graph's own `graph_hash` (== `workflow.graph_hash`).
    pub merged_graph_hash: String,
    /// The full single-graph receipt derived from the merged canonical form.
    pub workflow: WorkflowReceipt,
}

/// Canonical rendering of an object term — byte-identical to the object
/// rendering inside [`canonical_form`], so conflict values are nameable in
/// the same vocabulary the canonical form speaks.
fn render_object(o: &Object) -> String {
    match o {
        Object::Iri(iri) => format!("<{iri}>"),
        Object::Str(s) => {
            let mut out = String::with_capacity(s.len() + 2);
            out.push('"');
            for c in s.chars() {
                match c {
                    '\\' => out.push_str("\\\\"),
                    '"' => out.push_str("\\\""),
                    '\n' => out.push_str("\\n"),
                    '\t' => out.push_str("\\t"),
                    c => out.push(c),
                }
            }
            out.push('"');
            out
        }
        Object::Int(v) => format!("{v}"),
    }
}

/// Glue law: graphs may overlap on shared IRIs; on the overlap they must
/// AGREE. Same subject + functional `wf:` predicate ⇒ one object value across
/// all constituents, else a typed [`Refusal::GlueConflict`] naming subject,
/// predicate, and every conflicting value (byte-sorted). Exact-duplicate
/// triples are lawful agreement and dedup in the union. Deterministic; merge
/// order cannot matter by construction (sorted-set union: associative,
/// commutative, idempotent) — and is tested anyway.
pub fn compose_workflows(ttls: &[&str]) -> Result<ComposedGraph, Refusal> {
    if ttls.is_empty() {
        return Err(Refusal::InvalidInput {
            detail: "compose_workflows requires at least one constituent graph".to_string(),
        });
    }

    let mut sections = Vec::with_capacity(ttls.len());
    let mut union: BTreeSet<Triple> = BTreeSet::new();
    for ttl in ttls {
        let triples = parse_ttl(ttl)?;
        sections.push(graph_hash(&triples));
        union.extend(triples);
    }
    sections.sort_unstable();

    // Caps on the union: the merged form must itself be a lawful document.
    if union.len() > MAX_TRIPLES {
        return Err(Refusal::GraphCapExceeded {
            what: "triples".to_string(),
            cap: MAX_TRIPLES as u64,
            actual: union.len() as u64,
        });
    }

    // Glue check: functional wf: predicates carry at most one value per
    // subject across the whole composition.
    let functional: BTreeSet<String> = FUNCTIONAL_WF_LOCALS
        .iter()
        .map(|local| format!("{WF_NS}{local}"))
        .collect();
    let mut values: BTreeMap<(&str, &str), BTreeSet<String>> = BTreeMap::new();
    for t in &union {
        if functional.contains(&t.p) {
            values
                .entry((t.s.as_str(), t.p.as_str()))
                .or_default()
                .insert(render_object(&t.o));
        }
    }
    // First offending (s, p) in BTreeMap order — deterministic.
    for ((subject, predicate), vals) in &values {
        if vals.len() >= 2 {
            return Err(Refusal::GlueConflict {
                subject: (*subject).to_string(),
                predicate: (*predicate).to_string(),
                values: vals.iter().cloned().collect(),
            });
        }
    }

    let triples: Vec<Triple> = union.into_iter().collect();
    let canonical_ttl = canonical_form(&triples);
    if canonical_ttl.len() > MAX_TTL_BYTES {
        return Err(Refusal::GraphCapExceeded {
            what: "ttl_bytes".to_string(),
            cap: MAX_TTL_BYTES as u64,
            actual: canonical_ttl.len() as u64,
        });
    }
    let merged_graph_hash = graph_hash(&triples);

    Ok(ComposedGraph {
        sections,
        triples,
        merged_graph_hash,
        canonical_ttl,
    })
}

/// Compose then execute the merged canonical form through the untouched
/// single-document pipeline ([`execute_workflow`]). Every downstream shape
/// law (single `wf:Workflow` node, budget range, ground init, …) applies to
/// the merged graph unchanged; refusals pass through typed.
pub fn execute_composed(ttls: &[&str]) -> Result<ComposedWorkflowReceipt, Refusal> {
    let composed = compose_workflows(ttls)?;
    #[allow(deprecated)]
    let workflow = execute_workflow(&composed.canonical_ttl)?;
    debug_assert_eq!(
        workflow.graph_hash, composed.merged_graph_hash,
        "reparse of the canonical form is a fixpoint"
    );
    Ok(ComposedWorkflowReceipt {
        sections: composed.sections,
        merged_graph_hash: composed.merged_graph_hash,
        workflow,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_composition_is_a_typed_refusal() {
        let err = compose_workflows(&[]).expect_err("empty slice refuses");
        assert!(matches!(err, Refusal::InvalidInput { .. }));
    }

    #[test]
    fn single_constituent_composition_is_the_identity_on_the_graph() {
        let ttl = "@prefix ex: <http://example.org/> .\nex:a ex:p ex:b .\n";
        let triples = parse_ttl(ttl).expect("parse");
        let composed = compose_workflows(&[ttl]).expect("compose");
        assert_eq!(composed.merged_graph_hash, graph_hash(&triples));
        assert_eq!(composed.sections, vec![graph_hash(&triples)]);
        assert_eq!(composed.canonical_ttl, canonical_form(&triples));
    }
}
