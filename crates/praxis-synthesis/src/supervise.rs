//! Supervision topology — derived from the plan, never authored.
//!
//! NET-NEW: nothing in the constellation has this. knhk's real supervision
//! is hand-declared Erlang OTP (`genesis_custody_sup.erl`, one_for_one,
//! static children); its Rust side has classification and symptom analysis
//! but no supervisor tree. Here the tree is a *derived artifact*: stages
//! fall out of the plan's own data-dependency structure, the restart policy
//! is 8-bounded data, and the whole topology is content-addressed into the
//! plan's receipt lineage.
//!
//! Strategy vocabulary is OTP's, minus two deliberate absences:
//! - `OneForAll` is not in the enum at all. OTP justifies it by shared
//!   mutable fate between siblings; an acyclic data-flow plan cannot
//!   express that — siblings at the same depth are independent by
//!   construction. The absence IS the refusal.
//! - `SimpleOneForOne` (dynamic children) is absent: derived plans have
//!   static step sets. Salvage: incremental assertion is the growth path.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::dag::Dag;
use crate::sequence::{SequencePlan, SequenceProblem};
use crate::Refusal;

/// Maximum restart intensity — the byte governor applied to recovery.
pub const MAX_RESTART_INTENSITY: u8 = 8;

/// Restart strategy per stage. See the module docs for what is absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Strategy {
    /// A failed node restarts alone: nothing downstream consumes it.
    OneForOne,
    /// A failed node restarts together with its transitive dependents:
    /// its outputs feed them, so their work is invalidated.
    RestForOne,
}

/// Restart policy: intensity as data, windows in attempt ticks (no wall
/// clock — supervision must replay deterministically).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestartPolicy {
    /// Restarts allowed per node before lawful surrender (≤ 8).
    pub max_restarts: u8,
    /// The window, counted in restart attempts across the run.
    pub window: u32,
}

impl RestartPolicy {
    /// Construct, refusing intensities beyond the doctrine — a limit above
    /// 8 is refused, never silently clamped.
    pub fn new(max_restarts: u8, window: u32) -> Result<Self, Refusal> {
        if max_restarts > MAX_RESTART_INTENSITY {
            return Err(Refusal::InvalidInput {
                detail: format!(
                    "restart intensity {max_restarts} exceeds MAX_RESTART_INTENSITY \
                     ({MAX_RESTART_INTENSITY}); refused, not clamped"
                ),
            });
        }
        Ok(Self { max_restarts, window })
    }

    /// The doctrine default: 3 restarts (Erlang's customary intensity)
    /// within an 8-attempt window.
    #[must_use]
    pub fn default_policy() -> Self {
        Self { max_restarts: 3, window: 8 }
    }
}

/// One derived stage: the nodes at a dependency depth, with the strategy
/// their position earns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage {
    /// Dependency depth (0 = no data inputs).
    pub depth: usize,
    /// Strategy: `RestForOne` iff any member has downstream consumers.
    pub strategy: Strategy,
    /// Member node ids, sorted (content ids from [`Dag`]).
    pub nodes: Vec<String>,
}

/// The derived supervision topology. There is no way to construct one by
/// hand: [`SupervisionTopology::derive`] is the only door, and the sealed
/// field keeps struct literals out even inside the crate's dependents.
#[derive(Debug, Clone, Serialize)]
pub struct SupervisionTopology {
    /// Stages in depth order.
    pub stages: Vec<Stage>,
    /// The restart policy in force.
    pub policy: RestartPolicy,
    /// Content address over stages + policy + plan hash — the topology as
    /// a value in the plan's receipt lineage.
    pub topology_hash: String,
    /// Transitive dependents per node (the restart cohorts).
    downstream: BTreeMap<String, Vec<String>>,
    #[serde(skip)]
    _sealed: (),
}

impl SupervisionTopology {
    /// Derive the topology from a plan. The ONLY constructor.
    pub fn derive(
        plan: &SequencePlan,
        problem: &SequenceProblem,
        policy: RestartPolicy,
    ) -> Result<Self, Refusal> {
        if policy.max_restarts > MAX_RESTART_INTENSITY {
            return Err(Refusal::InvalidInput {
                detail: "restart policy exceeds the intensity doctrine".into(),
            });
        }
        let dag = Dag::from_plan(plan, problem);
        // Dependency depth per node: 0 for source nodes, else 1 + max input.
        let mut depth: BTreeMap<String, usize> = BTreeMap::new();
        // Dag nodes are acyclic (edges point earlier→later); iterate in
        // plan order via repeated relaxation (≤ |V| rounds, tiny sets).
        for _ in 0..dag.nodes.len().max(1) {
            let mut changed = false;
            for node in dag.nodes.values() {
                let d = node
                    .inputs
                    .iter()
                    .map(|i| depth.get(i).copied().map_or(usize::MAX, |x| x + 1))
                    .max()
                    .unwrap_or(0);
                if d != usize::MAX && depth.get(&node.id) != Some(&d) {
                    depth.insert(node.id.clone(), d);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        // Transitive downstream sets (restart cohorts).
        let mut direct: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for node in dag.nodes.values() {
            for input in &node.inputs {
                direct.entry(input.as_str()).or_default().push(node.id.as_str());
            }
        }
        let mut downstream: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for id in dag.nodes.keys() {
            let mut seen: BTreeSet<&str> = BTreeSet::new();
            let mut stack: Vec<&str> =
                direct.get(id.as_str()).cloned().unwrap_or_default();
            while let Some(next) = stack.pop() {
                if seen.insert(next) {
                    stack.extend(direct.get(next).cloned().unwrap_or_default());
                }
            }
            downstream
                .insert(id.clone(), seen.into_iter().map(String::from).collect());
        }
        // Stages: group by depth; strategy earned by having dependents.
        let mut by_depth: BTreeMap<usize, Vec<String>> = BTreeMap::new();
        for (id, d) in &depth {
            by_depth.entry(*d).or_default().push(id.clone());
        }
        let stages: Vec<Stage> = by_depth
            .into_iter()
            .map(|(d, mut nodes)| {
                nodes.sort_unstable();
                let strategy = if nodes
                    .iter()
                    .any(|n| downstream.get(n).is_some_and(|ds| !ds.is_empty()))
                {
                    Strategy::RestForOne
                } else {
                    Strategy::OneForOne
                };
                Stage { depth: d, strategy, nodes }
            })
            .collect();
        // Content-address the whole derivation.
        let canon = serde_json::json!({
            "stages": stages,
            "policy": policy,
            "plan_hash": plan.receipt.plan_hash,
            "problem_hash": problem.problem_hash(),
        });
        let topology_hash =
            chatman_common::provenance::content_address(canon.to_string().as_bytes());
        Ok(Self { stages, policy, topology_hash, downstream, _sealed: () })
    }

    /// The restart cohort for a node under its stage's strategy: the node
    /// alone (`OneForOne`) or the node plus its transitive dependents
    /// (`RestForOne`). Always includes the node itself, sorted.
    #[must_use]
    pub fn cohort(&self, node_id: &str) -> Vec<String> {
        let strategy = self
            .stages
            .iter()
            .find(|s| s.nodes.iter().any(|n| n == node_id))
            .map_or(Strategy::OneForOne, |s| s.strategy);
        let mut cohort = vec![node_id.to_string()];
        if strategy == Strategy::RestForOne {
            if let Some(ds) = self.downstream.get(node_id) {
                cohort.extend(ds.iter().cloned());
            }
        }
        cohort.sort_unstable();
        cohort.dedup();
        cohort
    }

    /// Direct + transitive dependents of a node (regardless of strategy) —
    /// what a Park must also disposition.
    #[must_use]
    pub fn dependents(&self, node_id: &str) -> &[String] {
        self.downstream.get(node_id).map_or(&[], Vec::as_slice)
    }

    /// Total supervised nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.stages.iter().map(|s| s.nodes.len()).sum()
    }
}
