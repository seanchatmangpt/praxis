//! `agent_registry` wired into `firing.rs::fire_hooks`: the graph-declared
//! agent registry (tool sets, spawn edges, layer depth) is extracted and the
//! depth-5 spawn law is judged as a pre-solve, global stage — same phase as
//! handler-existence judgment, before any hook evaluation. Proves the full
//! path RDF delta -> agent registry extraction -> spawn-depth law ->
//! folded/chained receipt, per the archaeology's decisive-connector plan.

use praxis_synthesis::agent_registry::AGENT_NS;
use praxis_synthesis::handlers::HANDLER_NS;
use praxis_synthesis::{
    agent_registry_hash, fire_hooks, replay_firing, FiringOutcome, HandlerRegistry,
    MeaningSource, Origin, Reference, Refusal,
};

const KERNEL: &str = include_str!("../ontology/lord_prayer.ttl");
const LIFE: &str = "http://seanchatmangpt.github.io/praxis/life#";

fn src(adds: &str) -> MeaningSource {
    MeaningSource { origin: Origin::Proposer, adds_ttl: adds.to_string(), removes_ttl: String::new() }
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

fn kernel_with_binding_and_agent(delegability: &str, handler_local: &str, agent_ttl: &str) -> String {
    format!("{}\n{agent_ttl}", kernel_with_binding(delegability, handler_local))
}

#[test]
fn no_agent_triples_is_unchanged_regression() {
    // Today's common case: no `agent:` triples at all. `extract_agents`
    // returns `[]`, `spawn_depth_law([])` is trivially `Ok`; the firing must
    // succeed identically to pre-wiring behavior.
    let base = kernel_with_binding("verifiable", "deterministic-v1");
    let reference = Reference::genesis(&base).expect("kernel admits");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}sean> <{LIFE}hasProvisionAnxiety> 1 ."));

    let receipt = fire_hooks(&reference, &source, &registry, &[]).expect("fires");
    assert_eq!(receipt.outcome, FiringOutcome::Completed);
    assert!(receipt.agents.is_empty(), "no agent: triples => empty registry");
    assert_eq!(
        receipt.agent_registry_hash,
        agent_registry_hash(&[]),
        "empty registry hashes to the canonical empty form"
    );
    replay_firing(&receipt, &base, &source, &registry, &[]).expect("replays");
}

#[test]
fn end_to_end_agent_registry_extracted_and_folded() {
    // RDF delta -> hook verdict -> PDDL-grounded action -> agent registry
    // extraction -> folded receipt, all in one `fire_hooks` call.
    let agent_ttl = format!(
        "@prefix agent: <{AGENT_NS}> .\n\
         @prefix ex: <http://seanchatmangpt.github.io/praxis/prayer#> .\n\
         ex:coordinator a agent:Agent ; agent:layerDepth 3 ; agent:tool \"Read\" ; \
         agent:canSpawn <{AGENT_NS}leaf-read> .\n"
    );
    let base = kernel_with_binding_and_agent("verifiable", "deterministic-v1", &agent_ttl);
    let reference = Reference::genesis(&base).expect("kernel + agent triples admit");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}sean> <{LIFE}hasProvisionAnxiety> 1 ."));

    let receipt = fire_hooks(&reference, &source, &registry, &[]).expect("fires");
    assert_eq!(receipt.outcome, FiringOutcome::Completed);
    assert_eq!(receipt.inner.len(), 1, "daily-bread grounded once");
    assert_eq!(receipt.agents.len(), 1);
    assert_eq!(receipt.agents[0].layer_depth, 3);
    assert_eq!(
        receipt.agent_registry_hash,
        agent_registry_hash(&receipt.agents),
        "receipt's claimed hash must match an independent recompute over its own agents"
    );

    replay_firing(&receipt, &base, &source, &registry, &[]).expect("replays end to end");
}

#[test]
fn depth_5_with_can_spawn_refuses_before_hook_evaluation() {
    let agent_ttl = format!(
        "@prefix agent: <{AGENT_NS}> .\n\
         @prefix ex: <http://seanchatmangpt.github.io/praxis/prayer#> .\n\
         ex:leaf a agent:Agent ; agent:layerDepth 5 ; \
         agent:canSpawn <{AGENT_NS}anything> .\n"
    );
    let base = kernel_with_binding_and_agent("verifiable", "deterministic-v1", &agent_ttl);
    let reference = Reference::genesis(&base).expect("kernel + agent triples admit");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}sean> <{LIFE}hasProvisionAnxiety> 1 ."));

    let receipt = fire_hooks(&reference, &source, &registry, &[]).expect("refusal is receipted");
    match &receipt.outcome {
        FiringOutcome::Refused { stage, reason } => {
            assert_eq!(stage, "agent-spawn-depth");
            assert!(reason.contains("terminal by absence"));
        }
        other => panic!("expected Refused(agent-spawn-depth), got {other:?}"),
    }
    assert!(receipt.inner.is_empty(), "refused BEFORE any solving/hook evaluation");
    assert!(receipt.verdicts.is_empty(), "hooks never evaluated once the spawn-depth law refuses");
    replay_firing(&receipt, &base, &source, &registry, &[])
        .expect("agent-spawn-depth refusal replays too — refusals are chained, never silent");
}

#[test]
fn replay_firing_stage_count_regression_covers_agent_registry_hash() {
    // A receipt produced by the new code path must still replay: the
    // `stages` array literal inside `replay_firing` grew, and a mis-ordered
    // insertion would silently miscompare hashes rather than fail loudly.
    let base = kernel_with_binding("verifiable", "deterministic-v1");
    let reference = Reference::genesis(&base).expect("kernel admits");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}sean> <{LIFE}hasProvisionAnxiety> 1 ."));
    let receipt = fire_hooks(&reference, &source, &registry, &[]).expect("fires");

    replay_firing(&receipt, &base, &source, &registry, &[]).expect("replays");

    // Forged agent_registry_hash behind an otherwise honest chain must fail
    // named, not silently pass (caught at the stage-comparison layer, since
    // the rederived value will differ from the forged claimed value).
    let mut forged = receipt.clone();
    forged.agent_registry_hash = "forged".to_string();
    match replay_firing(&forged, &base, &source, &registry, &[]) {
        Err(Refusal::VerificationFailed { failed }) => {
            assert!(
                failed[0].contains("agent_registry_hash") || failed[0].contains("chain"),
                "expected agent_registry_hash mismatch, got {failed:?}"
            );
        }
        other => panic!("expected VerificationFailed(agent_registry_hash), got {other:?}"),
    }
}
