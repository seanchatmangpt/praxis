#![cfg(test)]

//! TOGAF ADM full-lifecycle tests (Meridian Global Carrier, v26.7.13
//! increment 1): the 10 chained phase fixtures admit, plan as one 30-step
//! cycle, project hierarchically into 10 phase children by artifact
//! provenance, validate against the POWL structural shape, and replay to a
//! byte-identical digest. The 8-constraint split law is verified
//! mechanically, and its violation refuses typed. Case-study instance data
//! validates against the ea-togaf SHACL shapes (public-vocabulary
//! modeling), with a negative fixture proving the shapes actually bite.
//! All Turtle enters from on-disk fixtures; all SPARQL from the on-disk
//! query set.

use std::fs;

use chicago_tdd_tools::prelude::*;
use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::store::Store;

use super::{togaf_fixture_dir, verify_eight_constraint_split, ADM_PHASES};
use crate::bench::dispatch::shape_violations;
use crate::bench::templates::QuerySet;
use crate::pipeline::{generate_plan, hierarchical_projection, import_artifacts};
use crate::powl::{powl_to_turtle, CngRefusal, Powl};
use crate::shape::validate_powl_store;

const BASE_IRI: &str = "urn:chatman:powl:meridian-adm";

test!(
    full_adm_cycle_plans_projects_validates_and_replays_byte_identically,
    {
        // Arrange: admit the generated fixture directory (10 domain fragments +
        // 1 problem fragment + the non-PDDL case-study instance file, which
        // contributes no fragments).
        let dir = togaf_fixture_dir();

        // Act: the real manufacture chain, twice (replay determinism).
        let run = || {
            let artifacts = import_artifacts(&dir).expect("fixtures admit");
            let (tape, surface) = generate_plan(&artifacts).expect("cycle plan exists");
            verify_eight_constraint_split(&surface).expect("8-constraint split holds");
            let (powl, phase_sources) =
                hierarchical_projection(&tape, &surface).expect("hierarchical projection");
            let ttl = powl_to_turtle(&powl, BASE_IRI, Some("urn:chatman:plan:meridian-adm"));
            (tape, powl, phase_sources, ttl)
        };
        let (tape, powl, phase_sources, ttl) = run();
        let (_, _, _, ttl_replay) = run();

        // Assert: the full cycle is a 30-step linear plan (3 actions × 10
        // phases), forced through every phase by the precondition chain.
        assert_eq!(tape.ops.len(), 30, "3 actions per phase × 10 phases");
        let labels: Vec<&str> = tape.ops.iter().map(|op| op.label.as_str()).collect();
        assert_eq!(labels[0], "establish-ea-capability(meridian)");
        assert_eq!(labels[29], "produce-requirements-register(meridian)");

        // The hierarchical projection groups the plan into exactly 10 phase
        // children — one per contributing fixture artifact, in ADM order.
        assert_eq!(
            phase_sources.len(),
            10,
            "one provenance source per ADM phase"
        );
        let Powl::PartialOrder { children, .. } = &powl else {
            panic!("root must be a partial order over the 10 phases");
        };
        assert_eq!(children.len(), 10, "one POWL child per ADM phase");
        assert_eq!(ADM_PHASES.len(), 10);

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

test!(
    eight_constraint_split_violations_refuse_typed_not_silently,
    {
        // Arrange: the real merged surface, then concentrate constraints past
        // the PDDL8 bound in each of the two places the law guards.
        let artifacts = import_artifacts(&togaf_fixture_dir()).expect("fixtures admit");
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
        let err =
            verify_eight_constraint_split(&fat_goal).expect_err("9 goal conjuncts must refuse");
        assert!(
            matches!(&err, CngRefusal::UnsupportedConstruct(m) if m.contains("goal")),
            "typed goal-bound refusal expected, got {err:?}"
        );
    }
);

test!(
    case_study_instance_data_passes_the_ea_shapes_and_a_mutant_fails,
    {
        // Arrange: the public-vocabulary case-study instance data and the
        // generated ea-togaf shapes, validated through the SAME generic
        // shape-driven queries every other shape law in this crate uses.
        let instance = fs::read_to_string(togaf_fixture_dir().join("meridian-case-study.ttl"))
            .expect("case-study fixture reads");
        let shapes_path =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("shapes/ea-togaf-shapes.ttl");
        let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");

        // Act/Assert: the shipped instance data is violation-free.
        let violations =
            shape_violations(&instance, &shapes_path, &queries).expect("shape run evaluates");
        assert!(
            violations.is_empty(),
            "shipped case-study data must satisfy the ea shapes: {violations:?}"
        );

        // A capability stripped of its notation handle must be caught by the
        // missing-fields shape query — the shapes bite, they are not inert.
        let mutant = instance.replace("skos:notation \"CAP-FINOPS\" ;\n    ", "");
        assert_ne!(mutant, instance, "mutation must apply");
        let violations =
            shape_violations(&mutant, &shapes_path, &queries).expect("mutant shape run evaluates");
        assert!(
            !violations.is_empty(),
            "a capability without skos:notation must violate EaCapabilityShape"
        );
    }
);

test!(overgrounded_cycle_variant_refuses_typed_at_grounding, {
    // Arrange: same 10 domain fragments, but a problem variant whose object
    // roster pushes the ground-action count past PDDL8_MAX_GROUND (4096):
    // 30 one-parameter actions × 140 objects = 4200 ground actions. Built
    // by widening the on-disk problem fixture's object list — no PDDL is
    // authored in Rust.
    let dir = togaf_fixture_dir();
    let scratch = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests/togaf-overground")
        .join(format!("{}", std::process::id()));
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(&scratch).expect("scratch dir");
    for (file, _) in ADM_PHASES {
        fs::copy(dir.join(file), scratch.join(file)).expect("copy phase fragment");
    }
    let problem = fs::read_to_string(dir.join("adm-cycle-problem.ttl")).expect("problem reads");
    let wide_objects = (0..140)
        .map(|i| format!("m{i}"))
        .collect::<Vec<_>>()
        .join(" ");
    let overground = problem.replace("(:objects meridian)", &format!("(:objects {wide_objects})"));
    assert_ne!(overground, problem, "widening must apply");
    fs::write(scratch.join("adm-cycle-problem.ttl"), overground).expect("write variant");

    // Act
    let artifacts = import_artifacts(&scratch).expect("variant admits");
    // `(Pddl8Tape, AdmittedSurface)` has no Debug impl, so unpack the
    // refusal by match rather than expect_err.
    let err = match generate_plan(&artifacts) {
        Ok(_) => panic!("4200 ground actions must refuse"),
        Err(e) => e,
    };

    // Assert: typed grounding refusal, not a panic or a silent empty plan.
    assert!(
        matches!(&err, CngRefusal::UnsupportedConstruct(m) if m.contains("grounding failed")),
        "typed grounding refusal expected, got {err:?}"
    );
});
