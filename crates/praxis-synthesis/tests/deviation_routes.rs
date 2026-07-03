//! The remaining deviation routes of the prayer kernel: debt repair,
//! missing receipts, day-window overload, sponsor withdrawal — plus the
//! HumanOnly SCOPING law: delegability is judged per fired action against
//! the capabilities its derived plan uses, never against the whole graph.

use praxis_synthesis::handlers::HANDLER_NS;
use praxis_synthesis::hooks::HookVerdict;
use praxis_synthesis::{
    fire_hooks, replay_firing, FiringOutcome, HandlerRegistry, MeaningSource, Origin, Reference,
};

const KERNEL: &str = include_str!("../ontology/lord_prayer.ttl");
const LIFE: &str = "http://seanchatmangpt.github.io/praxis/life#";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

fn src(adds: &str) -> MeaningSource {
    MeaningSource {
        origin: Origin::Proposer,
        adds_ttl: adds.to_string(),
        removes_ttl: String::new(),
    }
}

fn kernel_with_cap_binding(cap_local: &str, delegability: &str) -> String {
    format!(
        "{KERNEL}\n\
         <http://seanchatmangpt.github.io/praxis/prayer#{cap_local}> \
         <http://seanchatmangpt.github.io/praxis/workflow#handler> <{HANDLER_NS}deterministic-v1> ;\n\
         <http://seanchatmangpt.github.io/praxis/workflow#delegability> \"{delegability}\" .\n"
    )
}

fn fire(base: &str, adds: &str) -> praxis_synthesis::HookFiringReceipt {
    let reference = Reference::genesis(base).expect("base admits");
    let registry = HandlerRegistry::builtin();
    let source = src(adds);
    let receipt = fire_hooks(&reference, &source, &registry, &[]).expect("receipted");
    replay_firing(&receipt, base, &source, &registry, &[]).expect("replays");
    receipt
}

#[test]
fn open_debt_fires_by_rule_and_grounds_confess_and_repair() {
    let receipt = fire(KERNEL, &format!("<{LIFE}debt42> <{RDF_TYPE}> <{LIFE}Debt> ."));
    assert_eq!(receipt.outcome, FiringOutcome::Completed);
    let debt = receipt.verdicts.iter().find(|r| r.hook_name == "debt-repair").unwrap();
    assert_eq!(debt.verdict, HookVerdict::Fired, "unrepaired debt is derivable");
    assert_eq!(receipt.inner.len(), 1, "confess-and-repair grounded once");
    let order: Vec<&str> =
        receipt.inner[0].plan.steps.iter().map(|s| s.capability.as_str()).collect();
    assert_eq!(
        order,
        ["confess-debt", "release-resentment", "repair-debt"],
        "the repair order falls out of the preconditions"
    );
}

#[test]
fn repaired_debt_quiets_the_debt_rule() {
    let receipt = fire(
        KERNEL,
        &format!(
            "<{LIFE}debt42> <{RDF_TYPE}> <{LIFE}Debt> .\n\
             <{LIFE}sean> <{LIFE}repairs> <{LIFE}debt42> ."
        ),
    );
    let debt = receipt.verdicts.iter().find(|r| r.hook_name == "debt-repair").unwrap();
    assert_eq!(debt.verdict, HookVerdict::NotFired, "a repaired debt is closed");
}

#[test]
fn missing_receipt_grounds_the_one_step_repair_fragment() {
    let receipt = fire(KERNEL, &format!("<{LIFE}monday> <{LIFE}hasMissingReceipt> 1 ."));
    assert_eq!(receipt.outcome, FiringOutcome::Completed);
    let gap = receipt.verdicts.iter().find(|r| r.hook_name == "receipt-missing").unwrap();
    assert_eq!(gap.verdict, HookVerdict::Fired);
    assert_eq!(receipt.inner.len(), 1);
    let order: Vec<&str> =
        receipt.inner[0].plan.steps.iter().map(|s| s.capability.as_str()).collect();
    assert_eq!(order, ["restore-receipt"], "one-step repair, nothing more");
}

#[test]
fn five_same_day_placements_in_one_delta_refuse_with_reschedule() {
    let mut adds = String::new();
    for i in 0..5 {
        adds.push_str(&format!("<{LIFE}task{i}> <{LIFE}scheduledToday> {i} .\n"));
    }
    let receipt = fire(KERNEL, &adds);
    match &receipt.outcome {
        FiringOutcome::Refused { stage, reason } => {
            assert_eq!(stage, "declared-refusal");
            assert!(reason.contains("refuse-or-reschedule"), "reason: {reason}");
        }
        other => panic!("expected declared refusal, got {other:?}"),
    }
    assert!(receipt.inner.is_empty());
}

#[test]
fn four_same_day_placements_do_not_trip_the_overload() {
    let mut adds = String::new();
    for i in 0..4 {
        adds.push_str(&format!("<{LIFE}task{i}> <{LIFE}scheduledToday> {i} .\n"));
    }
    let receipt = fire(KERNEL, &adds);
    assert_eq!(receipt.outcome, FiringOutcome::Completed, "4 is inside the bound");
}

#[test]
fn sponsor_withdrawal_refuses_and_parks_for_the_human() {
    let receipt = fire(
        KERNEL,
        &format!("<{LIFE}sponsor7> <{LIFE}withdrawsCapability> <{LIFE}ride-to-meeting> ."),
    );
    match &receipt.outcome {
        FiringOutcome::Refused { stage, reason } => {
            assert_eq!(stage, "declared-refusal");
            assert!(reason.contains("parks for the human"), "reason: {reason}");
        }
        other => panic!("expected declared refusal, got {other:?}"),
    }
}

#[test]
fn human_only_release_resentment_blocks_the_debt_firing() {
    // The agent cannot forgive for the user: release-resentment is in the
    // fired confess-and-repair plan, so its human-only grade refuses.
    let base = kernel_with_cap_binding("releaseResentment", "human-only");
    let reference = Reference::genesis(&base).expect("admits");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}debt42> <{RDF_TYPE}> <{LIFE}Debt> ."));
    let receipt = fire_hooks(&reference, &source, &registry, &[]).expect("receipted");
    match &receipt.outcome {
        FiringOutcome::Refused { stage, reason } => {
            assert_eq!(stage, "delegability");
            assert!(reason.contains("release-resentment"), "reason: {reason}");
            assert!(reason.contains("human-only"), "reason: {reason}");
        }
        other => panic!("expected Refused(delegability), got {other:?}"),
    }
    assert!(receipt.inner.is_empty(), "no executed receipts behind a delegability refusal");
    replay_firing(&receipt, &base, &source, &registry, &[]).expect("refusals replay");
}

#[test]
fn automatable_write_prayer_receipt_is_allowed() {
    let base = kernel_with_cap_binding("writePrayerReceipt", "automatable");
    let receipt = fire(&base, &format!("<{LIFE}sean> <{LIFE}hasProvisionAnxiety> 1 ."));
    assert_eq!(receipt.outcome, FiringOutcome::Completed, "automatable grade executes");
    assert_eq!(receipt.inner.len(), 1);
}

#[test]
fn human_only_release_resentment_does_not_block_an_unrelated_firing() {
    // Scoping law: the daily-bread plan never uses release-resentment, so
    // the same human-only binding must not refuse THIS firing.
    let base = kernel_with_cap_binding("releaseResentment", "human-only");
    let receipt = fire(&base, &format!("<{LIFE}sean> <{LIFE}hasProvisionAnxiety> 1 ."));
    assert_eq!(receipt.outcome, FiringOutcome::Completed);
    assert_eq!(receipt.inner.len(), 1, "daily-bread grounded despite the unrelated binding");
}
