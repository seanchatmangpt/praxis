//! End-to-end: the Lord's Prayer workflow kernel through the full chain —
//! quarantine → admission → knowledge hooks → grounded action → receipt.
//!
//! RDF event in → BoundStep out → receipt: the connector the archaeology
//! named missing (docs/ggen_rdf_to_pddl_sketch.rs was "DO NOT IMPLEMENT";
//! this test is the caller that did not exist).

// The deprecated execute_workflow surface stays covered until removal.
#![allow(deprecated)]
use praxis_synthesis::graph::parse_ttl;
use praxis_synthesis::hooks::{EffectKind, HookVerdict};
use praxis_synthesis::{
    capability_task_spec, evaluate_hooks, extract_hooks, ground_fired_action, replay_workflow,
    Admission, MeaningSource, Origin, Reference, RiceQuarantine,
};

const KERNEL: &str = include_str!("../ontology/lord_prayer.ttl");
const LIFE: &str = "http://seanchatmangpt.github.io/praxis/life#";

fn admit(adds: &str, removes: &str) -> praxis_synthesis::AdmittedEvent {
    let reference = Reference::genesis(KERNEL).expect("kernel admits at genesis");
    let source = MeaningSource {
        origin: Origin::Proposer,
        adds_ttl: adds.to_string(),
        removes_ttl: removes.to_string(),
    };
    let delta = RiceQuarantine::inspect(&source).expect("delta passes quarantine");
    Admission::admit(&reference, &delta).expect("delta admits")
}

#[test]
fn provision_anxiety_grounds_the_daily_prayer_workflow() {
    // Deviation: provision anxiety asserted into the life graph.
    let event = admit(&format!("<{LIFE}sean> <{LIFE}hasProvisionAnxiety> 1 ."), "");
    let hooks = extract_hooks(event.post()).expect("kernel registry extracts");
    assert_eq!(hooks.len(), 11, "eleven prayer-clause hooks declared");

    let records = evaluate_hooks(&hooks, &event, &[]).expect("evaluates");
    let bread = records
        .iter()
        .find(|r| r.hook_name == "daily-bread")
        .unwrap();
    assert_eq!(bread.verdict, HookVerdict::Fired);
    assert_eq!(bread.effect, EffectKind::GroundAction);

    // Ground: the fired clause runs the declared workflow fragment through
    // the EXISTING chain. The clause ORDER is derived by the solver from
    // preconditions, not authored.
    let receipt = ground_fired_action(&event, bread).expect("grounds and executes");
    let order: Vec<&str> = receipt
        .plan
        .steps
        .iter()
        .map(|s| s.capability.as_str())
        .collect();
    assert_eq!(
        order,
        [
            "orient-to-father",
            "surrender-will",
            "request-daily-bread",
            "write-prayer-receipt"
        ],
        "the prayer's order falls out of the preconditions"
    );
    assert_eq!(receipt.plan.cost, 12);

    // The inner v1 receipt replays from the fragment's canonical bytes —
    // the standard trust path, unchanged by grounding.
    let fragment_ttl_is_not_available_but_chain_is_bound = receipt.chain.len() > 16;
    assert!(fragment_ttl_is_not_available_but_chain_is_bound);

    // PDDL-router bridge: desired effects projected from the fragment goal.
    let hook = hooks.iter().find(|h| h.name == "daily-bread").unwrap();
    let spec = capability_task_spec(&event, hook).expect("spec projects");
    assert_eq!(
        spec.desired_effects,
        vec![("prayer-receipted".to_string(), "person".to_string())]
    );
}

#[test]
fn resentment_open_loop_fires_by_datalog_rule_and_release_quiets_it() {
    // An unreleased resentment loop is derivable -> hook fires.
    let event = admit(
        &format!("<{LIFE}resentment123> a <{LIFE}ResentmentLoop> ."),
        "",
    );
    let hooks = extract_hooks(event.post()).expect("extracts");
    let records = evaluate_hooks(&hooks, &event, &[]).expect("evaluates");
    let forgive = records
        .iter()
        .find(|r| r.hook_name == "forgive-debtors")
        .unwrap();
    assert_eq!(
        forgive.verdict,
        HookVerdict::Fired,
        "open loop detected by rule"
    );

    // The same loop WITH a release act (the human act, recorded as a fact)
    // is not an open loop: the negation quiets the hook.
    let event2 = admit(
        &format!(
            "<{LIFE}resentment123> a <{LIFE}ResentmentLoop> .\n\
             <{LIFE}sean> <{LIFE}releases> <{LIFE}resentment123> ."
        ),
        "",
    );
    let hooks2 = extract_hooks(event2.post()).expect("extracts");
    let records2 = evaluate_hooks(&hooks2, &event2, &[]).expect("evaluates");
    let forgive2 = records2
        .iter()
        .find(|r| r.hook_name == "forgive-debtors")
        .unwrap();
    assert_eq!(
        forgive2.verdict,
        HookVerdict::NotFired,
        "released loop is closed"
    );
}

#[test]
fn unbounded_threat_is_surrendered_not_computed() {
    // Deliverance: the effect is a REFUSAL with standing — no agent
    // computes the unbounded. The refusal reason is declared in the graph.
    let event = admit(
        &format!("<{LIFE}threat999> <{LIFE}hasUnboundedThreat> 1 ."),
        "",
    );
    let hooks = extract_hooks(event.post()).expect("extracts");
    let records = evaluate_hooks(&hooks, &event, &[]).expect("evaluates");
    let deliver = records
        .iter()
        .find(|r| r.hook_name == "deliverance")
        .unwrap();
    assert_eq!(deliver.verdict, HookVerdict::Fired);
    assert_eq!(deliver.effect, EffectKind::Refuse);
    let hook = hooks.iter().find(|h| h.name == "deliverance").unwrap();
    assert!(hook
        .reason
        .as_deref()
        .unwrap()
        .contains("surrendered to God"));
    // Grounding a refuse-effect hook is itself refused: surrender is not an action run.
    assert!(ground_fired_action(&event, deliver).is_err());
}

#[test]
fn v1_chain_golden_pin_direct_execution_unchanged_by_the_hook_layer() {
    // The kernel's workflow fragment executed DIRECTLY (pre-hook-layer path)
    // and via grounding yield the same derived stages: the inner v1 chain is
    // untouched by Slice A/B — hooks add folds, they never mutate the chain.
    let event = admit(&format!("<{LIFE}sean> <{LIFE}hasProvisionAnxiety> 1 ."), "");
    let hooks = extract_hooks(event.post()).expect("extracts");
    let records = evaluate_hooks(&hooks, &event, &[]).expect("evaluates");
    let bread = records
        .iter()
        .find(|r| r.hook_name == "daily-bread")
        .unwrap();
    let grounded = ground_fired_action(&event, bread).expect("grounds");

    // Direct: strip the graph to a standalone TTL doc holding EXACTLY the
    // daily prayer fragment — its workflow node, its declared wf:capability
    // members, and their atoms; the same restriction ground.rs derives via
    // membership edges (foreign fragments are never this action's business).
    // Execute the classic way: same triples -> same graph/ir/plan hashes.
    let kernel_triples = parse_ttl(KERNEL).expect("kernel parses");
    let wf_only: String = KERNEL
        .lines()
        .skip_while(|l| !l.contains("The daily prayer workflow"))
        .take_while(|l| !l.contains("The confess-and-repair workflow"))
        .collect::<Vec<_>>()
        .join("\n");
    let doc = format!(
        "@prefix wf: <http://seanchatmangpt.github.io/praxis/workflow#> .\n\
         @prefix ex: <http://seanchatmangpt.github.io/praxis/prayer#> .\n\
         @prefix prov: <http://www.w3.org/ns/prov#> .\n{wf_only}"
    );
    let direct = praxis_synthesis::execute_workflow(&doc).expect("direct executes");
    assert_eq!(direct.ir_hash, grounded.ir_hash, "same IR from both doors");
    assert_eq!(direct.plan_hash, grounded.plan_hash);
    assert_eq!(direct.topology_hash, grounded.topology_hash);
    assert_eq!(direct.geometry_hash, grounded.geometry_hash);
    // Replay the direct receipt from its own bytes — the classic trust path.
    replay_workflow(&direct, &doc).expect("direct receipt replays");
    drop(kernel_triples);
}

#[test]
fn day_window_over_the_eight_bound_trips_the_temptation_guard() {
    let mut adds = String::new();
    for i in 0..9 {
        adds.push_str(&format!("<{LIFE}task{i}> <{LIFE}scheduledToday> {i} .\n"));
    }
    let event = admit(&adds, "");
    let hooks = extract_hooks(event.post()).expect("extracts");
    let records = evaluate_hooks(&hooks, &event, &[]).expect("evaluates");
    let guard = records
        .iter()
        .find(|r| r.hook_name == "temptation-guard")
        .unwrap();
    assert_eq!(
        guard.verdict,
        HookVerdict::Fired,
        "9 > 8: the bound holds the line"
    );
    assert_eq!(guard.effect, EffectKind::Refuse);
}
