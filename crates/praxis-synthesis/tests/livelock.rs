//! AA livelock soundness model — detection programs, closure facts, the
//! spilled-milk disjunction, and the NO-INFINITE-REHEARSAL law.

use praxis_synthesis::graph::parse_ttl;
use praxis_synthesis::life;
use praxis_synthesis::livelock::{
    detect, detection_program, rehearsal_exceeded, LivelockClass, ALL_CLASSES, STEPS,
};
use praxis_synthesis::{
    fire_hooks, FiringOutcome, GraphDelta, HandlerRegistry, MeaningSource, Origin, Reference,
};

const LIFE: &str = "http://seanchatmangpt.github.io/praxis/life#";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

fn typed(subject: &str, class: &str) -> String {
    format!("<http://e/{subject}> <{RDF_TYPE}> <{class}> .\n")
}

fn edge(subject: &str, pred: &str, object: &str) -> String {
    format!("<http://e/{subject}> <{pred}> <http://e/{object}> .\n")
}

// TEST-14: an unreleased resentment loop is detected as an open livelock.
#[test]
fn test_14_resentment_livelock_detected() {
    let triples = parse_ttl(&typed("grudge", life::RESENTMENT_LOOP)).expect("parses");
    assert!(
        detect(LivelockClass::Resentment, &triples).expect("evaluates"),
        "unreleased ResentmentLoop is an open livelock"
    );
}

// TEST-15: the release fact converts the loop to closed, and an
// InventoryEntry query over the same graph sees the inventory row.
#[test]
fn test_15_release_fact_closes_and_inventory_sees_it() {
    let doc = format!(
        "{}{}{}",
        typed("grudge", life::RESENTMENT_LOOP),
        edge("release-act", life::RELEASES, "grudge"),
        typed("release-act", life::INVENTORY_ENTRY),
    );
    let triples = parse_ttl(&doc).expect("parses");
    assert!(
        !detect(LivelockClass::Resentment, &triples).expect("evaluates"),
        "released loop is closed"
    );
    assert!(life::open_resentments(&triples).is_empty());
    assert_eq!(
        life::subjects_of(&triples, life::INVENTORY_ENTRY),
        vec!["http://e/release-act"],
        "the closing act is visible to the inventory query"
    );
}

// TEST-16: spilled milk closes through ANY of repairs / learnsFrom /
// releases — and stays open with none of them.
#[test]
fn test_16_spilled_milk_closes_through_any_of_three() {
    let spill = typed("spill", life::IRREVERSIBLE_EVENT);
    let open = parse_ttl(&spill).expect("parses");
    assert!(
        detect(LivelockClass::SpilledMilk, &open).expect("evaluates"),
        "no repair/lesson/release: open"
    );

    for closer in [life::REPAIRS, life::LEARNS_FROM, life::RELEASES] {
        let doc = format!("{spill}{}", edge("act", closer, "spill"));
        let triples = parse_ttl(&doc).expect("parses");
        assert!(
            !detect(LivelockClass::SpilledMilk, &triples).expect("evaluates"),
            "{closer} alone closes the irreversible event"
        );
    }
}

// TEST-17: rehearsal_exceeded fires exactly at the bound, and a
// window-kind hook watching the loop predicate refuses the rehearsal —
// the unsound infinite loop is refused, not replayed.
#[test]
fn test_17_infinite_rehearsal_refused_at_bound() {
    let rehearses = format!("{LIFE}rehearses");
    let touch = || {
        GraphDelta::parse(
            &format!("<http://e/me> <{rehearses}> <http://e/grudge> ."),
            "",
        )
        .expect("delta parses")
    };
    let history: Vec<GraphDelta> = (0..3).map(|_| touch()).collect();
    assert!(
        !rehearsal_exceeded(&history[..2], "http://e/grudge", 3),
        "under the bound"
    );
    assert!(
        rehearsal_exceeded(&history, "http://e/grudge", 3),
        "fires AT the bound"
    );
    let unrelated = GraphDelta::parse("<http://e/x> <http://e/p> 1 .", "").expect("parses");
    assert!(
        !rehearsal_exceeded(&[unrelated], "http://e/grudge", 1),
        "deltas not touching the loop do not count"
    );

    // Window hook on the rehearsal predicate: 3rd touch within a 3-delta
    // window refuses.
    let base = format!(
        "@prefix hook: <http://seanchatmangpt.github.io/praxis/hook#> .\n\
         <http://e/no-rehearsal> a hook:Hook ;\n\
           hook:name \"no-infinite-rehearsal\" ;\n\
           hook:kind \"window\" ;\n\
           hook:var \"{rehearses}\" ;\n\
           hook:op \">=\" ; hook:k 3 ; hook:window 3 ;\n\
           hook:effect \"refuse\" ;\n\
           hook:reason \"rehearsal budget exhausted: park the loop, hand it off\" .\n"
    );
    let reference = Reference::genesis(&base).expect("base admits");
    let registry = HandlerRegistry::builtin();
    let source = MeaningSource {
        origin: Origin::Proposer,
        adds_ttl: format!("<http://e/me> <{rehearses}> <http://e/grudge> ."),
        removes_ttl: String::new(),
    };
    let receipt = fire_hooks(&reference, &source, &registry, &history[..2]).expect("receipted");
    match &receipt.outcome {
        FiringOutcome::Refused { stage, reason } => {
            assert_eq!(stage, "declared-refusal");
            assert!(
                reason.contains("rehearsal budget exhausted"),
                "reason: {reason}"
            );
        }
        other => panic!("expected refusal at the rehearsal bound, got {other:?}"),
    }
}

// Every LivelockClass detection program parses and evaluates on a synthetic
// graph: open instance detected, closing fact converts it to closed.
#[test]
fn every_class_program_parses_and_evaluates() {
    // (class, open TTL, closing TTL appended to the open doc)
    let cases: Vec<(LivelockClass, String, String)> = vec![
        (
            LivelockClass::Resentment,
            typed("loop", life::RESENTMENT_LOOP),
            edge("act", life::RELEASES, "loop"),
        ),
        (
            LivelockClass::Shame,
            typed("harm", life::HARM),
            edge("amend", life::AMENDS_FOR, "harm"),
        ),
        (
            LivelockClass::SelfWill,
            typed("plan", life::SELF_PLAN),
            edge("plan", life::SURRENDERED, "higher"),
        ),
        (
            LivelockClass::Fear,
            typed("worry", life::PROVISION_ANXIETY),
            edge("worry", life::HAS_BREAD_RECEIPT, "bread-receipt"),
        ),
        (
            LivelockClass::ReliefSeeking,
            typed("risk", life::TEMPTATION_RISK),
            edge("guard", life::GUARDS, "risk"),
        ),
        (
            LivelockClass::SpilledMilk,
            typed("spill", life::IRREVERSIBLE_EVENT),
            edge("lesson", life::LEARNS_FROM, "spill"),
        ),
    ];
    assert_eq!(cases.len(), ALL_CLASSES.len(), "every class exercised");
    for (class, open_ttl, closer_ttl) in cases {
        let (program, goal) = detection_program(class);
        assert!(
            program.contains(goal),
            "{class:?} program derives its own goal"
        );
        let open = parse_ttl(&open_ttl).expect("open doc parses");
        assert!(
            detect(class, &open).expect("evaluates"),
            "{class:?} open instance detected"
        );
        let closed = parse_ttl(&format!("{open_ttl}{closer_ttl}")).expect("closed doc parses");
        assert!(
            !detect(class, &closed).expect("evaluates"),
            "{class:?} closing fact closes"
        );
        assert!(
            !detect(class, &[]).expect("evaluates"),
            "{class:?} empty graph is clean"
        );
    }
}

// The twelve-step mapping is complete, ordered, and speaks soundness.
#[test]
fn twelve_steps_map_to_soundness_operations() {
    assert_eq!(STEPS.len(), 12);
    for (i, (n, recovery, soundness)) in STEPS.iter().enumerate() {
        assert_eq!(usize::from(*n), i + 1, "steps are 1..=12 in order");
        assert!(!recovery.is_empty());
        assert!(!soundness.is_empty());
    }
    assert!(STEPS[0].2.contains("livelock detection"));
    assert!(STEPS[9].2.contains("daily livelock detection"));
    assert!(STEPS[11].2.contains("service output"));
}
