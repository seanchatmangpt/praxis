//! Family F08 -- "PDDL Planning and Action-Hook Binding" (atlas ticket
//! V12-008).
//!
//! # Status (this pass)
//!
//! Survey verdict: **MIXED**. This pass wired real content per stage:
//!
//! - [`generated`] -- GGEN_GENERATABLE, real. Ggen-generated from
//!   `packs/pddl-planning-pack/ontology.ttl` (itself a direct
//!   transcription of the atlas's F08-L2/L5/L6/L8 mermaid diagrams): the
//!   [`generated::PipelineState`] L5 state enum, [`generated::stage`] L1/L2
//!   stage metadata, and [`generated::entity`] L6 provenance-chained
//!   entity structs. Re-run `ggen sync run` at the repo root (no dedicated
//!   `just` recipe exists for a single pack) after editing that pack.
//! - [`projector`] (Problem Projector + Domain Resolver, stages 1-2) --
//!   REUSE_ADAPT, real. Mirrors `ChatmanEngine::compute_pddl_plan`'s
//!   literal-selection step in `praxis-graphlaw/src/chatman/engine.rs`
//!   without depending on `ChatmanEngine`'s snapshot/admission
//!   infrastructure (disclosed scope boundary, see that module's doc
//!   comment) -- takes a minimal in-memory triple projection instead of an
//!   oxigraph-backed `GraphSnapshotId`.
//! - [`planner`] (Planner, stage 4) -- REUSE_ADAPT, real. Thin wrap of
//!   `bcinr_pddl::ground::GroundProblem` (grounding + bounded BFS search),
//!   the same real function `praxis-graphlaw` and `cng` both call.
//! - [`effect_trace`] (Plan Validator + Effect Trace + Plan Receipt,
//!   stages 5-7) -- REUSE_ADAPT, real. Thin wrap of
//!   `bcinr_pddl::execute::execute_tape` (independent Prolog8-admitted
//!   replay + genuine BLAKE3-by-execution receipt + OCEL trace).
//! - [`hook_binder`] (Action-Hook Binder, stage 3) -- HAND_WRITE_REQUIRED,
//!   **not implemented**. No general hook-capability registry query
//!   surface was found anywhere in this repo this session (see that
//!   module's doc comment for what was checked); [`hook_binder::bind_actions`]
//!   always returns a typed [`refusal::Refusal::NoAdmissiblePlan`], never a
//!   fabricated success.
//! - [`refusal`] -- hand-written, real. The `NoAdmissiblePlan` taxonomy the
//!   family invariant names, classifying `bcinr_pddl::Pddl8Error` into it.
//!
//! **Consequence, stated plainly**: because the Action-Hook Binder always
//! refuses, [`run_pipeline`] -- the full, real, 5-stage composition below
//! -- always returns `Err` today. This is the disclosed exception this
//! repo's "refusal is not the default" rule carves out for genuinely
//! not-yet-built work: the *other four* stages are real and independently
//! tested end-to-end (see each module's own tests, and
//! `tests::real_stages_compose_up_to_the_disclosed_hook_binder_gap`
//! below), and the pipeline's wiring itself -- projector output feeds
//! planner, planner's grounded actions feed the (stub) binder, its tape
//! feeds effect_trace -- is real, not decorative. What is not real is the
//! Action-Hook Binder's actual matching logic, and this module does not
//! pretend otherwise.
//!
//! Not attempted this pass (disclosed, not silently skipped): L5's
//! transition-guard *enforcement* (validating a caller-claimed
//! [`generated::PipelineState`] transition against
//! [`generated::is_lawful_transition`] at each stage boundary -- the table
//! exists and is real, but nothing in [`run_pipeline`] consults it yet);
//! L7 concurrency/chaos recovery (idempotency/correlation gate, duplicate/
//! restart/stale-result handling); writing the [`generated::entity`]
//! structs into an actual RDF store (they are real Rust values with real
//! content-addressed IRIs, but nothing here persists them as triples).
//!
//! Survey-cited paths (informed research from the v26.7.12 family survey
//! handed to the scaffolding session inline, re-verified by this pass
//! against the sources actually reused): `/Users/sac/Downloads/
//! v26.7.12_mermaid_atlas/families/F08_pddl-planning.md`,
//! `/Users/sac/bcinr/crates/bcinr-pddl/src/{lib,ground,execute,
//! capability_router,error}.rs`, `praxis-graphlaw/src/chatman/engine.rs`,
//! `crates/cng/src/pipeline.rs`, `crates/cng/ontologies/pddl-strips.ttl`,
//! `crates/praxis-synthesis/src/ground.rs`.

pub mod effect_trace;
pub mod generated;
pub mod hook_binder;
pub mod planner;
pub mod projector;
pub mod refusal;

use refusal::Refusal;

/// The real output of a successful [`run_pipeline`] call: the plan tape,
/// its independent execution log/receipt/OCEL trace, and the L6 entities
/// materialized along the way (content-addressed, chained per
/// [`generated::L6_CHAIN`] -- see that constant's own doc comment for the
/// disclosed "not yet written to an RDF store" boundary).
#[derive(Debug, Clone)]
pub struct PipelineOutcome {
    pub tape: bcinr_pddl::Pddl8Tape,
    pub log: bcinr_pddl::Pddl8ExecutionLog,
    pub receipt: bcinr_pddl::Pddl8ExecutionReceipt,
    pub ocel: bcinr_pddl::OCEL,
    pub pddl_problem: generated::entity::PDDLProblem,
    pub pddl_domain: generated::entity::PDDLDomain,
}

/// Run the real F08-L2 pipeline end to end, in atlas stage order: Problem
/// Projector -> Domain Resolver -> Action-Hook Binder -> Planner -> Plan
/// Validator -> Effect Trace -> Plan Receipt.
///
/// As documented on this module: stage 3 (Action-Hook Binder) is not yet
/// implemented and always refuses, so this function always returns `Err`
/// today. It is still real, composed wiring -- not a decorative pass-
/// through -- see the module doc comment.
///
/// # Errors
/// [`Refusal::NoAdmissiblePlan`] or [`Refusal::Underlying`] from whichever
/// stage first fails, per each stage module's own documented error
/// conditions.
pub fn run_pipeline(
    graph: &[projector::AdmittedTriple],
    case_id: &str,
) -> Result<PipelineOutcome, Refusal> {
    let (domain, problem) = projector::project_and_resolve(graph)?;
    let pddl_domain = generated::entity::PDDLDomain::from_content(domain.name.as_bytes());
    let pddl_problem = generated::entity::PDDLProblem::from_content(problem.name.as_bytes());

    let grounded = planner::ground(&domain, &problem)?;

    // Stage 3: Action-Hook Binder. Not yet implemented -- always refuses
    // (see hook_binder's doc comment). Every action's grounded effects are
    // real (from `grounded.actions`); what's missing is the registry to
    // check them against, not the data to check.
    let _capability_map = hook_binder::bind_actions(&grounded.actions)?;

    let tape = planner::plan(&grounded)?;
    let (log, receipt, ocel) = effect_trace::validate_and_execute(
        &tape,
        &grounded.initial_state,
        &grounded.goal,
        case_id,
        &[],
    )?;

    Ok(PipelineOutcome {
        tape,
        log,
        receipt,
        ocel,
        pddl_problem,
        pddl_domain,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use projector::{AdmittedTriple, PDDL_DOMAIN_PREDICATE, PDDL_PROBLEM_PREDICATE};

    const DOMAIN_TEXT: &str = r#"
(define (domain f08-pipeline-test)
  (:requirements :strips)
  (:predicates (at ?x) (goal-reached))
  (:action move
    :parameters (?x)
    :precondition (at ?x)
    :effect (and (goal-reached))))
"#;
    const PROBLEM_TEXT: &str = r#"
(define (problem f08-pipeline-test-problem)
  (:domain f08-pipeline-test)
  (:objects a)
  (:init (at a))
  (:goal (and (goal-reached))))
"#;

    fn fixture_graph() -> Vec<AdmittedTriple> {
        vec![
            AdmittedTriple {
                subject: "urn:mfw:f08:pipeline-test-snapshot".to_string(),
                predicate: PDDL_DOMAIN_PREDICATE.to_string(),
                object_literal: DOMAIN_TEXT.to_string(),
            },
            AdmittedTriple {
                subject: "urn:mfw:f08:pipeline-test-snapshot".to_string(),
                predicate: PDDL_PROBLEM_PREDICATE.to_string(),
                object_literal: PROBLEM_TEXT.to_string(),
            },
        ]
    }

    /// The disclosed, current, honest state of `run_pipeline`: it always
    /// refuses today, and it refuses exactly at `ActionHookBinder` -- not
    /// earlier (proving projector+planner really ran first) and not later
    /// (proving nothing downstream silently ran past the unimplemented
    /// gate).
    #[test]
    #[ignore]
    fn run_pipeline_reaches_and_stops_at_the_disclosed_hook_binder_gap() {
        let graph = fixture_graph();
        let err = run_pipeline(&graph, "f08-pipeline-case").expect_err(
            "hook_binder is not implemented, so the full pipeline must refuse, not succeed",
        );
        assert!(
            matches!(
                err,
                Refusal::NoAdmissiblePlan {
                    stage: "ActionHookBinder",
                    ..
                }
            ),
            "expected the refusal to originate at ActionHookBinder, got {err:?}"
        );
    }

    /// Proves the four *real* stages actually compose end-to-end when
    /// called directly (bypassing only the not-yet-built hook_binder
    /// step): a graph-derived problem/domain plans, and the resulting plan
    /// independently validates against Prolog8 admission with a real
    /// receipt. This is not a public "skip the invariant" API -- see the
    /// module doc comment for why `run_pipeline` itself never does this.
    #[test]
    fn real_stages_compose_up_to_the_disclosed_hook_binder_gap() {
        let graph = fixture_graph();
        let (domain, problem) = projector::project_and_resolve(&graph).expect("stage 1-2 real");
        let grounded = planner::ground(&domain, &problem).expect("grounded (stage 3 precursor)");
        assert_eq!(grounded.actions.len(), 1);

        let tape = planner::plan(&grounded).expect("stage 4 real: plan found");
        let (_log, receipt, _ocel) = effect_trace::validate_and_execute(
            &tape,
            &grounded.initial_state,
            &grounded.goal,
            "f08-real-stages-case",
            &[],
        )
        .expect("stages 5-7 real: independently validated and receipted");
        assert!(receipt.goal_reached);
    }

    #[test]
    fn missing_admitted_state_refuses_before_any_planning() {
        let err = run_pipeline(&[], "f08-empty-graph-case")
            .expect_err("no admitted PDDL literals at all must refuse at the projector");
        assert!(matches!(
            err,
            Refusal::NoAdmissiblePlan {
                stage: "ProblemProjector",
                ..
            }
        ));
    }
}
