//! Plan Validator + Effect Trace + Plan Receipt (F08-L2 stages 5-7):
//! REUSE_ADAPT, thin wrap of `bcinr_pddl::execute::execute_tape`.
//!
//! `execute_tape` independently replays the plan tape through a Prolog8
//! admission gate (not just re-checking the planner's own bookkeeping),
//! producing: a per-step [`bcinr_pddl::Pddl8ExecutionLog`] (the atlas's
//! Effect Trace), a chained BLAKE3 [`bcinr_pddl::Pddl8ExecutionReceipt`]
//! (the atlas's Plan Receipt), and an [`bcinr_pddl::OCEL`] trace. This is
//! the family invariant's "independent conformance check, not just planner
//! success" for real: replay is against Prolog8 admission, not just a
//! replay of the planner's own bookkeeping.
//!
//! **Correction made while wiring this** (caught by this module's own
//! test, not assumed): `bcinr_pddl::execute::execute_tape` does *not*
//! return `Err` when the goal is unreached -- it returns `Ok` with
//! `goal_reached: false` in the receipt/log, and its `Pddl8Error::
//! GoalNotReached` variant is dead code (defined in
//! `/Users/sac/bcinr/crates/bcinr-pddl/src/error.rs` but never constructed
//! anywhere in that crate as of v26.6.26 -- confirmed by grep this
//! session). So this wrapper checks `receipt.goal_reached` itself and
//! turns an unreached goal into [`Refusal::NoAdmissiblePlan`] -- the
//! "independent conformance check" the family invariant requires would
//! not exist without this, since a bare `execute_tape` call alone lets an
//! unreached goal look like `Ok`.
//!
//! The atlas's separate "ValidationTrace" entity (between Plan and
//! EffectTrace in the L6 chain) is not a distinct artifact `bcinr_pddl`
//! produces; this module's caller is responsible for deciding whether to
//! also materialize a `generated::entity::ValidationTrace` from this
//! call's `Ok`/`Err` outcome (see `super::run_pipeline`).

use std::collections::BTreeSet;

use bcinr_pddl::execute::execute_tape;
use bcinr_pddl::{Pddl8ExecutionLog, Pddl8ExecutionReceipt, Pddl8GroundAtom, Pddl8Tape, OCEL};

use super::refusal::Refusal;

/// Replay `tape` from `initial_state` through Prolog8 admission, checking
/// it reaches every atom in `goal`. `case_id` identifies this execution in
/// the returned OCEL log (must be 1-64 chars, validated by `execute_tape`
/// itself). `policy_rules` empty means every scheduled op is pre-admitted
/// (matches `ChatmanEngine`'s and `cng`'s own default use of
/// `execute_tape`); non-empty entries are `(head_label, body_labels)`
/// may-fire Horn rules for callers that need a real admission policy.
///
/// # Errors
/// [`Refusal::NoAdmissiblePlan`] if execution completes but the goal is
/// not reached (checked against `receipt.goal_reached`; see this module's
/// doc comment for why `execute_tape` alone does not catch this).
/// `Refusal::Underlying` for a denied step, invalid case id, or receipt-
/// integrity failure raised by `execute_tape` itself.
///
/// # Complexity
/// O(tape ops) for replay + receipt chaining; `case_id` validation is O(len).
pub fn validate_and_execute(
    tape: &Pddl8Tape,
    initial_state: &BTreeSet<Pddl8GroundAtom>,
    goal: &[Pddl8GroundAtom],
    case_id: &str,
    policy_rules: &[(&str, Vec<&str>)],
) -> Result<(Pddl8ExecutionLog, Pddl8ExecutionReceipt, OCEL), Refusal> {
    let (log, receipt, ocel) = execute_tape(tape, initial_state, goal, case_id, policy_rules)
        .map_err(|e| Refusal::from_pddl8("PlanValidator", e))?;
    if !receipt.goal_reached {
        return Err(Refusal::NoAdmissiblePlan {
            stage: "PlanValidator",
            reason: format!(
                "execution completed ({} step(s) admitted) but the goal was not reached; \
                 receipt chain_hash={}",
                receipt.step_count, receipt.chain_hash
            ),
        });
    }
    Ok((log, receipt, ocel))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::f08_pddl_planning::planner::{ground, plan};
    use bcinr_pddl::parse::{domain_from_pddl, problem_from_pddl};

    const DOMAIN_TEXT: &str = r#"
(define (domain f08-effect-trace-test)
  (:requirements :strips)
  (:predicates (at ?x) (goal-reached))
  (:action move
    :parameters (?x)
    :precondition (at ?x)
    :effect (and (goal-reached))))
"#;
    const PROBLEM_TEXT: &str = r#"
(define (problem f08-effect-trace-test-problem)
  (:domain f08-effect-trace-test)
  (:objects a)
  (:init (at a))
  (:goal (and (goal-reached))))
"#;

    #[test]
    fn a_real_plan_replays_and_reaches_the_goal_with_a_receipt() {
        let domain = domain_from_pddl(DOMAIN_TEXT).expect("domain parses");
        let problem = problem_from_pddl(PROBLEM_TEXT).expect("problem parses");
        let g = ground(&domain, &problem).expect("grounds");
        let tape = plan(&g).expect("plan found");

        let (log, receipt, ocel) = validate_and_execute(
            &tape,
            &g.initial_state,
            &g.goal,
            "f08-effect-trace-case-1",
            &[],
        )
        .expect("execution reaches the goal and is admitted throughout");

        assert!(
            receipt.goal_reached,
            "receipt must record the goal as reached"
        );
        assert_eq!(log.steps.len(), 1, "one executed step: move(a)");
        assert_eq!(ocel.events.len(), 1, "one OCEL event for the one step");
    }

    #[test]
    fn empty_tape_against_unmet_goal_refuses_no_admissible_plan() {
        let empty_tape = Pddl8Tape { ops: Vec::new() };
        let initial_state: BTreeSet<Pddl8GroundAtom> = BTreeSet::new();
        let goal = vec![Pddl8GroundAtom {
            pred: "goal-reached".to_string(),
            args: Vec::new(),
        }];
        let err = validate_and_execute(&empty_tape, &initial_state, &goal, "f08-empty-case", &[])
            .expect_err("an empty tape cannot reach an unmet goal");
        assert!(matches!(
            err,
            Refusal::NoAdmissiblePlan {
                stage: "PlanValidator",
                ..
            }
        ));
    }

    #[test]
    fn invalid_case_id_is_underlying_not_no_admissible_plan() {
        let empty_tape = Pddl8Tape { ops: Vec::new() };
        let initial_state: BTreeSet<Pddl8GroundAtom> = BTreeSet::new();
        let err = validate_and_execute(&empty_tape, &initial_state, &[], "", &[])
            .expect_err("empty case_id is invalid, not just goal-unreached");
        assert!(matches!(err, Refusal::Underlying { .. }));
    }
}
