#![cfg(test)]

//! External dispatch tests (PROJ-618/619/620): contract completeness
//! (CNG_R15), state-machine law (CNG_R16), staged lawful re-entry
//! (CNG_R17 at the named stage), refused-conformance compensation, timeout
//! escalation, closure laws via the on-disk dispatch-closure.rq, and
//! byte-determinism of the dispatch receipt digests. All Turtle enters
//! from on-disk templates/fixtures; all SPARQL from the on-disk query set.

use std::fs;
use std::path::PathBuf;

use chicago_tdd_tools::prelude::*;
use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::model::{LiteralRef, NamedNodeRef, TermRef};
use oxigraph::store::Store;

use super::{
    collect_consequence, workday_contract, DispatchAdapter, DispatchContract, DispatchOutcome,
    DispatchState, ExecutionClass, SynthesisMode,
};
use crate::bench::fill_template;
use crate::bench::roles::{select_rows, ObsWriter};
use crate::bench::templates::{load_templates, QuerySet};
use crate::powl::CngRefusal;

/// Scratch root for this test file. O(1).
fn scratch_dir(test_name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests/dispatch")
        .join(test_name);
    let _ = fs::remove_dir_all(&dir);
    dir
}

/// Crate-root path helper. O(1).
fn crate_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// Leaf fixture contract (no children, no closure). O(1).
fn fixture_contract() -> DispatchContract {
    let mut c = workday_contract(
        "fixture",
        "software-delivery",
        1,
        ExecutionClass::ExternalMachineDispatch,
    );
    c.recursive_depth = 0;
    c.closure_law = None;
    c
}

/// Count of observations with the given obsKind in `store` (typed oxigraph
/// pattern scan). O(matches).
fn kind_count(store: &Store, kind: &str) -> usize {
    let pred = NamedNodeRef::new("https://ggen.io/ontology/bench-obs#obsKind")
        .expect("obsKind IRI is valid");
    let lit = LiteralRef::new_simple_literal(kind);
    store
        .quads_for_pattern(None, Some(pred), Some(TermRef::from(lit)), None)
        .count()
}

test!(contract_missing_field_refuses_cng_r15, {
    // Arrange: a contract whose target actor and retry law are empty.
    let template = fs::read_to_string(crate_path("templates/dispatch-contract.template.ttl"))
        .expect("contract template reads");
    let mut contract = fixture_contract();
    contract.target_actor = String::new();
    contract.retry_law = "  ".to_string();

    // Act: render is the gate BEFORE the contract can leave the broker.
    let result = contract.render(&template);

    // Assert: typed CNG_R15 naming the dispatch and every missing field.
    match result {
        Err(CngRefusal::DispatchContractIncomplete { dispatch, missing }) => {
            assert_eq!(dispatch, "disp-fixture");
            assert!(missing.contains("TARGET_ACTOR"), "got {missing}");
            assert!(missing.contains("RETRY_LAW"), "got {missing}");
            assert_eq!(
                CngRefusal::DispatchContractIncomplete { dispatch, missing }.code(),
                "CNG_R15"
            );
        }
        other => panic!("expected DispatchContractIncomplete, got {other:?}"),
    }
});

test!(unlawful_state_transition_refuses_cng_r16, {
    // Arrange: a freshly manufactured contract.
    let mut contract = fixture_contract();

    // Act: MANUFACTURED -> COMPLETED skips the whole machine.
    let result = contract.advance(DispatchState::Completed);

    // Assert: typed CNG_R16 naming the dispatch and both states.
    match result {
        Err(CngRefusal::DispatchStateUnlawful { dispatch, from, to }) => {
            assert_eq!(dispatch, "disp-fixture");
            assert_eq!(from, "MANUFACTURED");
            assert_eq!(to, "COMPLETED");
            assert_eq!(
                CngRefusal::DispatchStateUnlawful { dispatch, from, to }.code(),
                "CNG_R16"
            );
        }
        other => panic!("expected DispatchStateUnlawful, got {other:?}"),
    }
});

test!(forged_inbox_correlation_refuses_at_correlation_stage, {
    // Arrange: the on-disk forged-correlation consequence fixture against
    // the matching fixture contract.
    let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
    let contract = fixture_contract();
    let consequence_ttl = fs::read_to_string(crate_path(
        "tests/fixtures/negative/dispatch-consequence-forged-correlation.ttl",
    ))
    .expect("fixture reads");

    // Act: the staged re-entry pipeline.
    let result = collect_consequence(
        &consequence_ttl,
        &contract,
        &crate_path("shapes/dispatch-shapes.ttl"),
        &queries,
    );

    // Assert: refused at EXACTLY the correlation stage (provenance passed).
    match result {
        Err(CngRefusal::ExternalConsequenceRefused { dispatch, stage }) => {
            assert_eq!(dispatch, "disp-fixture");
            assert_eq!(stage, "correlation");
            assert_eq!(
                CngRefusal::ExternalConsequenceRefused { dispatch, stage }.code(),
                "CNG_R17"
            );
        }
        other => panic!("expected ExternalConsequenceRefused, got {other:?}"),
    }
});

test!(
    semantic_conformance_failure_refuses_and_manufactures_compensation,
    {
        // Arrange: a wrong-artifact consequence fixture (correlation filled from
        // the contract, so only the semantic stage can refuse) injected through
        // the adapter's fixture seam.
        let out_dir = scratch_dir("semantic_refusal");
        let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
        let templates = load_templates().expect("templates load");
        let store = Store::new().expect("store");
        let mut writer =
            ObsWriter::new(&templates, &store, &out_dir.join("obs"), "test").expect("writer");
        let mut adapter = DispatchAdapter::new(&out_dir, &queries).expect("adapter constructs");
        let contract = fixture_contract();
        let fixture_raw = fs::read_to_string(crate_path(
            "tests/fixtures/negative/dispatch-consequence-wrong-artifact.ttl",
        ))
        .expect("fixture reads");
        let filled = fill_template(
            &fixture_raw,
            &[("CORRELATION_ID", contract.correlation_id.as_str())],
        );
        let fixture_path = out_dir.join("wrong-artifact.ttl");
        fs::write(&fixture_path, &filled).expect("fixture writes");

        // Act: full lifecycle with the fixture inbox, remediation budget 1.
        let outcome = adapter
            .dispatch(
                &mut writer,
                &store,
                contract,
                1,
                false,
                SynthesisMode::FixtureFile(&fixture_path),
                1,
            )
            .expect("lifecycle completes (refusal is evidence, not an error)");

        // Assert: refused at the semantic stage; the refusal landed as
        // observation evidence AND the declared compensation workflow was
        // manufactured and admitted through the same broker.
        assert_eq!(
            outcome,
            DispatchOutcome::Refused {
                stage: "semantic".to_string()
            }
        );
        assert_eq!(kind_count(&store, "consequence_refused"), 1);
        assert_eq!(kind_count(&store, "remediation_manufactured"), 1);
        assert_eq!(kind_count(&store, "consequence_admitted"), 1);
        assert_eq!(adapter.telemetry.refused, 1);
        assert_eq!(adapter.telemetry.remediations, 1);
        assert_eq!(adapter.telemetry.admitted, 1);
    }
);

test!(deadline_expiry_times_out_and_manufactures_escalation, {
    // Arrange: a contract whose deadline (0 logical ticks) expires before
    // any loopback consequence can arrive.
    let out_dir = scratch_dir("timeout_escalation");
    let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
    let templates = load_templates().expect("templates load");
    let store = Store::new().expect("store");
    let mut writer =
        ObsWriter::new(&templates, &store, &out_dir.join("obs"), "test").expect("writer");
    let mut adapter = DispatchAdapter::new(&out_dir, &queries).expect("adapter constructs");
    let mut contract = fixture_contract();
    contract.deadline_ticks = 0;

    // Act.
    let outcome = adapter
        .dispatch(
            &mut writer,
            &store,
            contract,
            2,
            false,
            SynthesisMode::LoopbackDeterministic,
            1,
        )
        .expect("lifecycle completes (timeout is evidence, not an error)");

    // Assert: TIMED_OUT, the escalation workflow manufactured through the
    // same broker, and the escalation's own consequence admitted.
    assert_eq!(outcome, DispatchOutcome::TimedOut);
    assert_eq!(kind_count(&store, "dispatch_timed_out"), 1);
    assert_eq!(kind_count(&store, "remediation_manufactured"), 1);
    assert_eq!(kind_count(&store, "consequence_admitted"), 1);
    assert_eq!(adapter.telemetry.timeouts, 1);
    assert_eq!(adapter.telemetry.remediations, 1);
});

/// Loads one dispatch_sent observation (template-rendered) into `store`.
/// O(|template|).
fn emit_sent(
    store: &Store,
    templates: &crate::bench::templates::Templates,
    seq: usize,
    dispatch_id: &str,
    parent: &str,
    law: &str,
) {
    let template = templates
        .obs
        .get("dispatch-sent")
        .expect("dispatch-sent template present");
    let seq_text = seq.to_string();
    let body = fill_template(
        template,
        &[
            ("SUBJECT", format!("obs-closure-{seq}").as_str()),
            ("SEQ", seq_text.as_str()),
            ("SET_ID", dispatch_id),
            ("TICK", "1"),
            ("DISPATCH_ID", dispatch_id),
            ("PARENT_DISPATCH", parent),
            ("EXECUTION_CLASS", "EXTERNAL_MACHINE_DISPATCH"),
            ("CORRELATION_ID", format!("corr-{dispatch_id}").as_str()),
            ("CLOSURE_LAW", law),
            ("DEADLINE_TICKS", "8"),
        ],
    );
    store
        .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), body.as_bytes())
        .expect("dispatch-sent observation parses");
}

/// Loads one consequence_admitted observation into `store`. O(|template|).
fn emit_admitted(
    store: &Store,
    templates: &crate::bench::templates::Templates,
    seq: usize,
    dispatch_id: &str,
) {
    let template = templates
        .obs
        .get("consequence-admitted")
        .expect("consequence-admitted template present");
    let seq_text = seq.to_string();
    let body = fill_template(
        template,
        &[
            ("SUBJECT", format!("obs-closure-adm-{seq}").as_str()),
            ("SEQ", seq_text.as_str()),
            ("SET_ID", dispatch_id),
            ("DISPATCH_ID", dispatch_id),
            ("CORRELATION_ID", format!("corr-{dispatch_id}").as_str()),
        ],
    );
    store
        .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), body.as_bytes())
        .expect("consequence-admitted observation parses");
}

test!(
    closure_all_children_required_with_unadmitted_child_stays_open,
    {
        // Arrange: parent under ALL_CHILDREN_REQUIRED with two children, only
        // one of which has an admitted consequence.
        let templates = load_templates().expect("templates load");
        let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
        let store = Store::new().expect("store");
        emit_sent(
            &store,
            &templates,
            0,
            "disp-parent",
            "none",
            "ALL_CHILDREN_REQUIRED",
        );
        emit_sent(
            &store,
            &templates,
            1,
            "disp-parent-c0",
            "disp-parent",
            "NONE",
        );
        emit_sent(
            &store,
            &templates,
            2,
            "disp-parent-c1",
            "disp-parent",
            "NONE",
        );
        emit_admitted(&store, &templates, 3, "disp-parent-c0");

        // Act: closure is READ from the on-disk query, never inferred.
        let rows = select_rows(
            &store,
            queries.get("dispatch-closure").expect("query present"),
        )
        .expect("closure query runs");

        // Assert: the parent is NOT satisfied — it stays open.
        assert!(
            rows.is_empty(),
            "expected no satisfied parents, got {rows:?}"
        );

        // Act 2: admit the second child; the law flips to satisfied.
        emit_admitted(&store, &templates, 4, "disp-parent-c1");
        let rows = select_rows(
            &store,
            queries.get("dispatch-closure").expect("query present"),
        )
        .expect("closure query runs");

        // Assert 2.
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].get("dispatch").map(String::as_str),
            Some("disp-parent")
        );
    }
);

test!(
    closure_any_child_sufficient_with_one_admitted_child_is_satisfied,
    {
        // Arrange: parent under ANY_CHILD_SUFFICIENT, one of two children
        // admitted.
        let templates = load_templates().expect("templates load");
        let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
        let store = Store::new().expect("store");
        emit_sent(
            &store,
            &templates,
            0,
            "disp-any",
            "none",
            "ANY_CHILD_SUFFICIENT",
        );
        emit_sent(&store, &templates, 1, "disp-any-c0", "disp-any", "NONE");
        emit_sent(&store, &templates, 2, "disp-any-c1", "disp-any", "NONE");
        emit_admitted(&store, &templates, 3, "disp-any-c0");

        // Act.
        let rows = select_rows(
            &store,
            queries.get("dispatch-closure").expect("query present"),
        )
        .expect("closure query runs");

        // Assert: one admitted child satisfies the law.
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].get("dispatch").map(String::as_str),
            Some("disp-any")
        );
    }
);

test!(recursive_dispatch_receipts_are_byte_deterministic, {
    // Arrange: two independent adapters, one contract (depth 1 → two
    // children through the SAME broker path + closure query).
    let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
    let templates = load_templates().expect("templates load");
    let run = |name: &str| {
        let out_dir = scratch_dir(name);
        let store = Store::new().expect("store");
        let mut writer =
            ObsWriter::new(&templates, &store, &out_dir.join("obs"), "test").expect("writer");
        let mut adapter = DispatchAdapter::new(&out_dir, &queries).expect("adapter constructs");
        let contract = workday_contract(
            "det",
            "software-delivery",
            3,
            ExecutionClass::ExternalMachineDispatch,
        );
        let outcome = adapter
            .dispatch(
                &mut writer,
                &store,
                contract,
                3,
                false,
                SynthesisMode::LoopbackDeterministic,
                1,
            )
            .expect("recursive lifecycle admits");
        assert_eq!(outcome, DispatchOutcome::Admitted);
        // Parent + 2 children all admitted.
        assert_eq!(kind_count(&store, "dispatch_sent"), 3);
        assert_eq!(kind_count(&store, "consequence_admitted"), 3);
        adapter.receipt_digests
    };

    // Act.
    let digests_a = run("recursive_a");
    let digests_b = run("recursive_b");

    // Assert: content-derived receipt digests are byte-identical across
    // independent runs (nothing path- or time-derived).
    assert_eq!(digests_a, digests_b);
    assert_eq!(digests_a.len(), 3);
});
