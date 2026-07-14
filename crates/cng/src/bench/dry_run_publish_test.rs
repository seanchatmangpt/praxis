#![cfg(test)]

//! Dry-run publish gate full-lifecycle structural tests (Kestrel Toolkit
//! release candidate, v26.7.13): the 6 chained gate fixtures admit, plan as
//! one 18-step cycle, project hierarchically into 6 gate children by artifact
//! provenance, validate against the POWL structural shape, and replay to a
//! byte-identical digest. The gate-completion chain, the init/goal atoms, and
//! the DRY-RUN-OVERCLAIM FENCE are verified mechanically, and each is
//! falsified by an adversarial mutant that must refuse typed.
//!
//! HONEST SCOPE: these tests validate the pack's PDDL MODEL (domain structure
//! and fence). They do NOT execute a real `cargo publish --dry-run`, do NOT
//! inspect the real workspace, and do NOT move the v26.7.13 Dry-Run Publish
//! Definition of Done off REFUSED. See `dry_run_publish.rs`'s module doc for
//! the full disclosure.
//!
//! `no_action_effect_ever_names_an_external_mutation` is the
//! DRY-RUN-OVERCLAIM FENCE's structural enforcement point: it greps the
//! parsed, merged domain's action effects programmatically for
//! `published`/`crates-io-uploaded`/`release-complete`, on both the real
//! fixture set (must be clean) and adversarial mutants (must refuse typed),
//! exactly as `soc2_test.rs` greps for `compliant`/`opinion`.

use chicago_tdd_tools::prelude::*;
use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::store::Store;

use super::{
    dry_run_publish_fixture_dir, validate_dry_run_publish_domain, verify_gate_completion_chain,
    verify_init_and_goal, verify_no_external_mutation_effects, DryRunPublishReport, DRY_RUN_PHASES,
};
use crate::pipeline::{generate_plan, hierarchical_projection, import_artifacts, AdmittedSurface};
use crate::powl::{powl_to_turtle, CngRefusal, Powl};
use crate::shape::validate_powl_store;

const BASE_IRI: &str = "urn:chatman:powl:kestrel-dry-run-publish";
const PLAN_IRI: &str = "urn:chatman:plan:kestrel-dry-run-publish";

test!(
    full_dry_run_publish_cycle_plans_projects_validates_and_replays_byte_identically,
    {
        // Arrange: admit the generated fixture directory (6 domain fragments +
        // 1 problem fragment + the non-PDDL case-study instance file, which
        // contributes no fragments).
        let dir = dry_run_publish_fixture_dir();

        // Act: the real manufacture chain, twice (replay determinism).
        let run = || {
            let artifacts = import_artifacts(&dir).expect("fixtures admit");
            let (tape, surface) = generate_plan(&artifacts).expect("cycle plan exists");
            let report =
                validate_dry_run_publish_domain(&surface).expect("model structure validates");
            let (powl, phase_sources) =
                hierarchical_projection(&tape, &surface).expect("hierarchical projection");
            let ttl = powl_to_turtle(&powl, BASE_IRI, Some(PLAN_IRI));
            (tape, powl, phase_sources, report, ttl)
        };
        let (tape, powl, phase_sources, report, ttl) = run();
        let (_, _, _, _, ttl_replay) = run();

        // Assert: the full cycle is an 18-step linear plan (3 actions × 6
        // gate phases), forced through every phase by the precondition chain.
        assert_eq!(tape.ops.len(), 18, "3 actions per phase × 6 gate phases");
        let labels: Vec<&str> = tape.ops.iter().map(|op| op.label.as_str()).collect();
        assert_eq!(labels[0], "define-publish-set-and-version(kestrel)");
        assert_eq!(
            labels[17],
            "confirm-byte-identical-replay-no-mutation(kestrel)"
        );

        // The hierarchical projection groups the plan into exactly 6 gate
        // children — one per contributing fixture artifact, in DoD order.
        assert_eq!(
            phase_sources.len(),
            6,
            "one provenance source per dry-run gate phase"
        );
        let Powl::PartialOrder { children, .. } = &powl else {
            panic!("root must be a partial order over the 6 gate phases");
        };
        assert_eq!(children.len(), 6, "one POWL child per dry-run gate phase");
        assert_eq!(DRY_RUN_PHASES.len(), 6);

        // The structural report summarizes the validated model.
        assert_eq!(
            report.validation_class,
            DryRunPublishReport::VALIDATION_CLASS
        );
        assert_eq!(report.gate_phases, 6);
        assert_eq!(report.actions_total, 18);
        assert_eq!(report.actions_per_phase, 3);
        assert_eq!(report.gate_chain_length, 7);
        assert_eq!(report.init_atom, "scope-engaged");
        assert_eq!(report.goal_atom, "dry-run-verified");
        assert_eq!(report.release_candidate, "kestrel");
        assert_eq!(
            report.external_mutation_effects, 0,
            "a validated model has zero external-mutation effects"
        );

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
    cycle_problem_init_is_scope_engaged_and_goal_is_dry_run_verified,
    {
        // Arrange: the real merged surface from real fixtures.
        let artifacts = import_artifacts(&dry_run_publish_fixture_dir()).expect("fixtures admit");
        let (_, surface) = generate_plan(&artifacts).expect("cycle plan exists");

        // Assert: the merged problem's init is exactly (scope-engaged kestrel)
        // and its goal is exactly (dry-run-verified kestrel).
        assert_eq!(surface.problem.init.len(), 1, "one init atom");
        assert_eq!(surface.problem.init[0].pred, "scope-engaged");
        assert_eq!(surface.problem.init[0].args, vec!["kestrel".to_string()]);
        assert_eq!(surface.problem.goal.len(), 1, "one goal atom");
        assert_eq!(surface.problem.goal[0].pred, "dry-run-verified");
        assert_eq!(surface.problem.goal[0].args, vec!["kestrel".to_string()]);
        verify_init_and_goal(&surface).expect("init/goal law holds on the shipped fixtures");

        // Fixture correspondence: every declared gate phase fixture exists on
        // disk under the fixture directory.
        for (file, _notation) in DRY_RUN_PHASES {
            let path = dry_run_publish_fixture_dir().join(file);
            assert!(path.exists(), "declared phase fixture {file} must exist");
        }
    }
);

test!(no_action_effect_ever_names_an_external_mutation, {
    // Arrange: the real merged surface, from real fixtures — no PDDL is
    // authored in this file.
    let artifacts = import_artifacts(&dry_run_publish_fixture_dir()).expect("fixtures admit");
    let (_, surface) = generate_plan(&artifacts).expect("cycle plan exists");
    // AdmittedSurface does not derive Clone; rebuild one from its Clone-able
    // parts for each mutation (same pattern as soc2_test.rs).
    let resurface = || AdmittedSurface {
        domain: surface.domain.clone(),
        problem: surface.problem.clone(),
        action_sources: surface.action_sources.clone(),
    };

    // Act/Assert 1: the DRY-RUN-OVERCLAIM FENCE holds over the REAL, shipped
    // 18-action domain — the terminal goal atom is dry-run-verified, and no
    // action anywhere effects a published/crates-io-uploaded/release-complete
    // atom.
    verify_no_external_mutation_effects(&surface)
        .expect("shipped domain must satisfy the dry-run-overclaim fence");

    // Act/Assert 2 (MUTANT): an action mutated to add a "published" effect
    // atom refuses typed — the fence is mechanical, not decorative.
    let mut published_mutant = resurface();
    published_mutant.domain.actions[0].add_effects[0].pred = "published".to_string();
    let err = verify_no_external_mutation_effects(&published_mutant)
        .expect_err("a 'published' effect atom must refuse");
    assert!(
        matches!(&err, CngRefusal::UnsupportedConstruct(m) if m.contains("dry-run-overclaim fence")),
        "typed fence refusal expected, got {err:?}"
    );

    // Act/Assert 3 (MUTANT): an action mutated to add a "release-complete"
    // effect atom refuses typed too — all three forbidden substrings bite.
    let mut release_mutant = resurface();
    let last = release_mutant.domain.actions.len() - 1;
    release_mutant.domain.actions[last].add_effects[0].pred = "release-complete".to_string();
    let err = verify_no_external_mutation_effects(&release_mutant)
        .expect_err("a 'release-complete' effect atom must refuse");
    assert!(
        matches!(&err, CngRefusal::UnsupportedConstruct(m) if m.contains("dry-run-overclaim fence")),
        "typed fence refusal expected, got {err:?}"
    );
});

test!(
    structural_mutants_break_the_chain_or_drop_the_goal_and_refuse_typed,
    {
        // Arrange: the real merged surface, from real fixtures.
        let artifacts = import_artifacts(&dry_run_publish_fixture_dir()).expect("fixtures admit");
        let (_, surface) = generate_plan(&artifacts).expect("cycle plan exists");
        let resurface = || AdmittedSurface {
            domain: surface.domain.clone(),
            problem: surface.problem.clone(),
            action_sources: surface.action_sources.clone(),
        };

        // MUTANT A: break the phase chain by rewiring the phase-1 -> phase-2
        // bridge. `scope-complete` is a precondition of exactly one action
        // (pin-ggen-version, the Deterministic Generation entry); rewire it to a
        // dangling atom, so `scope-complete` has zero consumers and the chain law
        // refuses.
        let mut broken_chain = resurface();
        for action in &mut broken_chain.domain.actions {
            for pre in &mut action.preconditions {
                if pre.pred == "scope-complete" {
                    pre.pred = "scope-dangling".to_string();
                }
            }
        }
        let err = verify_gate_completion_chain(&broken_chain)
            .expect_err("a broken phase-completion chain must refuse");
        assert!(
            matches!(&err, CngRefusal::UnsupportedConstruct(m) if m.contains("gate chain broken")),
            "typed gate-chain refusal expected, got {err:?}"
        );
        // The whole-domain orchestrator refuses the same mutant.
        assert!(
            validate_dry_run_publish_domain(&broken_chain).is_err(),
            "the orchestrator must reject a broken chain"
        );

        // MUTANT B: drop the terminal goal atom entirely — the plan would have no
        // goal, so the init/goal law refuses.
        let mut no_goal = resurface();
        no_goal.problem.goal.clear();
        let err = verify_init_and_goal(&no_goal).expect_err("a dropped goal atom must refuse");
        assert!(
            matches!(&err, CngRefusal::UnsupportedConstruct(m) if m.contains("goal law broken")),
            "typed goal-law refusal expected, got {err:?}"
        );
        assert!(
            validate_dry_run_publish_domain(&no_goal).is_err(),
            "the orchestrator must reject a missing goal atom"
        );

        // MUTANT C: change the release-candidate object on the sole init atom —
        // the init/goal law refuses (the cycle no longer plans over `kestrel`).
        let mut wrong_init = resurface();
        wrong_init.problem.init[0].args = vec!["not-kestrel".to_string()];
        let err = verify_init_and_goal(&wrong_init).expect_err("a wrong init object must refuse");
        assert!(
            matches!(&err, CngRefusal::UnsupportedConstruct(m) if m.contains("init law broken")),
            "typed init-law refusal expected, got {err:?}"
        );
    }
);
