//! Layer 4 — refinement-style admission (the Flux lesson).
//!
//! Not a type checker: an admission layer that runs machine-checkable
//! refinements over the pipeline's *own artifacts* — plan, DAG, receipts —
//! in the style of `praxis-core::verify::run_pipeline`: every check runs
//! (no short-circuit), every failure carries a witness, nothing is silent.

use serde::{Deserialize, Serialize};

use chatman_common::provenance::{fold_event, genesis_seed};

use crate::dag::{Dag, DagReceipt, DAG_CHAIN_DOMAIN};
use crate::datalog::Program;
use crate::sequence::{SequencePlan, SequenceProblem};

/// One refinement's outcome, with a human-legible witness either way.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckOutcome {
    /// Refinement name.
    pub name: String,
    /// Whether it held.
    pub ok: bool,
    /// Evidence: what was checked and what was found.
    pub witness: String,
}

/// The verdict over all refinements. `ok` iff every check passed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verdict {
    /// Conjunction of all checks.
    pub ok: bool,
    /// Every check's outcome, in declaration order.
    pub checks: Vec<CheckOutcome>,
}

impl Verdict {
    /// Names of failed checks.
    #[must_use]
    pub fn failed(&self) -> Vec<String> {
        self.checks.iter().filter(|c| !c.ok).map(|c| c.name.clone()).collect()
    }
}

/// Run all refinements over the pipeline artifacts. No short-circuit.
#[must_use]
pub fn admit(
    program: &mut Program,
    problem: &SequenceProblem,
    plan: &SequencePlan,
    dag: &Dag,
    receipt: &DagReceipt,
) -> Verdict {
    let mut checks = Vec::new();

    // PlanRespectsHorizon
    checks.push(CheckOutcome {
        name: "PlanRespectsHorizon".into(),
        ok: plan.steps.len() <= problem.horizon(),
        witness: format!("{} steps <= horizon {}", plan.steps.len(), problem.horizon()),
    });

    // PlanReachesGoal (independent replay — the differential guard)
    let reaches = problem.replay_reaches_goal(plan);
    checks.push(CheckOutcome {
        name: "PlanReachesGoal".into(),
        ok: reaches,
        witness: format!("independent replay of {} steps reaches goal: {reaches}", plan.steps.len()),
    });

    // DagAcyclic
    let topo = dag.topo_order();
    checks.push(CheckOutcome {
        name: "DagAcyclic".into(),
        ok: topo.is_ok(),
        witness: match &topo {
            Ok(order) => format!("Kahn ordered all {} nodes", order.len()),
            Err(e) => format!("{e}"),
        },
    });

    // EveryNodeReceipted
    checks.push(CheckOutcome {
        name: "EveryNodeReceipted".into(),
        ok: receipt.node_receipts.len() == dag.nodes.len(),
        witness: format!(
            "{} receipts for {} nodes",
            receipt.node_receipts.len(),
            dag.nodes.len()
        ),
    });

    // ChainRecomputes: refold every node frame, byte-compare final link.
    let mut chain = genesis_seed(DAG_CHAIN_DOMAIN);
    for nr in &receipt.node_receipts {
        let mut map = std::collections::BTreeMap::new();
        map.insert("node_id", serde_json::Value::String(nr.node_id.clone()));
        map.insert("action_hash", serde_json::Value::String(nr.action_hash.clone()));
        map.insert("input_hashes", serde_json::Value::Array(
            nr.input_hashes.iter().map(|h| serde_json::Value::String(h.clone())).collect()
        ));
        map.insert("output_hash", serde_json::Value::String(nr.output_hash.clone()));
        let frame = serde_json::to_string(&map).unwrap_or_default();
        chain = fold_event(&chain, frame.as_bytes());
    }
    let recorded = receipt.node_receipts.last().map_or_else(
        || genesis_seed(DAG_CHAIN_DOMAIN),
        |nr| nr.chain.clone(),
    );
    checks.push(CheckOutcome {
        name: "ChainRecomputes".into(),
        ok: chain == recorded,
        witness: format!("recomputed {} == recorded {}", &chain[..16], &recorded[..16]),
    });

    // FixpointClosed: one more saturation round derives nothing new.
    let closed = program.is_closed();
    checks.push(CheckOutcome {
        name: "FixpointClosed".into(),
        ok: matches!(closed, Ok(true)),
        witness: match closed {
            Ok(c) => format!("extra saturation round derived nothing new: {c}"),
            Err(e) => format!("re-saturation refused: {e}"),
        },
    });

    let ok = checks.iter().all(|c| c.ok);
    Verdict { ok, checks }
}
