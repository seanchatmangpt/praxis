//! Adversarial attacks test demonstrating security vulnerabilities in the
//! delegation boundaries, agent registry, and capability execution.

use praxis_synthesis::agent_registry::AGENT_NS;
use praxis_synthesis::handlers::HANDLER_NS;
use praxis_synthesis::hooks::HOOK_NS;
use praxis_synthesis::graph::WF_NS;
use praxis_synthesis::{
    fire_hooks, FiringOutcome, HandlerRegistry, MeaningSource, Origin, Reference, Refusal,
};

const LIFE: &str = "http://seanchatmangpt.github.io/praxis/life#";

fn src(adds: &str) -> MeaningSource {
    MeaningSource {
        origin: Origin::Proposer,
        adds_ttl: adds.to_string(),
        removes_ttl: String::new(),
    }
}

#[test]
fn test_surrender_boundary_bypass() {
    // 1. Surrender boundary bypass attack:
    // We declare a god-receives-unbounded clause (pk:deliverance) but OMIT its pk:action.
    // We expect it to be refused with BoundaryViolation because pk:action is missing.
    
    let ttl = format!(
        "@prefix wf:   <{WF_NS}> .\n\
         @prefix hook: <{HOOK_NS}> .\n\
         @prefix prx:  <{LIFE}> .\n\
         @prefix ex:   <http://seanchatmangpt.github.io/praxis/prayer#> .\n\
         @prefix pk:   <http://seanchatmangpt.github.io/praxis/prayer-kernel#> .\n\
         \n\
         pk:LordPrayerKernel a pk:Kernel ;\n\
             pk:clause pk:our-father, pk:hallowed-name, pk:kingdom-come,\n\
                 pk:will-be-done, pk:on-earth-as-heaven, pk:daily-bread,\n\
                 pk:forgive-debts, pk:forgive-debtors, pk:temptation-guard,\n\
                 pk:deliverance, pk:doxology .\n\
         \n\
         pk:our-father a pk:Clause ; pk:name \"our-father\" ; pk:problemClass \"c1\" ; pk:boundary \"human-only\" .\n\
         pk:hallowed-name a pk:Clause ; pk:name \"hallowed-name\" ; pk:problemClass \"c2\" ; pk:boundary \"human-only\" .\n\
         pk:kingdom-come a pk:Clause ; pk:name \"kingdom-come\" ; pk:problemClass \"c3\" ; pk:boundary \"god-receives-unbounded\" ; pk:action ex:RefuseHook .\n\
         pk:will-be-done a pk:Clause ; pk:name \"will-be-done\" ; pk:problemClass \"c4\" ; pk:boundary \"human-only\" .\n\
         pk:on-earth-as-heaven a pk:Clause ; pk:name \"on-earth-as-heaven\" ; pk:problemClass \"c5\" ; pk:boundary \"god-receives-unbounded\" ; pk:action ex:RefuseHook .\n\
         pk:daily-bread a pk:Clause ; pk:name \"daily-bread\" ; pk:problemClass \"c6\" ; pk:boundary \"automatable-support\" .\n\
         pk:forgive-debts a pk:Clause ; pk:name \"forgive-debts\" ; pk:problemClass \"c7\" ; pk:boundary \"god-receives-unbounded\" ; pk:action ex:RefuseHook .\n\
         pk:forgive-debtors a pk:Clause ; pk:name \"forgive-debtors\" ; pk:problemClass \"c8\" ; pk:boundary \"human-only\" .\n\
         pk:temptation-guard a pk:Clause ; pk:name \"temptation-guard\" ; pk:problemClass \"c9\" ; pk:boundary \"automatable-support\" .\n\
         pk:doxology a pk:Clause ; pk:name \"doxology\" ; pk:problemClass \"c11\" ; pk:boundary \"human-only\" .\n\
         \n\
         # The attack/vulnerability trigger: god-receives-unbounded boundary, but pk:action is OMITTED!\n\
         pk:deliverance a pk:Clause ;\n\
             pk:name \"deliverance\" ;\n\
             pk:problemClass \"unbounded-threat\" ;\n\
             pk:boundary \"god-receives-unbounded\" .\n\
         \n\
         # Refusing hook for other god-receives-unbounded clauses\n\
         ex:RefuseHook a hook:Hook ;\n\
             hook:name \"refuse-hook\" ;\n\
             hook:kind \"delta\" ;\n\
             hook:var \"{LIFE}someVar\" ;\n\
             hook:effect \"refuse\" ;\n\
             hook:reason \"refused\" .\n\
         \n\
         # And we register a ground-action hook instead of refuse hook!\n\
         ex:DeliveranceHook a hook:Hook ;\n\
             hook:name \"deliverance\" ;\n\
             hook:on \"assert\" ;\n\
             hook:kind \"delta\" ;\n\
             hook:var \"{LIFE}hasUnboundedThreat\" ;\n\
             hook:effect \"ground-action\" ;\n\
             hook:action ex:DummyWorkflow ;\n\
             hook:priority 0 .\n\
         \n\
         ex:DummyWorkflow a wf:Workflow ;\n\
             wf:budget 1 ;\n\
             wf:init ex:initAtom ;\n\
             wf:goal ex:goalAtom ;\n\
             wf:capability ex:dummyCap .\n\
         \n\
         ex:initAtom a wf:Atom ; wf:predicate \"init\" ; wf:arg0 \"x\" .\n\
         ex:goalAtom a wf:Atom ; wf:predicate \"goal\" ; wf:arg0 \"x\" .\n\
         \n\
         ex:dummyCap a wf:Capability ;\n\
             wf:name \"dummy-cap\" ;\n\
             wf:params 0 ;\n\
             wf:cost 1 ;\n\
             wf:pre ex:initAtom ;\n\
             wf:add ex:goalAtom .\n\
     "
    );

    let reference = Reference::genesis(&ttl).expect("kernel admits");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}threat> <{LIFE}hasUnboundedThreat> 1 ."));

    let result = fire_hooks(&reference, &source, &registry, &[]);
    
    assert!(result.is_ok(), "Expected fire_hooks to produce a receipted refusal");
    let receipt = result.unwrap();
    match &receipt.outcome {
        FiringOutcome::Refused { stage, reason } => {
            assert_eq!(stage, "kernel-boundary");
            assert!(reason.contains("is missing pk:action"), "reason: {reason}");
        }
        other => panic!("expected Refused(kernel-boundary), got {other:?}"),
    }
}

#[test]
fn test_human_only_delegability_bypass() {
    // 2. Human-only delegability bypass:
    // We declare a workflow with a capability (ex:dummyCap) representing a human-only action.
    // If we simply OMIT the wf:handler and wf:delegability declarations, the capability
    // bypasses judge_delegability completely and is executed by DeterministicRunner.
    
    let ttl = format!(
        "@prefix wf:   <{WF_NS}> .\n\
         @prefix hook: <{HOOK_NS}> .\n\
         @prefix prx:  <{LIFE}> .\n\
         @prefix ex:   <http://seanchatmangpt.github.io/praxis/prayer#> .\n\
         @prefix pk:   <http://seanchatmangpt.github.io/praxis/prayer-kernel#> .\n\
         \n\
         pk:LordPrayerKernel a pk:Kernel ;\n\
             pk:clause pk:our-father, pk:hallowed-name, pk:kingdom-come,\n\
                 pk:will-be-done, pk:on-earth-as-heaven, pk:daily-bread,\n\
                 pk:forgive-debts, pk:forgive-debtors, pk:temptation-guard,\n\
                 pk:deliverance, pk:doxology .\n\
         \n\
         pk:our-father a pk:Clause ; pk:name \"our-father\" ; pk:problemClass \"c1\" ; pk:boundary \"human-only\" .\n\
         pk:hallowed-name a pk:Clause ; pk:name \"hallowed-name\" ; pk:problemClass \"c2\" ; pk:boundary \"human-only\" .\n\
         pk:kingdom-come a pk:Clause ; pk:name \"kingdom-come\" ; pk:problemClass \"c3\" ; pk:boundary \"god-receives-unbounded\" ; pk:action ex:DeliveranceHook .\n\
         pk:will-be-done a pk:Clause ; pk:name \"will-be-done\" ; pk:problemClass \"c4\" ; pk:boundary \"human-only\" .\n\
         pk:on-earth-as-heaven a pk:Clause ; pk:name \"on-earth-as-heaven\" ; pk:problemClass \"c5\" ; pk:boundary \"god-receives-unbounded\" ; pk:action ex:DeliveranceHook .\n\
         pk:daily-bread a pk:Clause ; pk:name \"daily-bread\" ; pk:problemClass \"c6\" ; pk:boundary \"automatable-support\" .\n\
         pk:forgive-debts a pk:Clause ; pk:name \"forgive-debts\" ; pk:problemClass \"c7\" ; pk:boundary \"god-receives-unbounded\" ; pk:action ex:DeliveranceHook .\n\
         pk:forgive-debtors a pk:Clause ; pk:name \"forgive-debtors\" ; pk:problemClass \"c8\" ; pk:boundary \"human-only\" .\n\
         pk:temptation-guard a pk:Clause ; pk:name \"temptation-guard\" ; pk:problemClass \"c9\" ; pk:boundary \"automatable-support\" .\n\
         pk:doxology a pk:Clause ; pk:name \"doxology\" ; pk:problemClass \"c11\" ; pk:boundary \"human-only\" .\n\
         \n\
         pk:deliverance a pk:Clause ;\n\
             pk:name \"deliverance\" ;\n\
             pk:problemClass \"unbounded-threat\" ;\n\
             pk:boundary \"god-receives-unbounded\" ;\n\
             pk:action ex:DeliveranceHook .\n\
         \n\
         ex:DeliveranceHook a hook:Hook ;\n\
             hook:name \"deliverance\" ;\n\
             hook:kind \"delta\" ;\n\
             hook:var \"{LIFE}hasUnboundedThreat\" ;\n\
             hook:effect \"refuse\" ;\n\
             hook:reason \"refused\" .\n\
         \n\
         ex:TriggerHook a hook:Hook ;\n\
             hook:name \"trigger\" ;\n\
             hook:on \"assert\" ;\n\
             hook:kind \"delta\" ;\n\
             hook:var \"{LIFE}triggerState\" ;\n\
             hook:effect \"ground-action\" ;\n\
             hook:action ex:HumanOnlyWorkflow ;\n\
             hook:priority 0 .\n\
         \n\
         ex:HumanOnlyWorkflow a wf:Workflow ;\n\
             wf:budget 1 ;\n\
             wf:init ex:initAtom ;\n\
             wf:goal ex:goalAtom ;\n\
             wf:capability ex:humanCap .\n\
         \n\
         ex:initAtom a wf:Atom ; wf:predicate \"init\" ; wf:arg0 \"x\" .\n\
         ex:goalAtom a wf:Atom ; wf:predicate \"goal\" ; wf:arg0 \"x\" .\n\
         \n\
         # The capability is human-only, but we omit wf:handler and wf:delegability!\n\
         ex:humanCap a wf:Capability ;\n\
             wf:name \"human-cap\" ;\n\
             wf:params 0 ;\n\
             wf:cost 1 ;\n\
             wf:pre ex:initAtom ;\n\
             wf:add ex:goalAtom .\n\
    "
    );

    let reference = Reference::genesis(&ttl).expect("kernel admits");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}x> <{LIFE}triggerState> 1 ."));

    let result = fire_hooks(&reference, &source, &registry, &[]);
    assert!(result.is_ok(), "Expected fire_hooks to succeed with a receipted refusal");
    let receipt = result.unwrap();
    match &receipt.outcome {
        FiringOutcome::Refused { stage, reason } => {
            assert_eq!(stage, "delegability");
            assert!(reason.contains("delegability violation on 'human-cap'"), "reason: {reason}");
        }
        other => panic!("expected Refused(delegability), got {other:?}"),
    }
}

#[test]
fn test_unauthorized_agent_execution() {
    // 3. Unauthorized agent execution:
    // We declare an agent profile for deterministic-v1 with NO tools.
    // The capability requires tool "Read".
    // We verify that executing it fails because the agent lacks the required tool.
    
    let agent_ttl = format!(
        "@prefix agent: <{AGENT_NS}> .\n\
         <{HANDLER_NS}deterministic-v1> a agent:Agent ;\n\
             agent:layerDepth 1 .\n" // No tools declared
    );

    let ttl = format!(
        "@prefix wf:   <{WF_NS}> .\n\
         @prefix hook: <{HOOK_NS}> .\n\
         @prefix agent: <{AGENT_NS}> .\n\
         @prefix prx:  <{LIFE}> .\n\
         @prefix ex:   <http://seanchatmangpt.github.io/praxis/prayer#> .\n\
         @prefix pk:   <http://seanchatmangpt.github.io/praxis/prayer-kernel#> .\n\
         \n\
         pk:LordPrayerKernel a pk:Kernel ;\n\
             pk:clause pk:our-father, pk:hallowed-name, pk:kingdom-come,\n\
                 pk:will-be-done, pk:on-earth-as-heaven, pk:daily-bread,\n\
                 pk:forgive-debts, pk:forgive-debtors, pk:temptation-guard,\n\
                 pk:deliverance, pk:doxology .\n\
         \n\
         pk:our-father a pk:Clause ; pk:name \"our-father\" ; pk:problemClass \"c1\" ; pk:boundary \"human-only\" .\n\
         pk:hallowed-name a pk:Clause ; pk:name \"hallowed-name\" ; pk:problemClass \"c2\" ; pk:boundary \"human-only\" .\n\
         pk:kingdom-come a pk:Clause ; pk:name \"kingdom-come\" ; pk:problemClass \"c3\" ; pk:boundary \"god-receives-unbounded\" ; pk:action ex:DeliveranceHook .\n\
         pk:will-be-done a pk:Clause ; pk:name \"will-be-done\" ; pk:problemClass \"c4\" ; pk:boundary \"human-only\" .\n\
         pk:on-earth-as-heaven a pk:Clause ; pk:name \"on-earth-as-heaven\" ; pk:problemClass \"c5\" ; pk:boundary \"god-receives-unbounded\" ; pk:action ex:DeliveranceHook .\n\
         pk:daily-bread a pk:Clause ; pk:name \"daily-bread\" ; pk:problemClass \"c6\" ; pk:boundary \"automatable-support\" .\n\
         pk:forgive-debts a pk:Clause ; pk:name \"forgive-debts\" ; pk:problemClass \"c7\" ; pk:boundary \"god-receives-unbounded\" ; pk:action ex:DeliveranceHook .\n\
         pk:forgive-debtors a pk:Clause ; pk:name \"forgive-debtors\" ; pk:problemClass \"c8\" ; pk:boundary \"human-only\" .\n\
         pk:temptation-guard a pk:Clause ; pk:name \"temptation-guard\" ; pk:problemClass \"c9\" ; pk:boundary \"automatable-support\" .\n\
         pk:doxology a pk:Clause ; pk:name \"doxology\" ; pk:problemClass \"c11\" ; pk:boundary \"human-only\" .\n\
         \n\
         pk:deliverance a pk:Clause ;\n\
             pk:name \"deliverance\" ;\n\
             pk:problemClass \"unbounded-threat\" ;\n\
             pk:boundary \"god-receives-unbounded\" ;\n\
             pk:action ex:DeliveranceHook .\n\
         \n\
         ex:DeliveranceHook a hook:Hook ;\n\
             hook:name \"deliverance\" ;\n\
             hook:kind \"delta\" ;\n\
             hook:var \"{LIFE}hasUnboundedThreat\" ;\n\
             hook:effect \"refuse\" ;\n\
             hook:reason \"refused\" .\n\
         \n\
         ex:TriggerHook a hook:Hook ;\n\
             hook:name \"trigger\" ;\n\
             hook:on \"assert\" ;\n\
             hook:kind \"delta\" ;\n\
             hook:var \"{LIFE}triggerState\" ;\n\
             hook:effect \"ground-action\" ;\n\
             hook:action ex:ToolWorkflow ;\n\
             hook:priority 0 .\n\
         \n\
         ex:ToolWorkflow a wf:Workflow ;\n\
             wf:budget 1 ;\n\
             wf:init ex:initAtom ;\n\
             wf:goal ex:goalAtom ;\n\
             wf:capability ex:toolCap .\n\
         \n\
         ex:initAtom a wf:Atom ; wf:predicate \"init\" ; wf:arg0 \"x\" .\n\
         ex:goalAtom a wf:Atom ; wf:predicate \"goal\" ; wf:arg0 \"x\" .\n\
         \n\
         ex:toolCap a wf:Capability ;\n\
             wf:name \"read-tool-cap\" ;\n\
             wf:handler <{HANDLER_NS}deterministic-v1> ;\n\
             wf:delegability \"automatable\" ;\n\
             agent:tool \"Read\" ;\n\
             wf:params 0 ;\n\
             wf:cost 1 ;\n\
             wf:pre ex:initAtom ;\n\
             wf:add ex:goalAtom .\n\
         \n\
         {agent_ttl}\n\
    "
    );

    let reference = Reference::genesis(&ttl).expect("kernel admits");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}x> <{LIFE}triggerState> 1 ."));

    let result = fire_hooks(&reference, &source, &registry, &[]);
    assert!(result.is_err(), "Expected fire_hooks to fail because agent lacks tool");
    match result.unwrap_err() {
        Refusal::DelegabilityViolation { capability, required, declared } => {
            assert_eq!(capability, "read-tool-cap");
            assert!(required.contains("agent tool 'Read'"), "required: {required}");
            assert!(declared.contains("agent tools []"), "declared: {declared}");
        }
        other => panic!("expected DelegabilityViolation, got {other:?}"),
    }
}
