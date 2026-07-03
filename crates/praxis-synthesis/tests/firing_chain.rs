//! The outer firing chain: quarantine → admission → handler judgment →
//! hooks → grounded execution → one chained receipt; replay + tamper cases.

use praxis_synthesis::handlers::HANDLER_NS;
use praxis_synthesis::{
    fire_hooks, replay_firing, FiringOutcome, HandlerRegistry, MeaningSource, Origin, Reference,
    Refusal,
};

const KERNEL: &str = include_str!("../ontology/lord_prayer.ttl");
const LIFE: &str = "http://seanchatmangpt.github.io/praxis/life#";

fn src(adds: &str) -> MeaningSource {
    MeaningSource {
        origin: Origin::Proposer,
        adds_ttl: adds.to_string(),
        removes_ttl: String::new(),
    }
}

fn kernel_with_binding(delegability: &str, handler_local: &str) -> String {
    let mut base = KERNEL.to_string();
    for cap in &["orientToFather", "surrenderWill", "requestDailyBread", "writePrayerReceipt"] {
        base.push_str(&format!(
            "\n<http://seanchatmangpt.github.io/praxis/prayer#{cap}> \
             <http://seanchatmangpt.github.io/praxis/workflow#handler> <{HANDLER_NS}{handler_local}> ;\n\
             <http://seanchatmangpt.github.io/praxis/workflow#delegability> \"{delegability}\" .\n"
        ));
    }
    base
}

#[test]
fn completed_firing_chains_and_replays() {
    let base = kernel_with_binding("verifiable", "deterministic-v1");
    let reference = Reference::genesis(&base).expect("kernel admits");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}sean> <{LIFE}hasProvisionAnxiety> 1 ."));

    let receipt = fire_hooks(&reference, &source, &registry, &[]).expect("fires");
    assert_eq!(receipt.outcome, FiringOutcome::Completed);
    assert_eq!(receipt.inner.len(), 1, "daily-bread grounded once");
    assert_eq!(receipt.bindings.len(), 4);
    assert_eq!(receipt.verdicts.len(), 11, "all eleven hooks receipted");

    replay_firing(&receipt, &base, &source, &registry, &[]).expect("replays");
}

#[test]
fn unknown_handler_is_refused_before_solving_and_still_chained() {
    let base = kernel_with_binding("verifiable", "deterministic"); // suffix ≠ exact key
    let reference = Reference::genesis(&base).expect("admits");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}sean> <{LIFE}hasProvisionAnxiety> 1 ."));

    let receipt = fire_hooks(&reference, &source, &registry, &[]).expect("refusal is receipted");
    match &receipt.outcome {
        FiringOutcome::Refused { stage, reason } => {
            assert_eq!(stage, "handler");
            assert!(reason.contains("unknown handler"));
        }
        other => panic!("expected Refused(handler), got {other:?}"),
    }
    assert!(receipt.inner.is_empty(), "refused BEFORE any solving");
    assert!(receipt.verdicts.is_empty(), "hooks never evaluated");
    replay_firing(&receipt, &base, &source, &registry, &[]).expect("refusal receipts replay too");
}

#[test]
fn human_only_binding_is_a_chained_delegability_refusal() {
    // orientToFather IS in the fired daily-bread action's derived plan, so
    // the scoped delegability judgment still refuses.
    let base = kernel_with_binding("human-only", "deterministic-v1");
    let reference = Reference::genesis(&base).expect("admits");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}sean> <{LIFE}hasProvisionAnxiety> 1 ."));

    let receipt = fire_hooks(&reference, &source, &registry, &[]).expect("receipted");
    match &receipt.outcome {
        FiringOutcome::Refused { stage, reason } => {
            assert_eq!(stage, "delegability");
            assert!(reason.contains("human-only"));
        }
        other => panic!("expected Refused(delegability), got {other:?}"),
    }
    assert!(receipt.inner.is_empty(), "no executed receipts survive a delegability refusal");
}

#[test]
fn human_only_binding_on_an_unused_capability_does_not_refuse() {
    // restore-receipt lives in the RepairReceiptWorkflow fragment; the
    // daily-bread firing never grounds it, so its human-only grade is not
    // this firing's business.
    let mut base = kernel_with_binding("verifiable", "deterministic-v1");
    base.push_str(&format!(
        "\n<http://seanchatmangpt.github.io/praxis/prayer#restoreReceipt> \
         <http://seanchatmangpt.github.io/praxis/workflow#handler> <{HANDLER_NS}deterministic-v1> ;\n\
         <http://seanchatmangpt.github.io/praxis/workflow#delegability> \"human-only\" .\n"
    ));
    let reference = Reference::genesis(&base).expect("admits");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}sean> <{LIFE}hasProvisionAnxiety> 1 ."));

    let receipt = fire_hooks(&reference, &source, &registry, &[]).expect("fires");
    assert_eq!(receipt.outcome, FiringOutcome::Completed, "unrelated binding must not refuse");
    assert_eq!(receipt.inner.len(), 1, "daily-bread still grounded");
    replay_firing(&receipt, &base, &source, &registry, &[]).expect("replays");
}

#[test]
fn declared_refusal_surrender_is_chained_with_the_graph_reason() {
    let reference = Reference::genesis(KERNEL).expect("admits");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}threat999> <{LIFE}hasUnboundedThreat> 1 ."));
    let receipt = fire_hooks(&reference, &source, &registry, &[]).expect("receipted");
    match &receipt.outcome {
        FiringOutcome::Refused { stage, reason } => {
            assert_eq!(stage, "declared-refusal");
            assert!(reason.contains("surrendered to God"));
        }
        other => panic!("expected declared refusal, got {other:?}"),
    }
    replay_firing(&receipt, KERNEL, &source, &registry, &[]).expect("replays");
}

#[test]
fn forged_payloads_behind_honest_hashes_are_refused_by_name() {
    let base = kernel_with_binding("verifiable", "deterministic-v1");
    let reference = Reference::genesis(&base).expect("admits");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}sean> <{LIFE}hasProvisionAnxiety> 1 ."));
    let honest = fire_hooks(&reference, &source, &registry, &[]).expect("fires");

    let expect_fail = |receipt: &praxis_synthesis::HookFiringReceipt, what: &str| {
        match replay_firing(receipt, &base, &source, &registry, &[]) {
            Err(Refusal::VerificationFailed { failed }) => {
                assert!(failed[0].contains(what), "expected {what}, got {failed:?}");
            }
            other => panic!("expected VerificationFailed({what}), got {other:?}"),
        }
    };

    // Forged verdict body behind the honest hook_hash.
    let mut forged = honest.clone();
    forged.verdicts[0].hook_name = "forged".to_string();
    expect_fail(&forged, "verdict payload");

    // Forged binding body behind the honest handler_hash.
    let mut forged = honest.clone();
    forged.bindings[0].capability = "forged".to_string();
    expect_fail(&forged, "binding payload");

    // Forged admission record behind the honest admission_hash.
    let mut forged = honest.clone();
    forged.admission.epoch += 1;
    expect_fail(&forged, "admission payload");

    // Tampered chain itself.
    let mut forged = honest.clone();
    let flip = if forged.chain.ends_with('0') { "1" } else { "0" };
    forged.chain = format!("{}{flip}", &forged.chain[..forged.chain.len() - 1]);
    expect_fail(&forged, "chain");
}
