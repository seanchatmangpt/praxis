//! Adversarial repair-loop regressions (2026-07 review):
//!
//! 1. The surrender invariant is a runtime law: a delta that re-routes a
//!    `god-receives-unbounded` clause from refuse to ground-action is a
//!    kernel-boundary refusal, never a completed firing.
//! 2. Fragment scoping: grounding action A yields the identical inner
//!    receipt whether or not unrelated fragments share the admitted graph.
//! 3. Unstratifiable datalog hooks are refused at REGISTRATION.
//! 4. Empty-body (fact-injecting) and `t`-head datalog rules are refused
//!    at registration — a hook cannot fabricate EDB facts.
//! 5. Window verdicts are chained to the history that produced them:
//!    replaying a firing receipt against a different history is refused.

use praxis_synthesis::graph::parse_ttl;
use praxis_synthesis::hooks::HOOK_NS;
use praxis_synthesis::kernel::enforce_surrender_boundary;
use praxis_synthesis::{
    extract_hooks, fire_hooks, replay_firing, FiringOutcome, HandlerRegistry, MeaningSource,
    Origin, Reference, Refusal,
};

const KERNEL: &str = include_str!("../ontology/lord_prayer.ttl");
const LIFE: &str = "http://seanchatmangpt.github.io/praxis/life#";
const PRAYER: &str = "http://seanchatmangpt.github.io/praxis/prayer#";

fn src(adds: &str, removes: &str) -> MeaningSource {
    MeaningSource {
        origin: Origin::Proposer,
        adds_ttl: adds.to_string(),
        removes_ttl: removes.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Finding 1 — deliverance/surrender invariant enforced at firing time
// ---------------------------------------------------------------------------

#[test]
fn rerouting_deliverance_to_ground_action_is_a_kernel_boundary_refusal() {
    // The adversary's reproduction: one admitted delta flips the
    // DeliveranceHook effect from refuse to ground-action and asserts an
    // unbounded threat. Pre-repair this COMPLETED with a grounded plan —
    // the unbounded reached computed action.
    let reference = Reference::genesis(KERNEL).expect("kernel admits");
    let registry = HandlerRegistry::builtin();
    let source = src(
        &format!(
            "<{PRAYER}DeliveranceHook> <{HOOK_NS}effect> \"ground-action\" .\n\
             <{PRAYER}DeliveranceHook> <{HOOK_NS}action> <{PRAYER}DailyPrayerWorkflow> .\n\
             <{LIFE}threat1> <{LIFE}hasUnboundedThreat> 1 ."
        ),
        &format!("<{PRAYER}DeliveranceHook> <{HOOK_NS}effect> \"refuse\" ."),
    );
    let receipt = fire_hooks(&reference, &source, &registry, &[]).expect("refusal is receipted");
    match &receipt.outcome {
        FiringOutcome::Refused { stage, reason } => {
            assert_eq!(stage, "kernel-boundary");
            assert!(reason.contains("god-receives-unbounded"), "reason: {reason}");
        }
        other => panic!("expected kernel-boundary refusal, got {other:?}"),
    }
    assert!(receipt.inner.is_empty(), "the unbounded must never reach a computed plan");
    // The refusal itself chains and replays.
    replay_firing(&receipt, KERNEL, &source, &registry, &[]).expect("boundary refusal replays");
}

#[test]
fn a_ground_action_hook_watching_a_surrendered_var_is_refused() {
    // Keep DeliveranceHook intact, but swap the sponsor hook for a
    // threshold hook that grounds an action off `hasUnboundedThreat`
    // standing in the BASE state. The deliverance (delta-kind) hook is
    // quiet on an innocuous delta, so pre-repair the evil hook grounded a
    // plan while an unbounded threat stood.
    let needle = "    hook:kind \"delta\" ;\n    \
        hook:var \"http://seanchatmangpt.github.io/praxis/life#withdrawsCapability\" ;\n    \
        hook:effect \"refuse\" ;\n    \
        hook:reason \"sponsor capability withdrawn; reassignment parks for the human\" ;";
    let replacement = "    hook:kind \"threshold\" ;\n    \
        hook:var \"http://seanchatmangpt.github.io/praxis/life#hasUnboundedThreat\" ;\n    \
        hook:op \">\" ;\n    hook:k 0 ;\n    \
        hook:effect \"ground-action\" ;\n    \
        hook:action ex:DailyPrayerWorkflow ;";
    let base = KERNEL.replace(needle, replacement);
    assert_ne!(base, KERNEL, "the sponsor hook block must have been rewritten");
    let base = format!("{base}\n<{LIFE}threat9> <{LIFE}hasUnboundedThreat> 1 .\n");

    let reference = Reference::genesis(&base).expect("admits");
    let registry = HandlerRegistry::builtin();
    let source = src("<http://e/x> <http://e/ping> 1 .", "");
    let receipt = fire_hooks(&reference, &source, &registry, &[]).expect("receipted");
    match &receipt.outcome {
        FiringOutcome::Refused { stage, reason } => {
            assert_eq!(stage, "kernel-boundary");
            assert!(reason.contains("hasUnboundedThreat"), "reason: {reason}");
        }
        other => panic!("expected kernel-boundary refusal, got {other:?}"),
    }
    assert!(receipt.inner.is_empty());
}

#[test]
fn boundary_enforcement_is_conditional_on_a_declared_kernel() {
    // A graph with no prayer-kernel triples has no surrender law to
    // enforce: enforce_surrender_boundary is a no-op, not a refusal.
    let triples = parse_ttl("<http://e/a> <http://e/p> 1 .").expect("parses");
    let hooks = extract_hooks(&triples).expect("empty registry");
    enforce_surrender_boundary(&triples, &hooks).expect("no kernel, no law, no refusal");
}

#[test]
fn baseline_kernel_still_fires_clean_under_the_boundary_law() {
    // The untampered kernel satisfies the law: the classic daily-bread
    // firing still completes.
    let reference = Reference::genesis(KERNEL).expect("admits");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}sean> <{LIFE}hasProvisionAnxiety> 1 ."), "");
    let receipt = fire_hooks(&reference, &source, &registry, &[]).expect("fires");
    assert_eq!(receipt.outcome, FiringOutcome::Completed);
    assert_eq!(receipt.inner.len(), 1);
}

// ---------------------------------------------------------------------------
// Finding 3 — fragment scoping: foreign fragments cannot change A's plan
// ---------------------------------------------------------------------------

const WF: &str = "http://seanchatmangpt.github.io/praxis/workflow#";

fn fragment_a() -> String {
    format!(
        "@prefix wf: <{WF}> .\n@prefix hook: <{HOOK_NS}> .\n@prefix ex: <http://e/> .\n\
         ex:go a hook:Hook ; hook:name \"go\" ; hook:kind \"delta\" ; \
         hook:var \"http://e/ping\" ; hook:effect \"ground-action\" ; hook:action ex:A .\n\
         ex:A a wf:Workflow ; wf:budget 2 ; wf:init ex:s0 ; wf:goal ex:g ; \
         wf:capability ex:capA .\n\
         ex:s0 a wf:Atom ; wf:predicate \"s0\" .\n\
         ex:g a wf:Atom ; wf:predicate \"g\" .\n\
         ex:capA a wf:Capability ; wf:name \"capA\" ; wf:params 0 ; wf:cost 5 ; \
         wf:pre ex:s0 ; wf:add ex:g .\n"
    )
}

/// A foreign fragment that (pre-repair) hijacked A's grounding: a cheaper
/// capability that also adds A's goal, plus a global budget constraint.
fn foreign_fragment_b() -> &'static str {
    "ex:B a wf:Workflow ; wf:budget 1 ; wf:init ex:bs ; wf:goal ex:bg ; \
     wf:capability ex:capB ; wf:constraint ex:conB .\n\
     ex:bs a wf:Atom ; wf:predicate \"bs\" .\n\
     ex:bg a wf:Atom ; wf:predicate \"bg\" .\n\
     ex:capB a wf:Capability ; wf:name \"capB\" ; wf:params 0 ; wf:cost 1 ; \
     wf:pre ex:s0 ; wf:add ex:bg, ex:g .\n\
     ex:conB a wf:Constraint ; wf:kind \"budget\" ; wf:k 3 .\n"
}

#[test]
fn grounding_a_is_byte_identical_with_and_without_foreign_fragments() {
    let registry = HandlerRegistry::builtin();
    let source = src("<http://e/x> <http://e/ping> 1 .", "");

    let alone = fire_hooks(
        &Reference::genesis(&fragment_a()).expect("A admits"),
        &source,
        &registry,
        &[],
    )
    .expect("fires alone");
    let crowded = fire_hooks(
        &Reference::genesis(&format!("{}{}", fragment_a(), foreign_fragment_b()))
            .expect("A+B admits"),
        &source,
        &registry,
        &[],
    )
    .expect("fires with a foreign fragment present");

    assert_eq!(alone.outcome, FiringOutcome::Completed);
    assert_eq!(crowded.outcome, FiringOutcome::Completed);
    assert_eq!(alone.inner.len(), 1);
    assert_eq!(crowded.inner.len(), 1);
    // The DESIGNED law: A's inner receipt is byte-identical either way —
    // foreign capabilities, constraints, and budgets are not A's business.
    assert_eq!(
        serde_json::to_string(&alone.inner[0]).unwrap(),
        serde_json::to_string(&crowded.inner[0]).unwrap(),
        "foreign fragments leaked into A's grounded plan"
    );
    let steps: Vec<&str> =
        alone.inner[0].plan.steps.iter().map(|s| s.capability.as_str()).collect();
    assert_eq!(steps, ["capA"], "A executes only capabilities it declared membership for");
}

// ---------------------------------------------------------------------------
// Findings 4 & 5 — datalog registration hygiene
// ---------------------------------------------------------------------------

fn hook_doc(program: &str, goal: &str) -> String {
    format!(
        "@prefix hook: <{HOOK_NS}> .\n@prefix ex: <http://e/> .\n\
         ex:h a hook:Hook ; hook:name \"h\" ; hook:kind \"datalog\" ; \
         hook:program \"{program}\" ; hook:goal \"{goal}\" ; \
         hook:effect \"refuse\" ; hook:reason \"r\" .\n"
    )
}

#[test]
fn unstratifiable_program_is_refused_at_registration_not_firing() {
    let doc = hook_doc(
        "a(?0) :- t(?0, ?1, ?2), !b(?0). b(?0) :- t(?0, ?1, ?2), !a(?0).",
        "a",
    );
    let triples = parse_ttl(&doc).expect("parses");
    match extract_hooks(&triples) {
        Err(Refusal::HookIllFormed { detail, .. }) => {
            assert!(detail.contains("negation cycle") || detail.contains("stratif"),
                "detail: {detail}");
        }
        other => panic!("expected registration-time HookIllFormed, got {other:?}"),
    }
}

#[test]
fn empty_body_ground_rule_cannot_inject_edb_facts() {
    // A bodiless ground rule is a fact assertion smuggled through program
    // text — refused at registration.
    let doc = hook_doc("orphan(<http://e/x>).", "orphan");
    let triples = parse_ttl(&doc).expect("parses");
    match extract_hooks(&triples) {
        Err(Refusal::HookIllFormed { detail, .. }) => {
            assert!(detail.contains("positive body"), "detail: {detail}");
        }
        other => panic!("expected HookIllFormed(positive body), got {other:?}"),
    }
}

#[test]
fn t_head_rule_cannot_forge_the_edb_projection() {
    let doc = hook_doc(
        "t(?0, ?1, ?2) :- t(?0, ?1, ?2). goal(?0) :- t(?0, <http://e/p>, <http://e/o>).",
        "goal",
    );
    let triples = parse_ttl(&doc).expect("parses");
    match extract_hooks(&triples) {
        Err(Refusal::HookIllFormed { detail, .. }) => {
            assert!(detail.contains("reserved"), "detail: {detail}");
        }
        other => panic!("expected HookIllFormed(reserved t), got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Finding 6 — window verdicts are chained to their history
// ---------------------------------------------------------------------------

#[test]
fn replaying_a_firing_against_a_different_history_is_refused() {
    let base = format!(
        "@prefix hook: <{HOOK_NS}> .\n@prefix ex: <http://e/> .\n\
         ex:w a hook:Hook ; hook:name \"burst\" ; hook:kind \"window\" ; \
         hook:var \"http://e/tick\" ; hook:op \">=\" ; hook:k 3 ; hook:window 2 ; \
         hook:effect \"refuse\" ; hook:reason \"burst\" .\n"
    );
    let registry = HandlerRegistry::builtin();
    let source = src("<http://e/x> <http://e/tick> 1 .", "");
    let history = vec![praxis_synthesis::GraphDelta::parse(
        "<http://e/y> <http://e/tick> 1 .",
        "",
    )
    .expect("history delta parses")];

    let reference = Reference::genesis(&base).expect("admits");
    let receipt = fire_hooks(&reference, &source, &registry, &history).expect("fires");
    assert_eq!(receipt.outcome, FiringOutcome::Completed, "2 < 3: not fired");

    // Honest replay: same history.
    replay_firing(&receipt, &base, &source, &registry, &history).expect("replays");

    // Dishonest replay: an empty history yields the SAME verdicts (1 < 3),
    // but the receipt is chained to the history that produced it.
    match replay_firing(&receipt, &base, &source, &registry, &[]) {
        Err(Refusal::VerificationFailed { failed }) => {
            assert!(failed[0].contains("history"), "failed: {failed:?}");
        }
        other => panic!("expected history-bound verification failure, got {other:?}"),
    }
}
