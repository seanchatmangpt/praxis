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
    /// Declared ticks the node used. OUTSIDE the hashed frame (additive —
    /// old receipts and the foreign verifier are untouched).
    #[serde(default)]
    pub ticks_used: u64,
    /// The node's declared tick budget (0 = time-tier, no tick contract).
    #[serde(default)]
    pub tick_budget: u64,
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
        let input_hashes: Vec<String> = inputs.iter().map(|b| content_address(b)).collect();
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
    /// Insert a raw `(key, output)` entry — the WAL recovery path.
    pub fn insert_raw(&mut self, key: String, output: Vec<u8>) {
        self.entries.insert(key, output);
    }
    /// Iterate raw `(key, payload)` entries — lets non-memo WAL consumers
    /// (e.g. the park queue) recover their records. Memo keys are 64-hex
    /// content addresses; other subsystems namespace with a prefix, so the
    /// two populations can never collide.
    pub fn iter_raw(&self) -> impl Iterator<Item = (&String, &Vec<u8>)> {
        self.entries.iter()
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
                let id = content_address(format!("{base}#{occ}").as_bytes());
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
        self.execute_inner(runner, cache, None)
    }

    /// Execute with every cold-computed node journaled to a write-ahead log
    /// before the receipt advances — the durability contract: a node the
    /// receipt claims computed is on disk, fsynced, or the frame is absent.
    pub fn execute_journaled(
        &self,
        runner: &mut dyn NodeRunner,
        cache: &mut MemoCache,
        wal: &mut crate::wal::Wal,
    ) -> Result<DagReceipt, Refusal> {
        self.execute_inner(runner, cache, Some(wal))
    }

    fn execute_inner(
        &self,
        runner: &mut dyn NodeRunner,
        cache: &mut MemoCache,
        mut wal: Option<&mut crate::wal::Wal>,
    ) -> Result<DagReceipt, Refusal> {
        let order = self.topo_order()?;
        let mut outputs: HashMap<String, (String, Vec<u8>)> = HashMap::new();
        let mut chain = genesis_seed(DAG_CHAIN_DOMAIN);
        let mut node_receipts = Vec::with_capacity(order.len());
        let mut replayed_count = 0;

        for id in &order {
            let node = &self.nodes[id];
            let a_hash = action_hash(&node.action);
            let mut input_hashes: Vec<String> =
                node.inputs.iter().map(|i| outputs[i].0.clone()).collect();
            input_hashes.sort_unstable();
            let key = memo_key(&a_hash, &input_hashes);

            let (output, replayed) = if let Some(cached) = cache.entries.get(&key) {
                (cached.clone(), true)
            } else {
                let input_bytes: Vec<Vec<u8>> =
                    node.inputs.iter().map(|i| outputs[i].1.clone()).collect();
                let out = runner.run(node, &input_bytes);
                if let Some(w) = wal.as_deref_mut() {
                    w.append(&key, &out)?;
                }
                cache.entries.insert(key, out.clone());
                (out, false)
            };
            if replayed {
                replayed_count += 1;
            }
            let output_hash = content_address(&output);
            let mut map = BTreeMap::new();
            map.insert("node_id", serde_json::Value::String(id.clone()));
            map.insert("action_hash", serde_json::Value::String(a_hash.clone()));
            map.insert(
                "input_hashes",
                serde_json::Value::Array(
                    input_hashes
                        .iter()
                        .map(|h| serde_json::Value::String(h.clone()))
                        .collect(),
                ),
            );
            map.insert(
                "output_hash",
                serde_json::Value::String(output_hash.clone()),
            );
            let frame = serde_json::to_string(&map).expect("serialization");
            chain = fold_event(&chain, frame.as_bytes());
            node_receipts.push(NodeReceipt {
                node_id: id.clone(),
                action_hash: a_hash,
                input_hashes,
                output_hash: output_hash.clone(),
                chain: chain.clone(),
                replayed,
                ticks_used: 1,
                tick_budget: 0,
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

        Ok(DagReceipt {
            root_hash,
            node_receipts,
            replayed_count,
        })
    }
}

// ─── Supervised execution: the classify→actuate loop, closed ────────────────

use crate::budget::Ticks;
use crate::geometry::{
    Classification, CrashKind, CrashSnapshot, FailureClass, FailureGeometry, LawfulResponse,
};
use crate::park::{ParkCause, ParkManager, ParkedEntry};
use crate::supervise::SupervisionTopology;

/// Domain seed for the crash-receipt chain.
pub const GEOMETRY_CHAIN_DOMAIN: &str = "praxis-synthesis/geometry/v1";

/// A crash, as a value. Under `forbid(unsafe_code)` there are no panics to
/// catch: fallible runners return these, and the supervisor classifies them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeCrash {
    /// Environmental/I-O failure.
    Io {
        /// What failed.
        detail: String,
    },
    /// The node produced output failing its own validation.
    BadOutput {
        /// What was wrong.
        detail: String,
    },
    /// Budget exceeded (declared ticks).
    OverBudget {
        /// Ticks the attempt consumed.
        ticks: u64,
    },
    /// A planning-time precondition no longer holds.
    PreconditionLost {
        /// The lost fact, if known.
        fact: String,
    },
    /// The node refused (rendered refusal).
    Refused {
        /// Rendered refusal text.
        rendered: String,
    },
}

impl NodeCrash {
    /// The mechanical crash kind for geometry classification.
    #[must_use]
    pub fn kind(&self) -> CrashKind {
        match self {
            NodeCrash::Io { .. } => CrashKind::Io,
            NodeCrash::BadOutput { .. } => CrashKind::BadOutput,
            NodeCrash::OverBudget { .. } => CrashKind::OverBudget,
            NodeCrash::PreconditionLost { .. } => CrashKind::PreconditionLost,
            NodeCrash::Refused { .. } => CrashKind::Refused,
        }
    }
}

/// A runner that may crash. Blanket-implemented for every infallible
/// [`NodeRunner`], so existing runners work unchanged.
pub trait FallibleRunner {
    /// Produce output bytes and declared ticks, or crash with a value.
    fn try_run(
        &mut self,
        node: &DagNode,
        inputs: &[Vec<u8>],
        attempt: u8,
    ) -> Result<(Vec<u8>, Ticks), NodeCrash>;
}

impl<T: NodeRunner> FallibleRunner for T {
    fn try_run(
        &mut self,
        node: &DagNode,
        inputs: &[Vec<u8>],
        _attempt: u8,
    ) -> Result<(Vec<u8>, Ticks), NodeCrash> {
        Ok((self.run(node, inputs), Ticks(1)))
    }
}

/// One crash event, receipted and chained from the geometry hash — the
/// chain proves recovery followed the DERIVED map, not improvisation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrashReceipt {
    /// Crashing node.
    pub node_id: String,
    /// Named class the crash landed in.
    pub class: FailureClass,
    /// Whether a derived branch matched (false = GeometryGap).
    pub matched: bool,
    /// The response actually taken (rendered).
    pub response: String,
    /// Attempt index of the crashing execution.
    pub attempt: u8,
    /// Chain value after folding this receipt's frame.
    pub chain: String,
}

/// What finally happened to a node in a supervised run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Disposition {
    /// Executed (possibly after restarts).
    Completed {
        /// Restarts consumed.
        restarts: u8,
    },
    /// Parked, with cause.
    Parked {
        /// Why.
        cause: ParkCause,
    },
    /// Skipped because an ancestor was parked or gave up.
    SkippedBy {
        /// The ancestor node.
        ancestor: String,
    },
    /// Restart intensity exhausted — lawful surrender.
    GaveUp,
}

/// Terminal outcome of a supervised run. `GaveUp`/`Refused` are lawful
/// results, not errors: the receipt carries them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunOutcome {
    /// Every non-parked node completed.
    Completed,
    /// A node exhausted intensity; it and its cohort carry dispositions.
    GaveUp {
        /// The surrendering node.
        node_id: String,
    },
    /// A geometry branch prescribed refusal.
    Refused {
        /// The refusing node.
        node_id: String,
        /// The certificate core.
        core: Vec<String>,
    },
    /// A geometry branch escalated beyond this executor.
    Escalated {
        /// The escalating node.
        node_id: String,
    },
}

/// The supervised receipt: everything the run did, crash chain included.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupervisedReceipt {
    /// The derived topology in force.
    pub topology_hash: String,
    /// The derived geometry in force (crash-chain anchor).
    pub geometry_hash: String,
    /// Receipts for nodes that executed.
    pub node_receipts: Vec<NodeReceipt>,
    /// Order-independent root over executed nodes.
    pub root_hash: String,
    /// Nodes served from memo.
    pub replayed_count: usize,
    /// The crash chain, in event order.
    pub crash_receipts: Vec<CrashReceipt>,
    /// Final disposition per node.
    pub dispositions: BTreeMap<String, Disposition>,
    /// Total restart attempts consumed.
    pub restarts_total: u32,
    /// True iff every crash landed in a derived branch (no gaps).
    pub geometry_conformance: bool,
    /// Terminal outcome.
    pub outcome: RunOutcome,
}

impl Dag {
    /// Execute under supervision: crashes classify into the derived
    /// geometry's named branches and the prescribed responses actuate.
    /// Journals to the WAL when given (memo frames + park records share it).
    #[allow(clippy::too_many_lines, clippy::too_many_arguments)]
    pub fn execute_supervised(
        &self,
        topology: &SupervisionTopology,
        geometry: &FailureGeometry,
        runner: &mut dyn FallibleRunner,
        cache: &mut MemoCache,
        parks: &mut ParkManager,
        mut wal: Option<&mut crate::wal::Wal>,
        run_index: u64,
    ) -> Result<SupervisedReceipt, Refusal> {
        let order = self.topo_order()?;
        // Re-admission pass: parked nodes whose policy fires rejoin the run.
        let readmitted = parks.readmit(run_index, |node_id| {
            self.nodes.get(node_id).map(|n| {
                let mut ins: Vec<&str> = n.inputs.iter().map(String::as_str).collect();
                ins.sort_unstable();
                content_address(ins.join("\n").as_bytes())
            })
        });
        let _ = readmitted; // rejoining = simply not being parked anymore

        let mut outputs: HashMap<String, (String, Vec<u8>)> = HashMap::new();
        let mut chain = genesis_seed(DAG_CHAIN_DOMAIN);
        let mut crash_chain = fold_event(
            &genesis_seed(GEOMETRY_CHAIN_DOMAIN),
            geometry.geometry_hash.as_bytes(),
        );
        let mut node_receipts = Vec::new();
        let mut crash_receipts: Vec<CrashReceipt> = Vec::new();
        let mut dispositions: BTreeMap<String, Disposition> = BTreeMap::new();
        let mut replayed_count = 0usize;
        let mut restarts_total = 0u32;
        let mut conformance = true;
        let mut outcome = RunOutcome::Completed;
        let mut halted = false;

        'nodes: for id in &order {
            if halted {
                break;
            }
            let node = &self.nodes[id];
            // Skip: already parked, or fed by a parked/skipped/given-up node.
            if parks.is_parked(id) {
                dispositions.insert(
                    id.clone(),
                    Disposition::Parked {
                        cause: ParkCause::CrashLoop,
                    },
                );
                continue;
            }
            if let Some(bad) = node.inputs.iter().find(|i| {
                parks.is_parked(i)
                    || matches!(
                        dispositions.get(*i),
                        Some(
                            Disposition::Parked { .. }
                                | Disposition::SkippedBy { .. }
                                | Disposition::GaveUp
                        )
                    )
            }) {
                dispositions.insert(
                    id.clone(),
                    Disposition::SkippedBy {
                        ancestor: bad.clone(),
                    },
                );
                continue;
            }

            let a_hash = action_hash(&node.action);
            let mut input_hashes: Vec<String> =
                node.inputs.iter().map(|i| outputs[i].0.clone()).collect();
            input_hashes.sort_unstable();
            let key = memo_key(&a_hash, &input_hashes);

            let mut attempt: u8 = 0;
            let mut last_kind: Option<CrashKind> = None;
            let (output, ticks, replayed) = loop {
                if let Some(cached) = cache.entries.get(&key) {
                    break (cached.clone(), Ticks(0), true);
                }
                let input_bytes: Vec<Vec<u8>> =
                    node.inputs.iter().map(|i| outputs[i].1.clone()).collect();
                match runner.try_run(node, &input_bytes, attempt) {
                    Ok((out, ticks)) => {
                        if let Some(w) = wal.as_deref_mut() {
                            w.append(&key, &out)?;
                        }
                        cache.entries.insert(key.clone(), out.clone());
                        break (out, ticks, false);
                    }
                    Err(crash) => {
                        let snapshot = CrashSnapshot {
                            node_id: id.clone(),
                            attempt,
                            ticks_used: match &crash {
                                NodeCrash::OverBudget { ticks } => *ticks,
                                _ => 0,
                            },
                            tick_budget: Some(crate::budget::CHATMAN_CONSTANT),
                            tier: crate::fault::RuntimeClass::W1,
                            kind: crash.kind(),
                            refusal_head: match &crash {
                                NodeCrash::Refused { rendered } => Some(rendered.clone()),
                                _ => None,
                            },
                            upstream_parked: node.inputs.iter().any(|i| parks.is_parked(i)),
                            progressed: last_kind != Some(crash.kind()),
                        };
                        last_kind = Some(crash.kind());
                        let Classification {
                            class,
                            response,
                            matched,
                        } = geometry.classify(&snapshot);
                        conformance &= matched;
                        let frame = serde_json::json!({
                            "node_id": id,
                            "class": class,
                            "matched": matched,
                            "response": format!("{response:?}"),
                            "attempt": attempt,
                        })
                        .to_string();
                        crash_chain = fold_event(&crash_chain, frame.as_bytes());
                        crash_receipts.push(CrashReceipt {
                            node_id: id.clone(),
                            class,
                            matched,
                            response: format!("{response:?}"),
                            attempt,
                            chain: crash_chain.clone(),
                        });
                        match response {
                            LawfulResponse::Restart => {
                                restarts_total += 1;
                                if u32::from(attempt) + 1 >= u32::from(topology.policy.max_restarts)
                                {
                                    // Lawful surrender: receipt, never error.
                                    dispositions.insert(id.clone(), Disposition::GaveUp);
                                    outcome = RunOutcome::GaveUp {
                                        node_id: id.clone(),
                                    };
                                    continue 'nodes;
                                }
                                attempt += 1;
                            }
                            LawfulResponse::Park(readmission) => {
                                let cause = match class {
                                    FailureClass::BudgetBreach => ParkCause::TickBudgetExceeded,
                                    FailureClass::StarvedInput => ParkCause::UpstreamParked,
                                    _ => ParkCause::CrashLoop,
                                };
                                let mut ins: Vec<&str> =
                                    node.inputs.iter().map(String::as_str).collect();
                                ins.sort_unstable();
                                parks.park(
                                    ParkedEntry {
                                        node_id: id.clone(),
                                        cause: cause.clone(),
                                        readmission,
                                        parked_at_run: run_index,
                                        input_fingerprint: content_address(
                                            ins.join("\n").as_bytes(),
                                        ),
                                    },
                                    wal.as_deref_mut(),
                                )?;
                                dispositions.insert(id.clone(), Disposition::Parked { cause });
                                continue 'nodes;
                            }
                            LawfulResponse::Refuse { core } => {
                                outcome = RunOutcome::Refused {
                                    node_id: id.clone(),
                                    core,
                                };
                                halted = true;
                                continue 'nodes;
                            }
                            LawfulResponse::Escalate => {
                                outcome = RunOutcome::Escalated {
                                    node_id: id.clone(),
                                };
                                halted = true;
                                continue 'nodes;
                            }
                        }
                    }
                }
            };

            if replayed {
                replayed_count += 1;
            }
            let output_hash = content_address(&output);
            let mut map = std::collections::BTreeMap::new();
            map.insert("node_id", serde_json::Value::String(id.clone()));
            map.insert("action_hash", serde_json::Value::String(a_hash.clone()));
            map.insert(
                "input_hashes",
                serde_json::Value::Array(
                    input_hashes
                        .iter()
                        .map(|h| serde_json::Value::String(h.clone()))
                        .collect(),
                ),
            );
            map.insert(
                "output_hash",
                serde_json::Value::String(output_hash.clone()),
            );
            let frame = serde_json::to_string(&map).expect("serialization");
            chain = fold_event(&chain, frame.as_bytes());
            node_receipts.push(NodeReceipt {
                node_id: id.clone(),
                action_hash: a_hash,
                input_hashes,
                output_hash: output_hash.clone(),
                chain: chain.clone(),
                replayed,
                ticks_used: ticks.0,
                tick_budget: crate::budget::CHATMAN_CONSTANT,
            });
            dispositions.insert(id.clone(), Disposition::Completed { restarts: attempt });
            outputs.insert(id.clone(), (output_hash, output));
        }

        let mut pairs: Vec<String> = outputs
            .iter()
            .map(|(id, (h, _))| format!("{id}:{h}"))
            .collect();
        pairs.sort_unstable();
        let root_hash = content_address(pairs.join("\n").as_bytes());

        Ok(SupervisedReceipt {
            topology_hash: topology.topology_hash.clone(),
            geometry_hash: geometry.geometry_hash.clone(),
            node_receipts,
            root_hash,
            replayed_count,
            crash_receipts,
            dispositions,
            restarts_total,
            geometry_conformance: conformance,
            outcome,
        })
    }
}
