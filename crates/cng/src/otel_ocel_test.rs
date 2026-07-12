#![cfg(test)]

use std::path::PathBuf;

use chicago_tdd_tools::prelude::*;
use oxigraph::model::{GraphName, NamedNode, NamedNodeRef};
use oxigraph::store::Store;

use super::{
    graph_content_digest, insert_quads, load_source_graph, project_otel_to_ocel, GRAPH_LAYERS,
    OCEL_GRAPH_IRI, OTEL_GRAPH_IRI, RECEIPT_GRAPH_IRI, RESULT_GRAPH_IRI, SOURCE_GRAPH_IRI,
};
use crate::otel_rdf::{otel_rdf_test::admissible_span, project_admitted_spans};
use crate::powl::CngRefusal;

/// Reads the same real ontology fixture `otel_rdf_test.rs` already parses
/// (`otel_bridge_ontology_is_valid_turtle`), reused here as `G_SOURCE`
/// content rather than a synthesized fixture.
fn otel_bridge_ttl() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("praxis-graphlaw")
        .join("ontologies")
        .join("core")
        .join("otel-bridge.ttl");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("otel-bridge.ttl must read: {e}"))
}

fn graph_name_for(iri: &str) -> GraphName {
    GraphName::NamedNode(
        NamedNode::new(iri).unwrap_or_else(|e| panic!("bad test IRI {iri:?}: {e}")),
    )
}

/// Quad count scoped to one named graph, via the typed pattern API.
///
/// # Complexity
/// O(matches).
fn graph_quad_count(store: &Store, iri: &str) -> usize {
    let graph = graph_name_for(iri);
    store
        .quads_for_pattern(None, None, None, Some(graph.as_ref()))
        .count()
}

/// Quad count scoped to one named graph AND one predicate.
///
/// # Complexity
/// O(matches).
fn graph_predicate_count(store: &Store, graph_iri: &str, predicate_iri: &str) -> usize {
    let graph = graph_name_for(graph_iri);
    let predicate = NamedNode::new(predicate_iri).unwrap_or_else(|e| panic!("bad predicate: {e}"));
    store
        .quads_for_pattern(None, Some(predicate.as_ref()), None, Some(graph.as_ref()))
        .count()
}

/// Builds a store whose `urn:graph:otel` graph holds PROJ-763's admitted
/// fixture span, returning the store and the exact `Vec<Quad>` inserted.
fn store_with_admitted_span() -> Result<(Store, Vec<oxigraph::model::Quad>), CngRefusal> {
    let store = Store::new().map_err(|e| CngRefusal::IoRefused(format!("store: {e}")))?;
    let quads = project_admitted_spans(&[admissible_span()])?;
    insert_quads(&store, &quads)?;
    Ok((store, quads))
}

// Regression test for a real oxigraph 0.5.9 arithmetic-association bug
// found while building this query (see the .rq fixture's own header):
// isolates the calendar-math sub-expression otel-to-ocel.construct.rq
// uses for ocel:timestamp and asserts every derived field against two
// independently computed reference dates (datetime.utcfromtimestamp in
// Python) -- 1_700_000_000 -> 2023-11-14T22:13:20Z (era-typical case) and
// 4_107_542_400 -> 2100-03-01T00:00:00Z (the doe36524/leap-adjustment
// boundary case the associativity bug specifically corrupted).
test!(calendar_math_fixture_matches_independent_reference_dates, {
    use oxigraph::sparql::{QueryResults, SparqlEvaluator};

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("queries")
        .join("otel-ocel-calendar-math.rq");
    let query_text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("otel-ocel-calendar-math.rq must read: {e}"));

    let store = Store::new().map_err(|e| CngRefusal::IoRefused(format!("store: {e}")))?;
    let prepared = SparqlEvaluator::new()
        .parse_query(&query_text)
        .map_err(|e| CngRefusal::OcelConstructRefused {
            stage: "test_setup".to_string(),
            reason: e.to_string(),
        })?;
    let results =
        prepared
            .on_store(&store)
            .execute()
            .map_err(|e| CngRefusal::OcelConstructRefused {
                stage: "test_setup".to_string(),
                reason: e.to_string(),
            })?;

    let mut rows: Vec<(String, String, String, String, String, String)> = Vec::new();
    match results {
        QueryResults::Solutions(sols) => {
            for sol in sols {
                let sol = sol.map_err(|e| CngRefusal::OcelConstructRefused {
                    stage: "test_setup".to_string(),
                    reason: e.to_string(),
                })?;
                let get = |name: &str| {
                    sol.get(name)
                        .map(|t| t.to_string())
                        .unwrap_or_else(|| "<UNBOUND>".to_string())
                };
                rows.push((
                    get("year"),
                    get("month"),
                    get("day"),
                    get("hh"),
                    get("mi"),
                    get("ss"),
                ));
            }
        }
        _ => panic!("expected SELECT solutions from otel-ocel-calendar-math.rq"),
    }

    assert_eq!(
        rows.len(),
        2,
        "fixture supplies exactly two reference dates"
    );
    let expected_2023 = (
        "\"2023\"^^<http://www.w3.org/2001/XMLSchema#integer>".to_string(),
        "\"11\"^^<http://www.w3.org/2001/XMLSchema#integer>".to_string(),
        "\"14\"^^<http://www.w3.org/2001/XMLSchema#integer>".to_string(),
        "\"22\"^^<http://www.w3.org/2001/XMLSchema#integer>".to_string(),
        "\"13\"^^<http://www.w3.org/2001/XMLSchema#integer>".to_string(),
        "\"20\"^^<http://www.w3.org/2001/XMLSchema#integer>".to_string(),
    );
    let expected_2100 = (
        "\"2100\"^^<http://www.w3.org/2001/XMLSchema#integer>".to_string(),
        "\"3\"^^<http://www.w3.org/2001/XMLSchema#integer>".to_string(),
        "\"1\"^^<http://www.w3.org/2001/XMLSchema#integer>".to_string(),
        "\"0\"^^<http://www.w3.org/2001/XMLSchema#integer>".to_string(),
        "\"0\"^^<http://www.w3.org/2001/XMLSchema#integer>".to_string(),
        "\"0\"^^<http://www.w3.org/2001/XMLSchema#integer>".to_string(),
    );
    assert_eq!(
        rows[0], expected_2023,
        "1_700_000_000 must derive 2023-11-14T22:13:20Z"
    );
    assert_eq!(
        rows[1], expected_2100,
        "4_107_542_400 must derive 2100-03-01T00:00:00Z"
    );
});

test!(graph_layers_are_five_distinct_iris, {
    assert_eq!(GRAPH_LAYERS.len(), 5, "handoff doc names exactly 5 layers");
    let mut sorted = GRAPH_LAYERS.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        5,
        "the 5 layer IRIs must be genuinely distinct strings, not aliases"
    );
    assert_eq!(GRAPH_LAYERS[0], SOURCE_GRAPH_IRI);
    assert_eq!(GRAPH_LAYERS[1], OTEL_GRAPH_IRI);
    assert_eq!(GRAPH_LAYERS[2], OCEL_GRAPH_IRI);
    assert_eq!(GRAPH_LAYERS[3], RESULT_GRAPH_IRI);
    assert_eq!(GRAPH_LAYERS[4], RECEIPT_GRAPH_IRI);
});

test!(construct_projects_expected_ocel_event_and_object_triples, {
    let (store, _otel_quads) = store_with_admitted_span()?;

    let ocel_quads = project_otel_to_ocel(&store)?;
    assert!(!ocel_quads.is_empty(), "projection must produce triples");

    for quad in &ocel_quads {
        assert_eq!(
            quad.graph_name.to_string(),
            format!("<{OCEL_GRAPH_IRI}>"),
            "every projected quad must land in urn:graph:ocel: {quad}"
        );
    }

    let texts: Vec<String> = ocel_quads.iter().map(|q| q.to_string()).collect();
    let expect_present = |needle: &str| {
        assert!(
            texts.iter().any(|t| t.contains(needle)),
            "expected a quad containing {needle:?}; got:\n{}",
            texts.join("\n")
        );
    };
    let expect_absent = |needle: &str| {
        assert!(
            !texts.iter().any(|t| t.contains(needle)),
            "did not expect any quad containing {needle:?} in a pure OCEL projection; got:\n{}",
            texts.join("\n")
        );
    };

    let span_iri = "urn:otel:span:4bf92f3577b34da6a3ce929d0e0e4736:00f067aa0ba902b7";

    // Event core fields, derived (not copied) from G_OTEL bridge properties.
    expect_present(&format!(
        "<{span_iri}> <https://www.ocel-standard.org/2.0/eventId> \
         \"4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7\""
    ));
    expect_present(&format!(
        "<{span_iri}> <https://www.ocel-standard.org/2.0/activityName> \"ship-order\""
    ));
    expect_present(&format!(
        "<{span_iri}> <https://www.ocel-standard.org/2.0/hasEventType> \
         <urn:otel:ocel:eventtype:ship-order>"
    ));
    expect_present(
        "<urn:otel:ocel:eventtype:ship-order> <https://www.ocel-standard.org/2.0/eventTypeName> \
         \"ship-order\"",
    );
    expect_present(&format!(
        "<{span_iri}> <https://www.ocel-standard.org/2.0/timestamp>"
    ));

    // process.outcome / process.workflow.id become OCEL data-payload
    // attributes, not bridge-minted object properties.
    expect_present(
        "<urn:otel:ocel:attr:4bf92f3577b34da6a3ce929d0e0e4736:00f067aa0ba902b7:outcome> \
         <https://www.ocel-standard.org/2.0/attributeKey> \"process.outcome\"",
    );
    expect_present(
        "<urn:otel:ocel:attr:4bf92f3577b34da6a3ce929d0e0e4736:00f067aa0ba902b7:outcome> \
         <https://www.ocel-standard.org/2.0/attributeValue> \"completed\"",
    );
    expect_present(
        "<urn:otel:ocel:attr:4bf92f3577b34da6a3ce929d0e0e4736:00f067aa0ba902b7:workflowId> \
         <https://www.ocel-standard.org/2.0/attributeKey> \"process.workflow.id\"",
    );
    expect_present(
        "<urn:otel:ocel:attr:4bf92f3577b34da6a3ce929d0e0e4736:00f067aa0ba902b7:workflowId> \
         <https://www.ocel-standard.org/2.0/attributeValue> \"wf-42\"",
    );

    // Already-OCEL-shaped Object/ObjectType/relatesTo pass through unchanged.
    expect_present(&format!(
        "<{span_iri}> <https://www.ocel-standard.org/2.0/relatesTo> <urn:otel:object:order-7>"
    ));
    expect_present(
        "<urn:otel:object:order-7> <https://www.ocel-standard.org/2.0/objectId> \"order-7\"",
    );
    expect_present(
        "<urn:otel:objecttype:Order> <https://www.ocel-standard.org/2.0/objectTypeName> \"Order\"",
    );

    // Generic extra-attribute passthrough (praxis.extra.note fixture value).
    expect_present(&format!(
        "<{span_iri}> <https://www.ocel-standard.org/2.0/hasAttribute> \
         <urn:otel:attr:4bf92f3577b34da6a3ce929d0e0e4736:00f067aa0ba902b7:praxis.extra.note>"
    ));
    expect_present(
        "<urn:otel:attr:4bf92f3577b34da6a3ce929d0e0e4736:00f067aa0ba902b7:praxis.extra.note> \
         <https://www.ocel-standard.org/2.0/attributeValue> \"second-leg\"",
    );

    // G_OCEL is a pure OCEL 2.0 projection: raw OTLP wire-protocol fields
    // that have no OCEL equivalent must not be copied across.
    expect_absent("otel-bridge#traceId");
    expect_absent("otel-bridge#spanId");
    expect_absent("otel-bridge#spanStatusCode");
    expect_absent("otel-bridge#startTimeUnixNano");
    expect_absent("otel-bridge#activityIri");
    expect_absent("otel-bridge#hasOutcome");
    expect_absent("otel-bridge#workflowId");
});

test!(
    construct_timestamp_is_derived_from_producer_payload_not_wall_clock,
    {
        let (store, _quads) = store_with_admitted_span()?;
        let ocel_quads = project_otel_to_ocel(&store)?;

        let span_iri =
            NamedNode::new("urn:otel:span:4bf92f3577b34da6a3ce929d0e0e4736:00f067aa0ba902b7")
                .map_err(|e| CngRefusal::OcelConstructRefused {
                    stage: "test_setup".to_string(),
                    reason: e.to_string(),
                })?;
        let ts_predicate =
            NamedNode::new("https://www.ocel-standard.org/2.0/timestamp").map_err(|e| {
                CngRefusal::OcelConstructRefused {
                    stage: "test_setup".to_string(),
                    reason: e.to_string(),
                }
            })?;
        let ts_values: Vec<String> = ocel_quads
            .iter()
            .filter(|q| {
                // NamedNode's Display already renders the enclosing angle
                // brackets (`<iri>`); no extra wrapping needed here.
                q.subject.to_string() == span_iri.to_string()
                    && q.predicate == NamedNodeRef::from(ts_predicate.as_ref())
            })
            .map(|q| q.object.to_string())
            .collect();
        assert_eq!(
            ts_values.len(),
            1,
            "exactly one ocel:timestamp triple expected, got {ts_values:?}"
        );
        // start_time_unix_nano = 1_700_000_000_000_000_000 (admissible_span
        // fixture) -> 1_700_000_000 whole seconds since the Unix epoch ->
        // 2023-11-14T22:13:20Z, computed purely from that producer-supplied
        // payload value (see otel-to-ocel.construct.rq's own no-wall-clock
        // determinism note).
        assert!(
            ts_values[0].contains("2023-11-14T22:13:20"),
            "timestamp must be derived from ob:startTimeUnixNano's payload value, got {:?}",
            ts_values[0]
        );
        assert!(
            ts_values[0].contains("dateTime"),
            "timestamp literal must be typed xsd:dateTime, got {:?}",
            ts_values[0]
        );
    }
);

test!(construct_result_is_byte_identical_across_two_runs, {
    let (store, _quads) = store_with_admitted_span()?;

    let first = project_otel_to_ocel(&store)?;
    let second = project_otel_to_ocel(&store)?;

    let first_text: Vec<String> = first.iter().map(|q| q.to_string()).collect();
    let second_text: Vec<String> = second.iter().map(|q| q.to_string()).collect();
    assert_eq!(
        first_text, second_text,
        "same G_OTEL input must produce byte-identical G_OCEL output across runs"
    );
    assert!(
        !first_text.is_empty(),
        "sanity: projection must be non-trivial"
    );
});

test!(
    construct_on_empty_otel_graph_yields_empty_ocel_without_refusing,
    {
        let store = Store::new().map_err(|e| CngRefusal::IoRefused(format!("store: {e}")))?;
        let ocel_quads = project_otel_to_ocel(&store)?;
        assert!(
            ocel_quads.is_empty(),
            "an empty urn:graph:otel must construct an empty urn:graph:ocel, not a refusal"
        );
    }
);

test!(load_source_graph_refuses_malformed_turtle_cng_r28, {
    let store = Store::new().map_err(|e| CngRefusal::IoRefused(format!("store: {e}")))?;
    // No Turtle prefix declaration needed to be malformed: an unterminated
    // string literal is a syntax error on its own (and avoids embedding the
    // guarded Turtle-prefix-sigil needle that no_inline_ttl_guard.rs forbids
    // outside the serializer).
    let malformed = "<urn:ex:s> <urn:ex:p> \"unterminated";
    match load_source_graph(&store, malformed) {
        Err(refusal @ CngRefusal::OcelConstructRefused { .. }) => {
            assert_eq!(refusal.code(), "CNG_R28");
            match refusal {
                CngRefusal::OcelConstructRefused { stage, .. } => {
                    assert_eq!(stage, "source_load");
                }
                _ => panic!("unreachable"),
            }
        }
        other => panic!("expected OcelConstructRefused, got {other:?}"),
    }
});

test!(graph_content_digest_refuses_malformed_graph_iri_cng_r28, {
    let store = Store::new().map_err(|e| CngRefusal::IoRefused(format!("store: {e}")))?;
    // Not a legal absolute IRI (no scheme, embedded whitespace) -- this must
    // return a typed refusal, never panic, even though `graph_content_digest`
    // is a public function reachable with arbitrary caller-supplied input.
    let malformed = "not an iri at all";
    match graph_content_digest(&store, malformed) {
        Err(refusal @ CngRefusal::OcelConstructRefused { .. }) => {
            assert_eq!(refusal.code(), "CNG_R28");
            match refusal {
                CngRefusal::OcelConstructRefused { stage, .. } => {
                    assert_eq!(stage, "graph_iri_parse");
                }
                _ => panic!("unreachable"),
            }
        }
        other => panic!("expected OcelConstructRefused, got {other:?}"),
    }
});

test!(graph_content_digest_succeeds_on_a_real_graph_iri, {
    let (store, _quads) = store_with_admitted_span()?;
    // Sanity companion to the negative test above: a legal, populated graph
    // IRI must still succeed through the same code path.
    let digest = graph_content_digest(&store, OTEL_GRAPH_IRI)?;
    assert!(
        digest.starts_with("blake3:"),
        "digest must be blake3-prefixed, got {digest:?}"
    );
});

test!(five_layers_are_genuinely_separate_named_graphs, {
    let store = Store::new().map_err(|e| CngRefusal::IoRefused(format!("store: {e}")))?;

    let source_count = load_source_graph(&store, &otel_bridge_ttl())?;
    let otel_quads = project_admitted_spans(&[admissible_span()])?;
    insert_quads(&store, &otel_quads)?;
    let ocel_quads = project_otel_to_ocel(&store)?;
    insert_quads(&store, &ocel_quads)?;

    // Each populated layer holds real, non-trivial content.
    assert!(source_count > 100, "G_SOURCE must hold the full ontology");
    assert!(!otel_quads.is_empty());
    assert!(!ocel_quads.is_empty());

    // Every quad landed under its own named graph, not the default graph.
    let default_count = store
        .quads_for_pattern(None, None, None, Some(GraphName::DefaultGraph.as_ref()))
        .count();
    assert_eq!(default_count, 0, "no quad may land in the default graph");

    // Per-graph counts match exactly what was inserted into that graph.
    assert_eq!(graph_quad_count(&store, SOURCE_GRAPH_IRI), source_count);
    assert_eq!(graph_quad_count(&store, OTEL_GRAPH_IRI), otel_quads.len());
    assert_eq!(graph_quad_count(&store, OCEL_GRAPH_IRI), ocel_quads.len());

    // G_RESULT / G_RECEIPT are declared, distinct, reserved graph names —
    // the separation mechanism exists — but this ticket leaves their
    // content unpopulated (PROJ-765 / PROJ-766's scope; see module doc).
    assert_eq!(graph_quad_count(&store, RESULT_GRAPH_IRI), 0);
    assert_eq!(graph_quad_count(&store, RECEIPT_GRAPH_IRI), 0);

    // Genuinely separate content, not five names on one shared triple set:
    // ocel:eventId only ever appears in G_OCEL, never in G_SOURCE or
    // G_OTEL; the raw OTLP bridge property ob:traceId only ever appears in
    // G_OTEL, never in G_SOURCE (schema-only, no instance data) or G_OCEL
    // (pure OCEL projection, no bridge-only fields).
    let event_id_pred = "https://www.ocel-standard.org/2.0/eventId";
    assert_eq!(
        graph_predicate_count(&store, SOURCE_GRAPH_IRI, event_id_pred),
        0
    );
    assert_eq!(
        graph_predicate_count(&store, OTEL_GRAPH_IRI, event_id_pred),
        0
    );
    assert!(graph_predicate_count(&store, OCEL_GRAPH_IRI, event_id_pred) > 0);

    let trace_id_pred = "https://ggen.io/ontology/otel-bridge#traceId";
    assert_eq!(
        graph_predicate_count(&store, SOURCE_GRAPH_IRI, trace_id_pred),
        0
    );
    assert!(graph_predicate_count(&store, OTEL_GRAPH_IRI, trace_id_pred) > 0);
    assert_eq!(
        graph_predicate_count(&store, OCEL_GRAPH_IRI, trace_id_pred),
        0
    );

    // Whole-store sanity: total quads equal the sum of the three populated
    // layers plus the two empty reserved ones.
    let total = store
        .len()
        .map_err(|e| CngRefusal::IoRefused(format!("len: {e}")))?;
    assert_eq!(total, source_count + otel_quads.len() + ocel_quads.len());
});
