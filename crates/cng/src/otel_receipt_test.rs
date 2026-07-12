#![cfg(test)]

use chicago_tdd_tools::prelude::*;
use oxigraph::model::{GraphName, NamedNode};
use oxigraph::store::Store;

use super::{receipt_otel_to_ocel, verify_receipt_otel_to_ocel};
use crate::otel_ocel::{insert_quads, project_otel_to_ocel, RECEIPT_GRAPH_IRI};
use crate::otel_rdf::{otel_rdf_test::admissible_span, project_admitted_spans};
use crate::powl::CngRefusal;

fn graph_name_for(iri: &str) -> GraphName {
    GraphName::NamedNode(
        NamedNode::new(iri).unwrap_or_else(|e| panic!("bad test IRI {iri:?}: {e}")),
    )
}

/// Builds a store whose `urn:graph:otel` graph holds PROJ-763's admitted
/// fixture span (same fixture PROJ-764's `otel_ocel_test.rs` reuses),
/// returning just the store — this module's tests drive
/// [`receipt_otel_to_ocel`] directly rather than pre-computing `G_OCEL`
/// themselves, proving the receipt function's own internal call to
/// `project_otel_to_ocel` is what produces the output digest.
fn store_with_admitted_span() -> Result<Store, CngRefusal> {
    let store = Store::new().map_err(|e| CngRefusal::IoRefused(format!("store: {e}")))?;
    let quads = project_admitted_spans(&[admissible_span()])?;
    insert_quads(&store, &quads)?;
    Ok(store)
}

fn predicate_node(local: &str) -> NamedNode {
    NamedNode::new(format!("https://truex.io/ontology/cng-receipt#{local}"))
        .unwrap_or_else(|e| panic!("bad predicate {local:?}: {e}"))
}

fn prov_node(local: &str) -> NamedNode {
    NamedNode::new(format!("http://www.w3.org/ns/prov#{local}"))
        .unwrap_or_else(|e| panic!("bad prov predicate {local:?}: {e}"))
}

test!(receipt_writes_exclusively_into_the_receipt_graph, {
    let store = store_with_admitted_span()?;

    let receipt_quads = receipt_otel_to_ocel(&store)?;
    assert!(!receipt_quads.is_empty(), "receipt must be non-trivial");

    let receipts_graph = format!("<{RECEIPT_GRAPH_IRI}>");
    for quad in &receipt_quads {
        assert_eq!(
            quad.graph_name.to_string(),
            receipts_graph,
            "every receipt quad must land in urn:graph:receipts: {quad}"
        );
    }
});

test!(
    receipt_records_one_activity_typed_prov_activity_and_cngr_receipt,
    {
        let store = store_with_admitted_span()?;
        let receipt_quads = receipt_otel_to_ocel(&store)?;

        let rdf_type = NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")
            .map_err(|e| CngRefusal::IoRefused(format!("rdf:type: {e}")))?;
        let prov_activity = prov_node("Activity");
        let cngr_receipt = predicate_node("ConstructTransformationReceipt");

        let activity_subjects: Vec<_> = receipt_quads
            .iter()
            .filter(|q| {
                q.predicate == rdf_type.as_ref() && q.object == prov_activity.as_ref().into()
            })
            .map(|q| q.subject.clone())
            .collect();
        assert_eq!(
            activity_subjects.len(),
            1,
            "expected exactly one prov:Activity node, got {activity_subjects:?}"
        );

        let cngr_typed: Vec<_> = receipt_quads
            .iter()
            .filter(|q| {
                q.predicate == rdf_type.as_ref() && q.object == cngr_receipt.as_ref().into()
            })
            .map(|q| q.subject.clone())
            .collect();
        assert_eq!(
            cngr_typed, activity_subjects,
            "the same node must carry both prov:Activity and cngr:ConstructTransformationReceipt"
        );
    }
);

test!(receipt_activity_links_used_hadplan_generated, {
    let store = store_with_admitted_span()?;
    let receipt_quads = receipt_otel_to_ocel(&store)?;

    let used = prov_node("used");
    let had_plan = prov_node("hadPlan");
    let generated = prov_node("generated");

    for predicate in [&used, &had_plan, &generated] {
        let count = receipt_quads
            .iter()
            .filter(|q| q.predicate == predicate.as_ref())
            .count();
        assert_eq!(
            count, 1,
            "expected exactly one {predicate} triple on the activity, got {count}"
        );
    }

    // used -> the G_OTEL content-addressed entity; generated -> the G_OCEL
    // content-addressed entity; hadPlan -> the query content-addressed plan.
    // All three objects must be distinct urn:blake3: nodes (real content
    // differs across query text, admitted input, and derived output).
    let object_of = |predicate: &NamedNode| -> String {
        receipt_quads
            .iter()
            .find(|q| q.predicate == predicate.as_ref())
            .map(|q| q.object.to_string())
            .unwrap_or_default()
    };
    let used_obj = object_of(&used);
    let plan_obj = object_of(&had_plan);
    let generated_obj = object_of(&generated);
    assert!(used_obj.starts_with("<urn:blake3:"), "{used_obj}");
    assert!(plan_obj.starts_with("<urn:blake3:"), "{plan_obj}");
    assert!(generated_obj.starts_with("<urn:blake3:"), "{generated_obj}");
    assert_ne!(
        used_obj, plan_obj,
        "input graph and query digests must differ"
    );
    assert_ne!(
        used_obj, generated_obj,
        "input and output graph digests must differ"
    );
    assert_ne!(
        plan_obj, generated_obj,
        "query and output graph digests must differ"
    );
});

test!(receipt_head_chains_the_three_constitutional_digests, {
    let store = store_with_admitted_span()?;
    let receipt_quads = receipt_otel_to_ocel(&store)?;

    // Independently recompute the three constitutional digests outside the
    // function under test (mirrors this repo's established
    // recompute-and-compare receipt-verification pattern, e.g.
    // ArazzoProjectionReceipt::from_materials's own test discipline).
    let query_digest = format!(
        "blake3:{}",
        blake3::hash(crate::otel_ocel::OTEL_TO_OCEL_CONSTRUCT.as_bytes()).to_hex()
    );
    let input_digest =
        crate::otel_ocel::graph_content_digest(&store, crate::otel_ocel::OTEL_GRAPH_IRI)?;
    let ocel_quads = project_otel_to_ocel(&store)?;
    let mut ocel_lines: Vec<String> = ocel_quads.iter().map(|q| q.to_string()).collect();
    ocel_lines.sort();
    ocel_lines.dedup();
    let output_digest = format!(
        "blake3:{}",
        blake3::hash(ocel_lines.join("\n").as_bytes()).to_hex()
    );

    let mut hasher = blake3::Hasher::new();
    hasher.update(b"cng/otel-ocel/receipt-head/v1");
    hasher.update(b"\0");
    hasher.update(query_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update(input_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update(output_digest.as_bytes());
    let expected_head = format!("blake3:{}", hasher.finalize().to_hex());

    let receipt_head_pred = predicate_node("receiptHead");
    let recorded_heads: Vec<String> = receipt_quads
        .iter()
        .filter(|q| q.predicate == receipt_head_pred.as_ref())
        .map(|q| match &q.object {
            oxigraph::model::Term::Literal(l) => l.value().to_string(),
            other => other.to_string(),
        })
        .collect();
    assert_eq!(
        recorded_heads,
        vec![expected_head],
        "recorded cngr:receiptHead must equal the independently recomputed digest chain"
    );
});

test!(receipt_is_byte_identical_across_two_runs, {
    let store = store_with_admitted_span()?;

    let first = receipt_otel_to_ocel(&store)?;
    let second = receipt_otel_to_ocel(&store)?;

    let first_text: Vec<String> = first.iter().map(|q| q.to_string()).collect();
    let second_text: Vec<String> = second.iter().map(|q| q.to_string()).collect();
    assert_eq!(
        first_text, second_text,
        "same G_OTEL input must produce a byte-identical receipt across runs"
    );
});

test!(receipt_never_mutates_the_store_it_reads, {
    let store = store_with_admitted_span()?;
    let before = store
        .len()
        .map_err(|e| CngRefusal::IoRefused(format!("len: {e}")))?;

    let _receipt_quads = receipt_otel_to_ocel(&store)?;

    let after = store
        .len()
        .map_err(|e| CngRefusal::IoRefused(format!("len: {e}")))?;
    assert_eq!(
        before, after,
        "receipt_otel_to_ocel is a pure function; it must not insert into the store itself"
    );
    let receipts_graph = graph_name_for(RECEIPT_GRAPH_IRI);
    let receipts_in_store = store
        .quads_for_pattern(None, None, None, Some(receipts_graph.as_ref()))
        .count();
    assert_eq!(
        receipts_in_store, 0,
        "the caller inserts the receipt via otel_ocel::insert_quads, not this function"
    );
});

test!(
    receipt_over_empty_otel_graph_still_produces_a_well_formed_receipt,
    {
        let store = Store::new().map_err(|e| CngRefusal::IoRefused(format!("store: {e}")))?;

        let receipt_quads = receipt_otel_to_ocel(&store)?;
        assert!(
        !receipt_quads.is_empty(),
        "even an empty G_OTEL/G_OCEL pair must yield a real receipt describing that emptiness, \
         not a refusal or silent no-op"
    );

        let receipt_head_pred = predicate_node("receiptHead");
        let heads: usize = receipt_quads
            .iter()
            .filter(|q| q.predicate == receipt_head_pred.as_ref())
            .count();
        assert_eq!(heads, 1, "exactly one receipt head even for empty content");
    }
);

/// Pulls the `cngr:receiptHead` literal value out of a sealed receipt's
/// quads — the "claimed head" a downstream consumer would carry forward and
/// later hand to [`verify_receipt_otel_to_ocel`].
fn claimed_head_of(receipt_quads: &[oxigraph::model::Quad]) -> Result<String, CngRefusal> {
    let receipt_head_pred = predicate_node("receiptHead");
    receipt_quads
        .iter()
        .find(|q| q.predicate == receipt_head_pred.as_ref())
        .map(|q| match &q.object {
            oxigraph::model::Term::Literal(l) => l.value().to_string(),
            other => other.to_string(),
        })
        .ok_or_else(|| {
            CngRefusal::IoRefused("no cngr:receiptHead literal in receipt quads".to_string())
        })
}

// ── verify_receipt_otel_to_ocel: the real "recompute and compare" ──────────
// consumer `docs/otel-rdf-handoff.md` describes, and the independent check
// `receipt_head_chains_the_three_constitutional_digests` above cannot stand
// in for: that test's oracle is a hand-copied restatement of
// `fold_receipt_head`'s own formula (by necessity — it cannot call a
// private production function), so a bug in the fold itself could pass both
// the production code and its own test oracle. These tests instead exercise
// the real `pub fn verify_receipt_otel_to_ocel`.

test!(verify_receipt_accepts_the_claimed_head_of_a_real_receipt, {
    let store = store_with_admitted_span()?;
    let receipt_quads = receipt_otel_to_ocel(&store)?;
    let claimed_head = claimed_head_of(&receipt_quads)?;

    verify_receipt_otel_to_ocel(&store, &claimed_head)?;
});

test!(verify_receipt_refuses_a_tampered_claimed_head, {
    let store = store_with_admitted_span()?;
    // Real receipt quads exist and carry a real head, but the caller hands
    // the verifier a different (tampered/stale) claimed head instead of
    // that real one.
    let _receipt_quads = receipt_otel_to_ocel(&store)?;
    let tampered_head = format!("blake3:{}", "0".repeat(64));

    match verify_receipt_otel_to_ocel(&store, &tampered_head) {
        Err(CngRefusal::AuditMismatch(_)) => {}
        other => panic!(
            "a tampered claimed_head must refuse as CngRefusal::AuditMismatch, got {other:?}"
        ),
    }
});

test!(verify_receipt_over_empty_otel_graph_accepts_its_own_head, {
    let store = Store::new().map_err(|e| CngRefusal::IoRefused(format!("store: {e}")))?;
    let receipt_quads = receipt_otel_to_ocel(&store)?;
    let claimed_head = claimed_head_of(&receipt_quads)?;

    verify_receipt_otel_to_ocel(&store, &claimed_head)?;
});
