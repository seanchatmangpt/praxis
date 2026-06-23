use genesis_types_v2::{
    BoundedRepairOperator, Error, RepairAdmissionReport, ResidualVector, Result, VisualGapReport,
};
use std::collections::HashMap;

/// Trait representing the Measurement Function (\mu) and execution environment
/// necessary to maintain the Chatman Equation (A = \mu(O)).
#[async_trait::async_trait]
pub trait MeasurementEnvironment {
    /// Compute \mu(O) against A and return the freshness-guaranteed gap report.
    async fn measure_gap(&self) -> Result<VisualGapReport>;

    /// Apply a specific BoundedRepairOperator to the artifact A.
    /// Must cross a real boundary (e.g., execute a deterministic change).
    async fn apply_repair(&self, operator: &BoundedRepairOperator) -> Result<()>;

    /// Rollback the last repair if it was not admitted.
    async fn rollback_repair(&self, operator: &BoundedRepairOperator) -> Result<()>;

    /// Fetch available repair operators.
    async fn available_operators(&self) -> Result<Vec<BoundedRepairOperator>>;
}

/// The core Autonomic Loop for the praxis-reconciler.
/// Continuously monitors structural drift and applies residual-vector repair loops.
pub struct PraxisReconciler {
    pub env: Box<dyn MeasurementEnvironment + Send + Sync>,
}

impl PraxisReconciler {
    pub fn new(env: Box<dyn MeasurementEnvironment + Send + Sync>) -> Self {
        Self { env }
    }

    /// Executes the autonomic repair loop.
    /// Evaluates the Chatman Equation and applies repairs until the residual vector is 0 (all passing)
    /// or repair potential is exhausted.
    pub async fn reconcile(&self) -> Result<()> {
        loop {
            // 1. Measure current state \mu(O) vs A
            let report = self.env.measure_gap().await?;
            report.assert_fresh()?;

            let current_residual = report.residuals;

            // 2. Check Chatman Equation compliance: if all passing, A = \mu(O) is satisfied.
            if current_residual.all_passing() {
                // Structural drift is 0. System is in equilibrium.
                return Ok(());
            }

            // 3. Structural drift detected. Identify dominant dimension.
            let dominant_dim = match &current_residual.dominant {
                Some(dim) => dim.clone(),
                None => {
                    return Err(Error::StateError(
                        "Drift detected but no dominant dimension found in residual vector".into(),
                    ))
                }
            };

            // 4. Find applicable repair operators for the dominant dimension.
            let operators = self.env.available_operators().await?;
            let mut applicable_ops: Vec<BoundedRepairOperator> = operators
                .into_iter()
                .filter(|op| op.targets_dimension == dominant_dim)
                .collect();

            if applicable_ops.is_empty() {
                return Err(Error::StateError(format!(
                    "No repair operators available for dominant dimension: {}",
                    dominant_dim
                )));
            }

            // Sort operators based on some heuristic if needed; here we just iterate.
            let mut repair_successful = false;

            for operator in applicable_ops {
                // 5. Apply residual-vector repair loop boundary crossing
                self.env.apply_repair(&operator).await?;

                // 6. Measure new state
                let after_report = self.env.measure_gap().await?;
                after_report.assert_fresh()?;
                let after_residual = after_report.residuals;

                // 7. Validate repair admission
                let admission = RepairAdmissionReport::compute(
                    operator.id.clone(),
                    current_residual.clone(),
                    after_residual.clone(),
                );

                if admission.admitted {
                    // Repair was successful and admitted
                    repair_successful = true;
                    break; 
                } else {
                    // Repair failed to improve the residual vector; rollback and try next operator
                    self.env.rollback_repair(&operator).await?;
                }
            }

            if !repair_successful {
                return Err(Error::StateError(format!(
                    "Exhausted all repair operators for dimension {}. Chatman Equation cannot be reconciled.",
                    dominant_dim
                )));
            }
            
            // Loop continues to check if further dimensions need repair until all_passing() is true.
        }
    }
}
