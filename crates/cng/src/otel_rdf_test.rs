#![cfg(test)]

use std::path::PathBuf;

use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::store::Store;

use super::{
    admit, admitted_spans_to_trig, project_admitted_spans, OtlpSpan, SpanStatus, SpanStatusCode,
    OTEL_GRAPH_IRI,
};
use crate::powl::CngRefusal;
use crate::telemetry_gen;

/// The hand-added "Span-occurrence instance vocabulary" section in
/// `otel-bridge.ttl` (this module's sole vocabulary source) must itself
/// remain valid Turtle — no `.ttl` file in this repo gets a free pass on
/// syntax just because no generated artifact currently re-parses it.
#[test]
fn otel_bridge_ontology_is_valid_turtle() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("praxis-graphlaw")
        .join("ontologies")
        .join("core")
        .join("otel-bridge.ttl");
    let turtle = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("otel-bridge.ttl must be readable at {path:?}: {e}"));
    let store = Store::new().expect("in-memory store construction must succeed");
    store
        .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
        .expect("otel-bridge.ttl must parse as valid Turtle");
    let count = store.len().expect("store length must be queryable");
    assert!(
        count > 100,
        "expected well over 100 triples from the full ontology (schema + span vocabulary), got {count}"
    );
}

/// A real, hand-constructed admissible span: all five required
/// `event.praxis.activity_executed` attributes present, `process.outcome`
/// in the closed vocabulary, plus one extra attribute
/// (`praxis.extra.note`) to exercise the generic `ocel:Attribute`
/// projection path.
///
/// `pub(crate)`: PROJ-764's `otel_ocel_test.rs` reuses this exact fixture
/// (same trace/span ids, same attribute values) rather than inventing a
/// second one, so the two test suites stay provably consistent about what
/// an admitted span looks like.
pub(crate) fn admissible_span() -> OtlpSpan {
    OtlpSpan {
        trace_id: "4bf92f3577b34da6a3ce929d0e0e4736".to_string(),
        span_id: "00f067aa0ba902b7".to_string(),
        parent_span_id: None,
        name: telemetry_gen::REGISTRY_GROUP_ID.to_string(),
        start_time_unix_nano: 1_700_000_000_000_000_000,
        end_time_unix_nano: 1_700_000_000_500_000_000,
        attributes: vec![
            (
                telemetry_gen::ATTR_WORKFLOW_ID.to_string(),
                "wf-42".to_string(),
            ),
            (
                telemetry_gen::ATTR_OBJECT_ID.to_string(),
                "order-7".to_string(),
            ),
            (
                telemetry_gen::ATTR_OBJECT_TYPE.to_string(),
                "Order".to_string(),
            ),
            (
                telemetry_gen::ATTR_ACTIVITY_IRI.to_string(),
                "urn:praxis:activity:ship-order".to_string(),
            ),
            (
                telemetry_gen::ATTR_OUTCOME.to_string(),
                "completed".to_string(),
            ),
            // Extra attribute beyond the five required — proves the
            // generic ocel:Attribute path and, placed first here on
            // purpose, that extras are sorted before projection.
            ("praxis.extra.note".to_string(), "second-leg".to_string()),
        ],
        status: SpanStatus {
            code: SpanStatusCode::Ok,
            message: None,
        },
    }
}

#[test]
fn admit_accepts_a_fully_conformant_span() {
    let span = admissible_span();
    assert!(admit(&span).is_ok(), "conformant span must be admitted");
}

#[test]
fn admit_refuses_missing_required_attribute_cng_r27() {
    let mut span = admissible_span();
    span.attributes
        .retain(|(k, _)| k != telemetry_gen::ATTR_OUTCOME);
    match admit(&span) {
        Err(refusal @ CngRefusal::OtelSpanRefused { .. }) => {
            assert_eq!(refusal.code(), "CNG_R27");
            assert!(
                refusal.to_string().contains("process.outcome"),
                "refusal must name the missing attribute: {refusal}"
            );
        }
        other => panic!("expected OtelSpanRefused, got {other:?}"),
    }
}

#[test]
fn admit_refuses_outcome_outside_closed_vocabulary() {
    let mut span = admissible_span();
    for (k, v) in span.attributes.iter_mut() {
        if k == telemetry_gen::ATTR_OUTCOME {
            *v = "maybe".to_string();
        }
    }
    match admit(&span) {
        Err(CngRefusal::OtelSpanRefused { reason, .. }) => {
            assert!(
                reason.contains("closed vocabulary"),
                "reason must cite the closed vocabulary: {reason}"
            );
        }
        other => panic!("expected OtelSpanRefused, got {other:?}"),
    }
}

#[test]
fn admit_refuses_empty_trace_id() {
    let mut span = admissible_span();
    span.trace_id = String::new();
    match admit(&span) {
        Err(CngRefusal::OtelSpanRefused { reason, .. }) => {
            assert!(reason.contains("trace_id"), "reason was: {reason}");
        }
        other => panic!("expected OtelSpanRefused, got {other:?}"),
    }
}

#[test]
fn admit_refuses_unparseable_activity_iri() {
    let mut span = admissible_span();
    for (k, v) in span.attributes.iter_mut() {
        if k == telemetry_gen::ATTR_ACTIVITY_IRI {
            // A space makes this an illegal IRI under RFC 3987.
            *v = "not a valid iri".to_string();
        }
    }
    match admit(&span) {
        Err(CngRefusal::OtelSpanRefused { reason, .. }) => {
            assert!(reason.contains("not a legal IRI"), "reason was: {reason}");
        }
        other => panic!("expected OtelSpanRefused, got {other:?}"),
    }
}

#[test]
fn projection_lands_in_urn_graph_otel_with_expected_triples() {
    let span = admissible_span();
    let quads = project_admitted_spans(&[span]).expect("admissible span must project");
    assert!(!quads.is_empty(), "projection must produce triples");

    // Every quad is in the named graph urn:graph:otel — no triple is
    // dropped into the default graph or any other graph.
    for quad in &quads {
        assert_eq!(
            quad.graph_name.to_string(),
            format!("<{OTEL_GRAPH_IRI}>"),
            "quad not in urn:graph:otel: {quad}"
        );
    }

    let span_iri = "urn:otel:span:4bf92f3577b34da6a3ce929d0e0e4736:00f067aa0ba902b7";
    let texts: Vec<String> = quads.iter().map(|q| q.to_string()).collect();

    let expect_present = |needle: &str| {
        assert!(
            texts.iter().any(|t| t.contains(needle)),
            "expected a quad containing {needle:?}; got:\n{}",
            texts.join("\n")
        );
    };

    // Occurrence typing (public terms, no minted wrapper class).
    expect_present(&format!(
        "<{span_iri}> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/prov#Activity>"
    ));
    expect_present(&format!(
        "<{span_iri}> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://www.ocel-standard.org/2.0/Event>"
    ));
    // Raw OTLP structural fields (bridge-minted).
    expect_present(&format!(
        "<{span_iri}> <https://ggen.io/ontology/otel-bridge#traceId> \"4bf92f3577b34da6a3ce929d0e0e4736\""
    ));
    expect_present(&format!(
        "<{span_iri}> <https://ggen.io/ontology/otel-bridge#spanId> \"00f067aa0ba902b7\""
    ));
    expect_present(&format!(
        "<{span_iri}> <https://ggen.io/ontology/otel-bridge#spanStatusCode> \"ok\""
    ));
    // No parentSpanId triple for a root span (parent_span_id: None).
    assert!(
        !texts.iter().any(|t| t.contains("otel-bridge#parentSpanId")),
        "root span must not carry a parentSpanId triple"
    );
    // Semantic attributes: OCEL 2.0 public terms for object/object type,
    // bridge-minted for workflow id and outcome.
    expect_present(&format!(
        "<{span_iri}> <https://ggen.io/ontology/otel-bridge#workflowId> \"wf-42\""
    ));
    expect_present(&format!(
        "<{span_iri}> <https://ggen.io/ontology/otel-bridge#activityIri> <urn:praxis:activity:ship-order>"
    ));
    expect_present(&format!(
        "<{span_iri}> <https://ggen.io/ontology/otel-bridge#hasOutcome> <https://ggen.io/ontology/otel-bridge#OutcomeCompleted>"
    ));
    expect_present(
        "<urn:otel:object:order-7> <https://www.ocel-standard.org/2.0/objectId> \"order-7\"",
    );
    expect_present(
        "<urn:otel:objecttype:Order> <https://www.ocel-standard.org/2.0/objectTypeName> \"Order\"",
    );
    expect_present(&format!(
        "<{span_iri}> <https://www.ocel-standard.org/2.0/relatesTo> <urn:otel:object:order-7>"
    ));
    // Generic extra-attribute projection.
    expect_present("<https://www.ocel-standard.org/2.0/attributeKey> \"praxis.extra.note\"");
    expect_present("<https://www.ocel-standard.org/2.0/attributeValue> \"second-leg\"");
    expect_present(&format!(
        "<{span_iri}> <https://www.ocel-standard.org/2.0/hasAttribute>"
    ));
}

#[test]
fn trig_serialization_is_byte_identical_across_two_runs() {
    let spans = vec![admissible_span()];
    let first = admitted_spans_to_trig(&spans).expect("first serialization succeeds");
    let second = admitted_spans_to_trig(&spans).expect("second serialization succeeds");
    assert_eq!(
        first, second,
        "same input must produce byte-identical TriG output across runs"
    );
    assert!(
        first.contains(OTEL_GRAPH_IRI),
        "serialized TriG must name the urn:graph:otel graph: {first}"
    );
}

#[test]
fn trig_serialization_is_independent_of_attribute_order() {
    let mut reordered = admissible_span();
    reordered.attributes.reverse();
    let canonical = admitted_spans_to_trig(&[admissible_span()]).expect("serializes");
    let from_reordered = admitted_spans_to_trig(&[reordered]).expect("serializes");
    assert_eq!(
        canonical, from_reordered,
        "attribute order in the source span must not affect serialized output"
    );
}

#[test]
fn batch_projection_refuses_whole_batch_on_one_inadmissible_span() {
    let good = admissible_span();
    let mut bad = admissible_span();
    bad.span_id = "11f067aa0ba902b8".to_string();
    bad.attributes
        .retain(|(k, _)| k != telemetry_gen::ATTR_OBJECT_ID);
    match project_admitted_spans(&[good, bad]) {
        Err(CngRefusal::OtelSpanRefused { reason, .. }) => {
            assert!(reason.contains("process.object.id"), "reason was: {reason}");
        }
        other => panic!("expected OtelSpanRefused for the batch, got {other:?}"),
    }
}
