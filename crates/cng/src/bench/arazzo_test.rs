#![cfg(test)]

//! Arazzo projection tests (PROJ-621): the refused-feature fixture
//! (CNG_R18 naming the feature), typed step projection from the admitted
//! example (order + dependsOn + retry law), and the end-to-end projection
//! through the loopback dispatch adapter. Fixture RDF enters only from
//! on-disk files; SPARQL only from the on-disk query set.

use std::fs;
use std::path::PathBuf;

use chicago_tdd_tools::prelude::*;
use oxigraph::store::Store;

use super::{admit_arazzo, default_description_path, project_steps, run_arazzo_projection};
use crate::bench::dispatch::DispatchAdapter;
use crate::bench::roles::ObsWriter;
use crate::bench::templates::{load_templates, QuerySet};
use crate::powl::CngRefusal;

/// Scratch root for this test file. O(1).
fn scratch_dir(test_name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests/arazzo")
        .join(test_name);
    let _ = fs::remove_dir_all(&dir);
    dir
}

test!(
    xpath_criterion_fixture_refuses_cng_r18_naming_the_feature,
    {
        // Arrange: the on-disk fixture using the REFUSED criterionType "xpath".
        let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/negative/arazzo-xpath-criterion.ttl");

        // Act: admission is the profile gate.
        let result = admit_arazzo(&fixture, &queries);

        // Assert: typed CNG_R18 naming the refused feature.
        match result {
            Err(CngRefusal::ArazzoProfileRefused { feature }) => {
                assert!(
                    feature.contains("xpath"),
                    "refusal names the feature, got {feature}"
                );
                assert_eq!(
                    CngRefusal::ArazzoProfileRefused { feature }.code(),
                    "CNG_R18"
                );
            }
            Err(other) => panic!("expected ArazzoProfileRefused, got {other:?}"),
            Ok(_) => panic!("expected ArazzoProfileRefused, but the fixture admitted"),
        }
    }
);

test!(example_description_projects_four_ordered_steps, {
    // Arrange: the shipped 4-step order-fulfillment example.
    let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
    let store = admit_arazzo(&default_description_path(), &queries)
        .expect("shipped example admits under the 80/20 profile");

    // Act.
    let steps = project_steps(&store).expect("projection succeeds");

    // Assert: typed step structure — stepIndex order, dependsOn chain,
    // declared retry law on authorizePayment (declarative-only ticks).
    let ids: Vec<&str> = steps.iter().map(|s| s.step_id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "createOrder",
            "authorizePayment",
            "capturePayment",
            "confirmOrder"
        ]
    );
    assert_eq!(steps[0].depends_on, Vec::<String>::new());
    assert_eq!(steps[1].depends_on, vec!["createOrder".to_string()]);
    assert_eq!(steps[2].depends_on, vec!["authorizePayment".to_string()]);
    assert_eq!(steps[3].depends_on, vec!["capturePayment".to_string()]);
    assert!(
        steps[1].retry_law.contains("limit=3"),
        "retry law carries retryLimit, got {}",
        steps[1].retry_law
    );
    assert!(steps[1].retry_law.contains("declarative-only"));
    assert_eq!(steps[0].operation_id, "createOrder");
    assert_eq!(steps[0].parameters, vec!["customerId", "sku"]);
});

test!(
    arazzo_projection_dispatches_every_step_through_the_loopback_adapter,
    {
        // Arrange: a fresh obs store + adapter (no workday needed — the
        // projection is the unit under test).
        let out_dir = scratch_dir("projection_e2e");
        let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
        let templates = load_templates().expect("templates load");
        let store = Store::new().expect("store");
        let mut writer =
            ObsWriter::new(&templates, &store, &out_dir.join("obs"), "test").expect("writer");
        let mut adapter = DispatchAdapter::new(&out_dir, &queries).expect("adapter constructs");

        // Act: admit + validate + project + dispatch, dependsOn enforced.
        let dispatched = run_arazzo_projection(
            &mut adapter,
            &mut writer,
            &store,
            &default_description_path(),
            "tick-0000",
            "api-orchestration",
            0,
        )
        .expect("projection dispatches end-to-end");

        // Assert: all four steps dispatched and admitted through the broker.
        assert_eq!(dispatched, 4);
        assert_eq!(adapter.telemetry.sent, 4);
        assert_eq!(adapter.telemetry.admitted, 4);
        assert_eq!(adapter.telemetry.refused, 0);
        assert_eq!(adapter.telemetry.timeouts, 0);
        assert_eq!(adapter.receipt_digests.len(), 4);
    }
);
