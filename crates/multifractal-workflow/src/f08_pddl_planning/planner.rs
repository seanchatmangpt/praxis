//! Planner (F08-L2 stage 4): REUSE_ADAPT, thin wrap of
//! `bcinr_pddl::ground::GroundProblem` -- the same grounder + bounded BFS
//! forward-search `praxis-graphlaw/src/chatman/engine.rs::compute_pddl_plan`
//! and `crates/cng/src/pipeline.rs` both call for the identical
//! ground-then-plan step. No search logic is reimplemented here.
//!
//! `ground()` is exposed separately from `plan()` (rather than only
//! offering a combined "parse and search" entry point) because the Action-
//! Hook Binder (F08-L2 stage 3, [`super::hook_binder`]) needs the grounded
//! action list -- `GroundProblem::actions` -- *before* search runs, per the
//! atlas's stage order (Action-Hook Binder precedes Planner). See
//! [`super::run_pipeline`] for how the two compose.

use bcinr_pddl::ground::GroundProblem;
use bcinr_pddl::{Pddl8Domain, Pddl8Problem, Pddl8Tape};

use super::refusal::Refusal;

/// Ground `domain`/`problem` into a [`GroundProblem`] (action instantiation
/// + precondition index), without running search.
///
/// # Errors
/// [`Refusal::NoAdmissiblePlan`] if grounding yields zero applicable
/// actions ([`bcinr_pddl::Pddl8Error::EmptyGrounding`]); `Refusal::Underlying`
/// for a bound-exceeded/unknown-predicate structural failure.
///
/// # Complexity
/// Bounded by `PDDL8_MAX_GROUND` (`bcinr_pddl::ground` grounds one
/// candidate action per type-compatible parameter binding, capped).
pub fn ground(domain: &Pddl8Domain, problem: &Pddl8Problem) -> Result<GroundProblem, Refusal> {
    GroundProblem::build(domain, problem, None).map_err(|e| Refusal::from_pddl8("Planner", e))
}

/// Run bounded BFS forward search over an already-grounded problem.
///
/// # Errors
/// [`Refusal::NoAdmissiblePlan`] if search exhausts the bounded depth
/// without reaching the goal ([`bcinr_pddl::Pddl8Error::NoAdmittedPlan`]).
///
/// # Complexity
/// Bounded BFS, `PDDL8_MAX_PLAN_DEPTH` deep, over the grounded action set.
pub fn plan(ground: &GroundProblem) -> Result<Pddl8Tape, Refusal> {
    ground
        .find_plan()
        .into_result()
        .map_err(|e| Refusal::from_pddl8("Planner", e.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bcinr_pddl::parse::{domain_from_pddl, problem_from_pddl};

    const DOMAIN_TEXT: &str = r#"
(define (domain f08-planner-test)
  (:requirements :strips)
  (:predicates (at ?x) (goal-reached))
  (:action move
    :parameters (?x)
    :precondition (at ?x)
    :effect (and (goal-reached))))
"#;
    const PROBLEM_TEXT: &str = r#"
(define (problem f08-planner-test-problem)
  (:domain f08-planner-test)
  (:objects a)
  (:init (at a))
  (:goal (and (goal-reached))))
"#;
    const UNSOLVABLE_PROBLEM_TEXT: &str = r#"
(define (problem f08-planner-test-unsolvable)
  (:domain f08-planner-test)
  (:objects a)
  (:init )
  (:goal (and (goal-reached))))
"#;

    #[test]
    fn ground_then_plan_finds_a_real_one_step_plan() {
        let domain = domain_from_pddl(DOMAIN_TEXT).expect("domain parses");
        let problem = problem_from_pddl(PROBLEM_TEXT).expect("problem parses");
        let g = ground(&domain, &problem).expect("grounds to >=1 action");
        assert_eq!(g.actions.len(), 1, "exactly one grounding of move(a)");
        let tape = plan(&g).expect("plan is found");
        assert_eq!(tape.ops.len(), 1, "one-step plan: move(a)");
    }

    #[test]
    fn unreachable_goal_refuses_no_admissible_plan() {
        let domain = domain_from_pddl(DOMAIN_TEXT).expect("domain parses");
        let problem = problem_from_pddl(UNSOLVABLE_PROBLEM_TEXT).expect("problem parses");
        let g = ground(&domain, &problem).expect("still grounds (action exists, just unreachable)");
        let err = plan(&g).expect_err("goal is unreachable from empty init");
        assert!(matches!(
            err,
            Refusal::NoAdmissiblePlan {
                stage: "Planner",
                ..
            }
        ));
    }
}
