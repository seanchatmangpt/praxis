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
        .join(format!(
            "../../target/chatman/cng-tests/dispatch_{}",
            std::process::id()
        ))
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

test!(
    deadline_expiry_with_zero_remediation_budget_reaches_blocked,
    {
        // Arrange: the SAME forced-timeout setup as
        // deadline_expiry_times_out_and_manufactures_escalation above
        // (deadline_ticks = 0, so no loopback consequence can arrive
        // before the deadline expires), but with remediation_budget = 0 —
        // the coordinator has nothing left to spend on this timeout. This
        // is one of the 4 ->BLOCKED edges in the 16-state transition table
        // (REMOTE_IN_PROGRESS/REFUSED/TIMED_OUT/COMPENSATING -> BLOCKED)
        // that no prior test drove with real data: TIMED_OUT -> BLOCKED,
        // the terminal "stuck, needs operator intervention" state.
        let out_dir = scratch_dir("timeout_blocked_zero_budget");
        let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
        let templates = load_templates().expect("templates load");
        let store = Store::new().expect("store");
        let mut writer =
            ObsWriter::new(&templates, &store, &out_dir.join("obs"), "test").expect("writer");
        let mut adapter = DispatchAdapter::new(&out_dir, &queries).expect("adapter constructs");
        let mut contract = fixture_contract();
        contract.deadline_ticks = 0;
        let dispatch_id = contract.dispatch_id.clone();

        // Act: remediation_budget = 0 forces the coordinator-timeout branch
        // that skips COMPENSATING/escalation entirely.
        let outcome = adapter
            .dispatch(
                &mut writer,
                &store,
                contract,
                2,
                false,
                SynthesisMode::LoopbackDeterministic,
                0,
            )
            .expect("lifecycle completes (timeout is evidence, not an error)");

        // Assert: TIMED_OUT is still the reported outcome, but with zero
        // remediation budget NO escalation workflow was manufactured — the
        // load-bearing difference from the remediation_budget=1 sibling
        // test above.
        assert_eq!(outcome, DispatchOutcome::TimedOut);
        assert_eq!(kind_count(&store, "dispatch_timed_out"), 1);
        assert_eq!(kind_count(&store, "remediation_manufactured"), 0);
        assert_eq!(kind_count(&store, "consequence_admitted"), 0);
        assert_eq!(adapter.telemetry.timeouts, 1);
        assert_eq!(adapter.telemetry.remediations, 0);

        // Assert: the durable ledger proves the state machine actually
        // crossed TIMED_OUT -> BLOCKED, not TIMED_OUT -> COMPENSATING ->
        // COMPLETED — BLOCKED is a genuinely distinct terminal, reached by
        // a real code path (dispatch.rs's `remediation_budget > 0` guard),
        // not merely declared lawful in the transition table.
        let entries = adapter.ledger.entries(&dispatch_id).expect("entries read");
        let trajectory: Vec<(&str, &str)> = entries
            .iter()
            .map(|e| (e.from_state.as_str(), e.to_state.as_str()))
            .collect();
        assert_eq!(
            trajectory,
            vec![
                ("MANUFACTURED", "ARAZZO_RENDERED"),
                ("ARAZZO_RENDERED", "DISPATCH_READY"),
                ("DISPATCH_READY", "DISPATCHED"),
                ("DISPATCHED", "ACKNOWLEDGED"),
                ("ACKNOWLEDGED", "REMOTE_STARTED"),
                ("REMOTE_STARTED", "REMOTE_IN_PROGRESS"),
                ("REMOTE_IN_PROGRESS", "TIMED_OUT"),
                ("TIMED_OUT", "BLOCKED"),
            ]
        );
        assert_eq!(
            trajectory.last(),
            Some(&("TIMED_OUT", "BLOCKED")),
            "must reach the terminal BLOCKED state, not COMPENSATING/COMPLETED"
        );
    }
);

test!(
    semantic_refusal_with_zero_remediation_budget_reaches_blocked,
    {
        // Arrange: the SAME wrong-artifact fixture as
        // semantic_conformance_failure_refuses_and_manufactures_compensation
        // above, but with remediation_budget = 0 (mirrors how
        // deadline_expiry_with_zero_remediation_budget_reaches_blocked
        // mirrors ITS budget=1 sibling above). This drives the
        // REFUSED -> BLOCKED edge — 2 of the 4 ->BLOCKED edges in the
        // 16-state table (REMOTE_IN_PROGRESS/REFUSED/TIMED_OUT/COMPENSATING
        // -> BLOCKED) now real-data-proven.
        let out_dir = scratch_dir("semantic_refusal_blocked");
        let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
        let templates = load_templates().expect("templates load");
        let store = Store::new().expect("store");
        let mut writer =
            ObsWriter::new(&templates, &store, &out_dir.join("obs"), "test").expect("writer");
        let mut adapter = DispatchAdapter::new(&out_dir, &queries).expect("adapter constructs");
        let contract = fixture_contract();
        let dispatch_id = contract.dispatch_id.clone();
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

        // Act: zero remediation budget forces the refused-terminal branch
        // that skips COMPENSATING/compensation entirely.
        let outcome = adapter
            .dispatch(
                &mut writer,
                &store,
                contract,
                1,
                false,
                SynthesisMode::FixtureFile(&fixture_path),
                0,
            )
            .expect("lifecycle completes (refusal is evidence, not an error)");

        // Assert: still refused at the semantic stage, but with zero
        // remediation budget NO compensation workflow was manufactured —
        // the load-bearing difference from the remediation_budget=1 sibling
        // test above.
        assert_eq!(
            outcome,
            DispatchOutcome::Refused {
                stage: "semantic".to_string()
            }
        );
        assert_eq!(kind_count(&store, "consequence_refused"), 1);
        assert_eq!(kind_count(&store, "remediation_manufactured"), 0);
        assert_eq!(kind_count(&store, "consequence_admitted"), 0);
        assert_eq!(adapter.telemetry.refused, 1);
        assert_eq!(adapter.telemetry.remediations, 0);

        // Assert: the durable ledger proves REFUSED -> BLOCKED, a genuinely
        // distinct terminal from REFUSED -> COMPENSATING -> COMPLETED.
        let entries = adapter.ledger.entries(&dispatch_id).expect("entries read");
        let trajectory: Vec<(&str, &str)> = entries
            .iter()
            .map(|e| (e.from_state.as_str(), e.to_state.as_str()))
            .collect();
        assert_eq!(
            trajectory.last(),
            Some(&("REFUSED", "BLOCKED")),
            "must reach the terminal BLOCKED state, not COMPENSATING/COMPLETED"
        );
    }
);

test!(
    unimplemented_closure_law_leaves_parent_remote_in_progress_blocked,
    {
        // Arrange: a recursive (depth 1, CHILD_FAN_OUT=2 children) parent
        // whose declared closure law is one of the FOUR laws
        // `queries/dispatch-closure.rq` documents as "declared in
        // shapes/dispatch-shapes.ttl but not yet emitted by the workday
        // broker" (QUORUM_REQUIRED, ORDERED_SUBSET_REQUIRED, POLICY_DECIDES,
        // FIRST_CONFORMANT_RESULT) — the query returns NO satisfied row for
        // ANY parent declaring one of these four, regardless of how many
        // children admit. This is the REAL, documented (not fabricated)
        // trigger for REMOTE_IN_PROGRESS -> BLOCKED: unlike the other 3
        // ->BLOCKED edges (all failure paths: timeout/refusal), this one is
        // the "closure law the broker cannot yet evaluate" path — the
        // children succeed lawfully, only the PARENT's closure gate blocks.
        let out_dir = scratch_dir("closure_unimplemented_blocked");
        let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
        let templates = load_templates().expect("templates load");
        let store = Store::new().expect("store");
        let mut writer =
            ObsWriter::new(&templates, &store, &out_dir.join("obs"), "test").expect("writer");
        let mut adapter = DispatchAdapter::new(&out_dir, &queries).expect("adapter constructs");
        // workday_contract's ExternalMachineDispatch branch declares
        // recursive_depth=1, closure_law=ALL_CHILDREN_REQUIRED; override
        // ONLY the closure law to one dispatch-closure.rq cannot evaluate.
        let mut contract = workday_contract(
            "quorum",
            "software-delivery",
            5,
            ExecutionClass::ExternalMachineDispatch,
        );
        contract.closure_law = Some("QUORUM_REQUIRED");
        let dispatch_id = contract.dispatch_id.clone();

        // Act: children dispatch and admit deterministically (loopback), but
        // the PARENT's closure check can never be satisfied for this law.
        let outcome = adapter
            .dispatch(
                &mut writer,
                &store,
                contract,
                5,
                false,
                SynthesisMode::LoopbackDeterministic,
                0,
            )
            .expect("lifecycle completes (unsatisfied closure is evidence, not an error)");

        // Assert: the parent stays OPEN (BLOCKED) even though both children
        // admitted lawfully — the closure law, not the children, is why.
        assert_eq!(outcome, DispatchOutcome::Open);
        assert_eq!(kind_count(&store, "dispatch_sent"), 3); // parent + 2 children
        assert_eq!(kind_count(&store, "consequence_admitted"), 2); // children only

        // Assert: the durable ledger proves the PARENT crossed
        // REMOTE_IN_PROGRESS -> BLOCKED directly — it never reaches
        // RESULT_AVAILABLE/RESULT_RECEIVED/RESULT_ADMITTED/COMPLETED,
        // because the closure gate short-circuits before the parent's own
        // consequence is ever polled for.
        let entries = adapter.ledger.entries(&dispatch_id).expect("entries read");
        let trajectory: Vec<(&str, &str)> = entries
            .iter()
            .map(|e| (e.from_state.as_str(), e.to_state.as_str()))
            .collect();
        assert_eq!(
            trajectory,
            vec![
                ("MANUFACTURED", "ARAZZO_RENDERED"),
                ("ARAZZO_RENDERED", "DISPATCH_READY"),
                ("DISPATCH_READY", "DISPATCHED"),
                ("DISPATCHED", "ACKNOWLEDGED"),
                ("ACKNOWLEDGED", "REMOTE_STARTED"),
                ("REMOTE_STARTED", "REMOTE_IN_PROGRESS"),
                ("REMOTE_IN_PROGRESS", "BLOCKED"),
            ]
        );
    }
);

// --- COMPENSATING -> BLOCKED: investigated, found vestigial (declared
// lawful in DispatchState::lawful_to, never driven by any production code
// path), the same finding class as DISPATCH_READY -> REFUSED (see the type
// doc on DispatchState above). Both production call sites that advance a
// contract INTO Compensating (deadline-expiry-with-remediation-budget and
// refused-with-remediation-budget, both above) unconditionally advance it
// to Completed next once `remediate()` returns Ok; remediate()'s only other
// exit is Err (a HardcodingSuspicion that propagates out of dispatch()
// entirely via `?`, never reaching a Blocked ledger entry for THIS
// contract). No code path in dispatch.rs or engine.rs ever calls
// `advance_ledgered(.., DispatchState::Blocked, ..)` on a contract whose
// current state is Compensating. Confirmed by exhaustive grep: every
// `DispatchState::Compensating` reference in `crates/cng/src/` is either
// the enum declaration, the transition-table entry, or one of these two
// call sites — no third site exists. Forcing this edge would require adding
// a NEW code path whose only purpose is to make the assertion pass, which
// is exactly the fabricated-trigger this investigation was asked not to do;
// documenting it here instead.

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

test!(sixteen_state_transition_law_is_exact, {
    // Arrange: the full lawful transition set from the PROJ-720 table.
    use std::collections::BTreeSet;
    let expected: BTreeSet<(&str, &str)> = [
        ("MANUFACTURED", "ARAZZO_RENDERED"),
        ("ARAZZO_RENDERED", "DISPATCH_READY"),
        ("DISPATCH_READY", "DISPATCHED"),
        ("DISPATCH_READY", "REFUSED"),
        ("DISPATCHED", "ACKNOWLEDGED"),
        ("DISPATCHED", "TIMED_OUT"),
        ("ACKNOWLEDGED", "REMOTE_STARTED"),
        ("ACKNOWLEDGED", "TIMED_OUT"),
        ("REMOTE_STARTED", "REMOTE_IN_PROGRESS"),
        ("REMOTE_IN_PROGRESS", "RESULT_AVAILABLE"),
        ("REMOTE_IN_PROGRESS", "TIMED_OUT"),
        ("REMOTE_IN_PROGRESS", "BLOCKED"),
        ("RESULT_AVAILABLE", "RESULT_RECEIVED"),
        ("RESULT_RECEIVED", "RESULT_ADMITTED"),
        ("RESULT_RECEIVED", "REFUSED"),
        ("RESULT_ADMITTED", "COMPLETED"),
        ("REFUSED", "COMPENSATING"),
        ("REFUSED", "BLOCKED"),
        ("TIMED_OUT", "COMPENSATING"),
        ("TIMED_OUT", "BLOCKED"),
        ("COMPENSATING", "COMPLETED"),
        ("COMPENSATING", "BLOCKED"),
    ]
    .into_iter()
    .collect();

    // Act: enumerate ALL 16×16 pairs against lawful_to.
    let mut lawful: BTreeSet<(&str, &str)> = BTreeSet::new();
    for from in DispatchState::ALL {
        for to in DispatchState::ALL {
            if from.lawful_to(to) {
                lawful.insert((from.as_str(), to.as_str()));
            }
        }
    }

    // Assert: the lawful set is EXACTLY the documented table — no extra
    // lawful transition, no missing one; terminals have no exits.
    assert_eq!(lawful, expected);
    assert_eq!(DispatchState::ALL.len(), 16);
    for terminal in ["COMPLETED", "BLOCKED", "UNKNOWN"] {
        assert!(
            !lawful.iter().any(|(from, _)| *from == terminal),
            "{terminal} must be terminal"
        );
    }
});

test!(shapes_ttl_state_individuals_match_the_enum, {
    // Arrange: parse the on-disk shapes law (the declarative authority).
    use std::collections::BTreeSet;
    let shapes_ttl =
        fs::read_to_string(crate_path("shapes/dispatch-shapes.ttl")).expect("shapes file reads");
    let store = Store::new().expect("store");
    store
        .load_from_slice(
            RdfParser::from_format(RdfFormat::Turtle),
            shapes_ttl.as_bytes(),
        )
        .expect("shapes parse");
    let type_pred =
        NamedNodeRef::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").expect("rdf:type IRI");
    let state_class = NamedNodeRef::new("https://truex.io/ontology/dispatch#DispatchState")
        .expect("DispatchState IRI");

    // Act: collect the disp:DispatchState individuals' local names.
    let mut declared: BTreeSet<String> = BTreeSet::new();
    for quad in store.quads_for_pattern(
        None,
        Some(type_pred),
        Some(TermRef::from(state_class)),
        None,
    ) {
        let quad = quad.expect("shape scan");
        let iri = quad.subject.to_string();
        let local = iri
            .trim_start_matches("<https://truex.io/ontology/dispatch#")
            .trim_end_matches('>')
            .to_string();
        declared.insert(local);
    }
    let in_rust: BTreeSet<String> = DispatchState::ALL
        .iter()
        .map(|s| s.as_str().to_string())
        .collect();

    // Assert: drift test — the TTL individual set IS the as_str set.
    assert_eq!(declared, in_rust);
});

test!(ledger_records_every_advance_and_replays_chain_verified, {
    // Arrange: one full admitted lifecycle through the adapter.
    let out_dir = scratch_dir("ledger_lifecycle");
    let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
    let templates = load_templates().expect("templates load");
    let store = Store::new().expect("store");
    let mut writer =
        ObsWriter::new(&templates, &store, &out_dir.join("obs"), "test").expect("writer");
    let mut adapter = DispatchAdapter::new(&out_dir, &queries).expect("adapter constructs");
    let contract = fixture_contract();
    let dispatch_id = contract.dispatch_id.clone();

    // Act.
    let outcome = adapter
        .dispatch(
            &mut writer,
            &store,
            contract,
            1,
            false,
            SynthesisMode::LoopbackDeterministic,
            0,
        )
        .expect("lifecycle admits");
    assert_eq!(outcome, DispatchOutcome::Admitted);

    // Assert: the durable ledger holds the exact state trajectory (one
    // StateEntry per advance, seq-ordered).
    let entries = adapter.ledger.entries(&dispatch_id).expect("entries read");
    let trajectory: Vec<(&str, &str)> = entries
        .iter()
        .map(|e| (e.from_state.as_str(), e.to_state.as_str()))
        .collect();
    assert_eq!(
        trajectory,
        vec![
            ("MANUFACTURED", "ARAZZO_RENDERED"),
            ("ARAZZO_RENDERED", "DISPATCH_READY"),
            ("DISPATCH_READY", "DISPATCHED"),
            ("DISPATCHED", "ACKNOWLEDGED"),
            ("ACKNOWLEDGED", "REMOTE_STARTED"),
            ("REMOTE_STARTED", "REMOTE_IN_PROGRESS"),
            ("REMOTE_IN_PROGRESS", "RESULT_AVAILABLE"),
            ("RESULT_AVAILABLE", "RESULT_RECEIVED"),
            ("RESULT_RECEIVED", "RESULT_ADMITTED"),
            ("RESULT_ADMITTED", "COMPLETED"),
        ]
    );

    // Assert 2: a fresh sink over the same directory replays and
    // chain-verifies every entry (resume machinery, PROJ-724).
    let reloaded = super::FileLedgerSink::new(&out_dir.join("dispatch").join("ledger"))
        .expect("ledger reloads chain-verified");
    assert_eq!(reloaded.total_entries(), entries.len() as u64);
});

test!(replayed_consequence_refuses_cng_r25_double_admit, {
    // Arrange: one admitted lifecycle, then the SAME contract (same
    // idempotency key) presented again through the same adapter.
    let out_dir = scratch_dir("double_admit");
    let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
    let templates = load_templates().expect("templates load");
    let store = Store::new().expect("store");
    let mut writer =
        ObsWriter::new(&templates, &store, &out_dir.join("obs"), "test").expect("writer");
    let mut adapter = DispatchAdapter::new(&out_dir, &queries).expect("adapter constructs");
    let first = adapter
        .dispatch(
            &mut writer,
            &store,
            fixture_contract(),
            1,
            false,
            SynthesisMode::LoopbackDeterministic,
            0,
        )
        .expect("first lifecycle admits");
    assert_eq!(first, DispatchOutcome::Admitted);

    // Act: the replay reaches the admission gate and must refuse there.
    let result = adapter.dispatch(
        &mut writer,
        &store,
        fixture_contract(),
        2,
        false,
        SynthesisMode::LoopbackDeterministic,
        0,
    );

    // Assert: typed CNG_R25 naming the dispatch and the processed key.
    match result {
        Err(CngRefusal::DoubleAdmit {
            dispatch,
            idempotency_key,
        }) => {
            assert_eq!(dispatch, "disp-fixture");
            assert_eq!(idempotency_key, fixture_contract().idempotency_key);
            assert_eq!(
                CngRefusal::DoubleAdmit {
                    dispatch,
                    idempotency_key
                }
                .code(),
                "CNG_R25"
            );
        }
        other => panic!("expected DoubleAdmit, got {other:?}"),
    }
});

test!(
    arazzo_projection_gate_admits_when_render_digest_matches_receipt,
    {
        // Arrange: a scratch project_root doubling as both the workday
        // out_dir (DispatchAdapter::new) AND the ggen sync project root
        // (PROJ-745) — a rendered arazzo.yaml and a receipt recording its
        // true digest, exactly the shape a real `ggen sync run` of
        // arazzo-pack leaves behind.
        let out_dir = scratch_dir("arazzo_gate_match");
        fs::create_dir_all(out_dir.join("generated")).expect("generated dir");
        fs::create_dir_all(out_dir.join(".ggen-v2")).expect(".ggen-v2 dir");
        let rendered_yaml: &[u8] = b"arazzo: \"1.1.0\"\ninfo:\n  title: gate_match\n";
        fs::write(out_dir.join("generated/arazzo.yaml"), rendered_yaml).expect("write yaml");
        let digest = blake3::hash(rendered_yaml).to_hex().to_string();
        fs::write(
            out_dir.join(".ggen-v2/receipt.json"),
            format!("{{\"payload\":{{\"outputs\":{{\"generated/arazzo.yaml\":\"{digest}\"}}}}}}"),
        )
        .expect("write receipt");
        let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
        let templates = load_templates().expect("templates load");
        let store = Store::new().expect("store");
        let mut writer =
            ObsWriter::new(&templates, &store, &out_dir.join("obs"), "test").expect("writer");
        let mut adapter = DispatchAdapter::new(&out_dir, &queries).expect("adapter constructs");

        // Act: run_arazzo_projection's wired gate (verify_arazzo_render_
        // digest(adapter.project_root())) must pass before any step's
        // dispatch reaches DispatchState::ArazzoRendered/DispatchReady.
        let dispatched = crate::bench::arazzo::run_arazzo_projection(
            &mut adapter,
            &mut writer,
            &store,
            &crate::bench::arazzo::default_description_path(),
            "gate-match",
            "api-orchestration",
            0,
        )
        .expect("digest matches receipt: projection dispatches end-to-end");

        // Assert: every step of the shipped example reached DISPATCH_READY
        // and admitted through the loopback — the gate did not block a
        // correctly-receipted render.
        assert_eq!(dispatched, 4);
        assert_eq!(adapter.telemetry.sent, 4);
        assert_eq!(adapter.telemetry.admitted, 4);
        assert_eq!(adapter.telemetry.refused, 0);
    }
);

test!(
    arazzo_projection_gate_refuses_cng_r11_before_any_step_dispatches,
    {
        // Arrange: a scratch project_root with NO rendered arazzo.yaml and
        // NO ggen receipt — the honest state of a project where a `ggen
        // sync run` was never performed (or was performed against a
        // different pack). run_arazzo_projection must refuse before any
        // step's dispatch transitions the state machine.
        let out_dir = scratch_dir("arazzo_gate_missing_render");
        let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
        let templates = load_templates().expect("templates load");
        let store = Store::new().expect("store");
        let mut writer =
            ObsWriter::new(&templates, &store, &out_dir.join("obs"), "test").expect("writer");
        let mut adapter = DispatchAdapter::new(&out_dir, &queries).expect("adapter constructs");

        // Act.
        let result = crate::bench::arazzo::run_arazzo_projection(
            &mut adapter,
            &mut writer,
            &store,
            &crate::bench::arazzo::default_description_path(),
            "gate-missing",
            "api-orchestration",
            0,
        );

        // Assert: typed CNG_R11 AuditMismatch, and — the load-bearing
        // assertion — zero steps were sent. The gate runs before the
        // per-step loop, so a missing/mismatched render blocks the whole
        // projection, not just the step that would have used it.
        match result {
            Err(CngRefusal::AuditMismatch(msg)) => {
                assert!(
                    msg.contains("not auditable"),
                    "message names the missing render, got {msg}"
                );
                assert_eq!(CngRefusal::AuditMismatch(msg).code(), "CNG_R11");
            }
            Err(other) => panic!("expected AuditMismatch, got {other:?}"),
            Ok(_) => panic!("expected AuditMismatch for a missing render, but projection ran"),
        }
        assert_eq!(
            adapter.telemetry.sent, 0,
            "the render-digest gate must run before any step dispatches"
        );
        let outbox = out_dir.join("dispatch").join("outbox");
        let outbox_entries = fs::read_dir(&outbox)
            .expect("outbox dir exists (created by DispatchAdapter::new)")
            .count();
        assert_eq!(
            outbox_entries, 0,
            "no contract should have been written to the outbox before the gate"
        );
    }
);
