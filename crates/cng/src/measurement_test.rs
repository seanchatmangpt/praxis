#![cfg(test)]

use chicago_tdd_tools::prelude::*;
use oxigraph::model::{GraphName, NamedNode};
use oxigraph::store::Store;

use super::{
    build_measurement_profile, compute_execution_measure, project_measurement_profile,
    DeclaredProcessScale, ExecutionMeasure,
};
use crate::otel_ocel::{insert_quads, project_otel_to_ocel, RESULT_GRAPH_IRI};
use crate::otel_rdf::{otel_rdf_test::admissible_span, project_admitted_spans, OtlpSpan};
use crate::powl::CngRefusal;

/// A second admitted span for a different workflow family (`wf-99`,
/// activity `receive-payment`, object type `Payment` rather than `Order`),
/// so mass-by-family queries (workflow, activity, and object-type scales)
/// all have more than one distinct family to group over.
fn second_span() -> OtlpSpan {
    let mut span = admissible_span();
    span.trace_id = "5cf92f3577b34da6a3ce929d0e0e4737".to_string();
    span.span_id = "11f067aa0ba902b8".to_string();
    for (k, v) in span.attributes.iter_mut() {
        if k == crate::telemetry_gen::ATTR_WORKFLOW_ID {
            *v = "wf-99".to_string();
        }
        if k == crate::telemetry_gen::ATTR_ACTIVITY_IRI {
            *v = "urn:praxis:activity:receive-payment".to_string();
        }
        if k == crate::telemetry_gen::ATTR_OBJECT_ID {
            *v = "order-8".to_string();
        }
        if k == crate::telemetry_gen::ATTR_OBJECT_TYPE {
            *v = "Payment".to_string();
        }
    }
    span
}

/// Builds a store whose `urn:graph:ocel` graph holds two admitted spans
/// from two distinct workflow families / activities, via the real
/// PROJ-763/764 admission + CONSTRUCT pipeline (never hand-built OCEL).
fn store_with_two_families() -> Result<Store, CngRefusal> {
    let store = Store::new().map_err(|e| CngRefusal::IoRefused(format!("store: {e}")))?;
    let otel_quads = project_admitted_spans(&[admissible_span(), second_span()])?;
    insert_quads(&store, &otel_quads)?;
    let ocel_quads = project_otel_to_ocel(&store)?;
    insert_quads(&store, &ocel_quads)?;
    Ok(store)
}

fn graph_name_for(iri: &str) -> GraphName {
    GraphName::NamedNode(
        NamedNode::new(iri).unwrap_or_else(|e| panic!("bad test IRI {iri:?}: {e}")),
    )
}

test!(mass_by_workflow_groups_two_distinct_families, {
    let store = store_with_two_families()?;

    let mut measures = compute_execution_measure(&store, DeclaredProcessScale::Workflow)?;
    measures.sort_by(|a, b| a.family.cmp(&b.family));

    assert_eq!(
        measures,
        vec![
            ExecutionMeasure {
                family: "wf-42".to_string(),
                mass: 1,
            },
            ExecutionMeasure {
                family: "wf-99".to_string(),
                mass: 1,
            },
        ]
    );
});

test!(mass_by_activity_groups_two_distinct_families, {
    let store = store_with_two_families()?;

    let mut measures = compute_execution_measure(&store, DeclaredProcessScale::Activity)?;
    measures.sort_by(|a, b| a.family.cmp(&b.family));

    assert_eq!(
        measures,
        vec![
            ExecutionMeasure {
                family: "receive-payment".to_string(),
                mass: 1,
            },
            ExecutionMeasure {
                family: "ship-order".to_string(),
                mass: 1,
            },
        ]
    );
});

test!(
    scale_with_no_real_data_source_refuses_measurement_evidence_insufficient_cng_r29,
    {
        let store = store_with_two_families()?;

        match compute_execution_measure(&store, DeclaredProcessScale::EnterpriseGoal) {
            Err(refusal @ CngRefusal::MeasurementEvidenceInsufficient { .. }) => {
                assert_eq!(refusal.code(), "CNG_R29");
                match refusal {
                    CngRefusal::MeasurementEvidenceInsufficient { scale, reason } => {
                        assert_eq!(scale, "enterprise goal");
                        assert!(reason.contains("not instrumented anywhere upstream"));
                    }
                    _ => panic!("unreachable"),
                }
            }
            other => panic!("expected MeasurementEvidenceInsufficient, got {other:?}"),
        }
    }
);

test!(mass_by_object_type_groups_two_distinct_families, {
    let store = store_with_two_families()?;

    let mut measures =
        compute_execution_measure(&store, DeclaredProcessScale::ObjectCentricAggregationLevel)?;
    measures.sort_by(|a, b| a.family.cmp(&b.family));

    assert_eq!(
        measures,
        vec![
            ExecutionMeasure {
                family: "Order".to_string(),
                mass: 1,
            },
            ExecutionMeasure {
                family: "Payment".to_string(),
                mass: 1,
            },
        ]
    );
});

test!(
    each_of_the_eight_no_data_scales_refuses_with_a_distinct_scale_specific_reason,
    {
        let store = store_with_two_families()?;

        let no_data_scales = [
            DeclaredProcessScale::EnterpriseGoal,
            DeclaredProcessScale::Program,
            DeclaredProcessScale::Process,
            DeclaredProcessScale::Subprocess,
            DeclaredProcessScale::ChildWorkflow,
            DeclaredProcessScale::BrokerActuation,
            DeclaredProcessScale::RecursivePowlDepth,
            DeclaredProcessScale::BoundedExecutionCostBand,
        ];

        let mut reasons: Vec<String> = Vec::new();
        for scale in no_data_scales {
            match compute_execution_measure(&store, scale) {
                Err(CngRefusal::MeasurementEvidenceInsufficient { scale: got, reason }) => {
                    assert_eq!(got, scale.as_str());
                    assert!(
                        !reason.is_empty(),
                        "scale {got:?} must carry a non-empty, scale-specific refusal reason"
                    );
                    reasons.push(reason);
                }
                other => {
                    panic!("expected MeasurementEvidenceInsufficient for {scale:?}, got {other:?}")
                }
            }
        }

        let mut deduped = reasons.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(
            deduped.len(),
            reasons.len(),
            "every one of the 8 no-data scales must carry its own distinct reason, not a shared \
             blanket excuse: {reasons:?}"
        );
    }
);

test!(
    empty_ocel_graph_refuses_measurement_evidence_insufficient,
    {
        let store = Store::new().map_err(|e| CngRefusal::IoRefused(format!("store: {e}")))?;

        match compute_execution_measure(&store, DeclaredProcessScale::Workflow) {
            Err(CngRefusal::MeasurementEvidenceInsufficient { reason, .. }) => {
                assert!(reason.contains("zero admitted"));
            }
            other => panic!("expected MeasurementEvidenceInsufficient, got {other:?}"),
        }
    }
);

test!(
    build_measurement_profile_refuses_below_declared_min_evidence_threshold,
    {
        let store = store_with_two_families()?;

        // Two families are measured; declare a threshold of 3 -- must refuse.
        match build_measurement_profile(
            &store,
            DeclaredProcessScale::Workflow,
            vec![-5, -1, 0, 1, 5],
            "log-log-regression".to_string(),
            3,
            "n/a".to_string(),
        ) {
            Err(CngRefusal::MeasurementEvidenceInsufficient { reason, .. }) => {
                assert!(reason.contains("below the declared minimum evidence threshold of 3"));
            }
            other => panic!("expected MeasurementEvidenceInsufficient, got {other:?}"),
        }
    }
);

test!(
    build_measurement_profile_computes_source_ocel_digest_not_asserted,
    {
        let store = store_with_two_families()?;

        let (profile, measures) = build_measurement_profile(
            &store,
            DeclaredProcessScale::Workflow,
            vec![-5, -1, 0, 1, 5],
            "log-log-regression".to_string(),
            2,
            "n/a".to_string(),
        )?;

        assert_eq!(profile.scale, DeclaredProcessScale::Workflow);
        assert_eq!(measures.len(), 2);
        assert!(profile.source_ocel_digest.starts_with("blake3:"));

        let independently_recomputed =
            crate::otel_ocel::graph_content_digest(&store, crate::otel_ocel::OCEL_GRAPH_IRI)?;
        assert_eq!(
            profile.source_ocel_digest, independently_recomputed,
            "source_ocel_digest must equal an independently recomputed G_OCEL content digest"
        );
    }
);

test!(
    projected_measurement_profile_lands_exclusively_in_result_graph,
    {
        let store = store_with_two_families()?;
        let (profile, measures) = build_measurement_profile(
            &store,
            DeclaredProcessScale::Workflow,
            vec![-5, -1, 0, 1, 5],
            "log-log-regression".to_string(),
            2,
            "n/a".to_string(),
        )?;

        let quads = project_measurement_profile(&profile, &measures)?;
        assert!(!quads.is_empty());

        let result_graph = format!("<{RESULT_GRAPH_IRI}>");
        for quad in &quads {
            assert_eq!(
                quad.graph_name.to_string(),
                result_graph,
                "every projected quad must land in urn:graph:results: {quad}"
            );
        }
    }
);

test!(
    projected_measurement_profile_carries_one_execution_measure_per_family,
    {
        let store = store_with_two_families()?;
        let (profile, measures) = build_measurement_profile(
            &store,
            DeclaredProcessScale::Workflow,
            vec![-5, -1, 0, 1, 5],
            "log-log-regression".to_string(),
            2,
            "n/a".to_string(),
        )?;
        assert_eq!(measures.len(), 2);

        let quads = project_measurement_profile(&profile, &measures)?;
        let mass_pred = NamedNode::new("https://truex.io/ontology/cng-measurement#mass")
            .map_err(|e| CngRefusal::IoRefused(format!("bad predicate: {e}")))?;
        let mass_triples = quads.iter().filter(|q| q.predicate == mass_pred).count();
        assert_eq!(
            mass_triples,
            measures.len(),
            "expected one mn:mass triple per measured family"
        );
    }
);

test!(
    projected_measurement_profile_is_byte_identical_across_two_runs,
    {
        let store = store_with_two_families()?;
        let (profile, measures) = build_measurement_profile(
            &store,
            DeclaredProcessScale::Workflow,
            vec![-5, -1, 0, 1, 5],
            "log-log-regression".to_string(),
            2,
            "n/a".to_string(),
        )?;

        let first = project_measurement_profile(&profile, &measures)?;
        let second = project_measurement_profile(&profile, &measures)?;
        let first_text: Vec<String> = first.iter().map(|q| q.to_string()).collect();
        let second_text: Vec<String> = second.iter().map(|q| q.to_string()).collect();
        assert_eq!(
            first_text, second_text,
            "same profile + measures must project byte-identically across runs"
        );
    }
);

test!(inserted_measurement_profile_is_queryable_in_result_graph, {
    let store = store_with_two_families()?;
    let (profile, measures) = build_measurement_profile(
        &store,
        DeclaredProcessScale::Workflow,
        vec![-5, -1, 0, 1, 5],
        "log-log-regression".to_string(),
        2,
        "n/a".to_string(),
    )?;
    let quads = project_measurement_profile(&profile, &measures)?;
    insert_quads(&store, &quads)?;

    let result_graph = graph_name_for(RESULT_GRAPH_IRI);
    let count = store
        .quads_for_pattern(None, None, None, Some(result_graph.as_ref()))
        .count();
    assert_eq!(count, quads.len());
});

test!(declared_process_scale_as_str_matches_prd_wording, {
    assert_eq!(
        DeclaredProcessScale::ObjectCentricAggregationLevel.as_str(),
        "object-centric aggregation level"
    );
    assert_eq!(
        DeclaredProcessScale::BoundedExecutionCostBand.as_str(),
        "bounded execution cost band"
    );
    assert_eq!(
        DeclaredProcessScale::RecursivePowlDepth.as_str(),
        "recursive POWL depth"
    );
});
