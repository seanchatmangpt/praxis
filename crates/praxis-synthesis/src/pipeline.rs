//! The composition: facts → saturate → sequence → dag-execute → verify →
//! one receipt.
//!
//! [`Synthesis::run`] threads the four layers together and folds every stage
//! receipt into a single rolling chain, so the whole pipeline run is one
//! auditable content-addressed object.

use serde::{Deserialize, Serialize};

use chatman_common::provenance::RollingChain;

use crate::dag::{Dag, DagReceipt, MemoCache, NodeRunner};
use crate::datalog::{Atom, Program, SaturationReceipt};
use crate::sequence::{Capability, SequencePlan, SequenceProblem, Solver};
use crate::verify::{admit, Verdict};
use crate::Refusal;

/// Domain string seeding the pipeline chain.
pub const PIPELINE_CHAIN_DOMAIN: &str = "praxis-synthesis/pipeline/v1";

/// The single receipt a full pipeline run emits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SynthesisReceipt {
    /// Layer 1: saturation receipt (fixpoint content address inside).
    pub saturation: SaturationReceipt,
    /// Layer 2: plan content address.
    pub plan_hash: String,
    /// Layer 2: plan cost and length.
    pub plan_steps: usize,
    /// Layer 3: DAG execution receipt.
    pub dag: DagReceipt,
    /// Layer 4: refinement verdict.
    pub verdict: Verdict,
    /// Rolling chain over (fixpoint_hash, plan_hash, dag root_hash, verdict):
    /// the whole run as one hash.
    pub chain: String,
}

/// The pipeline runner.
#[derive(Debug, Default, Clone, Copy)]
pub struct Synthesis;

impl Synthesis {
    /// Run the full pipeline. The plan is refused (with the verdict as
    /// salvage) if any refinement fails — admission is not optional.
    pub fn run(
        program: &mut Program,
        capabilities: Vec<Capability>,
        goal: Vec<Atom>,
        horizon: usize,
        solver: &dyn Solver,
        runner: &mut dyn NodeRunner,
        cache: &mut MemoCache,
    ) -> Result<SynthesisReceipt, Refusal> {
        // Layer 1: saturate.
        let saturation = program.saturate()?;
        // Layer 2: sequence against the saturated database.
        let problem = SequenceProblem::new(program, capabilities, goal, horizon, Vec::new())?;
        let plan: SequencePlan = solver.solve(&problem)?;
        // Layer 3: derive + execute the content-addressed DAG.
        let dag = Dag::from_plan(&plan, &problem);
        let dag_receipt = dag.execute(runner, cache)?;
        // Layer 4: admit.
        let verdict = admit(program, &problem, &plan, &dag, &dag_receipt);
        if !verdict.ok {
            return Err(Refusal::VerificationFailed { failed: verdict.failed() });
        }
        // Fold the run into one chain.
        let mut chain = RollingChain::new(PIPELINE_CHAIN_DOMAIN);
        chain.push(saturation.fixpoint_hash.as_bytes());
        chain.push(plan.receipt.plan_hash.as_bytes());
        chain.push(dag_receipt.root_hash.as_bytes());
        chain.push(
            serde_json::to_string(&verdict)
                .unwrap_or_default()
                .as_bytes(),
        );
        Ok(SynthesisReceipt {
            saturation,
            plan_hash: plan.receipt.plan_hash.clone(),
            plan_steps: plan.steps.len(),
            dag: dag_receipt,
            verdict,
            chain: chain.finalize(),
        })
    }
}
