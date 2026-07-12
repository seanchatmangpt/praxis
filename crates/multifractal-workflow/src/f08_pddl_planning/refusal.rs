//! F08's typed refusal taxonomy.
//!
//! The atlas's F08-L2/L4 diagrams name exactly one refusal type,
//! `NoAdmissiblePlan`, raised at three component boundaries (Domain
//! Resolver, Action-Hook Binder, Plan Receipt -- see
//! [`super::generated::stage::refusal_stages`]). This module keeps that
//! single named refusal as the outward-facing variant while preserving the
//! underlying `bcinr_pddl::Pddl8Error` (or a stage name alone, for logic
//! that has no such error to wrap) as context, so a caller can match on
//! `Refusal::NoAdmissiblePlan` per the family invariant without losing the
//! debugging detail this repo's no-silent-defaults discipline requires.
//!
//! `Pddl8Error::GoalNotReached` is classified below for forward
//! compatibility, but as of `bcinr-pddl` v26.6.26 it is dead code -- never
//! actually constructed anywhere in that crate (confirmed by grep this
//! session); `execute::execute_tape` reports an unreached goal via
//! `receipt.goal_reached == false` inside an `Ok`, not via `Err`. See
//! [`super::effect_trace`] for where that field is actually checked.

use std::fmt;

use bcinr_pddl::Pddl8Error;

/// F08's refusal taxonomy. `NoAdmissiblePlan` is the only variant the atlas
/// names by name; `stage` records which of the 7 [`super::generated::stage::Stage`]s
/// raised it so a caller can correlate against
/// [`super::generated::stage::refusal_stages`] or
/// [`super::generated::REFUSAL_SOURCES`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// No admissible plan: derived from a [`Pddl8Error`] that means the
    /// planning surface itself is infeasible (`EmptyGrounding`,
    /// `NoAdmittedPlan`, `GoalNotReached`), or from an F08-local admission
    /// check with no underlying `Pddl8Error` (e.g. a missing PDDL literal
    /// on the admitted graph, or the Action-Hook Binder's not-yet-built
    /// matching step -- see [`super::hook_binder`]).
    NoAdmissiblePlan { stage: &'static str, reason: String },
    /// A [`Pddl8Error`] that is not itself a "no plan" condition (parse
    /// error, bound exceeded, admission load error, step denied, receipt
    /// integrity failure, invalid case id) -- surfaced verbatim rather than
    /// folded into `NoAdmissiblePlan`, since conflating "malformed input"
    /// with "search found nothing" would lose real debugging signal.
    Underlying {
        stage: &'static str,
        source: Pddl8Error,
    },
}

impl Refusal {
    /// Classify a [`Pddl8Error`] from pipeline stage `stage` into this
    /// taxonomy: the two "no plan is reachable" variants become
    /// [`Refusal::NoAdmissiblePlan`] (matching the family invariant's exact
    /// name); everything else is preserved verbatim as
    /// [`Refusal::Underlying`].
    #[must_use]
    pub fn from_pddl8(stage: &'static str, source: Pddl8Error) -> Self {
        match &source {
            Pddl8Error::EmptyGrounding
            | Pddl8Error::NoAdmittedPlan
            | Pddl8Error::GoalNotReached => Refusal::NoAdmissiblePlan {
                stage,
                reason: source.to_string(),
            },
            _ => Refusal::Underlying { stage, source },
        }
    }

    /// The stage that raised this refusal (one of
    /// [`super::generated::stage::STAGES`]' labels).
    #[must_use]
    pub fn stage(&self) -> &'static str {
        match self {
            Refusal::NoAdmissiblePlan { stage, .. } | Refusal::Underlying { stage, .. } => stage,
        }
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::NoAdmissiblePlan { stage, reason } => {
                write!(f, "NoAdmissiblePlan at {stage}: {reason}")
            }
            Refusal::Underlying { stage, source } => {
                write!(f, "{stage} refused: {source}")
            }
        }
    }
}

impl std::error::Error for Refusal {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_grounding_classifies_as_no_admissible_plan() {
        let r = Refusal::from_pddl8("Planner", Pddl8Error::EmptyGrounding);
        assert!(matches!(
            r,
            Refusal::NoAdmissiblePlan {
                stage: "Planner",
                ..
            }
        ));
    }

    #[test]
    fn no_admitted_plan_classifies_as_no_admissible_plan() {
        let r = Refusal::from_pddl8("Planner", Pddl8Error::NoAdmittedPlan);
        assert!(matches!(
            r,
            Refusal::NoAdmissiblePlan {
                stage: "Planner",
                ..
            }
        ));
    }

    #[test]
    fn goal_not_reached_classifies_as_no_admissible_plan() {
        let r = Refusal::from_pddl8("PlanValidator", Pddl8Error::GoalNotReached);
        assert!(matches!(
            r,
            Refusal::NoAdmissiblePlan {
                stage: "PlanValidator",
                ..
            }
        ));
    }

    #[test]
    fn parse_error_is_preserved_as_underlying_not_folded_into_no_admissible_plan() {
        let r = Refusal::from_pddl8(
            "ProblemProjector",
            Pddl8Error::ParseError("bad token".into()),
        );
        assert!(matches!(
            r,
            Refusal::Underlying {
                stage: "ProblemProjector",
                source: Pddl8Error::ParseError(_)
            }
        ));
    }

    #[test]
    fn stage_accessor_matches_constructor_for_both_variants() {
        let a = Refusal::from_pddl8("Planner", Pddl8Error::EmptyGrounding);
        assert_eq!(a.stage(), "Planner");
        let b = Refusal::from_pddl8("DomainResolver", Pddl8Error::ParseError("x".into()));
        assert_eq!(b.stage(), "DomainResolver");
    }

    #[test]
    fn display_names_no_admissible_plan_explicitly() {
        let r = Refusal::NoAdmissiblePlan {
            stage: "ActionHookBinder",
            reason: "no hook registered for effect predicate 'painted'".to_string(),
        };
        let s = r.to_string();
        assert!(s.contains("NoAdmissiblePlan"));
        assert!(s.contains("ActionHookBinder"));
    }
}
