//! Test for quarantine/admission bypass attempts.
use praxis_synthesis::graph::WF_NS;
use praxis_synthesis::handlers::HANDLER_NS;
use praxis_synthesis::hooks::HOOK_NS;
use praxis_synthesis::{fire_hooks, HandlerRegistry, MeaningSource, Origin, Reference, Refusal};

const BASE: &str = "@prefix ex: <http://e/> .\nex:a ex:p ex:b .\n";
const LIFE: &str = "http://seanchatmangpt.github.io/praxis/life#";

fn src(adds: &str) -> MeaningSource {
    MeaningSource {
        origin: Origin::Proposer,
        adds_ttl: adds.to_string(),
        removes_ttl: String::new(),
    }
}

/// Proof-of-concept attack: propose a brand-new `wf:Capability` class
/// definition, plus a matching hook/workflow that fires on the same delta.
/// Once a genuine attack, now a closed regression: `Admission::admit`
/// refuses defining a new instance of the closed-world `wf:Capability` class
/// via a delta (`AdmissionRefused`), so the evil workflow never grounds —
/// firing itself is refused before any hook can evaluate it.
#[test]
fn execute_custom_workflow_injected_via_delta_proposer() {
    let reference = Reference::genesis(BASE).expect("base admits");
    let registry = HandlerRegistry::builtin();

    // The adversary proposes a new hook, a new workflow, a new capability, and the trigger fact in a single delta!
    let proposal = format!(
        "@prefix wf: <{WF_NS}> .\n\
         @prefix hook: <{HOOK_NS}> .\n\
         @prefix ex: <http://e/> .\n\
         \n\
         # 1. Trigger fact\n\
         ex:sean <{LIFE}hasCustomAnxiety> 1 .\n\
         \n\
         # 2. Custom Hook\n\
         ex:evilHook a hook:Hook ;\n\
             hook:name \"evilHook\" ;\n\
             hook:kind \"delta\" ;\n\
             hook:var \"{LIFE}hasCustomAnxiety\" ;\n\
             hook:effect \"ground-action\" ;\n\
             hook:action ex:evilWorkflow .\n\
         \n\
         # 3. Custom Workflow\n\
         ex:evilWorkflow a wf:Workflow ;\n\
             wf:budget 8 ;\n\
             wf:init ex:s0 ;\n\
             wf:goal ex:g ;\n\
             wf:capability ex:evilCap ;\n\
             wf:handler <{HANDLER_NS}deterministic-v1> ;\n\
             wf:delegability \"verifiable\" .\n\
         \n\
         # 4. Custom Capability\n\
         ex:evilCap a wf:Capability ;\n\
             wf:name \"evilCap\" ;\n\
             wf:params 0 ;\n\
             wf:cost 1 ;\n\
             wf:pre ex:s0 ;\n\
             wf:add ex:g .\n\
         ex:s0 a wf:Atom ; wf:predicate \"s0\" .\n\
         ex:g a wf:Atom ; wf:predicate \"g\" .\n"
    );

    let source = src(&proposal);

    match fire_hooks(&reference, &source, &registry, &[]) {
        Err(Refusal::AdmissionRefused { subject, detail }) => {
            assert_eq!(subject, "http://e/evilCap");
            assert!(detail.contains("forbidden in deltas"));
        }
        other => {
            panic!("expected AdmissionRefused for a delta-proposed Capability class, got {other:?}")
        }
    }
}
