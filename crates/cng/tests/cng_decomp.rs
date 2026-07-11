//! PROJ-712 (Track P slice): the canonical potato scenario driven through
//! the full no-LLM decomposition pipeline — `examples/pddl-strips-potato.ttl`
//! (hand-authored pddl-strips graph, PROJ-701) → graph→surface bridge
//! (render domain + problem, parse via the unchanged bcinr parser) →
//! `decompose` (lift, Datalog edges, bounded candidate search, verified s′
//! replay, interference + release-closure proofs, nested POWL composition,
//! deterministic selection, per-candidate receipts).
//!
//! Fixture data enters ONLY from the on-disk example graph; assertions are
//! typed Rust comparisons plus typed-pattern inspection of the emitted
//! result graph — no inline Turtle/PDDL/SPARQL in this file.

#![cfg(feature = "bench")]

use std::fs;
use std::path::{Path, PathBuf};

use chicago_tdd_tools::prelude::*;

use cng::bench::decomp::{
    decomp_queries_dir, decompose, decompose_with, strips_graph_to_surface, DecompositionOutcome,
    SINGLE_ACTOR_CANDIDATE_ID,
};
use cng::bench::QuerySet;
use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::model::NamedNodeRef;
use oxigraph::store::Store;

fn scratch_dir(test_name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests/decomp-integration")
        .join(test_name);
    fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

fn potato_store() -> Store {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/pddl-strips-potato.ttl");
    let turtle = fs::read_to_string(&path).expect("read potato example graph");
    let store = Store::new().expect("store");
    store
        .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
        .expect("potato example graph must parse");
    store
}

fn template(name: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("templates")
            .join(name),
    )
    .expect("read template")
}

test!(potato_graph_bridges_to_a_parsed_surface, {
    // Arrange.
    let store = potato_store();
    let queries = QuerySet::load(&decomp_queries_dir()).expect("load decomp queries");

    // Act: graph → rendered PDDL → parsed structs.
    let (domain, problem) = strips_graph_to_surface(
        &store,
        &queries,
        &template("decomp-domain.template.pddl"),
        &template("decomp-problem.template.pddl"),
    )?;

    // Assert: the fixture inventory (7 action schemas, 9 objects, 13 init
    // atoms, 2 goal atoms) survives the bridge.
    assert_eq!(domain.name, "potato-kitchen");
    assert_eq!(domain.actions.len(), 7);
    assert_eq!(problem.objects.len(), 9);
    assert_eq!(problem.init.len(), 13);
    assert_eq!(problem.goal.len(), 2);
});

test!(potato_decomposition_is_typed_receipted_and_replayable, {
    // Arrange.
    let store = potato_store();
    let queries = QuerySet::load(&decomp_queries_dir()).expect("load decomp queries");
    let (domain, problem) = strips_graph_to_surface(
        &store,
        &queries,
        &template("decomp-domain.template.pddl"),
        &template("decomp-problem.template.pddl"),
    )?;
    let out = scratch_dir("potato");

    // Act.
    let result = decompose(&domain, &problem, &out, "urn:cng:test:decomp:potato")?;

    // Assert: the outcome is one of the three TYPED results (never a
    // silent fallback); candidate 0 is always the single-actor plan and is
    // receipted; every receipt carries a canonical id.
    assert_eq!(
        result.candidate_receipts[0].candidate_id,
        SINGLE_ACTOR_CANDIDATE_ID
    );
    assert!(
        result.candidate_receipts.len() >= 2,
        "splits must be examined"
    );
    match &result.outcome {
        DecompositionOutcome::Selected { subworkflows, .. } => {
            // The 2-actor decomposition: helper ∥ main with proven s′.
            assert_eq!(*subworkflows, 2);
            assert_eq!(result.subworkflows.len(), 2);
            assert_eq!(result.subworkflows[0].role, "helper");
            assert_eq!(result.subworkflows[1].role, "main");
            assert!(!result.interface_atoms.is_empty());
        }
        DecompositionOutcome::NoBeneficialDecomposition { .. }
        | DecompositionOutcome::NoAdmissibleDecomposition { .. } => {
            // Typed single-actor outcome — still carries the plan.
            assert_eq!(result.subworkflows.len(), 1);
            assert_eq!(result.subworkflows[0].role, "single");
        }
    }

    // The emitted result graph parses and carries one DecompositionResult
    // plus one CandidateReceipt per examined candidate (typed pattern API).
    let graph = fs::read_to_string(&result.result_graph_path).expect("read result graph");
    let result_store = Store::new().expect("store");
    result_store
        .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), graph.as_bytes())
        .expect("result graph must parse");
    let rdf_type =
        NamedNodeRef::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").expect("iri");
    let receipt_class =
        NamedNodeRef::new("https://truex.io/ontology/decomp#CandidateReceipt").expect("iri");
    let receipt_count = result_store
        .quads_for_pattern(None, Some(rdf_type), Some(receipt_class.into()), None)
        .count();
    assert_eq!(receipt_count, result.candidate_receipts.len());

    // Determinism: a second manufacture is byte-identical.
    let out2 = scratch_dir("potato-again");
    let again = decompose(&domain, &problem, &out2, "urn:cng:test:decomp:potato")?;
    let bytes_a = fs::read(&result.result_graph_path).expect("read a");
    let bytes_b = fs::read(&again.result_graph_path).expect("read b");
    assert_eq!(bytes_a, bytes_b);
});

test!(forcing_an_unknown_candidate_refuses_cng_r21, {
    // Arrange.
    let store = potato_store();
    let queries = QuerySet::load(&decomp_queries_dir()).expect("load decomp queries");
    let (domain, problem) = strips_graph_to_surface(
        &store,
        &queries,
        &template("decomp-domain.template.pddl"),
        &template("decomp-problem.template.pddl"),
    )?;
    let out = scratch_dir("potato-forced");

    // Act: demand a candidate that was never enumerated.
    let refusal = decompose_with(
        &domain,
        &problem,
        &out,
        "urn:cng:test:decomp:potato-forced",
        Some("no-such-candidate"),
    )
    .unwrap_err();

    // Assert: typed CNG_R21, never a silent fallback.
    assert_eq!(refusal.code(), "CNG_R21");
});
