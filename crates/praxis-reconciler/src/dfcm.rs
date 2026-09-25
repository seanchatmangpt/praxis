//! DfCM SELECT: preserve maximal reversible lawful possibilities before choosing.

use crate::model::{
    digest, AdmittedObservation, CandidateEdge, Refusal, RefusalCode, RepairOperator, Selection,
};
use std::collections::BTreeSet;

/// Hard cap on dimensions accepted by the reconciliation kernel.
pub const MAX_DIMENSIONS: usize = 64;
/// Hard cap on candidate repair operators examined per checkpoint.
pub const MAX_OPERATORS: usize = 256;

/// Admit a raw observation as `O*` after bounded structural checks.
///
/// # Errors
///
/// Returns [`RefusalCode::InvalidObservation`] for malformed or over-bound input,
/// or [`RefusalCode::Serialization`] if the deterministic digest cannot be built.
pub fn admit_observation(
    observation: crate::model::Observation,
) -> Result<AdmittedObservation, Refusal> {
    if observation.subject.trim().is_empty() {
        return Err(Refusal::new(
            RefusalCode::InvalidObservation,
            "subject must be non-empty",
        ));
    }
    if observation.identity.trim().is_empty() {
        return Err(Refusal::new(
            RefusalCode::InvalidObservation,
            "identity must be non-empty",
        ));
    }
    if observation.residuals.dimensions.len() > MAX_DIMENSIONS {
        return Err(Refusal::new(
            RefusalCode::InvalidObservation,
            format!(
                "residual dimensions {} exceed cap {MAX_DIMENSIONS}",
                observation.residuals.dimensions.len()
            ),
        ));
    }
    if observation
        .residuals
        .dimensions
        .keys()
        .any(|key| key.trim().is_empty())
    {
        return Err(Refusal::new(
            RefusalCode::InvalidObservation,
            "residual dimension names must be non-empty",
        ));
    }

    let observation_digest = digest(&observation)?;
    Ok(AdmittedObservation {
        observation,
        observation_digest,
    })
}

fn validate_operators(operators: &[RepairOperator]) -> Result<(), Refusal> {
    if operators.len() > MAX_OPERATORS {
        return Err(Refusal::new(
            RefusalCode::BudgetExceeded,
            format!("operator count {} exceeds cap {MAX_OPERATORS}", operators.len()),
        ));
    }

    let mut operator_ids = BTreeSet::new();
    for operator in operators {
        if operator.id.trim().is_empty() {
            return Err(Refusal::new(
                RefusalCode::InvalidOperator,
                "repair operator id must be non-empty",
            ));
        }
        if operator.targets.is_empty() {
            return Err(Refusal::new(
                RefusalCode::InvalidOperator,
                format!("repair operator {} has no target dimensions", operator.id),
            ));
        }
        if operator.targets.len() > MAX_DIMENSIONS
            || operator.expected_reduction.len() > MAX_DIMENSIONS
        {
            return Err(Refusal::new(
                RefusalCode::InvalidOperator,
                format!("repair operator {} exceeds the dimension cap", operator.id),
            ));
        }
        if operator.targets.iter().any(|target| target.trim().is_empty()) {
            return Err(Refusal::new(
                RefusalCode::InvalidOperator,
                format!("repair operator {} has an empty target dimension", operator.id),
            ));
        }
        if operator
            .expected_reduction
            .keys()
            .any(|dimension| !operator.targets.contains(dimension))
        {
            return Err(Refusal::new(
                RefusalCode::InvalidOperator,
                format!(
                    "repair operator {} declares reduction outside its target set",
                    operator.id
                ),
            ));
        }
        if !operator_ids.insert(operator.id.clone()) {
            return Err(Refusal::new(
                RefusalCode::InvalidOperator,
                format!("duplicate repair operator id {}", operator.id),
            ));
        }
    }
    Ok(())
}

fn candidate_edge(
    admitted: &AdmittedObservation,
    before_total: u64,
    operator: RepairOperator,
) -> CandidateEdge {
    let relevant = operator.targets.iter().any(|dimension| {
        admitted.observation.residuals.get(dimension) > 0
            && operator.expected_reduction.get(dimension).copied().unwrap_or(0) > 0
    });
    let predicted_reduction = operator
        .targets
        .iter()
        .map(|dimension| {
            let reduction = operator
                .expected_reduction
                .get(dimension)
                .copied()
                .unwrap_or(0);
            admitted
                .observation
                .residuals
                .get(dimension)
                .min(reduction)
        })
        .fold(0_u64, u64::saturating_add);
    let predicted_total = before_total.saturating_sub(predicted_reduction);

    let exclusion = if !relevant {
        Some("NO_RELEVANT_RESIDUAL_REDUCTION".to_string())
    } else if !operator.reversible {
        Some("IRREVERSIBLE_AUTOMATIC_EDGE".to_string())
    } else if operator.authority_scope.trim().is_empty() {
        Some("EMPTY_AUTHORITY_SCOPE".to_string())
    } else {
        None
    };
    let admitted_for_auto_do = exclusion.is_none();

    CandidateEdge {
        operator,
        predicted_total,
        admitted_for_auto_do,
        exclusion,
    }
}

/// Explore all bounded operators, preserving excluded edges as topology, then
/// choose the strongest reversible lawful edge deterministically.
///
/// # Errors
///
/// Returns a typed refusal when the operator budget is exceeded, operator identity
/// is ambiguous, no reversible lawful edge exists, or digest manufacture fails.
pub fn select_maximal_reversible(
    admitted: &AdmittedObservation,
    mut operators: Vec<RepairOperator>,
) -> Result<Selection, Refusal> {
    validate_operators(&operators)?;
    operators.sort_by(|left, right| left.id.cmp(&right.id));

    let before_total = admitted.observation.residuals.total();
    let edges: Vec<_> = operators
        .into_iter()
        .map(|operator| candidate_edge(admitted, before_total, operator))
        .collect();

    // Preserve the complete examined topology, but rank lawful reversible edges by:
    // 1. lowest predicted residual, 2. lowest cost, 3. stable operator id.
    let selected = edges
        .iter()
        .filter(|edge| edge.admitted_for_auto_do)
        .min_by(|left, right| {
            left.predicted_total
                .cmp(&right.predicted_total)
                .then_with(|| {
                    left.operator
                        .estimated_cost
                        .cmp(&right.operator.estimated_cost)
                })
                .then_with(|| left.operator.id.cmp(&right.operator.id))
        })
        .ok_or_else(|| {
            let irreversible = edges.iter().any(|edge| {
                edge.exclusion.as_deref() == Some("IRREVERSIBLE_AUTOMATIC_EDGE")
            });
            let code = if irreversible {
                RefusalCode::IrreversibleAutomaticActuation
            } else {
                RefusalCode::NoLawfulCandidate
            };
            Refusal::new(code, "no reversible admitted repair edge is available")
                .with_salvage("examined_edges", edges.len().to_string())
        })?;

    let selected_operator_id = selected.operator.id.clone();
    let selection_digest = digest(&(&edges, &selected_operator_id))?;
    Ok(Selection {
        edges,
        selected_operator_id,
        selection_digest,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Observation, ResidualVector};
    use std::collections::{BTreeMap, BTreeSet};

    fn op(id: &str, reversible: bool, reduction: u64, cost: u64) -> RepairOperator {
        RepairOperator {
            id: id.to_string(),
            targets: BTreeSet::from(["build".to_string()]),
            authority_scope: "praxis:repair/build".to_string(),
            reversible,
            estimated_cost: cost,
            expected_reduction: BTreeMap::from([("build".to_string(), reduction)]),
        }
    }

    #[test]
    fn dfcm_preserves_edges_before_selection() {
        let admitted_result = admit_observation(Observation {
            subject: "repo".to_string(),
            identity: "sha:1".to_string(),
            logical_time: 7,
            residuals: ResidualVector {
                dimensions: BTreeMap::from([("build".to_string(), 5)]),
            },
        });
        let admitted = match admitted_result {
            Ok(value) => value,
            Err(error) => panic!("fixture must admit: {error}"),
        };

        let selection_result = select_maximal_reversible(
            &admitted,
            vec![
                op("cheap", true, 2, 1),
                op("strong", true, 5, 9),
                op("unsafe", false, 5, 0),
            ],
        );
        let selection = match selection_result {
            Ok(value) => value,
            Err(error) => panic!("lawful reversible edges must select: {error}"),
        };

        assert_eq!(selection.edges.len(), 3);
        assert_eq!(selection.selected_operator_id, "strong");
        assert_eq!(
            selection.edges[2].exclusion.as_deref(),
            Some("IRREVERSIBLE_AUTOMATIC_EDGE")
        );
    }
}
