//! `corpus` — ingest reusable open-ontologies vocabularies as manufacturing
//! *inputs* (feature `ggen`).
//!
//! The `mfg` lane manufactures PDDL from one hand-authored `pdl:` ontology.
//! This module widens the intake: it consumes three dependency-free Turtle
//! vocabularies vendored under `ontology/vendor/` (provenance recorded in each
//! file header) and manufactures byte-deterministic Rust from them, so the
//! taxonomies below are corpus-driven rather than hand-maintained:
//!
//! - **truex-ecosystem.ttl** — `truex:Failure` individuals are the ecosystem's
//!   *refusal conditions*. [`refusal_category`] maps every one of them onto a
//!   praxis [`RefusalCategory`]; [`every_failure_has_category`] (and the test
//!   of the same intent) proves the map is **total over the corpus**.
//! - **mcpp-proof-chain.ttl** — the `mcpp:Verdict` model
//!   (`Admitted`/`Refused`/`Partial`) is manufactured into a Rust enum.
//! - **shared-receipt-shapes.ttl** — the `sr:status` enumeration is surfaced
//!   for downstream receipt validators (already wired via `receipt_shacl`).
//!
//! Manufacture is *round-tripped*: [`emit_rust_enum`] emits deterministic
//! source and [`parse_enum_variants`] reads the variant set back, so the tests
//! assert `parse(emit(corpus)) == corpus`.

use bcinr_powl_receipt::denial::DenialPolarity;
use ggen_graph::prelude::DeterministicGraph;
use oxigraph::{model::Term, sparql::QueryResults};
use praxis_core::RefusalCategory;
use serde::{Deserialize, Serialize};

use crate::mfg::{self, MfgError};

/// Vendored truex ecosystem vocabulary (obligation/refusal surface).
pub const TRUEX_TTL: &str = include_str!("../ontology/vendor/truex-ecosystem.ttl");
/// Vendored MCPP proof-chain vocabulary (Admitted/Refused/Partial verdicts).
pub const MCPP_TTL: &str = include_str!("../ontology/vendor/mcpp-proof-chain.ttl");
/// Vendored shared-receipt SHACL shapes (status enumeration lives here).
pub const SHARED_RECEIPT_TTL: &str = include_str!("../ontology/vendor/shared-receipt-shapes.ttl");

/// A term ingested from the corpus: its `rdfs:label` and optional definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorpusTerm {
    /// The `rdfs:label` value — used verbatim as the manufactured Rust variant.
    pub label: String,
    /// A human definition (`skos:definition`/`rdfs:comment`), if present.
    pub definition: Option<String>,
}

type Result<T> = std::result::Result<T, MfgError>;

fn select(graph: &DeterministicGraph, query: &str) -> Result<Vec<Vec<(String, Term)>>> {
    let mut rows = Vec::new();
    match graph
        .query(query)
        .map_err(|e| MfgError::Graph(e.to_string()))?
    {
        QueryResults::Solutions(sols) => {
            for sol in sols {
                let sol = sol.map_err(|e| MfgError::Shape(e.to_string()))?;
                rows.push(
                    sol.iter()
                        .map(|(v, t)| (v.as_str().to_string(), t.clone()))
                        .collect(),
                );
            }
        }
        QueryResults::Boolean(_) | QueryResults::Graph(_) => {
            return Err(MfgError::Shape("expected SELECT results".to_string()));
        }
    }
    Ok(rows)
}

fn literal(row: &[(String, Term)], var: &str) -> Option<String> {
    row.iter()
        .find(|(v, _)| v == var)
        .and_then(|(_, t)| match t {
            Term::Literal(l) => Some(l.value().to_string()),
            _ => None,
        })
}

/// Ingest the `truex:Failure` refusal conditions from the vendored corpus,
/// ordered by label for deterministic manufacture.
///
/// # Errors
/// Returns [`MfgError`] if the vendored Turtle fails to parse or query.
pub fn truex_failures() -> Result<Vec<CorpusTerm>> {
    let graph = mfg::load_graph(TRUEX_TTL)?;
    let q = "\
        PREFIX truex: <https://open-ontologies.org/profile/truex#>\n\
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\n\
        PREFIX skos: <http://www.w3.org/2004/02/skos/core#>\n\
        SELECT ?label ?def WHERE {\n\
          ?f a truex:Failure ; rdfs:label ?label .\n\
          OPTIONAL { ?f skos:definition ?def }\n\
        } ORDER BY ?label";
    Ok(select(&graph, q)?
        .iter()
        .filter_map(|row| {
            literal(row, "label").map(|label| CorpusTerm {
                label,
                definition: literal(row, "def"),
            })
        })
        .collect())
}

/// Ingest the `mcpp:Verdict` individuals (Admitted/Refused/Partial) from the
/// vendored corpus, ordered by label.
///
/// # Errors
/// Returns [`MfgError`] if the vendored Turtle fails to parse or query.
pub fn mcpp_verdicts() -> Result<Vec<CorpusTerm>> {
    let graph = mfg::load_graph(MCPP_TTL)?;
    let q = "\
        PREFIX mcpp: <urn:ontostar:mcpp:>\n\
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\n\
        SELECT ?label ?def WHERE {\n\
          ?v a mcpp:Verdict ; rdfs:label ?label .\n\
          OPTIONAL { ?v rdfs:comment ?def }\n\
        } ORDER BY ?label";
    Ok(select(&graph, q)?
        .iter()
        .filter_map(|row| {
            literal(row, "label").map(|label| CorpusTerm {
                label,
                definition: literal(row, "def"),
            })
        })
        .collect())
}

/// Map a truex refusal condition (by its `rdfs:label`) onto the praxis
/// [`RefusalCategory`] it manifests as. This is the one hand-authored surface
/// — a deliberate semantic judgement per condition — but its **totality over
/// the corpus** is machine-checked by [`every_failure_has_category`] and the
/// module test, so a newly-added truex `Failure` that lacks a mapping is a
/// test failure, never a silent gap.
///
/// Returns `None` for an unknown condition (a corpus/table drift signal).
#[must_use]
pub fn refusal_category(condition: &str) -> Option<RefusalCategory> {
    Some(match condition {
        // Missing/absent raw execution evidence — an unmet prerequisite.
        "SummaryOnlyProof" | "MissingBoundary" | "OCELLaundering" | "NonDerivableExecution" => {
            RefusalCategory::Prerequisites
        }
        // Provenance/identity: evidence or artifact origin does not match.
        "CloneTrace" | "ArtifactOriginMismatch" => RefusalCategory::Identity,
        // Object state-machine position disagreement.
        "StateTransitionMismatch" => RefusalCategory::Lifecycle,
        // Causal/temporal ordering law violated.
        "TemporalOrderingViolation" => RefusalCategory::Temporal,
        // Structural projection into the OCEL graph failed.
        "BoundaryProjectionFailure" => RefusalCategory::Topology,
        _ => return None,
    })
}

/// Convenience: the `DenialPolarity` lane a truex condition composes into,
/// via its praxis [`RefusalCategory`]. Bridges the ingested vocabulary to the
/// receipt denial word used across praxis.
#[must_use]
pub fn denial_lane_for(condition: &str) -> Option<DenialPolarity> {
    refusal_category(condition).map(|cat| match cat {
        RefusalCategory::Prerequisites => DenialPolarity::PRECONDITION_FAILED,
        RefusalCategory::Identity => DenialPolarity::AUTHORIZATION_DENIED,
        RefusalCategory::Lifecycle => DenialPolarity::OBJECT_LIFECYCLE_VIOLATION,
        RefusalCategory::Temporal => DenialPolarity::SLA_BREACH,
        RefusalCategory::Topology => DenialPolarity::CONFORMANCE_GATE_FAILED,
        RefusalCategory::Capacity => DenialPolarity::RESOURCE_EXHAUSTED,
        RefusalCategory::Authorization => DenialPolarity::AUTHORIZATION_DENIED,
        RefusalCategory::Reserved => DenialPolarity::CONFORMANCE_GATE_FAILED,
    })
}

/// Total assertion form of [`refusal_category`]: `Ok(())` iff every ingested
/// truex `Failure` maps to a category, else `Err(unmapped_labels)`.
///
/// # Errors
/// Returns the sorted list of unmapped condition labels.
pub fn every_failure_has_category() -> std::result::Result<(), Vec<String>> {
    let failures = truex_failures().map_err(|e| vec![format!("ingest error: {e}")])?;
    let unmapped: Vec<String> = failures
        .iter()
        .filter(|c| refusal_category(&c.label).is_none())
        .map(|c| c.label.clone())
        .collect();
    if unmapped.is_empty() {
        Ok(())
    } else {
        Err(unmapped)
    }
}

/// Ingest the `sr:status` enumeration (the `sh:in` list) from the vendored
/// shared-receipt SHACL shapes — the set of admissible receipt statuses
/// exchanged between wasm4pm and mcpp. Returned sorted for determinism.
///
/// # Errors
/// Returns [`MfgError`] if the vendored Turtle fails to parse or query.
pub fn shared_receipt_status_values() -> Result<Vec<String>> {
    let graph = mfg::load_graph(SHARED_RECEIPT_TTL)?;
    let q = "\
        PREFIX sh: <http://www.w3.org/ns/shacl#>\n\
        PREFIX sr: <urn:ontostar:shared-receipt:>\n\
        PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
        SELECT DISTINCT ?status WHERE {\n\
          ?prop sh:path sr:status ; sh:in/rdf:rest*/rdf:first ?status .\n\
        } ORDER BY ?status";
    Ok(select(&graph, q)?
        .iter()
        .filter_map(|row| literal(row, "status"))
        .collect())
}

// ── manufacture: RDF individuals -> byte-deterministic Rust enum ───────────────

/// Emit a deterministic Rust enum `name` with one PascalCase variant per
/// corpus term (already ordered). The header records provenance; the body is a
/// pure function of `(name, terms)` — identical input yields identical bytes.
#[must_use]
pub fn emit_rust_enum(name: &str, source_label: &str, terms: &[CorpusTerm]) -> String {
    let mut s = String::new();
    s.push_str("// GENERATED by praxis::corpus — do not hand-edit.\n");
    s.push_str(&format!("// manufactured-from: {source_label}\n"));
    s.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n");
    s.push_str(&format!("pub enum {name} {{\n"));
    for t in terms {
        if let Some(def) = &t.definition {
            s.push_str(&format!("    /// {}\n", def.replace('\n', " ")));
        }
        s.push_str(&format!("    {},\n", t.label));
    }
    s.push_str("}\n");
    s
}

/// Read the variant identifiers back out of enum source emitted by
/// [`emit_rust_enum`] — the round-trip inverse.
#[must_use]
pub fn parse_enum_variants(src: &str) -> Vec<String> {
    src.lines()
        .map(str::trim)
        .filter(|l| !l.starts_with("//") && !l.starts_with("/// ") && l.ends_with(','))
        .filter_map(|l| l.strip_suffix(','))
        .filter(|id| id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
        .filter(|id| !id.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The vendored corpus parses and yields the expected refusal conditions.
    #[test]
    fn truex_corpus_ingests_failures() {
        let failures = truex_failures().expect("ingest truex failures");
        assert!(
            failures.len() >= 9,
            "corpus has 9 documented Failure conditions"
        );
        let labels: Vec<&str> = failures.iter().map(|c| c.label.as_str()).collect();
        assert!(labels.contains(&"OCELLaundering"));
        assert!(labels.contains(&"TemporalOrderingViolation"));
        // ordered deterministically by label
        let mut sorted = labels.clone();
        sorted.sort_unstable();
        assert_eq!(labels, sorted);
    }

    /// THE mapping test: every truex RefusalCondition maps to a praxis
    /// RefusalCategory — totality over the corpus, machine-checked.
    #[test]
    fn every_truex_failure_has_a_praxis_refusal_category() {
        match every_failure_has_category() {
            Ok(()) => {}
            Err(unmapped) => {
                panic!("truex Failures without a praxis RefusalCategory: {unmapped:?}")
            }
        }
        // spot-check a couple of the semantic judgements
        assert_eq!(
            refusal_category("SummaryOnlyProof"),
            Some(RefusalCategory::Prerequisites)
        );
        assert_eq!(
            refusal_category("TemporalOrderingViolation"),
            Some(RefusalCategory::Temporal)
        );
        assert_eq!(
            refusal_category("StateTransitionMismatch"),
            Some(RefusalCategory::Lifecycle)
        );
        // and the bridge to the receipt denial word
        assert_eq!(
            denial_lane_for("TemporalOrderingViolation"),
            Some(DenialPolarity::SLA_BREACH)
        );
    }

    /// The refusal-category table must not carry entries the corpus dropped:
    /// every mapped label is present in the ingested corpus (no stale drift).
    #[test]
    fn refusal_table_has_no_stale_entries() {
        let corpus: std::collections::BTreeSet<String> = truex_failures()
            .unwrap()
            .into_iter()
            .map(|c| c.label)
            .collect();
        for known in [
            "SummaryOnlyProof",
            "MissingBoundary",
            "OCELLaundering",
            "NonDerivableExecution",
            "CloneTrace",
            "ArtifactOriginMismatch",
            "StateTransitionMismatch",
            "TemporalOrderingViolation",
            "BoundaryProjectionFailure",
        ] {
            assert!(
                corpus.contains(known),
                "mapped label {known} missing from corpus"
            );
        }
    }

    /// The mcpp verdict model (Admitted/Refused/Partial) is ingested.
    #[test]
    fn mcpp_verdict_model_ingests() {
        let verdicts = mcpp_verdicts().expect("ingest mcpp verdicts");
        let labels: Vec<&str> = verdicts.iter().map(|c| c.label.as_str()).collect();
        assert_eq!(
            labels,
            vec!["Admitted", "Partial", "Refused"],
            "the three-verdict model"
        );
    }

    /// The shared-receipt `sr:status` enumeration is ingested from the third
    /// vendored ontology, and the mcpp verdict model is a (lower-cased) subset
    /// of it — the two corpora agree on the admission vocabulary.
    #[test]
    fn shared_receipt_status_enum_ingests_and_aligns_with_mcpp() {
        let statuses = shared_receipt_status_values().expect("ingest sr:status");
        let expected: Vec<String> = [
            "accepted", "admitted", "failed", "partial", "refused", "success",
        ]
        .iter()
        .map(|s| (*s).to_string())
        .collect();
        assert_eq!(statuses, expected, "the six SharedReceiptV1 status values");

        // every mcpp verdict (Admitted/Refused/Partial) appears, lower-cased,
        // in the shared-receipt status vocabulary.
        let set: std::collections::BTreeSet<String> = statuses.into_iter().collect();
        for v in mcpp_verdicts().unwrap() {
            assert!(
                set.contains(&v.label.to_lowercase()),
                "mcpp verdict {} missing from shared-receipt status enum",
                v.label
            );
        }
    }

    /// Manufacture is byte-deterministic and round-trips through the emitted
    /// Rust source.
    #[test]
    fn manufactured_enum_is_deterministic_and_roundtrips() {
        let verdicts = mcpp_verdicts().unwrap();
        let a = emit_rust_enum(
            "McppVerdict",
            "ontology/vendor/mcpp-proof-chain.ttl",
            &verdicts,
        );
        let b = emit_rust_enum(
            "McppVerdict",
            "ontology/vendor/mcpp-proof-chain.ttl",
            &verdicts,
        );
        assert_eq!(a, b, "manufacture must be byte-deterministic");

        let parsed = parse_enum_variants(&a);
        let expected: Vec<String> = verdicts.iter().map(|c| c.label.clone()).collect();
        assert_eq!(
            parsed, expected,
            "emit -> parse must round-trip the variant set"
        );

        // and the refusal-condition enum round-trips too
        let failures = truex_failures().unwrap();
        let src = emit_rust_enum(
            "RefusalCondition",
            "ontology/vendor/truex-ecosystem.ttl",
            &failures,
        );
        let re = parse_enum_variants(&src);
        assert_eq!(
            re,
            failures.iter().map(|c| c.label.clone()).collect::<Vec<_>>()
        );
    }
}
