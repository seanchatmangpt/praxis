//! Layer 3 — content-addressed DAG execution (the OxyMake lesson).
//!
//! A [`SequencePlan`] is linear, but its true shape is a DAG: step B depends
//! on step A only if A produces an atom B consumes. [`Dag::from_plan`] derives
//! those data edges, and [`Dag::execute`] runs nodes in deterministic
//! topological order, hashing every output with BLAKE3 and chaining node
//! frames with `chatman_common::provenance::fold_event`.
//!
//! Reproducibility is content-addressed, not order-addressed: a node's memo
//! key is `(action_hash, input_hashes)`, so a re-run replays from cache and
//! the DAG's `root_hash` is identical regardless of step declaration order.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};

use chatman_common::provenance::{content_address, fold_event, genesis_seed};

use crate::sequence::{BoundStep, SequencePlan, SequenceProblem};
use crate::Refusal;

/// Domain string seeding the node-receipt chain.
pub const DAG_CHAIN_DOMAIN: &str = "praxis-synthesis/dag/v1";

/// A node: one bound step plus the nodes whose outputs it consumes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DagNode {
    /// Content-based identity: hash of (capability, binding, occurrence).
    pub id: String,
    /// The bound step this node executes.
    pub action: BoundStep,
    /// IDs of nodes this node depends on (data edges), sorted.
    pub inputs: Vec<String>,
}

/// The derived DAG.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dag {
    /// Nodes keyed by content id.
    pub nodes: BTreeMap<String, DagNode>,
}

/// Per-node execution receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeReceipt {
    /// Node id.
    pub node_id: String,
    /// Hash of the bound action.
    pub action_hash: String,
    /// Sorted output hashes of the node's inputs.
    pub input_hashes: Vec<String>,
    /// BLAKE3 of the node's output bytes.
    pub output_hash: String,
    /// Rolling chain value after folding this node's frame.
    pub chain: String,
    /// Whether the node was replayed from the memo cache.
    pub replayed: bool,
}

/// Receipt for one DAG execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DagReceipt {
    /// Order-independent content address of the executed DAG: BLAKE3 over the
    /// sorted `node_id:output_hash` pairs.
    pub root_hash: String,
    /// Node receipts in execution (topological) order.
    pub node_receipts: Vec<NodeReceipt>,
    /// How many nodes were served from the memo cache.
    pub replayed_count: usize,
}

/// Executes one node given its resolved input output-bytes. Implementations
/// must be deterministic in `(action, inputs)` for memoization to be sound.
pub trait NodeRunner {
    /// Produce the node's output bytes.
    fn run(&mut self, node: &DagNode, inputs: &[Vec<u8>]) -> Vec<u8>;
}

/// Default runner: output = canonical frame of the action + input hashes.
/// Deterministic by construction; stands in for real side-effectful runners.
#[derive(Debug, Default, Clone, Copy)]
pub struct HashRunner;

impl NodeRunner for HashRunner {
    fn run(&mut self, node: &DagNode, inputs: &[Vec<u8>]) -> Vec<u8> {
        let input_hashes: Vec<String> =
            inputs.iter().map(|b| content_address(b)).collect();
        serde_json::json!({
            "capability": node.action.capability,
            "binding": node.action.binding,
            "inputs": input_hashes,
        })
        .to_string()
        .into_bytes()
    }
}

/// Content-addressed memo cache: `(action_hash ‖ input_hashes)` → output bytes.
#[derive(Debug, Default, Clone)]
pub struct MemoCache {
    entries: HashMap<String, Vec<u8>>,
}

impl MemoCache {
    /// Empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Number of memoized outputs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    /// Whether the cache is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

fn action_hash(step: &BoundStep) -> String {
    let canon = serde_json::to_string(step).unwrap_or_default();
    content_address(canon.as_bytes())
}

fn memo_key(action: &str, input_hashes: &[String]) -> String {
    let mut bytes = action.as_bytes().to_vec();
    for h in input_hashes {
        bytes.extend_from_slice(h.as_bytes());
    }
    content_address(&bytes)
}

impl Dag {
    /// Derive the DAG from a plan. Node identity is content-based (hash of
    /// capability, binding, and occurrence index among identical steps); an
    /// edge A→B exists iff an add-effect of A (the *latest* such producer
    /// before B) matches a ground precondition of B (data edges only).
    #[must_use]
    pub fn from_plan(plan: &SequencePlan, problem: &SequenceProblem) -> Self {
        // Content-based ids, disambiguated by occurrence.
        let mut seen: HashMap<String, u32> = HashMap::new();
        let ids: Vec<String> = plan
            .steps
            .iter()
            .map(|s| {
                let base = serde_json::to_string(s).unwrap_or_default();
                let occ = seen.entry(base.clone()).or_insert(0);
                let id = content_address(
                    format!("{base}#{occ}").as_bytes(),
                );
                *occ += 1;
                id
            })
            .collect();
        // Producer map: ground atom -> latest step index that added it.
        let mut nodes: BTreeMap<String, DagNode> = BTreeMap::new();
        let mut producer: HashMap<(u32, Vec<u32>), usize> = HashMap::new();
        for (i, step) in plan.steps.iter().enumerate() {
            let mut inputs: BTreeSet<String> = BTreeSet::new();
            for atom in problem.step_preconditions(step) {
                if let Some(&p) = producer.get(&atom) {
                    inputs.insert(ids[p].clone());
                }
            }
            for atom in problem.step_effects(step) {
                producer.insert(atom, i);
            }
            nodes.insert(
                ids[i].clone(),
                DagNode {
                    id: ids[i].clone(),
                    action: step.clone(),
                    inputs: inputs.into_iter().collect(),
                },
            );
        }
        Self { nodes }
    }

    /// Deterministic topological order (Kahn's algorithm, ties broken by
    /// content id). Refuses on a cycle — which `from_plan` cannot produce,
    /// but hand-built DAGs can.
    pub fn topo_order(&self) -> Result<Vec<String>, Refusal> {
        let mut indegree: BTreeMap<&str, usize> =
            self.nodes.keys().map(|k| (k.as_str(), 0)).collect();
        let mut dependents: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for node in self.nodes.values() {
            for input in &node.inputs {
                if !self.nodes.contains_key(input) {
                    return Err(Refusal::InvalidInput {
                        detail: format!("node {} names unknown input {input}", node.id),
                    });
                }
                *indegree.get_mut(node.id.as_str()).expect("node present") += 1;
                dependents.entry(input.as_str()).or_default().push(&node.id);
            }
        }
        let mut ready: BTreeSet<&str> = indegree
            .iter()
            .filter_map(|(k, d)| (*d == 0).then_some(*k))
            .collect();
        let mut order = Vec::with_capacity(self.nodes.len());
        while let Some(&next) = ready.iter().next() {
            ready.remove(next);
            order.push(next.to_string());
            for dep in dependents.get(next).into_iter().flatten() {
                let d = indegree.get_mut(dep).expect("dep present");
                *d -= 1;
                if *d == 0 {
                    ready.insert(dep);
                }
            }
        }
        if order.len() != self.nodes.len() {
            return Err(Refusal::InvalidInput {
                detail: format!(
                    "cycle: only {} of {} nodes orderable",
                    order.len(),
                    self.nodes.len()
                ),
            });
        }
        Ok(order)
    }

    /// Execute in topological order, memoized and content-addressed.
    pub fn execute(
        &self,
        runner: &mut dyn NodeRunner,
        cache: &mut MemoCache,
    ) -> Result<DagReceipt, Refusal> {
        let order = self.topo_order()?;
        let mut outputs: HashMap<String, (String, Vec<u8>)> = HashMap::new();
        let mut chain = genesis_seed(DAG_CHAIN_DOMAIN);
        let mut node_receipts = Vec::with_capacity(order.len());
        let mut replayed_count = 0;

        for id in &order {
            let node = &self.nodes[id];
            let a_hash = action_hash(&node.action);
            let mut input_hashes: Vec<String> = node
                .inputs
                .iter()
                .map(|i| outputs[i].0.clone())
                .collect();
            input_hashes.sort_unstable();
            let key = memo_key(&a_hash, &input_hashes);

            let (output, replayed) = if let Some(cached) = cache.entries.get(&key) {
                (cached.clone(), true)
            } else {
                let input_bytes: Vec<Vec<u8>> =
                    node.inputs.iter().map(|i| outputs[i].1.clone()).collect();
                let out = runner.run(node, &input_bytes);
                cache.entries.insert(key, out.clone());
                (out, false)
            };
            if replayed {
                replayed_count += 1;
            }
            let output_hash = content_address(&output);
            let frame = serde_json::json!({
                "node_id": id,
                "action_hash": a_hash,
                "input_hashes": input_hashes,
                "output_hash": output_hash,
            })
            .to_string();
            chain = fold_event(&chain, frame.as_bytes());
            node_receipts.push(NodeReceipt {
                node_id: id.clone(),
                action_hash: a_hash,
                input_hashes,
                output_hash: output_hash.clone(),
                chain: chain.clone(),
                replayed,
            });
            outputs.insert(id.clone(), (output_hash, output));
        }

        // Order-independent root: sorted (node_id, output_hash) pairs.
        let mut pairs: Vec<String> = outputs
            .iter()
            .map(|(id, (h, _))| format!("{id}:{h}"))
            .collect();
        pairs.sort_unstable();
        let root_hash = content_address(pairs.join("\n").as_bytes());

        Ok(DagReceipt { root_hash, node_receipts, replayed_count })
    }
}
