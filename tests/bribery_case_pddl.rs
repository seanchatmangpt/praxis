//! Stage-1 live verification that the Solvane Global bribery-compliance
//! PDDL8 domain (`crates/multifractal-workflow/fixtures/bribery-case/
//! pddl-domain.ttl`) is not a hand-authored resemblance of a generated
//! artifact -- it IS one. This test feeds the `pddl:`-vocabulary RDF Turtle
//! through the same real, already-tested pipeline `tests/mfg_golden.rs`
//! pins for `ontology/lawobject.ttl`:
//!   `mfg::manufacture` (SPARQL extraction + `enforce_pddl8` bound
//!   checking + PDDL8 text emission) -> `mfg::validate` (real
//!   `bcinr_pddl::domain_from_pddl` / `problem_from_pddl` /
//!   `GroundProblem::build` / `GroundProblem::find_plan`).
//!
//! Two scenarios, both genuinely ground+solve:
//!   1. `pddl-problem-closable.ttl`: all 3 obligations' evidence is
//!      suppliable -> the case reaches `(in-stage ... receipted)`.
//!   2. `pddl-problem-blocked.ttl`: one obligation's evidence is positively
//!      unavailable -> the case reaches `(in-stage ... blocked)`, a typed
//!      non-closure terminal state distinct from `receipted` -- proving
//!      this domain does not force every path to fabricate a closure.
//!
//! This test ITSELF found a real STRIPS8 design bug in an earlier version
//! of pddl-domain.ttl (a shared `obligation-cleared(?ob)` predicate let the
//! real grounder alias all 3 of `close-obligations`' parameters to one
//! cleared obligation) -- see that file's "BUG FOUND AND FIXED THIS
//! SESSION" comment for the fix and how running this test caught it.
#![cfg(feature = "ggen")]

use my_conforming_project::mfg;

const DOMAIN_TTL: &str =
    include_str!("../crates/multifractal-workflow/fixtures/bribery-case/pddl-domain.ttl");
const PROBLEM_CLOSABLE_TTL: &str =
    include_str!("../crates/multifractal-workflow/fixtures/bribery-case/pddl-problem-closable.ttl");
const PROBLEM_BLOCKED_TTL: &str =
    include_str!("../crates/multifractal-workflow/fixtures/bribery-case/pddl-problem-blocked.ttl");

/// STRIPS8 bound check by inspection (this task's explicit verification
/// requirement, cross-checked against the same PDDL8_MAX_* constants
/// `ontology/lawobject.ttl` documents): every action in this domain stays
/// at <= 4 params / <= 3 precondition conjuncts; every atom used stays at
/// <= 2 args. All well under PDDL8_MAX_ARITY / PDDL8_MAX_CONJUNCTS /
/// PDDL8_MAX_PARAMS = 8. `mfg::enforce_pddl8` (called inside
/// `mfg::manufacture` below) is the LIVE re-check of this claim -- if it
/// were wrong, manufacture() would return `Err(MfgError::BoundExceeded)`
/// and the test would fail at the `.expect(...)` below, not silently pass.
#[test]
fn domain_is_strips8_safe_and_manufactures_real_pddl8_text() {
    // `mfg::manufacture` extracts a domain AND a problem from one graph
    // (`extract_problem` requires exactly one `pddl:Problem` instance), so
    // this bound-checking test concatenates the domain with one problem
    // file (arbitrarily, the closable one) purely to give it a Problem to
    // extract -- the assertions below are about the DOMAIN text only.
    let combined = format!("{DOMAIN_TTL}\n{PROBLEM_CLOSABLE_TTL}");
    let manufactured = mfg::manufacture(&combined, "fixtures/bribery-case/pddl-domain.ttl")
        .expect("pddl-domain.ttl must satisfy PDDL8 bounds (arity/conjuncts/params <= 8)");
    assert!(manufactured
        .project_domain_text()
        .contains("close-obligations"));
    assert!(manufactured
        .domain_text
        .contains("clear-transaction-obligation"));
    assert!(manufactured
        .domain_text
        .contains("clear-authorization-obligation"));
    assert!(manufactured
        .project_domain_text()
        .contains("clear-policy-obligation"));
    assert!(manufactured
        .domain_text
        .contains("block-for-missing-evidence"));
    eprintln!(
        "manufactured domain text:\n{}",
        manufactured.project_domain_text()
    );
}

/// Scenario 1: lawful closure. Concatenates pddl-domain.ttl with
/// pddl-problem-closable.ttl (see DESIGN.md for why the domain/problem
/// split is 3 files, not 1: `mfg::extract_problem` assumes exactly one
/// `pddl:Problem` instance per graph), manufactures real PDDL8 text, and
/// calls the REAL `bcinr_pddl` grounder+solver -- not a mock.
#[test]
fn closable_case_grounds_and_solves_to_receipted() {
    let combined = format!("{DOMAIN_TTL}\n{PROBLEM_CLOSABLE_TTL}");
    let manufactured = mfg::manufacture(&combined, "bribery-case (closable scenario)")
        .expect("domain+closable-problem must manufacture");
    eprintln!(
        "manufactured problem text:\n{}",
        manufactured.project_problem_text()
    );

    let report = mfg::solve_ir(&manufactured);
    assert!(
        report.parsed,
        "must round-trip through bcinr-pddl's parser: {:?}",
        report.error
    );
    assert!(
        report.solvable,
        "GroundProblem::find_plan must find a real plan reaching receipted: {:?}",
        report.error
    );
    eprintln!(
        "REAL bcinr_pddl plan ({} steps): {:?}",
        report.plan_len, report.plan_steps
    );

    // The plan must supply evidence for + clear all 3 named obligations
    // (one distinct clear-*-obligation action per obligation type -- see
    // this file's module doc for the aliasing bug this exact assertion
    // catches if it ever regresses), close them as a group, then
    // judge/admit/receipt -- 10 actions total, ending in exactly `receipt`.
    assert_eq!(report.plan_len, 10, "plan: {:?}", report.plan_steps);
    assert_eq!(
        report.plan_steps.last().map(String::as_str),
        Some("receipt")
    );
    assert_eq!(
        report
            .plan_steps
            .iter()
            .filter(|s| s.as_str() == "supply-evidence")
            .count(),
        3
    );
    for role_action in [
        "clear-transaction-obligation",
        "clear-authorization-obligation",
        "clear-policy-obligation",
    ] {
        assert_eq!(
            report
                .plan_steps
                .iter()
                .filter(|s| s.as_str() == role_action)
                .count(),
            1,
            "expected exactly one {role_action} in the plan: {:?}",
            report.plan_steps
        );
    }
    assert_eq!(
        report
            .plan_steps
            .iter()
            .filter(|s| s.as_str() == "close-obligations")
            .count(),
        1
    );
    assert!(report.plan_steps.contains(&"judge".to_string()));
    assert!(report.plan_steps.contains(&"admit".to_string()));
    // block-for-missing-evidence must NEVER appear in a lawful closure plan.
    assert!(!report
        .plan_steps
        .contains(&"block-for-missing-evidence".to_string()));
}

/// Scenario 2: the typed non-closure path. One obligation's evidence is
/// positively unavailable -- the REAL solver must reach `blocked`, not
/// `receipted`, and must do so WITHOUT ever invoking judge/admit/receipt
/// (proving `blocked` is a genuine sibling terminal state, not a renamed
/// step on the closure path).
#[test]
fn blocked_case_grounds_and_solves_to_blocked_not_receipted() {
    let combined = format!("{DOMAIN_TTL}\n{PROBLEM_BLOCKED_TTL}");
    let manufactured = mfg::manufacture(&combined, "bribery-case (blocked scenario)")
        .expect("domain+blocked-problem must manufacture");
    eprintln!(
        "manufactured problem text:\n{}",
        manufactured.project_problem_text()
    );

    let report = mfg::solve_ir(&manufactured);
    assert!(
        report.parsed,
        "must round-trip through bcinr-pddl's parser: {:?}",
        report.error
    );
    assert!(
        report.solvable,
        "GroundProblem::find_plan must find a real plan reaching blocked: {:?}",
        report.error
    );
    eprintln!(
        "REAL bcinr_pddl plan ({} steps): {:?}",
        report.plan_len, report.plan_steps
    );

    assert_eq!(
        report.plan_steps,
        vec!["block-for-missing-evidence".to_string()],
        "the blocked scenario's shortest real plan is exactly one action"
    );
    for lawful_closure_action in ["judge", "admit", "receipt"] {
        assert!(
            !report
                .plan_steps
                .contains(&lawful_closure_action.to_string()),
            "the blocked path must never pass through {lawful_closure_action}"
        );
    }
}
