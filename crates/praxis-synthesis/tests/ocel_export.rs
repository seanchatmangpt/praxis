//! PROJ-304 — OCEL V2 event export from existing firing receipts.
//!
//! `to_ocel_event` is a pure, read-only projection of `HookFiringReceipt`:
//! no I/O, no new hash folded into the firing chain. A firing with no bound
//! `RealityAddressRecord` omits `time` rather than inventing a wall clock.

use praxis_synthesis::graph::WF_NS;
use praxis_synthesis::handlers::HANDLER_NS;
use praxis_synthesis::hooks::HOOK_NS;
use praxis_synthesis::reality::RealityAddressRecord;
use praxis_synthesis::{
    fire_hooks, graph::parse_ttl, to_ocel_event, FiringOutcome, HandlerRegistry, MeaningSource,
    Origin, Reference,
};

const LIFE: &str = "http://seanchatmangpt.github.io/praxis/life#";

fn src(adds: &str) -> MeaningSource {
    MeaningSource {
        origin: Origin::Proposer,
        adds_ttl: adds.to_string(),
        removes_ttl: String::new(),
    }
}

fn kernel_ttl() -> String {
    format!(
        "@prefix wf:   <{WF_NS}> .\n\
         @prefix hook: <{HOOK_NS}> .\n\
         @prefix prx:  <{LIFE}> .\n\
         @prefix ex:   <http://e/> .\n\
         @prefix prov: <http://www.w3.org/ns/prov#> .\n\
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
             wf:capability ex:plainCap ;\n\
             prov:wasAttributedTo ex:authority .\n\
         \n\
         ex:initAtom a wf:Atom ; wf:predicate \"init\" ; wf:arg0 \"x\" .\n\
         ex:goalAtom a wf:Atom ; wf:predicate \"goal\" ; wf:arg0 \"x\" .\n\
         \n\
         ex:plainCap a wf:Capability ;\n\
             wf:name \"plain-cap\" ;\n\
             wf:params 0 ;\n\
             wf:cost 1 ;\n\
             wf:pre ex:initAtom ;\n\
             wf:add ex:goalAtom ;\n\
             wf:handler <{HANDLER_NS}deterministic-v1> ;\n\
             wf:delegability \"automatable\" .\n\
    "
    )
}

#[test]
fn completed_firing_renders_expected_shape_with_no_time_when_unanchored() {
    let ttl = kernel_ttl();
    let reference = Reference::genesis(&ttl).expect("kernel admits");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}x> <{LIFE}triggerState> 1 ."));

    let receipt = fire_hooks(&reference, &source, &registry, &[]).expect("fires");
    assert_eq!(receipt.outcome, FiringOutcome::Completed);

    // No RealityAddressRecord bound for this firing (the source event carries
    // no OWL-Time/GeoSPARQL/PROV-O anchor of its own) — time must be absent,
    // not fabricated.
    let event = to_ocel_event(&receipt, None);
    let obj = event.as_object().expect("event is a JSON object");
    assert_eq!(
        obj["id"],
        serde_json::Value::String(receipt.outcome_hash.clone())
    );
    assert_eq!(obj["type"], "hook-firing");
    assert!(
        !obj.contains_key("time"),
        "time must be omitted, not fabricated: {obj:?}"
    );
    let rels = obj["relationships"]
        .as_array()
        .expect("relationships is an array");
    assert!(!rels.is_empty());
    assert_eq!(rels[0]["qualifier"], "handler-binding");
    assert_eq!(obj["attributes"]["outcome"], "Completed");
    assert_eq!(obj["attributes"]["hook_hash"], receipt.hook_hash);
    assert_eq!(obj["attributes"]["event_hash"], receipt.event_hash);
}

#[test]
fn completed_firing_includes_time_when_a_reality_record_is_bound() {
    let ttl = kernel_ttl();
    let reference = Reference::genesis(&ttl).expect("kernel admits");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}x> <{LIFE}triggerState> 1 ."));
    let receipt = fire_hooks(&reference, &source, &registry, &[]).expect("fires");

    // Bind a reality address on the action node directly from the admitted
    // kernel triples (the action node already carries prov:wasAttributedTo;
    // add a time anchor separately to exercise the honest-time-only path).
    let anchored_ttl = format!(
        "{ttl}<http://e/PlainWorkflow> <http://www.w3.org/2006/time#inXSDDateTimeStamp> \"2026-07-04T00:00:00Z\" ."
    );
    let triples = parse_ttl(&anchored_ttl).expect("parses");
    let record = RealityAddressRecord::bind(&triples, "http://e/PlainWorkflow").expect("binds");

    let event = to_ocel_event(&receipt, Some(&record));
    assert_eq!(event["time"], "2026-07-04T00:00:00Z");
}

#[test]
fn refused_firing_renders_refused_outcome_with_stage_and_reason() {
    // A hook:effect "refuse" fires and is receipted as Refused, never
    // executed — to_ocel_event must render that honestly too.
    let ttl = format!(
        "@prefix wf:   <{WF_NS}> .\n\
         @prefix hook: <{HOOK_NS}> .\n\
         @prefix ex:   <http://e/> .\n\
         ex:RefuseHook a hook:Hook ;\n\
             hook:name \"refuse\" ;\n\
             hook:kind \"delta\" ;\n\
             hook:var \"{LIFE}hasThreat\" ;\n\
             hook:effect \"refuse\" ;\n\
             hook:reason \"surrendered\" .\n\
    "
    );
    let reference = Reference::genesis(&ttl).expect("kernel admits");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}x> <{LIFE}hasThreat> 1 ."));
    let receipt =
        fire_hooks(&reference, &source, &registry, &[]).expect("fires (refused, not erred)");

    match &receipt.outcome {
        FiringOutcome::Refused { .. } => {}
        other => panic!("expected Refused outcome, got {other:?}"),
    }
    let event = to_ocel_event(&receipt, None);
    assert_eq!(event["attributes"]["outcome"], "Refused");
    assert!(event["attributes"]["stage"].is_string());
    assert!(event["attributes"]["reason"].is_string());
    // A refused firing grounds no action, so no handler bindings exist.
    assert_eq!(event["relationships"].as_array().unwrap().len(), 0);
}
