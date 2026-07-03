//! Receipt-tamper coverage gaps named against the ggen ReplayVerifier,
//! star-toml release_verifier, and lsp-max receipt_chain patterns:
//! structural substitution (a whole valid sub-object swapped in from
//! elsewhere) and staleness/rollback replay, as distinct from the existing
//! single-field-tamper and different-history suites.

use praxis_synthesis::handlers::HANDLER_NS;
use praxis_synthesis::hooks::HOOK_NS;
use praxis_synthesis::{
    fire_hooks, replay_firing, FiringOutcome, GraphDelta, HandlerRegistry, MeaningSource, Origin,
    Reference, Refusal,
};

const KERNEL: &str = include_str!("../ontology/lord_prayer.ttl");
const LIFE: &str = "http://seanchatmangpt.github.io/praxis/life#";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

fn src(adds: &str) -> MeaningSource {
    MeaningSource { origin: Origin::Proposer, adds_ttl: adds.to_string(), removes_ttl: String::new() }
}

/// The kernel document itself never binds handlers (a closed-world law
/// covered separately in `kernel_coverage.rs`); tests that exercise the
/// full `fire_hooks` path bind every clause capability to the built-in
/// deterministic handler at `verifiable`, mirroring `deviation_routes.rs`'s
/// `kernel_with_bindings` — every used capability now REQUIRES an explicit
/// `wf:delegability` grade (an unbound capability defaults to
/// `human-only` and refuses), so this augmented genesis is what makes a
/// full-graph `Reference::genesis(KERNEL)` firing completable at all.
fn kernel_with_bindings() -> String {
    let mut base = KERNEL.to_string();
    let all_caps = [
        "orientToFather", "surrenderWill", "requestDailyBread", "writePrayerReceipt",
        "confessDebt", "releaseResentment", "repairDebt", "restoreReceipt",
    ];
    for cap in &all_caps {
        base.push_str(&format!(
            "\n<http://seanchatmangpt.github.io/praxis/prayer#{cap}> \
             <http://seanchatmangpt.github.io/praxis/workflow#handler> <{HANDLER_NS}deterministic-v1> ;\n\
             <http://seanchatmangpt.github.io/praxis/workflow#delegability> \"verifiable\" .\n"
        ));
    }
    base
}

/// Cross-receipt / chain-splice replay (ggen ReplayVerifier pattern): a
/// whole valid `chain` value from an honest OTHER firing (not a bit-flip)
/// is spliced onto a structurally self-consistent receipt for a DIFFERENT
/// firing. The spliced chain is itself a real, honestly-produced value —
/// it is simply not the one this receipt's own stage hashes committed to.
/// `replay_firing` must still catch it via the stage-by-stage rederive,
/// not just single-field bit-flip tamper.
#[test]
fn splicing_a_whole_valid_chain_from_a_different_firing_is_refused() {
    let base = kernel_with_bindings();
    let reference = Reference::genesis(&base).expect("kernel admits");
    let registry = HandlerRegistry::builtin();

    let source_bread = src(&format!("<{LIFE}sean> <{LIFE}hasProvisionAnxiety> 1 ."));
    let source_debt = src(&format!("<{LIFE}debt42> <{RDF_TYPE}> <{LIFE}Debt> ."));

    let bread = fire_hooks(&reference, &source_bread, &registry, &[]).expect("bread fires");
    let debt = fire_hooks(&reference, &source_debt, &registry, &[]).expect("debt fires");
    assert_eq!(bread.outcome, FiringOutcome::Completed);
    assert_eq!(debt.outcome, FiringOutcome::Completed);
    assert_ne!(bread.chain, debt.chain, "two different firings must chain differently");

    // Confused-deputy splice: steal the whole (valid elsewhere) `chain`
    // value from `debt` and attach it to an otherwise-honest `bread`
    // receipt, leaving every one of bread's own stage hashes untouched.
    let mut spliced = bread.clone();
    spliced.chain = debt.chain.clone();

    match replay_firing(&spliced, &base, &source_bread, &registry, &[]) {
        Err(Refusal::VerificationFailed { failed }) => {
            assert_eq!(failed, vec!["chain".to_string()]);
        }
        other => panic!("expected VerificationFailed(chain), got {other:?}"),
    }
}

/// Monotonic/rollback replay (star-toml release_verifier pattern): a
/// receipt that was genuinely valid against the window history AT THE TIME
/// it fired must be refused if replayed against a NEWER history state —
/// i.e. an attacker re-presenting a once-valid receipt after the window
/// has moved forward. Distinct from
/// `replaying_a_firing_against_a_different_history_is_refused` (which
/// swaps to an unrelated/empty history): here the history is extended
/// forward in time, the append-only staleness case.
#[test]
fn replaying_an_old_receipt_against_a_since_advanced_history_is_refused() {
    let var = format!("{LIFE}tick");
    let base = format!(
        "@prefix hook: <{HOOK_NS}> .\n@prefix ex: <http://e/> .\n\
         ex:w a hook:Hook ; hook:name \"burst\" ; hook:kind \"window\" ; \
         hook:var \"{var}\" ; hook:op \">=\" ; hook:k 3 ; hook:window 2 ; \
         hook:effect \"refuse\" ; hook:reason \"burst\" .\n"
    );
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}x> <{var}> 1 ."));

    // At the time of firing: one prior history delta touching `tick` once.
    // window 2 => this delta (1) + the most recent 1 history delta (1) = 2
    // < 3: not fired, firing completes and is receipted.
    let history_at_firing =
        vec![GraphDelta::parse(&format!("<{LIFE}y> <{var}> 1 ."), "").expect("parses")];
    let reference = Reference::genesis(&base).expect("admits");
    let receipt = fire_hooks(&reference, &source, &registry, &history_at_firing)
        .expect("fires and completes");
    assert_eq!(receipt.outcome, FiringOutcome::Completed, "2 < 3: not fired at firing time");

    // Honest replay against the SAME (stale) history still succeeds — the
    // receipt is not simply broken, it is bound to a specific snapshot.
    replay_firing(&receipt, &base, &source, &registry, &history_at_firing)
        .expect("replays against its own contemporaneous history");

    // The world has since moved forward: a new delta touching `tick` twice
    // is now the most recent history entry (prepended). window 2 still
    // takes only the most recent 1 history delta, so the SAME receipt,
    // replayed against this advanced history, now corresponds to a
    // verdict that would have REFUSED (1 + 2 = 3 >= 3) — a rollback/replay
    // of a once-valid receipt must be refused, not silently accepted.
    let history_advanced = vec![
        GraphDelta::parse(&format!("<{LIFE}z1> <{var}> 1 .\n<{LIFE}z2> <{var}> 1 ."), "")
            .expect("parses"),
        history_at_firing[0].clone(),
    ];
    match replay_firing(&receipt, &base, &source, &registry, &history_advanced) {
        Err(Refusal::VerificationFailed { failed }) => {
            assert!(
                failed[0].contains("history") || failed[0].contains("hook"),
                "expected a history/hook-bound rejection, got {failed:?}"
            );
        }
        other => panic!("expected a stale receipt to be refused on replay, got {other:?}"),
    }
}

/// `receipt.inner` substitution without touching `.chain` (lsp-max
/// receipt_chain pattern): swap `receipt.inner[0]` for a different, but
/// individually well-formed, `WorkflowReceipt` lifted whole from a
/// DIFFERENT firing (different capability set, its own internally
/// consistent `chain`) — while leaving the outer `chain`/`hook_hash`/
/// `outcome_hash` fields untouched. Confirms the outer receipt actually
/// binds inner CONTENT (via the final rederive-and-compare in
/// `replay_firing`), not merely inner receipt count.
#[test]
fn substituting_inner_receipt_from_a_different_firing_is_refused() {
    let base = kernel_with_bindings();
    let reference = Reference::genesis(&base).expect("kernel admits");
    let registry = HandlerRegistry::builtin();

    let source_bread = src(&format!("<{LIFE}sean> <{LIFE}hasProvisionAnxiety> 1 ."));
    let source_debt = src(&format!("<{LIFE}debt42> <{RDF_TYPE}> <{LIFE}Debt> ."));

    let bread = fire_hooks(&reference, &source_bread, &registry, &[]).expect("bread fires");
    let debt = fire_hooks(&reference, &source_debt, &registry, &[]).expect("debt fires");
    assert_eq!(bread.inner.len(), 1, "daily-bread grounded once");
    assert_eq!(debt.inner.len(), 1, "confess-and-repair grounded once");
    assert_ne!(
        bread.inner[0].chain, debt.inner[0].chain,
        "the two inner receipts must be genuinely different plans"
    );

    // Swap in debt's structurally valid inner receipt, leaving bread's own
    // outer chain/hook_hash/outcome_hash exactly as honestly computed.
    let mut forged = bread.clone();
    forged.inner[0] = debt.inner[0].clone();
    assert_eq!(forged.chain, bread.chain, "outer chain field itself is untouched by the swap");

    match replay_firing(&forged, &base, &source_bread, &registry, &[]) {
        Err(Refusal::VerificationFailed { failed }) => {
            assert_eq!(failed, vec!["inner chains".to_string()]);
        }
        other => panic!("expected VerificationFailed(inner chains), got {other:?}"),
    }
}

/// Demonstrates that a forged plan steps payload behind an honest plan hash
/// is now caught by `replay_workflow` (the trustless replay verifier).
#[test]
fn mutating_plan_steps_retains_honest_plan_hash_and_passes_replay() {
    #[allow(deprecated)]
    let ttl_demo = include_str!("../ontology/workflow_demo.ttl");
    #[allow(deprecated)]
    let mut receipt = praxis_synthesis::execute_workflow(ttl_demo).expect("demo executes");

    // Clear steps of the plan, which is a major forgery of the plan's payload body.
    assert!(!receipt.plan.steps.is_empty());
    receipt.plan.steps.clear();

    // Verify it with replay_workflow.
    // It is rejected because replay_workflow now hashes receipt.plan.steps and compares it.
    let result = praxis_synthesis::replay_workflow(&receipt, ttl_demo);
    let err = result.expect_err("replay_workflow must reject the mutated plan steps");
    assert!(matches!(err, Refusal::VerificationFailed { .. }));
}

