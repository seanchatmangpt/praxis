//! Problem Projector + Domain Resolver (F08-L2 stages 1-2): REUSE_ADAPT.
//!
//! Mirrors the literal-selection + parse step
//! `ChatmanEngine::compute_pddl_plan` performs internally in
//! `praxis-graphlaw/src/chatman/engine.rs` (same two well-known predicate
//! IRIs, same `bcinr_pddl::parse::{domain_from_pddl, problem_from_pddl}`
//! calls) -- but deliberately does NOT depend on `ChatmanEngine`'s
//! oxigraph-backed snapshot/admission infrastructure (`GraphSnapshotId`,
//! `AdmissionSpec`, stage sealing, etc.). That infrastructure is out of
//! scope for this pass: wiring it for real would mean replicating a large
//! part of chatman's own test harness, not "thinly wrapping" it. Instead
//! this module takes a minimal, explicit, in-memory triple projection the
//! caller is responsible for building from whatever admitted-state source
//! it has (a real RDF store, a fixture, or (currently, since no caller
//! exists yet) a hand-built `Vec` in a test) -- see the module doc comment
//! on `f08_pddl_planning` for this disclosed scope boundary.
//!
//! The invariant "the planner problem is graph-derived, not hand-authored
//! text" is satisfied the same way `ChatmanEngine` satisfies it: the PDDL
//! domain/problem are still PDDL *text*, but that text is only reachable
//! through a predicate-keyed selection over admitted facts, not passed to
//! the planner directly by a caller.

use bcinr_pddl::parse::{domain_from_pddl, problem_from_pddl};
use bcinr_pddl::{Pddl8Domain, Pddl8Problem};

use super::refusal::Refusal;

/// Same predicate IRIs `praxis-graphlaw/src/chatman/engine.rs` uses
/// (`PDDL_DOMAIN_PREDICATE` / `PDDL_PROBLEM_PREDICATE`), reused verbatim so
/// admitted-graph fixtures are interchangeable between the two.
pub const PDDL_DOMAIN_PREDICATE: &str = "urn:chatman:engine#pddlDomain";
pub const PDDL_PROBLEM_PREDICATE: &str = "urn:chatman:engine#pddlProblem";

/// One admitted RDF fact, reduced to what this module needs: a predicate
/// IRI and its literal object value. Not a general triple/store type --
/// see this module's doc comment for why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedTriple {
    pub subject: String,
    pub predicate: String,
    pub object_literal: String,
}

/// First admitted fact whose predicate is `predicate`, or `None`. Linear
/// scan, matching `ChatmanEngine::select_literal`'s own semantics (first
/// match wins, no ordering guarantee beyond caller-supplied order).
///
/// # Complexity
/// O(n) over `graph`.
#[must_use]
pub fn select_literal<'a>(graph: &'a [AdmittedTriple], predicate: &str) -> Option<&'a str> {
    graph
        .iter()
        .find(|t| t.predicate == predicate)
        .map(|t| t.object_literal.as_str())
}

/// Problem Projector (stage 1) + Domain Resolver (stage 2) combined:
/// extract the PDDL domain/problem literal text from `graph` by predicate,
/// then parse both through `bcinr_pddl::parse`. Refuses
/// [`Refusal::NoAdmissiblePlan`] if either literal is absent (an
/// unresolved boundary, per the atlas's F08-L4 refusal sequence); parse
/// failures are `Refusal::Underlying` (a different failure mode than "no
/// plan reachable" -- see [`Refusal::from_pddl8`]).
///
/// # Errors
/// See above.
pub fn project_and_resolve(
    graph: &[AdmittedTriple],
) -> Result<(Pddl8Domain, Pddl8Problem), Refusal> {
    let domain_text =
        select_literal(graph, PDDL_DOMAIN_PREDICATE).ok_or_else(|| Refusal::NoAdmissiblePlan {
            stage: "ProblemProjector",
            reason: format!(
                "no PDDL domain literal at <{PDDL_DOMAIN_PREDICATE}> in admitted graph"
            ),
        })?;
    let problem_text =
        select_literal(graph, PDDL_PROBLEM_PREDICATE).ok_or_else(|| Refusal::NoAdmissiblePlan {
            stage: "ProblemProjector",
            reason: format!(
                "no PDDL problem literal at <{PDDL_PROBLEM_PREDICATE}> in admitted graph"
            ),
        })?;
    let domain =
        domain_from_pddl(domain_text).map_err(|e| Refusal::from_pddl8("DomainResolver", e))?;
    let problem =
        problem_from_pddl(problem_text).map_err(|e| Refusal::from_pddl8("ProblemProjector", e))?;
    Ok((domain, problem))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal STRIPS domain/problem pair, structurally identical in
    /// shape to the fixtures `crates/cng/tests/cng_pipeline.rs` and
    /// `praxis-graphlaw/tests/chatman_pddl_to_powl_projection.rs` already
    /// use for this exact domain/problem literal parsing path.
    const DOMAIN_TEXT: &str = r#"
(define (domain f08-test)
  (:requirements :strips)
  (:predicates (at ?x) (goal-reached))
  (:action move
    :parameters (?x)
    :precondition (at ?x)
    :effect (and (goal-reached))))
"#;
    const PROBLEM_TEXT: &str = r#"
(define (problem f08-test-problem)
  (:domain f08-test)
  (:objects a)
  (:init (at a))
  (:goal (and (goal-reached))))
"#;

    fn fixture_graph() -> Vec<AdmittedTriple> {
        vec![
            AdmittedTriple {
                subject: "urn:mfw:f08:test-snapshot".to_string(),
                predicate: PDDL_DOMAIN_PREDICATE.to_string(),
                object_literal: DOMAIN_TEXT.to_string(),
            },
            AdmittedTriple {
                subject: "urn:mfw:f08:test-snapshot".to_string(),
                predicate: PDDL_PROBLEM_PREDICATE.to_string(),
                object_literal: PROBLEM_TEXT.to_string(),
            },
        ]
    }

    #[test]
    fn select_literal_finds_by_predicate() {
        let graph = fixture_graph();
        assert_eq!(
            select_literal(&graph, PDDL_DOMAIN_PREDICATE),
            Some(DOMAIN_TEXT)
        );
        assert_eq!(select_literal(&graph, "urn:nope"), None);
    }

    #[test]
    fn project_and_resolve_parses_a_real_admitted_graph() {
        let graph = fixture_graph();
        let (domain, problem) = project_and_resolve(&graph).expect("real STRIPS fixture parses");
        assert_eq!(domain.name, "f08-test");
        assert_eq!(problem.name, "f08-test-problem");
    }

    #[test]
    fn missing_domain_literal_refuses_no_admissible_plan() {
        let graph = vec![AdmittedTriple {
            subject: "s".to_string(),
            predicate: PDDL_PROBLEM_PREDICATE.to_string(),
            object_literal: PROBLEM_TEXT.to_string(),
        }];
        let err = project_and_resolve(&graph).expect_err("no domain literal must refuse");
        assert!(matches!(
            err,
            Refusal::NoAdmissiblePlan {
                stage: "ProblemProjector",
                ..
            }
        ));
    }

    #[test]
    fn missing_problem_literal_refuses_no_admissible_plan() {
        let graph = vec![AdmittedTriple {
            subject: "s".to_string(),
            predicate: PDDL_DOMAIN_PREDICATE.to_string(),
            object_literal: DOMAIN_TEXT.to_string(),
        }];
        let err = project_and_resolve(&graph).expect_err("no problem literal must refuse");
        assert!(matches!(
            err,
            Refusal::NoAdmissiblePlan {
                stage: "ProblemProjector",
                ..
            }
        ));
    }

    #[test]
    fn malformed_domain_text_is_underlying_not_no_admissible_plan() {
        let graph = vec![
            AdmittedTriple {
                subject: "s".to_string(),
                predicate: PDDL_DOMAIN_PREDICATE.to_string(),
                object_literal: "not pddl at all".to_string(),
            },
            AdmittedTriple {
                subject: "s".to_string(),
                predicate: PDDL_PROBLEM_PREDICATE.to_string(),
                object_literal: PROBLEM_TEXT.to_string(),
            },
        ];
        let err = project_and_resolve(&graph).expect_err("malformed PDDL text must refuse");
        assert!(matches!(err, Refusal::Underlying { .. }));
    }
}
