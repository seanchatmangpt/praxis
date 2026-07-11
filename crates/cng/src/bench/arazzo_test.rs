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

use super::{
    admit_arazzo, default_description_path, project_steps, run_arazzo_projection,
    verify_arazzo_render_digest,
};
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
        // PROJ-745: run_arazzo_projection now gates on
        // verify_arazzo_render_digest(adapter.project_root()) before any
        // step dispatches — seed the scratch out_dir with a rendered YAML
        // and a matching ggen receipt so this unit test exercises the
        // dispatch-side wiring, not just the admit/project path.
        fs::create_dir_all(out_dir.join("generated")).expect("generated dir");
        fs::create_dir_all(out_dir.join(".ggen-v2")).expect(".ggen-v2 dir");
        let rendered_yaml: &[u8] = b"arazzo: \"1.1.0\"\ninfo:\n  title: projection_e2e\n";
        fs::write(out_dir.join("generated/arazzo.yaml"), rendered_yaml).expect("write yaml");
        let rendered_digest = blake3::hash(rendered_yaml).to_hex().to_string();
        fs::write(
            out_dir.join(".ggen-v2/receipt.json"),
            format!(
                "{{\"payload\":{{\"outputs\":{{\"generated/arazzo.yaml\":\"{rendered_digest}\"}}}}}}"
            ),
        )
        .expect("write receipt");

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

test!(arazzo_render_digest_match_verifies_against_ggen_receipt, {
    // Arrange: a scratch ggen project root with a rendered YAML and a
    // .ggen-v2/receipt.json recording that file's true BLAKE3 digest
    // (the exact seam: packs/arazzo-pack/README.md "Downstream
    // verification seam", per PROJ-745).
    let root = scratch_dir("render_digest_match");
    fs::create_dir_all(root.join("generated")).expect("generated dir");
    fs::create_dir_all(root.join(".ggen-v2")).expect(".ggen-v2 dir");
    let yaml_bytes: &[u8] = b"arazzo: \"1.1.0\"\ninfo:\n  title: test\n";
    fs::write(root.join("generated/arazzo.yaml"), yaml_bytes).expect("write yaml");
    let digest = blake3::hash(yaml_bytes).to_hex().to_string();
    let receipt =
        format!("{{\"payload\":{{\"outputs\":{{\"generated/arazzo.yaml\":\"{digest}\"}}}}}}");
    fs::write(root.join(".ggen-v2/receipt.json"), receipt).expect("write receipt");

    // Act.
    let result = verify_arazzo_render_digest(&root);

    // Assert: matching digest verifies Ok, carrying the recomputed
    // digest the caller uses to advance to DispatchState::ArazzoRendered.
    let verification = result.expect("matching digest verifies");
    assert_eq!(verification.output_path, "generated/arazzo.yaml");
    assert_eq!(verification.digest, digest);
});

test!(
    arazzo_render_digest_mismatch_refuses_cng_r11_audit_mismatch,
    {
        // Arrange: same shape as the matching case, but the rendered file
        // on disk is corrupted (one byte flipped) after the receipt
        // recorded the original digest — simulating a tampered/stale
        // render.
        let root = scratch_dir("render_digest_mismatch");
        fs::create_dir_all(root.join("generated")).expect("generated dir");
        fs::create_dir_all(root.join(".ggen-v2")).expect(".ggen-v2 dir");
        let original_bytes: &[u8] = b"arazzo: \"1.1.0\"\ninfo:\n  title: test\n";
        let recorded_digest = blake3::hash(original_bytes).to_hex().to_string();
        let receipt = format!(
            "{{\"payload\":{{\"outputs\":{{\"generated/arazzo.yaml\":\"{recorded_digest}\"}}}}}}"
        );
        fs::write(root.join(".ggen-v2/receipt.json"), receipt).expect("write receipt");
        let mut corrupted_bytes = original_bytes.to_vec();
        corrupted_bytes[0] ^= 0xFF; // flip a byte: no longer matches the receipt digest
        fs::write(root.join("generated/arazzo.yaml"), &corrupted_bytes)
            .expect("write corrupted yaml");

        // Act.
        let result = verify_arazzo_render_digest(&root);

        // Assert: typed CNG_R11 refusal naming the mismatch — never a
        // silent pass-through, never a panic.
        match result {
            Err(CngRefusal::AuditMismatch(msg)) => {
                assert!(
                    msg.contains("digest mismatch"),
                    "message names the mismatch, got {msg}"
                );
                assert_eq!(CngRefusal::AuditMismatch(msg).code(), "CNG_R11");
            }
            Err(other) => panic!("expected AuditMismatch, got {other:?}"),
            Ok(_) => {
                panic!("expected AuditMismatch for a corrupted rendered file, but verification succeeded")
            }
        }
    }
);
