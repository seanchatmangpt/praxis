#![cfg(test)]

//! SOC2 audit-engagement full-lifecycle tests (Arclight Cloud Platform,
//! v26.7.12/13; rescaled from Solace Cloud in the Stage 2 rescale): the 10
//! chained phase fixtures admit, plan as one 30-step
//! cycle, project hierarchically into 10 phase children by artifact
//! provenance, validate against the POWL structural shape, and replay to a
//! byte-identical digest. The 8-constraint split law is verified
//! mechanically (reusing `togaf::verify_eight_constraint_split` verbatim),
//! and its violation refuses typed. Case-study instance data validates
//! against the soc2 SHACL shapes (public-vocabulary modeling), with a
//! negative fixture proving the shapes actually bite. All Turtle enters
//! from on-disk fixtures; all SPARQL from the on-disk query set.
//!
//! `no_action_effect_ever_asserts_compliance_or_opinion` is the
//! COMPLIANCE-OVERCLAIM FENCE's structural enforcement point: it greps the
//! parsed, merged `Pddl8Domain`'s action effects programmatically for
//! "compliant"/"opinion" substrings, on both the real fixture set (must be
//! clean) and two adversarial mutants (must refuse typed). See
//! `soc2.rs`'s module doc for the full fence disclosure.

use std::fs;

use chicago_tdd_tools::prelude::*;
use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::store::Store;

use super::{
    compute_evidence_metrics, soc2_fixture_dir, verify_no_compliance_or_opinion_effects,
    Soc2EvidenceMetrics, SOC2_PHASES,
};
use crate::bench::dispatch::shape_violations;
use crate::bench::templates::QuerySet;
use crate::bench::togaf::verify_eight_constraint_split;
use crate::pipeline::{generate_plan, hierarchical_projection, import_artifacts};
use crate::powl::{powl_to_turtle, CngRefusal, Powl};
use crate::shape::validate_powl_store;

const BASE_IRI: &str = "urn:chatman:powl:arclight-soc2";

test!(
    full_audit_cycle_plans_projects_validates_and_replays_byte_identically,
    {
        // Arrange: admit the generated fixture directory (10 domain fragments +
        // 1 problem fragment + the non-PDDL case-study instance file, which
        // contributes no fragments).
        let dir = soc2_fixture_dir();

        // Act: the real manufacture chain, twice (replay determinism).
        let run = || {
            let artifacts = import_artifacts(&dir).expect("fixtures admit");
            let (tape, surface) = generate_plan(&artifacts).expect("cycle plan exists");
            verify_eight_constraint_split(&surface).expect("8-constraint split holds");
            verify_no_compliance_or_opinion_effects(&surface)
                .expect("compliance-overclaim fence holds");
            let (powl, phase_sources) =
                hierarchical_projection(&tape, &surface).expect("hierarchical projection");
            let ttl = powl_to_turtle(&powl, BASE_IRI, Some("urn:chatman:plan:arclight-soc2"));
            (tape, powl, phase_sources, ttl)
        };
        let (tape, powl, phase_sources, ttl) = run();
        let (_, _, _, ttl_replay) = run();

        // Assert: the full cycle is a 30-step linear plan (3 actions × 10
        // phases), forced through every phase by the precondition chain.
        assert_eq!(tape.ops.len(), 30, "3 actions per phase × 10 phases");
        let labels: Vec<&str> = tape.ops.iter().map(|op| op.label.as_str()).collect();
        assert_eq!(labels[0], "define-system-boundary(arclight)");
        assert_eq!(labels[29], "confirm-evidence-bundle-complete(arclight)");

        // The hierarchical projection groups the plan into exactly 10 phase
        // children — one per contributing fixture artifact, in engagement
        // order.
        assert_eq!(
            phase_sources.len(),
            10,
            "one provenance source per SOC2 audit phase"
        );
        let Powl::PartialOrder { children, .. } = &powl else {
            panic!("root must be a partial order over the 10 phases");
        };
        assert_eq!(children.len(), 10, "one POWL child per SOC2 audit phase");
        assert_eq!(SOC2_PHASES.len(), 10);

        // Structural shape validation over the exported Turtle.
        let store = Store::new().expect("store");
        store
            .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), ttl.as_bytes())
            .expect("exported POWL parses");
        validate_powl_store(&store, true).expect("POWL structural shape holds");

        // Replay determinism: byte-identical export, byte-identical digest.
        assert_eq!(
            blake3::hash(ttl.as_bytes()),
            blake3::hash(ttl_replay.as_bytes()),
            "re-manufacture must reproduce the POWL export byte-identically"
        );
    }
);

test!(eight_constraint_split_violations_refuse_typed, {
    // Arrange: the real merged surface, then concentrate constraints past
    // the PDDL8 bound in each of the two places the law guards. Reuses
    // togaf::verify_eight_constraint_split verbatim — it is already
    // generic over AdmittedSurface, not TOGAF-specific in its signature.
    let artifacts = import_artifacts(&soc2_fixture_dir()).expect("fixtures admit");
    let (_, surface) = generate_plan(&artifacts).expect("cycle plan exists");
    // AdmittedSurface does not derive Clone; rebuild one from its
    // Clone-able parts for each mutation.
    let resurface = || crate::pipeline::AdmittedSurface {
        domain: surface.domain.clone(),
        problem: surface.problem.clone(),
        action_sources: surface.action_sources.clone(),
    };

    // Act/Assert 1: an action with 9 precondition conjuncts refuses.
    let mut fat_action = resurface();
    let extra = fat_action.domain.actions[0].preconditions[0].clone();
    for _ in 0..9 {
        fat_action.domain.actions[0]
            .preconditions
            .push(extra.clone());
    }
    let err = verify_eight_constraint_split(&fat_action)
        .expect_err("9 precondition conjuncts must refuse");
    assert!(
        matches!(&err, CngRefusal::UnsupportedConstruct(m) if m.contains("8-constraint split")),
        "typed 8-constraint refusal expected, got {err:?}"
    );

    // Act/Assert 2: a merged goal with 9 conjuncts refuses — phase ordering
    // must ride precondition chains, never goal conjuncts.
    let mut fat_goal = resurface();
    let goal_atom = fat_goal.problem.goal[0].clone();
    for _ in 0..9 {
        fat_goal.problem.goal.push(goal_atom.clone());
    }
    let err = verify_eight_constraint_split(&fat_goal).expect_err("9 goal conjuncts must refuse");
    assert!(
        matches!(&err, CngRefusal::UnsupportedConstruct(m) if m.contains("goal")),
        "typed goal-bound refusal expected, got {err:?}"
    );
});

test!(
    case_study_instance_data_passes_the_soc2_shapes_and_a_mutant_fails,
    {
        // Arrange: the public-vocabulary case-study instance data and the
        // generated soc2 shapes, validated through the SAME generic
        // shape-driven queries every other shape law in this crate uses.
        let instance = fs::read_to_string(soc2_fixture_dir().join("arclight-case-study.ttl"))
            .expect("case-study fixture reads");
        let shapes_path =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("shapes/soc2-shapes.ttl");
        let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");

        // Act/Assert: the shipped instance data is violation-free.
        let violations =
            shape_violations(&instance, &shapes_path, &queries).expect("shape run evaluates");
        assert!(
            violations.is_empty(),
            "shipped case-study data must satisfy the soc2 shapes: {violations:?}"
        );

        // A control point stripped of its notation handle must be caught by
        // the missing-fields shape query — the shapes bite, they are not
        // inert.
        let mutant = instance.replace("skos:notation \"CTRL-DATA-CLASSIFICATION\" ;\n    ", "");
        assert_ne!(mutant, instance, "mutation must apply");
        let violations =
            shape_violations(&mutant, &shapes_path, &queries).expect("mutant shape run evaluates");
        assert!(
            !violations.is_empty(),
            "a control point without skos:notation must violate Soc2ControlPointShape \
             (and Soc2ConceptShape)"
        );
    }
);

test!(no_action_effect_ever_asserts_compliance_or_opinion, {
    // Arrange: the real merged surface, from real fixtures — no PDDL is
    // authored in this file.
    let artifacts = import_artifacts(&soc2_fixture_dir()).expect("fixtures admit");
    let (_, surface) = generate_plan(&artifacts).expect("cycle plan exists");
    let resurface = || crate::pipeline::AdmittedSurface {
        domain: surface.domain.clone(),
        problem: surface.problem.clone(),
        action_sources: surface.action_sources.clone(),
    };

    // Act/Assert 1: the compliance-overclaim fence holds over the REAL,
    // shipped 30-action domain — the terminal goal atom is
    // evidence-bundle-complete, nothing else, and no action anywhere
    // effects a "compliant" or "opinion" atom.
    verify_no_compliance_or_opinion_effects(&surface)
        .expect("shipped domain must satisfy the compliance-overclaim fence");

    // Act/Assert 2: an action mutated to add a "...compliant..." effect
    // atom refuses typed — the fence is mechanical, not decorative.
    let mut compliant_mutant = resurface();
    compliant_mutant.domain.actions[0].add_effects[0].pred = "audit-compliant".to_string();
    let err = verify_no_compliance_or_opinion_effects(&compliant_mutant)
        .expect_err("a 'compliant' effect atom must refuse");
    assert!(
        matches!(&err, CngRefusal::UnsupportedConstruct(m) if m.contains("compliance-overclaim fence")),
        "typed fence refusal expected, got {err:?}"
    );

    // Act/Assert 3: an action mutated to add an "...opinion..." effect atom
    // refuses typed too — both forbidden substrings are checked, not just
    // one.
    let mut opinion_mutant = resurface();
    opinion_mutant.domain.actions[29].add_effects[0].pred = "auditor-opinion-issued".to_string();
    let err = verify_no_compliance_or_opinion_effects(&opinion_mutant)
        .expect_err("an 'opinion' effect atom must refuse");
    assert!(
        matches!(&err, CngRefusal::UnsupportedConstruct(m) if m.contains("compliance-overclaim fence")),
        "typed fence refusal expected, got {err:?}"
    );
});

test!(
    soc2_evidence_metrics_measure_the_shipped_case_study_instance_data,
    {
        // Arrange: the on-disk metric-soc2-*.rq queries (v26.7.12/13 Stage
        // 2) over the SAME shipped Arclight Cloud Platform case-study
        // instance data the SHACL test above validates.
        let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
        let instance_path = soc2_fixture_dir().join("arclight-case-study.ttl");

        // Act: the real chain — Turtle parse + 3 measured SELECT counts +
        // one Rust-computed DERIVED_ARITHMETIC ratio.
        let metrics =
            compute_evidence_metrics(&instance_path, &queries).expect("evidence metrics compute");

        // Assert: measured against the shipped fixture's actual content (16
        // documented control points -- the Fortune-5 rescale's 5-TSC-category
        // scope, up from Solace Cloud's 3; the Exception Identification and
        // Management Response & Remediation phases each generate exactly
        // one deliverable entity for this engagement, unaffected by the
        // control-point count since those are per-phase deliverables, not
        // per-control).
        assert_eq!(
            metrics.measurement_class,
            Soc2EvidenceMetrics::MEASUREMENT_CLASS
        );
        assert_eq!(
            metrics.evidenced_controls, 16,
            "3 Security + 1 CUEC (Sequoia carve-out) + 3 Availability + 3 Confidentiality \
             + 3 Processing Integrity + 3 Privacy control points"
        );
        assert_eq!(
            metrics.exception_register_artifacts, 1,
            "one Exception Register deliverable (DELIV-EXCEPTION-REGISTER)"
        );
        assert_eq!(
            metrics.remediation_log_artifacts, 1,
            "one Management Response & Remediation Log deliverable (DELIV-REMEDIATION-LOG)"
        );
        assert_eq!(
            metrics.derived_exception_register_ratio_class,
            Soc2EvidenceMetrics::DERIVED_ARITHMETIC,
            "the ratio field must be machine-tagged DERIVED_ARITHMETIC, not left implicit"
        );
        // DERIVED_ARITHMETIC: 1 / 16, computed in Rust from the two measured
        // counts above — never a SPARQL aggregate.
        let expected_ratio = 1.0_f64 / 16.0_f64;
        assert!(
            (metrics.derived_exception_register_ratio - expected_ratio).abs() < 1e-12,
            "derived ratio must be exactly exception_register_artifacts / evidenced_controls, \
             got {}",
            metrics.derived_exception_register_ratio
        );

        // A control point without any deliverables in the graph at all
        // (a directory holding zero AUDIT-EXCEPTION-ID / AUDIT-REMEDIATION
        // phase activities) must divide-by-zero SAFELY to 0.0, never
        // silently fabricate a nonzero ratio or panic. Reuse the SHACL
        // test's own control-notation-stripping mutant as a source of a
        // still-parseable but structurally different graph, mutated further
        // to drop every prov:Activity entirely.
        let instance = fs::read_to_string(&instance_path).expect("instance reads");
        let no_controls = instance
            .lines()
            .filter(|line| !line.contains("prov:Plan"))
            .collect::<Vec<_>>()
            .join("\n");
        let scratch_dir = std::env::temp_dir().join("cng-soc2-evidence-metrics-test");
        fs::create_dir_all(&scratch_dir).expect("scratch dir");
        let scratch_path = scratch_dir.join("no-controls.ttl");
        fs::write(&scratch_path, &no_controls).expect("scratch write");
        let empty_metrics = compute_evidence_metrics(&scratch_path, &queries)
            .expect("empty-control metrics compute");
        assert_eq!(empty_metrics.evidenced_controls, 0);
        assert_eq!(
            empty_metrics.derived_exception_register_ratio, 0.0,
            "zero evidenced controls must yield a documented 0.0 ratio, not a panic or a \
             fabricated nonzero value"
        );
    }
);
