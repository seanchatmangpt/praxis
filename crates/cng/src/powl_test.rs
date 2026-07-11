#![cfg(test)]

use super::*;
use bcinr_pddl::Pddl8Tape;
use chicago_tdd_tools::prelude::*;
use oxigraph::model::NamedNode;
use oxigraph::store::Store;

/// Parses serializer output into an in-memory store so assertions run
/// over the parsed graph via `crate::shape::validate_powl_store` and the
/// typed `quads_for_pattern` API — never substring matching on Turtle
/// and never inline SPARQL strings.
fn store_from_turtle(turtle: &str) -> Store {
    let store = Store::new().expect("in-memory store must construct");
    store
        .load_from_slice(
            oxigraph::io::RdfParser::from_format(oxigraph::io::RdfFormat::Turtle),
            turtle.as_bytes(),
        )
        .expect("serializer output must be valid Turtle");
    store
}

/// Objects of `<subject_iri> <predicate_iri> ?o` in the default graph,
/// via the typed pattern API. O(matches).
fn objects_of(store: &Store, subject_iri: &str, predicate_iri: &str) -> Vec<String> {
    let subject = NamedNode::new(subject_iri).expect("test subject IRI must parse");
    let predicate = NamedNode::new(predicate_iri).expect("test predicate IRI must parse");
    store
        .quads_for_pattern(
            Some(subject.as_ref().into()),
            Some(predicate.as_ref()),
            None,
            None,
        )
        .map(|quad| quad.expect("quad must decode").object.to_string())
        .collect()
}

/// Count of quads carrying `<predicate_iri>` anywhere in the store, via
/// the typed pattern API. O(matches).
fn predicate_count(store: &Store, predicate_iri: &str) -> usize {
    let predicate = NamedNode::new(predicate_iri).expect("test predicate IRI must parse");
    store
        .quads_for_pattern(None, Some(predicate.as_ref()), None, None)
        .count()
}

test!(empty_tape_refuses_plan_unsolvable, {
    let empty = Pddl8Tape { ops: vec![] };
    match project_tape_to_powl(&empty) {
        Err(refusal @ CngRefusal::PlanUnsolvable(_)) => {
            assert_eq!(refusal.code(), "CNG_R04");
            assert!(!refusal.message().is_empty());
        }
        other => panic!("expected PlanUnsolvable, got {other:?}"),
    }
});

test!(audit_mismatch_refusal_has_stable_code, {
    let refusal = CngRefusal::AuditMismatch("digest drift".to_string());
    assert_eq!(refusal.code(), "CNG_R11");
    assert_eq!(refusal.message(), "digest drift");
    assert_eq!(format!("{refusal}"), "CNG_R11: digest drift");
});

test!(provenance_serializer_emits_one_source_per_leaf, {
    let model = Powl::PartialOrder {
        children: vec![
            Powl::Leaf(Some("a(x)".to_string())),
            Powl::Leaf(Some("b(x)".to_string())),
        ],
        order: [(0usize, 1usize)].into_iter().collect(),
    };
    let sources = vec!["urn:blake3:aa".to_string(), "urn:blake3:bb".to_string()];
    let turtle = powl_to_turtle_with_provenance(&model, "urn:t", Some("urn:src"), &sources)
        .expect("aligned provenance must serialize");
    let store = store_from_turtle(&turtle);
    let prov_iri = format!("{PROV_PREFIX}wasDerivedFrom");
    for (idx, expected_source) in sources.iter().enumerate() {
        assert_eq!(
            objects_of(&store, &format!("urn:t/n0/c{idx}"), &prov_iri),
            vec![format!("<{expected_source}>")],
            "leaf {idx} must carry exactly its own source's provenance"
        );
    }
    assert_eq!(predicate_count(&store, &prov_iri), sources.len());
    // Misaligned provenance refuses (UnsupportedConstruct, CNG_R05).
    match powl_to_turtle_with_provenance(&model, "urn:t", None, &sources[..1]) {
        Err(r @ CngRefusal::UnsupportedConstruct(_)) => assert_eq!(r.code(), "CNG_R05"),
        other => panic!("expected UnsupportedConstruct, got {other:?}"),
    }
});

test!(turtle_is_deterministic_and_derived_from_is_root_only, {
    let model = Powl::PartialOrder {
        children: vec![
            Powl::Leaf(Some("a(x)".to_string())),
            Powl::Leaf(Some("b(x)".to_string())),
        ],
        order: [(0usize, 1usize)].into_iter().collect(),
    };
    // Determinism: whole-output byte equality (String equality, not
    // substring matching).
    let a = powl_to_turtle(&model, "urn:t", Some("urn:src"));
    let b = powl_to_turtle(&model, "urn:t", Some("urn:src"));
    assert_eq!(a, b, "same inputs must serialize byte-identically");
    // Root-only provenance, asserted over the parsed graph.
    let store = store_from_turtle(&a);
    let derived_iri = format!("{POWL2_PREFIX}derivedFrom");
    assert_eq!(predicate_count(&store, &derived_iri), 1);
    assert_eq!(
        objects_of(&store, "urn:t/n0", &derived_iri),
        vec!["<urn:src>".to_string()],
        "the single powl2:derivedFrom triple must sit on the root"
    );
});

/// Builds a synthetic tape op with a given `(schema_name, label)`; the
/// action's preconditions/effects are irrelevant to projection.
fn tape_op(index: u8, pred_mask: u64, schema_name: &str) -> bcinr_pddl::Pddl8TapeOp {
    bcinr_pddl::Pddl8TapeOp {
        index,
        label: format!("{schema_name}()"),
        pred_mask,
        action: bcinr_pddl::Pddl8GroundAction {
            schema_name: schema_name.to_string(),
            label: format!("{schema_name}()"),
            preconditions: vec![],
            add_effects: vec![],
            del_effects: vec![],
        },
    }
}

/// Three artifacts, tape order A,A,B,C — a run of consecutive same-source
/// ops followed by two single-op phases from distinct artifacts.
fn three_phase_tape_and_sources() -> (Pddl8Tape, BTreeMap<String, String>) {
    let tape = Pddl8Tape {
        ops: vec![
            tape_op(0, 0, "act_a1"),
            tape_op(1, 1, "act_a2"),
            tape_op(2, 2, "act_b1"),
            tape_op(3, 4, "act_c1"),
        ],
    };
    let mut sources = BTreeMap::new();
    sources.insert("act_a1".to_string(), "urn:blake3:aa".to_string());
    sources.insert("act_a2".to_string(), "urn:blake3:aa".to_string());
    sources.insert("act_b1".to_string(), "urn:blake3:bb".to_string());
    sources.insert("act_c1".to_string(), "urn:blake3:cc".to_string());
    (tape, sources)
}

test!(
    hierarchical_projection_groups_consecutive_same_source_runs,
    {
        let (tape, sources) = three_phase_tape_and_sources();
        let (model, phase_sources) =
            project_tape_to_powl_hierarchical(&tape, &sources).expect("must project");

        assert_eq!(
            phase_sources,
            vec![
                "urn:blake3:aa".to_string(),
                "urn:blake3:bb".to_string(),
                "urn:blake3:cc".to_string(),
            ]
        );
        let Powl::PartialOrder { children, order } = &model else {
            panic!("expected root PartialOrder");
        };
        assert_eq!(children.len(), 3, "3 phases: [a1,a2], [b1], [c1]");
        assert_eq!(order.len(), 3, "C(3,2) root-level precedence pairs");
        let Powl::PartialOrder {
            children: phase0_leaves,
            order: phase0_order,
        } = &children[0]
        else {
            panic!("expected phase 0 to be a PartialOrder");
        };
        assert_eq!(phase0_leaves.len(), 2, "phase 0 groups act_a1 and act_a2");
        assert_eq!(phase0_order.len(), 1, "C(2,2) intra-phase precedence pair");
        let Powl::PartialOrder {
            children: phase1_leaves,
            ..
        } = &children[1]
        else {
            panic!("expected phase 1 to be a PartialOrder");
        };
        assert_eq!(phase1_leaves.len(), 1, "phase 1 is the lone act_b1 op");
    }
);

test!(hierarchical_projection_refuses_empty_tape, {
    let empty = Pddl8Tape { ops: vec![] };
    match project_tape_to_powl_hierarchical(&empty, &BTreeMap::new()) {
        Err(refusal @ CngRefusal::PlanUnsolvable(_)) => {
            assert_eq!(refusal.code(), "CNG_R04");
        }
        other => panic!("expected PlanUnsolvable, got {other:?}"),
    }
});

test!(hierarchical_projection_refuses_untracked_action, {
    let tape = Pddl8Tape {
        ops: vec![tape_op(0, 0, "act_unknown")],
    };
    match project_tape_to_powl_hierarchical(&tape, &BTreeMap::new()) {
        Err(refusal @ CngRefusal::HardcodingSuspicion(_)) => {
            assert_eq!(refusal.code(), "CNG_R09");
        }
        other => panic!("expected HardcodingSuspicion, got {other:?}"),
    }
});

test!(phase_provenance_serializer_emits_one_source_per_phase, {
    let (tape, sources) = three_phase_tape_and_sources();
    let (model, phase_sources) =
        project_tape_to_powl_hierarchical(&tape, &sources).expect("must project");

    let turtle =
        powl_to_turtle_with_phase_provenance(&model, "urn:t", Some("urn:src"), &phase_sources)
            .expect("aligned phase provenance must serialize");
    let store = store_from_turtle(&turtle);

    // The nested model passes the crate's own structural validator —
    // this doubles as the shape.rs regression test for hierarchical
    // output (root Model + 4 PartialOrders + 4 labelled leaves + 7
    // bindings: 3 root-level, 4 leaf-level).
    let report =
        crate::shape::validate_powl_store(&store, true).expect("nested model must validate");
    assert_eq!(report.models, 1);
    assert_eq!(report.partial_orders, 4, "root + 3 phase PartialOrders");
    assert_eq!(report.activity_leaves, 4, "all 4 tape ops are leaves");
    assert_eq!(
        report.child_bindings, 7,
        "3 phase bindings + 4 leaf bindings"
    );
    assert_eq!(report.derived_from, 1);

    // One prov:wasDerivedFrom per phase node (n0/c0, n0/c1, n0/c2), each
    // pointing at that phase's contributing source IRI — asserted with
    // the typed pattern API over the parsed graph.
    let prov_iri = format!("{PROV_PREFIX}wasDerivedFrom");
    for (phase_idx, expected_source) in phase_sources.iter().enumerate() {
        assert_eq!(
            objects_of(&store, &format!("urn:t/n0/c{phase_idx}"), &prov_iri),
            vec![format!("<{expected_source}>")],
            "phase {phase_idx} must carry exactly its own source's provenance"
        );
    }
    assert_eq!(
        predicate_count(&store, &prov_iri),
        3,
        "exactly one prov:wasDerivedFrom triple per phase"
    );
});

test!(phase_provenance_serializer_refuses_flat_model, {
    // A flat model (top-level children are Leaf, not PartialOrder) is not
    // a hierarchical shape — refuses CNG_R05, points callers at the flat
    // provenance function instead.
    let flat_model = Powl::PartialOrder {
        children: vec![
            Powl::Leaf(Some("a(x)".to_string())),
            Powl::Leaf(Some("b(x)".to_string())),
        ],
        order: [(0usize, 1usize)].into_iter().collect(),
    };
    let sources = vec!["urn:blake3:aa".to_string(), "urn:blake3:bb".to_string()];
    match powl_to_turtle_with_phase_provenance(&flat_model, "urn:t", None, &sources) {
        Err(r @ CngRefusal::UnsupportedConstruct(_)) => assert_eq!(r.code(), "CNG_R05"),
        other => panic!("expected UnsupportedConstruct, got {other:?}"),
    }
});

test!(
    phase_provenance_serializer_refuses_misaligned_source_count,
    {
        let (tape, sources) = three_phase_tape_and_sources();
        let (model, phase_sources) =
            project_tape_to_powl_hierarchical(&tape, &sources).expect("must project");
        match powl_to_turtle_with_phase_provenance(
            &model,
            "urn:t",
            None,
            &phase_sources[..phase_sources.len() - 1],
        ) {
            Err(r @ CngRefusal::UnsupportedConstruct(_)) => assert_eq!(r.code(), "CNG_R05"),
            other => panic!("expected UnsupportedConstruct, got {other:?}"),
        }
    }
);

test!(
    existing_flat_functions_are_unaffected_by_hierarchical_additions,
    {
        // Regression guard: the pre-existing flat projection/serialization
        // shape is unchanged after adding the hierarchical siblings, verified
        // by the crate's own structural validator over the parsed output —
        // no substring matching, no inline query strings.
        let tape = Pddl8Tape {
            ops: vec![tape_op(0, 0, "a"), tape_op(1, 1, "b")],
        };
        let model = project_tape_to_powl(&tape).expect("flat projection");
        let turtle = powl_to_turtle(&model, "urn:t", Some("urn:src"));
        let store = store_from_turtle(&turtle);

        let report =
            crate::shape::validate_powl_store(&store, true).expect("flat model must validate");
        assert_eq!(report.models, 1);
        assert_eq!(report.partial_orders, 1, "flat model has one PartialOrder");
        assert_eq!(
            report.activity_leaves, 2,
            "both flat tape ops must serialize as labelled ActivityLeafs"
        );
        assert_eq!(report.child_bindings, 2);
        assert_eq!(report.precedes, 1, "C(2,2) = 1 closed order pair");
        assert_eq!(report.derived_from, 1);
    }
);

/// One representative instance of every `CngRefusal` variant, in `code()`
/// order (`CNG_R01`..`CNG_R25`).
///
/// The internal match over the constructed values has no wildcard arm: a
/// future `CNG_R26` variant fails to compile here until this list is
/// extended, mirroring the exhaustiveness already enforced by `code()`,
/// `message()`, and `hint()` themselves.
fn all_refusal_variants() -> Vec<CngRefusal> {
    let variants = vec![
        CngRefusal::MalformedTtl("bad turtle: unexpected token".to_string()),
        CngRefusal::MissingDomain("no domain fragment found".to_string()),
        CngRefusal::MissingProblem("no problem fragment found".to_string()),
        CngRefusal::PlanUnsolvable("empty PDDL plan tape".to_string()),
        CngRefusal::UnsupportedConstruct("nested POWL is not supported".to_string()),
        CngRefusal::InvalidPowl("PartialOrder missing hasChild".to_string()),
        CngRefusal::RunnerMismatch("runner executed out of projected order".to_string()),
        CngRefusal::Nondeterminism("repeated manufacture produced different bytes".to_string()),
        CngRefusal::HardcodingSuspicion("leaf has no contributing source".to_string()),
        CngRefusal::IoRefused("permission denied writing output".to_string()),
        CngRefusal::AuditMismatch("digest drift".to_string()),
        CngRefusal::StandingAmbiguous {
            tick: 3,
            candidate_count: 2,
        },
        CngRefusal::UnreceiptedActuation {
            workflow: "wf-1".to_string(),
            category: "manufacture".to_string(),
        },
        CngRefusal::DialectRegistryRefused {
            entry: "urn:entry:1".to_string(),
            missing: "sourceIri".to_string(),
        },
        CngRefusal::DispatchContractIncomplete {
            dispatch: "d-1".to_string(),
            missing: "callback_url".to_string(),
        },
        CngRefusal::DispatchStateUnlawful {
            dispatch: "d-1".to_string(),
            from: "Sent".to_string(),
            to: "Acknowledged".to_string(),
        },
        CngRefusal::ExternalConsequenceRefused {
            dispatch: "d-1".to_string(),
            stage: "authority".to_string(),
        },
        CngRefusal::ArazzoProfileRefused {
            feature: "criterionType=xpath".to_string(),
        },
        CngRefusal::EvidenceGateFailed {
            gate: "unreceipted-actuations".to_string(),
            count: 3,
        },
        CngRefusal::MarkerFalse {
            marker: "AUTONOMIC_LOOP_CLOSED".to_string(),
            value: 1,
        },
        CngRefusal::DecompositionInadmissible {
            candidate: "0-single".to_string(),
            reason: "interference".to_string(),
        },
        CngRefusal::InterferenceDetected {
            helper_action: "pickup(a)".to_string(),
            main_action: "stack(a,b)".to_string(),
            atom: "holding(a)".to_string(),
        },
        CngRefusal::InterfaceStateMismatch {
            step: 2,
            atom: "clear(b)".to_string(),
        },
        CngRefusal::ResourceUnreleased {
            resource: "lock(room1)".to_string(),
            holder: "helper".to_string(),
        },
        CngRefusal::DoubleAdmit {
            dispatch: "d-2".to_string(),
            idempotency_key: "key-123".to_string(),
        },
    ];
    for refusal in &variants {
        match refusal {
            CngRefusal::MalformedTtl(_) => {}
            CngRefusal::MissingDomain(_) => {}
            CngRefusal::MissingProblem(_) => {}
            CngRefusal::PlanUnsolvable(_) => {}
            CngRefusal::UnsupportedConstruct(_) => {}
            CngRefusal::InvalidPowl(_) => {}
            CngRefusal::RunnerMismatch(_) => {}
            CngRefusal::Nondeterminism(_) => {}
            CngRefusal::HardcodingSuspicion(_) => {}
            CngRefusal::IoRefused(_) => {}
            CngRefusal::AuditMismatch(_) => {}
            CngRefusal::StandingAmbiguous { .. } => {}
            CngRefusal::UnreceiptedActuation { .. } => {}
            CngRefusal::DialectRegistryRefused { .. } => {}
            CngRefusal::DispatchContractIncomplete { .. } => {}
            CngRefusal::DispatchStateUnlawful { .. } => {}
            CngRefusal::ExternalConsequenceRefused { .. } => {}
            CngRefusal::ArazzoProfileRefused { .. } => {}
            CngRefusal::EvidenceGateFailed { .. } => {}
            CngRefusal::MarkerFalse { .. } => {}
            CngRefusal::DecompositionInadmissible { .. } => {}
            CngRefusal::InterferenceDetected { .. } => {}
            CngRefusal::InterfaceStateMismatch { .. } => {}
            CngRefusal::ResourceUnreleased { .. } => {}
            CngRefusal::DoubleAdmit { .. } => {}
        }
    }
    variants
}

test!(every_refusal_variant_has_a_specific_actionable_hint, {
    let variants = all_refusal_variants();
    assert_eq!(
        variants.len(),
        25,
        "expected one instance per CNG_R01..CNG_R25 variant"
    );
    let mut seen_hints = std::collections::BTreeSet::new();
    for refusal in &variants {
        let code = refusal.code();
        let hint = refusal.hint();
        assert!(
            hint.len() >= 40,
            "{code}: hint too short to carry an actionable next step: {hint:?}"
        );
        assert_ne!(
            hint,
            refusal.message(),
            "{code}: hint must add actionable guidance, not restate message()"
        );
        assert!(
            seen_hints.insert(hint),
            "{code}: hint duplicates another variant's hint verbatim (copy-paste)"
        );
    }
    assert_eq!(
        seen_hints.len(),
        25,
        "every variant must carry its own distinct, non-generic hint"
    );
});
