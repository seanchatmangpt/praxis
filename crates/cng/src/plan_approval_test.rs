#![cfg(test)]

use super::*;
use chicago_tdd_tools::prelude::*;

/// Path to the shared 2-op seeded fixture template also rendered by
/// `tests/cng_pipeline.rs` (act0(obj) -> act1(obj), a solvable linear
/// STRIPS chain) — Turtle never lives in a Rust string; see that
/// integration test's own header for the placeholder convention.
fn template_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pddl-pair.template.ttl")
}

/// Renders the shared template with seed-derived names into a scratch dir
/// under `target/chatman/cng-tests/plan_approval/<test_name>/` and returns
/// the artifact directory (one admitted PDDL Turtle artifact carrying a
/// 2-action linear STRIPS domain/problem).
fn render_fixture(test_name: &str, seed: u64) -> PathBuf {
    let hex = blake3::hash(&seed.to_le_bytes()).to_hex().to_string();
    let s = &hex[..8];
    let template = fs::read_to_string(template_path()).expect("read fixture template");
    let rendered = template
        .replace("{{DOMAIN_NAME}}", &format!("dom-{s}"))
        .replace("{{OBJ}}", &format!("obj-{s}"))
        .replace("{{ACTION_0}}", &format!("act0-{s}"))
        .replace("{{ACTION_1}}", &format!("act1-{s}"))
        .replace("{{PRED_0}}", &format!("pred0-{s}"))
        .replace("{{PRED_1}}", &format!("pred1-{s}"))
        .replace("{{PRED_2}}", &format!("pred2-{s}"));
    assert!(
        !rendered.contains("{{"),
        "unsubstituted placeholder left in rendered fixture"
    );
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests/plan_approval")
        .join(test_name);
    fs::create_dir_all(&dir).expect("create fixture dir");
    fs::write(dir.join("pddl-pair.ttl"), rendered).expect("write rendered fixture");
    dir
}

/// A fresh (removed-if-present) ledger dir under the same scratch root, so
/// each test gets an isolated ledger.
fn ledger_scratch_dir(test_name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests/plan_approval")
        .join(test_name)
        .join("ledger");
    let _ = fs::remove_dir_all(&dir);
    dir
}

test!(present_plan_halts_without_side_effect_and_is_idempotent, {
    // Arrange
    let dir = render_fixture("present_halts", 1);
    let ledger_dir = ledger_scratch_dir("present_halts");

    // Act: present twice against the same admitted artifact set.
    let first = present_plan(&dir, &ledger_dir).expect("present_plan must admit a solvable plan");
    let ledger_file = ledger_dir.join("plan-ledger.jsonl");
    let first_len = fs::metadata(&ledger_file)
        .expect("ledger file must exist after present")
        .len();
    let second =
        present_plan(&dir, &ledger_dir).expect("re-presenting the same plan must be idempotent");
    let second_len = fs::metadata(&ledger_file)
        .expect("ledger file must still exist")
        .len();

    // Assert: same digest, same steps, and re-presenting appended nothing
    // further (present halts -- it never executes).
    assert_eq!(first.plan_digest, second.plan_digest);
    assert_eq!(first.steps, second.steps);
    assert_eq!(first.steps.len(), 2, "fixture is a 2-action linear plan");
    assert_eq!(
        first_len, second_len,
        "re-presenting an already-presented digest must not append a second Presented event"
    );
});

test!(check_admits_the_exact_next_step, {
    // Arrange
    let dir = render_fixture("check_admits", 2);
    let ledger_dir = ledger_scratch_dir("check_admits");
    let presented = present_plan(&dir, &ledger_dir).expect("present_plan");

    // Act
    let outcome = check_action(&ledger_dir, &presented.plan_digest, &presented.steps[0]);

    // Assert
    match outcome {
        Ok(o) => assert_eq!(o.step_index, 0),
        Err(e) => panic!("expected admission for the exact next step, got {e}"),
    }
});

test!(check_refuses_an_unlisted_action, {
    // Arrange
    let dir = render_fixture("check_refuses_unlisted", 3);
    let ledger_dir = ledger_scratch_dir("check_refuses_unlisted");
    let presented = present_plan(&dir, &ledger_dir).expect("present_plan");

    // Act
    let outcome = check_action(
        &ledger_dir,
        &presented.plan_digest,
        "totally-unrelated-action(z)",
    );

    // Assert
    match outcome {
        Err(refusal @ CngRefusal::ActionNotNextApprovedStep { .. }) => {
            assert_eq!(refusal.code(), "CNG_R31");
        }
        other => panic!("expected CNG_R31 ActionNotNextApprovedStep, got {other:?}"),
    }
});

test!(check_refuses_an_unpresented_plan_digest, {
    // Arrange
    let ledger_dir = ledger_scratch_dir("check_refuses_unpresented");
    fs::create_dir_all(&ledger_dir).expect("create empty ledger dir");

    // Act
    let outcome = check_action(&ledger_dir, "blake3:never-presented", "any-action(x)");

    // Assert
    match outcome {
        Err(refusal @ CngRefusal::PlanNotPresented { .. }) => {
            assert_eq!(refusal.code(), "CNG_R30");
        }
        other => panic!("expected CNG_R30 PlanNotPresented, got {other:?}"),
    }
});

test!(
    step_refuses_without_approved_flag_and_never_mutates_the_ledger,
    {
        // Arrange
        let dir = render_fixture("step_refuses_unapproved", 4);
        let ledger_dir = ledger_scratch_dir("step_refuses_unapproved");
        let presented = present_plan(&dir, &ledger_dir).expect("present_plan");
        let ledger_file = ledger_dir.join("plan-ledger.jsonl");
        let before = fs::read_to_string(&ledger_file).expect("read ledger before step attempt");

        // Act
        let outcome = execute_approved_step(&ledger_dir, &presented.plan_digest, false);

        // Assert
        match outcome {
            Err(refusal @ CngRefusal::StepNotApproved { .. }) => {
                assert_eq!(refusal.code(), "CNG_R32");
            }
            other => panic!("expected CNG_R32 StepNotApproved, got {other:?}"),
        }
        let after = fs::read_to_string(&ledger_file).expect("read ledger after step attempt");
        assert_eq!(
            before, after,
            "an unapproved step must never mutate the ledger"
        );
    }
);

test!(
    step_approved_executes_exactly_one_step_and_advances_the_ledger,
    {
        // Arrange
        let dir = render_fixture("step_executes_one", 5);
        let ledger_dir = ledger_scratch_dir("step_executes_one");
        let presented = present_plan(&dir, &ledger_dir).expect("present_plan");
        assert_eq!(presented.steps.len(), 2);

        // Act: first step admitted then executed.
        let admitted_first = check_action(&ledger_dir, &presented.plan_digest, &presented.steps[0])
            .expect("first step must be admitted before executing it");
        let receipt_one = execute_approved_step(&ledger_dir, &presented.plan_digest, true)
            .expect("approved step must execute");

        // Assert: exactly step 0 executed, matching what was checked.
        assert_eq!(admitted_first.step_index, 0);
        assert_eq!(receipt_one.step_index, 0);
        assert_eq!(receipt_one.step_label, presented.steps[0]);
        assert!(!receipt_one.chain_hash.is_empty());

        // Act: the already-executed step is no longer admissible; the second
        // step is now the sole lawful next.
        let repeat_refused = check_action(&ledger_dir, &presented.plan_digest, &presented.steps[0]);
        let second_admitted =
            check_action(&ledger_dir, &presented.plan_digest, &presented.steps[1])
                .expect("second step must now be the lawful next step");
        let receipt_two = execute_approved_step(&ledger_dir, &presented.plan_digest, true)
            .expect("second approved step must execute");

        // Assert
        match repeat_refused {
            Err(refusal @ CngRefusal::ActionNotNextApprovedStep { .. }) => {
                assert_eq!(refusal.code(), "CNG_R31");
            }
            other => panic!("re-proposing the already-executed step must refuse, got {other:?}"),
        }
        assert_eq!(second_admitted.step_index, 1);
        assert_eq!(receipt_two.step_index, 1);
        assert_ne!(receipt_two.chain_hash, receipt_one.chain_hash);

        // Act: the plan is now exhausted.
        let exhausted = execute_approved_step(&ledger_dir, &presented.plan_digest, true);
        match exhausted {
            Err(
                refusal @ CngRefusal::ActionNotNextApprovedStep {
                    expected_next: None,
                    ..
                },
            ) => {
                assert_eq!(refusal.code(), "CNG_R31");
            }
            other => panic!(
                "stepping an exhausted plan must refuse with expected_next=None, got {other:?}"
            ),
        }
    }
);

test!(reload_detects_a_tampered_ledger_line, {
    // Arrange
    let dir = render_fixture("reload_detects_tamper", 6);
    let ledger_dir = ledger_scratch_dir("reload_detects_tamper");
    let presented = present_plan(&dir, &ledger_dir).expect("present_plan");
    execute_approved_step(&ledger_dir, &presented.plan_digest, true).expect("first step executes");
    let ledger_file = ledger_dir.join("plan-ledger.jsonl");
    let original = fs::read_to_string(&ledger_file).expect("read ledger");
    // Corrupt the recorded chain_hash on the StepExecuted line.
    let tampered = original.replace("\"chain_hash\":\"", "\"chain_hash\":\"tampered-");
    assert_ne!(
        original, tampered,
        "the replace must actually have found the target field"
    );
    fs::write(&ledger_file, tampered).expect("write tampered ledger");

    // Act
    let outcome = check_action(&ledger_dir, &presented.plan_digest, &presented.steps[1]);

    // Assert
    match outcome {
        Err(refusal @ CngRefusal::AuditMismatch(_)) => {
            assert_eq!(refusal.code(), "CNG_R11");
        }
        other => panic!("expected CNG_R11 AuditMismatch on a tampered chain hash, got {other:?}"),
    }
});

// PROJ (live role-inference wiring): a real, non-bench-fixture plan
// artifact directory — the shared PDDL fixture PLUS a plain roster.ttl
// (never bench's `ObsWriter`/`roster_admitted` observation shape) — run
// through the live, non-bench `present_plan` path, proving real
// Datalog-derived role facts for it.
#[cfg(feature = "role-inference")]
test!(present_plan_derives_real_datalog_roles_for_a_non_bench_roster_artifact, {
    // Arrange: the shared 2-action PDDL fixture plus an independent,
    // hand-authored roster.ttl using the live path's own plain vocabulary
    // (ceng:rosterDeclaredRole / ceng:rosterDepartment) — not generated by
    // any bench fixture generator or ObsWriter template.
    let dir = render_fixture("live_roster_roles", 7);
    let ledger_dir = ledger_scratch_dir("live_roster_roles");
    let roster_ttl_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/live-roster.ttl");
    let roster_ttl = fs::read_to_string(&roster_ttl_path).expect("read live-roster.ttl fixture");
    fs::write(dir.join("roster.ttl"), roster_ttl).expect("write live roster.ttl");

    // Act: the live plan-admit path, unchanged call site.
    let presented = present_plan(&dir, &ledger_dir).expect("present_plan over PDDL + roster");

    // Assert: the plan itself is unaffected (still the 2-action PDDL plan)…
    assert_eq!(presented.steps.len(), 2);
    // …and real Datalog materialization derived roles/obligations for every
    // roster worker, over the SAME praxis-graphlaw engine bench uses,
    // sourced from a plain non-bench Turtle vocabulary.
    assert_eq!(
        presented.roster_roles.get("worker-avery").map(String::as_str),
        Some("reviewer")
    );
    assert_eq!(
        presented.roster_roles.get("worker-bao").map(String::as_str),
        Some("approver")
    );
    assert_eq!(
        presented.roster_roles.get("worker-caro").map(String::as_str),
        Some("auditor")
    );
    assert_eq!(
        presented
            .roster_obligations
            .get("worker-avery")
            .map(String::as_str),
        Some("review-then-escalate-to-approver")
    );
    assert_eq!(
        presented
            .roster_obligations
            .get("worker-bao")
            .map(String::as_str),
        Some("authorize-transition")
    );
    assert_eq!(
        presented
            .roster_obligations
            .get("worker-caro")
            .map(String::as_str),
        Some("verify-evidence-chain")
    );
});

// The disclosed generalization boundary: an arbitrary PDDL-only artifact
// (no roster triples at all) genuinely yields no role facts — reported
// honestly as an empty map, never fabricated.
#[cfg(feature = "role-inference")]
test!(present_plan_yields_no_roster_roles_for_a_plain_pddl_only_artifact, {
    let dir = render_fixture("live_no_roster", 8);
    let ledger_dir = ledger_scratch_dir("live_no_roster");

    let presented = present_plan(&dir, &ledger_dir).expect("present_plan over PDDL-only dir");

    assert!(
        presented.roster_roles.is_empty(),
        "a PDDL-only artifact with no roster triples must yield zero role facts, not fabricated ones"
    );
    assert!(presented.roster_obligations.is_empty());
});
