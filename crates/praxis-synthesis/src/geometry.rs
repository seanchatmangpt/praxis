//! Failure geometry — the crash space, derived before the crash.
//!
//! Armstrong made failure survivable when crash space was unknowable; the
//! LHC/EHT lesson is to predict the event signatures before the event and
//! reconstruct from lawful projections. Here the *planner's own analysis*
//! manufactures the failure map: every plan node gets an ordered list of
//! named branches (class, signal conjunction, lawful response), fragile
//! preconditions are detected from the same sole-producer analysis Solver8
//! uses for unsat certificates, and the whole geometry is content-addressed
//! so the crash-receipt chain can prove recovery followed the *derived* map
//! rather than improvisation.
//!
//! [`FailureClass::GeometryGap`] is the implicit no-match outcome — it is
//! never a stored branch, so it can never be shadowed or deleted. The map
//! admits its own incompleteness by construction.
//!
//! Class × tier is a matrix in data, not types: eight semantic classes
//! (the doctrine's cap), with knhk's R1/W1/C1 as an orthogonal
//! [`RuntimeClass`] tier carried on snapshots — fusing them would explode
//! to 24 variants.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::dag::Dag;
use crate::fault::RuntimeClass;
use crate::park::ReAdmission;
use crate::sequence::{SequencePlan, SequenceProblem};
use crate::supervise::SupervisionTopology;

/// The eight failure classes — the semantic axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureClass {
    /// The node's own computation is wrong (bad output, internal error).
    LogicFault,
    /// A budget was breached (ticks/time beyond the node's tier).
    BudgetBreach,
    /// A required authority/support fact is absent and nothing produces it.
    AuthorityVacuum,
    /// A transient environmental fault (I/O, resource blip).
    TransientFault,
    /// No progress across attempts without a crash.
    Stall,
    /// An upstream dependency was parked; inputs are starved.
    StarvedInput,
    /// The solver certified unsatisfiability at runtime.
    CertifiedUnsat,
    /// No derived branch matched — the honest residue. Never stored.
    GeometryGap,
}

/// What kind of crash the runner reported — the mechanical axis, mapped
/// from execution errors by the supervised executor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrashKind {
    /// Environmental/I-O failure.
    Io,
    /// The node produced output that failed its own validation.
    BadOutput,
    /// The node exceeded its budget.
    OverBudget,
    /// A precondition that held at planning time no longer holds.
    PreconditionLost,
    /// The node refused (carries a rendered refusal in the snapshot).
    Refused,
}

/// Everything the classifier may look at. One snapshot per crash event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashSnapshot {
    /// Content id of the crashing node.
    pub node_id: String,
    /// Zero-based attempt index (0 = first execution).
    pub attempt: u8,
    /// Declared ticks used by this attempt.
    pub ticks_used: u64,
    /// The node's tier budget in ticks (R1) — `None` for time tiers.
    pub tick_budget: Option<u64>,
    /// The node's runtime class (the tier axis).
    pub tier: RuntimeClass,
    /// The mechanical crash kind.
    pub kind: CrashKind,
    /// Rendered refusal head when `kind == Refused` (e.g. "unsat (certified)").
    pub refusal_head: Option<String>,
    /// Whether any transitive input of this node is currently parked.
    pub upstream_parked: bool,
    /// Whether this attempt made observable progress over the last.
    pub progressed: bool,
}

/// A signal predicate over a snapshot. Branch signals are conjunctions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureSignal {
    /// Attempt index at or above `n`.
    RetriesAtLeast(u8),
    /// Declared ticks above the node's tick budget (R1 tier).
    TicksAboveBudget,
    /// Crash kind equals.
    CrashKind(CrashKind),
    /// Refusal head starts with the given string.
    RefusalHeadIs(String),
    /// No observable progress this attempt.
    NoProgress,
    /// Some upstream dependency is parked.
    UpstreamParked,
}

impl FailureSignal {
    /// Evaluate against a snapshot.
    #[must_use]
    pub fn fires(&self, s: &CrashSnapshot) -> bool {
        match self {
            FailureSignal::RetriesAtLeast(n) => s.attempt >= *n,
            FailureSignal::TicksAboveBudget => s.tick_budget.is_some_and(|b| s.ticks_used > b),
            FailureSignal::CrashKind(k) => s.kind == *k,
            FailureSignal::RefusalHeadIs(h) => s
                .refusal_head
                .as_deref()
                .is_some_and(|r| r.starts_with(h.as_str())),
            FailureSignal::NoProgress => !s.progressed,
            FailureSignal::UpstreamParked => s.upstream_parked,
        }
    }
}

/// The four lawful verbs a branch may prescribe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LawfulResponse {
    /// Re-run the node's restart cohort (within intensity).
    Restart,
    /// Park the node with the stated way back.
    Park(ReAdmission),
    /// Halt the run with a certificate (named culprits).
    Refuse {
        /// The core: what makes continuation impossible.
        core: Vec<String>,
    },
    /// Outside this executor's lawful responses; hand upward.
    Escalate,
}

/// One named branch: class + signal conjunction + response. Ordered lists;
/// first match wins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeometryBranch {
    /// The named class a matching crash lands in.
    pub class: FailureClass,
    /// ALL signals must fire (conjunction).
    pub signals: Vec<FailureSignal>,
    /// The prescribed response.
    pub response: LawfulResponse,
}

impl GeometryBranch {
    fn matches(&self, s: &CrashSnapshot) -> bool {
        self.signals.iter().all(|sig| sig.fires(s))
    }
}

/// The derived failure geometry: per-node ordered branch lists, content-
/// addressed. [`FailureGeometry::derive`] is the only constructor.
// The `_sealed` field is stronger than `#[non_exhaustive]`: it forbids
// struct literals even inside this crate's other modules, so a geometry
// can never exist without passing through `derive`.
#[allow(clippy::manual_non_exhaustive)]
#[derive(Debug, Clone, Serialize)]
pub struct FailureGeometry {
    /// Branches per node id, in match order.
    pub branches: BTreeMap<String, Vec<GeometryBranch>>,
    /// Content address over the branch map + topology hash — the anchor of
    /// the crash-receipt chain (spec-hash == exec-hash, applied to failure).
    pub geometry_hash: String,
    #[serde(skip)]
    _sealed: (),
}

/// Outcome of classification: the named branch (or the honest gap).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Classification {
    /// The class the crash landed in.
    pub class: FailureClass,
    /// The prescribed response (`Restart` for gaps — the safe default,
    /// receipted as a gap).
    pub response: LawfulResponse,
    /// Whether a derived branch matched (false = GeometryGap).
    pub matched: bool,
}

impl FailureGeometry {
    /// Derive the geometry from the topology and the problem. Per node:
    ///
    /// 1. **AuthorityVacuum** branches for *fragile preconditions* — mined
    ///    with the same sole-producer analysis Solver8's certificates use:
    ///    a precondition predicate absent from the initial state with at
    ///    most one producing capability. If it is lost at runtime, nothing
    ///    can lawfully re-produce it → `Refuse` with the fact named.
    /// 2. **CertifiedUnsat** — a runtime refusal whose head is the
    ///    certified-unsat rendering → `Refuse` carrying the head.
    /// 3. **BudgetBreach** — ticks above budget → tier-dependent: R1
    ///    refuses (hot path never retries), W1/C1 park with `AfterRuns(1)`.
    /// 4. **StarvedInput** — upstream parked → park `OnInputChange`.
    /// 5. **Stall** — no progress by the second attempt → restart.
    /// 6. **TransientFault** — I/O crash → restart.
    /// 7. **LogicFault** — bad output persisting to the second attempt →
    ///    park `Manual` (a wrong computation does not heal by retry).
    ///
    /// `GeometryGap` is implicit: it is what classification returns when no
    /// branch matches, and it cannot appear in the stored lists.
    #[must_use]
    pub fn derive(
        topology: &SupervisionTopology,
        plan: &SequencePlan,
        problem: &SequenceProblem,
    ) -> Self {
        // Map node content ids back to their capabilities for fact mining.
        let dag = Dag::from_plan(plan, problem);
        let mut branches: BTreeMap<String, Vec<GeometryBranch>> = BTreeMap::new();
        for stage in &topology.stages {
            for node_id in &stage.nodes {
                let capability = dag.nodes.get(node_id).map(|n| n.action.capability.clone());
                branches.insert(
                    node_id.clone(),
                    Self::node_branches(capability.as_deref(), problem),
                );
            }
        }
        let canon = serde_json::json!({
            "branches": branches,
            "topology_hash": topology.topology_hash,
        });
        let geometry_hash =
            chatman_common::provenance::content_address(canon.to_string().as_bytes());
        Self {
            branches,
            geometry_hash,
            _sealed: (),
        }
    }

    fn node_branches(capability: Option<&str>, problem: &SequenceProblem) -> Vec<GeometryBranch> {
        let mut list = Vec::new();
        // 1. Fragile preconditions → AuthorityVacuum → Refuse{MissingFact}.
        let fragile = capability
            .map(|c| problem.fragile_precondition_names(c))
            .unwrap_or_default();
        for fact in fragile {
            list.push(GeometryBranch {
                class: FailureClass::AuthorityVacuum,
                signals: vec![FailureSignal::CrashKind(CrashKind::PreconditionLost)],
                response: LawfulResponse::Refuse {
                    core: vec![format!("MissingFact({fact})")],
                },
            });
        }
        // 2. Certified unsat at runtime.
        list.push(GeometryBranch {
            class: FailureClass::CertifiedUnsat,
            signals: vec![
                FailureSignal::CrashKind(CrashKind::Refused),
                FailureSignal::RefusalHeadIs("unsat (certified)".into()),
            ],
            response: LawfulResponse::Refuse {
                core: vec!["runtime unsat certificate".into()],
            },
        });
        // 3. Budget breach (tier-dependent response; the R1 variant is the
        //    hot-path discipline: never retry over budget).
        list.push(GeometryBranch {
            class: FailureClass::BudgetBreach,
            signals: vec![FailureSignal::TicksAboveBudget],
            response: LawfulResponse::Refuse {
                core: vec!["R1 tick budget breached on the hot path".into()],
            },
        });
        list.push(GeometryBranch {
            class: FailureClass::BudgetBreach,
            signals: vec![FailureSignal::CrashKind(CrashKind::OverBudget)],
            response: LawfulResponse::Park(ReAdmission::AfterRuns(1)),
        });
        // 4. Starved input.
        list.push(GeometryBranch {
            class: FailureClass::StarvedInput,
            signals: vec![FailureSignal::UpstreamParked],
            response: LawfulResponse::Park(ReAdmission::OnInputChange),
        });
        // 5. Stall.
        list.push(GeometryBranch {
            class: FailureClass::Stall,
            signals: vec![FailureSignal::RetriesAtLeast(1), FailureSignal::NoProgress],
            response: LawfulResponse::Restart,
        });
        // 6. Transient I/O.
        list.push(GeometryBranch {
            class: FailureClass::TransientFault,
            signals: vec![FailureSignal::CrashKind(CrashKind::Io)],
            response: LawfulResponse::Restart,
        });
        // 7. Persistent logic fault: retry does not heal wrong computation.
        list.push(GeometryBranch {
            class: FailureClass::LogicFault,
            signals: vec![
                FailureSignal::CrashKind(CrashKind::BadOutput),
                FailureSignal::RetriesAtLeast(1),
            ],
            response: LawfulResponse::Park(ReAdmission::Manual),
        });
        // First bad-output attempt gets one restart (could be transient).
        list.push(GeometryBranch {
            class: FailureClass::LogicFault,
            signals: vec![FailureSignal::CrashKind(CrashKind::BadOutput)],
            response: LawfulResponse::Restart,
        });
        list
    }

    /// Classify a crash: first matching branch wins; no match is the
    /// implicit, unshadowable [`FailureClass::GeometryGap`].
    #[must_use]
    pub fn classify(&self, snapshot: &CrashSnapshot) -> Classification {
        if let Some(list) = self.branches.get(&snapshot.node_id) {
            for branch in list {
                if branch.matches(snapshot) {
                    return Classification {
                        class: branch.class,
                        response: branch.response.clone(),
                        matched: true,
                    };
                }
            }
        }
        Classification {
            class: FailureClass::GeometryGap,
            response: LawfulResponse::Restart,
            matched: false,
        }
    }

    /// Total branches derived (all nodes).
    #[must_use]
    pub fn branch_count(&self) -> usize {
        self.branches.values().map(Vec::len).sum()
    }
}
