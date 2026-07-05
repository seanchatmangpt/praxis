//! PROJ-303 — authority ledger via firing-time provenance check.
//!
//! A `ground-action` firing whose action node lacks a PROV-O
//! `prov:wasAttributedTo` triple is refused (`Refusal::WorkflowIllFormed`,
//! reusing `reality.rs`'s `provenance_anchor`, no new ledger subsystem); the
//! identical fragment WITH the triple grounds and executes normally.

use praxis_synthesis::graph::WF_NS;
use praxis_synthesis::hooks::HOOK_NS;
use praxis_synthesis::{fire_hooks, HandlerRegistry, MeaningSource, Origin, Reference, Refusal};

const LIFE: &str = "http://seanchatmangpt.github.io/praxis/life#";

fn src(adds: &str) -> MeaningSource {
    MeaningSource {
        origin: Origin::Proposer,
        adds_ttl: adds.to_string(),
        removes_ttl: String::new(),
    }
}

fn kernel_ttl(action_authority_triple: &str) -> String {
    format!(
        "@prefix wf:   <{WF_NS}> .\n\
         @prefix hook: <{HOOK_NS}> .\n\
         @prefix prx:  <{LIFE}> .\n\
         @prefix ex:   <http://seanchatmangpt.github.io/praxis/prayer#> .\n\
         @prefix pk:   <http://seanchatmangpt.github.io/praxis/prayer-kernel#> .\n\
         @prefix prov: <http://www.w3.org/ns/prov#> .\n\
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
             hook:action ex:PlainWorkflow ;\n\
             hook:priority 0 .\n\
         \n\
         ex:PlainWorkflow a wf:Workflow ;\n\
             wf:budget 1 ;\n\
             wf:init ex:initAtom ;\n\
             wf:goal ex:goalAtom ;\n\
             wf:capability ex:plainCap{action_authority_triple} .\n\
         \n\
         ex:initAtom a wf:Atom ; wf:predicate \"init\" ; wf:arg0 \"x\" .\n\
         ex:goalAtom a wf:Atom ; wf:predicate \"goal\" ; wf:arg0 \"x\" .\n\
         \n\
         ex:plainCap a wf:Capability ;\n\
             wf:name \"plain-cap\" ;\n\
             wf:params 0 ;\n\
             wf:cost 1 ;\n\
             wf:pre ex:initAtom ;\n\
             wf:add ex:goalAtom .\n\
    "
    )
}

#[test]
fn ground_action_without_authority_anchor_is_refused() {
    let ttl = kernel_ttl("");
    let reference = Reference::genesis(&ttl).expect("kernel admits");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}x> <{LIFE}triggerState> 1 ."));

    let result = fire_hooks(&reference, &source, &registry, &[]);
    match result {
        Err(Refusal::WorkflowIllFormed { subject, detail }) => {
            assert_eq!(
                subject,
                "http://seanchatmangpt.github.io/praxis/prayer#PlainWorkflow"
            );
            assert!(detail.contains("no authority anchor"), "detail: {detail}");
        }
        other => panic!("expected WorkflowIllFormed(no authority anchor), got {other:?}"),
    }
}

#[test]
fn ground_action_with_authority_anchor_succeeds() {
    let ttl = kernel_ttl(" ;\n             prov:wasAttributedTo ex:authority");
    let reference = Reference::genesis(&ttl).expect("kernel admits");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}x> <{LIFE}triggerState> 1 ."));

    let result = fire_hooks(&reference, &source, &registry, &[]);
    assert!(result.is_ok(), "expected fire_hooks to succeed: {result:?}");
}
