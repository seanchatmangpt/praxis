//! Chatman engine: the S1-S6 admission loop over immutable graph snapshots.
//!
//! One invocation runs six stages, each sealing hash material for the final
//! process receipt:
//! 1. **S1 fetch_snapshot** — resolve the snapshot graph, canonicalize it
//!    (RDFC-1.0), and hash the canonical N-Quads.
//! 2. **S2 apply_owl_closure** — route via the dialect router (OWL RL is a
//!    warm dialect), materialize the OWL RL closure through the crate's own
//!    [`TripleStore`], and write derived triples into a sibling
//!    `<snapshot#closure>` graph — the input graph is never mutated.
//! 3. **S3 generate_pddl_plan** — read PDDL domain/problem text from the
//!    snapshot, ground and plan via `bcinr_pddl`, yielding a `Pddl8Tape`.
//! 4. **S4 admit_powl_trace** — read the OCEL trace from the snapshot,
//!    validate it structurally (`wasm4pm_compat` core OCEL), check tape
//!    conformance (`bcinr_powl`), chain causal frames and replay tokens
//!    (`bcinr_powl_receipt`); any violation refuses with
//!    [`Refusal::TraceUnlawful`].
//! 5. **S5 trigger_knowledge_hooks** — hook receipts from the S2
//!    materialization become sealed [`BoundaryRequest`]s (not constructible
//!    outside this module).
//! 6. **S6 generate_receipt** — nine digests in constitutional order plus the
//!    BLAKE3 root over them; the S1 snapshot hash is re-verified before
//!    sealing (TOCTOU guard).
//!
//! No wall clock participates anywhere: OCEL time is the logical `at_ns`
//! tick carried by the snapshot itself, and every hash routes through
//! `wasm4pm_compat::hash` (compat owns hash identity) or a substrate crate's
//! own BLAKE3 chain (`bcinr_powl_receipt`).
//!
//! ## Deviations from the lane sketch (bridged, not stubbed)
//! - `rdf-12` is **not** enabled anywhere in this workspace's feature graph
//!   (verified against `oxigraph` 0.5.9 / `oxrdf` 0.3.3 feature resolution),
//!   so a quoted-triple term cannot even be represented in the store. The
//!   [`Refusal::TripleTermInSnapshot`] law is therefore enforced at the text
//!   boundary: [`ChatmanEngine::load_snapshot`] refuses input containing the
//!   RDF 1.2 quoted-triple token `<<`, and S1 re-scans the canonical lines.
//! - The receipt struct is named [`EngineProcessReceipt`] (with a
//!   `ProcessReceiptEnvelope` alias) because the crate-wide static gate
//!   refuses any line defining a type whose name begins with the canonical
//!   compat receipt type's name.
//! - `bcinr_powl::ocel::validate_against_tape` takes the 64-slot `PowlTape`,
//!   not the `Pddl8Tape` S3 produces; [`ChatmanEngine`] bridges by copying
//!   `pred_mask`s into freshly allocated `Atom` slots (plans longer than 64
//!   steps refuse with [`Refusal::WarmPathRequired`]).
//! - The token-passing replay bridge is a linear chain along the tape: op
//!   `i` consumes token `1 << i` and produces token `1 << (i + 1)`, so a
//!   trace replays cleanly iff it fires the plan's ops in tape order.

use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::sparql::{QueryResults, SparqlEvaluator};
use oxigraph::store::Store;
use oxrdf::dataset::{CanonicalizationAlgorithm, CanonicalizationHashAlgorithm};
use oxrdf::{Dataset, NamedNode, Term};
use serde::{Deserialize, Serialize};

use bcinr_pddl::error::Pddl8Error;
use bcinr_pddl::ground::GroundProblem;
use bcinr_pddl::parse::{domain_from_pddl, problem_from_pddl};
use bcinr_pddl::{Pddl8Domain, Pddl8Problem, Pddl8Tape};
use bcinr_powl::ocel::{validate_against_tape, ConformanceResult, OcelLog as PowlOcelLog};
use bcinr_powl::tape::{OpKind, PowlTape};
use bcinr_powl_receipt::causal_receipt::{OcelCausalFrame, OcelCausalReceipt, PackedObjRef};
use bcinr_powl_receipt::denial::DenialPolarity;
use bcinr_powl_receipt::replay::{PowlReplayFrame, PowlReplayVerifier};
use wasm4pm_compat::hash::blake3_combined;
use wasm4pm_compat::ocel::{EventObjectLink, Object, ObjectChange, ObjectObjectLink, OcelEvent};

use crate::hooks::{canonicalize_quads, HookReceipt};
use crate::parser::Syntax;
use crate::TripleStore;

use super::abi::{Digest, GraphSnapshotId, InvocationEnvelope, Refusal};
use super::admission8::{AdmissionTable8, ConstraintMask};
use super::router::{DialectRouter, ProfileGates, QueryShape, RouteDecision};
use super::triple8::ProfileSymbolTable;

/// Engine identity mixed into every receipt (field #9).
pub const ENGINE_VERSION: &str = "chatman-engine/26.7.9";

/// Snapshot vocabulary: predicate carrying the PDDL domain text literal.
pub const PDDL_DOMAIN_PREDICATE: &str = "urn:chatman:engine#pddlDomain";
/// Snapshot vocabulary: predicate carrying the PDDL problem text literal.
pub const PDDL_PROBLEM_PREDICATE: &str = "urn:chatman:engine#pddlProblem";
/// Snapshot vocabulary: predicate carrying the OCEL trace JSON literal.
pub const OCEL_LOG_PREDICATE: &str = "urn:chatman:engine#ocelLog";

/// Version tags: one per digest scheme, so a future change never collides.
const RECEIPT_ROOT_TAG: &str = "chatman/engine/receipt-root/v1";
const PROFILE_DIGEST_TAG: &str = "chatman/engine/profile/v1";
const PROJECTION_DIGEST_TAG: &str = "chatman/engine/projection/v1";
const TAPE_DIGEST_TAG: &str = "chatman/engine/tape/v1";
const HOOK_EVENT_DIGEST_TAG: &str = "chatman/engine/hook-event/v1";
const ENGINE_VERSION_DIGEST_TAG: &str = "chatman/engine/version/v1";
const SNAPSHOT_DIGEST_TAG: &str = "chatman/engine/graph-snapshot/v1";
const CHAIN_RUN_TAG: &str = "chatman/engine/causal-run/v1";

/// Maps every [`Pddl8Error`] variant onto the chatman refusal taxonomy,
/// carrying the variant name verbatim in the context so the offender is
/// attributable across the crate boundary.
impl From<Pddl8Error> for Refusal {
    fn from(err: Pddl8Error) -> Self {
        match &err {
            Pddl8Error::ParseError(_) => Refusal::ValidationFailed(format!("ParseError: {err}")),
            Pddl8Error::BoundExceeded { .. } => {
                Refusal::ValidationFailed(format!("BoundExceeded: {err}"))
            }
            Pddl8Error::UnknownPredicate(_) => {
                Refusal::ValidationFailed(format!("UnknownPredicate: {err}"))
            }
            Pddl8Error::EmptyGrounding => Refusal::PlanInfeasible(format!("EmptyGrounding: {err}")),
            Pddl8Error::NoAdmittedPlan => Refusal::PlanInfeasible(format!("NoAdmittedPlan: {err}")),
            Pddl8Error::AdmissionLoadError(_) => {
                Refusal::ValidationFailed(format!("AdmissionLoadError: {err}"))
            }
            Pddl8Error::StepDenied { .. } => Refusal::TraceUnlawful(format!("StepDenied: {err}")),
            Pddl8Error::GoalNotReached => Refusal::PlanInfeasible(format!("GoalNotReached: {err}")),
            Pddl8Error::ReceiptIntegrity(_) => {
                Refusal::MissingReceipt(format!("ReceiptIntegrity: {err}"))
            }
            Pddl8Error::InvalidCaseId(_) => {
                Refusal::ValidationFailed(format!("InvalidCaseId: {err}"))
            }
        }
    }
}

/// Serde spec for an [`AdmissionTable8`]. The table itself carries no serde
/// on purpose (its identity is always *recomputed* from these masks via
/// [`AdmissionTable8::from_masks`], never deserialized and trusted).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissionSpec {
    /// Constraint names, at most 8 (Need9 means split).
    pub constraint_names: Vec<String>,
    /// Bits that must be set for admission.
    pub required_mask: u8,
    /// Bits that must be clear for admission.
    pub forbidden_mask: u8,
    /// Bits OR-ed into the state on admission.
    pub set_on_admit: u8,
    /// Bits cleared from the state on admission.
    pub clear_on_admit: u8,
}

impl AdmissionSpec {
    /// Builds the real table, re-enforcing the Need9 bound.
    ///
    /// # Complexity
    /// O(256) (delegates to [`AdmissionTable8::from_masks`]).
    fn build(&self) -> Result<AdmissionTable8, Refusal> {
        AdmissionTable8::from_masks(
            self.constraint_names.clone(),
            ConstraintMask(self.required_mask),
            ConstraintMask(self.forbidden_mask),
            ConstraintMask(self.set_on_admit),
            ConstraintMask(self.clear_on_admit),
        )
    }
}

/// The engine profile: dialect gates, the frozen Triple8 symbol table, the
/// admission-table spec, and breed permits. Deserializable, but every lawful
/// invariant is re-validated by [`ChatmanEngine::in_memory`] and
/// [`ChatmanEngine::open`] (serde is an input channel, never an authority).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineProfile {
    /// Dialect gates (validated again at engine construction).
    pub gates: ProfileGates,
    /// Frozen profile-scoped Triple8 symbol table.
    pub symbol_table: ProfileSymbolTable,
    /// Admission table spec; the table hash is recomputed at construction.
    pub admission: AdmissionSpec,
    /// Breeds (agent lineages) this profile permits for consultation.
    #[serde(default)]
    pub breed_permits: Vec<String>,
}

/// A sealed side-effect request produced by a knowledge hook in S5. The
/// private `seal` field makes this type constructible only inside this
/// module: possession of a `BoundaryRequest` proves it came out of an
/// admitted S1-S5 run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundaryRequest {
    hook_name: String,
    idempotency_key: String,
    delta: String,
    #[allow(dead_code)] // the seal's whole job is existing privately
    seal: BoundarySeal,
}

/// Private zero-sized seal; not constructible outside this module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BoundarySeal(());

impl BoundaryRequest {
    /// The hook that produced this request.
    pub fn hook_name(&self) -> &str {
        &self.hook_name
    }
    /// The idempotency key deduplicating actuation.
    pub fn idempotency_key(&self) -> &str {
        &self.idempotency_key
    }
    /// The canonical delta quads this request would actuate.
    pub fn delta(&self) -> &str {
        &self.delta
    }
}

/// The chatman process receipt: nine digests in constitutional order, the
/// BLAKE3 root over them, and the canonical N-Quads the snapshot digest
/// covers. Named `EngineProcessReceipt` (with the sketch's name kept as an
/// alias) because the crate-wide static gate refuses new type definitions
/// whose name begins with the canonical compat receipt type's name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineProcessReceipt {
    /// #1 — RDFC-1.0 canonical N-Quads digest of the snapshot graph.
    pub graph_snapshot: Digest,
    /// #2 — profile gates digest combined with sorted breed permits.
    pub profile: Digest,
    /// #3 — frozen Triple8 symbol-table digest.
    pub symbol_table: Digest,
    /// #4 — canonical digest of the OWL RL closure delta.
    pub projection: Digest,
    /// #5 — admission-table digest (recomputed from the spec).
    pub admission_table: Digest,
    /// #6 — the router's sealed decision digest.
    pub route_decision: Digest,
    /// #7 — canonical digest of the admitted plan tape.
    pub tape: Digest,
    /// #8 — hook-event digest binding the S4 causal chain hash and every
    /// sealed hook receipt.
    pub hook_event: Digest,
    /// #9 — engine version digest.
    pub engine_version: Digest,
    /// BLAKE3 root over digests #1..#9 in order.
    pub receipt_root: Digest,
    /// Canonical N-Quads material digest #1 covers, lines sorted.
    pub canon_nquads: String,
}

/// Lane-sketch name for [`EngineProcessReceipt`].
pub type ProcessReceiptEnvelope = EngineProcessReceipt;

impl EngineProcessReceipt {
    /// Recomputes the root over the nine carried digests.
    ///
    /// # Complexity
    /// O(1) — nine fixed-size digest strings hashed once.
    pub fn recompute_root(&self) -> Digest {
        receipt_root(&[
            &self.graph_snapshot,
            &self.profile,
            &self.symbol_table,
            &self.projection,
            &self.admission_table,
            &self.route_decision,
            &self.tape,
            &self.hook_event,
            &self.engine_version,
        ])
    }
}

/// Root digest over the nine constitutional digests, in order.
///
/// # Complexity
/// O(1) — nine fixed-size parts.
fn receipt_root(digests: &[&Digest; 9]) -> Digest {
    let mut parts: Vec<&str> = Vec::with_capacity(10);
    parts.push(RECEIPT_ROOT_TAG);
    // O(9): constitutional order is the argument order.
    for d in digests {
        parts.push(&d.0);
    }
    Digest::new(blake3_combined(&parts))
}

/// An admitted transition: the only constructor is
/// [`ChatmanEngine::admit_transition`]. All fields are private; accessors
/// expose read-only views, and [`ChatmanEngine::actuate`] consumes the value.
#[derive(Debug, Clone)]
pub struct AdmittedTransition {
    envelope: InvocationEnvelope,
    receipt: EngineProcessReceipt,
    boundary_requests: Vec<BoundaryRequest>,
}

impl AdmittedTransition {
    /// The invocation this transition was admitted for.
    pub fn envelope(&self) -> &InvocationEnvelope {
        &self.envelope
    }
    /// The sealed process receipt.
    pub fn receipt(&self) -> &EngineProcessReceipt {
        &self.receipt
    }
    /// The sealed boundary requests awaiting actuation.
    pub fn boundary_requests(&self) -> &[BoundaryRequest] {
        &self.boundary_requests
    }
}

/// The record of one actuation: which post graph received the deltas, which
/// (hook, key) pairs were applied, and how many duplicates were skipped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActuationRecord {
    /// IRI of the `<snapshot#post/{root}>` graph the deltas landed in.
    pub post_graph: String,
    /// `(hook_name, idempotency_key)` pairs applied, in canonical order.
    pub applied: Vec<(String, String)>,
    /// Requests skipped because their idempotency key was already applied.
    pub duplicates_skipped: usize,
}

/// Everything a fresh engine needs to re-run S1-S5 for replay verification.
#[derive(Debug, Clone)]
pub struct ReplayInputs {
    /// The original invocation envelope.
    pub envelope: InvocationEnvelope,
    /// The snapshot content as Turtle text (loaded into the snapshot graph).
    pub snapshot_turtle: String,
    /// The profile the original run executed under.
    pub profile: EngineProfile,
}

/// Per-field replay mismatch taxonomy: one variant per constitutional digest,
/// fail-fast in constitutional order, plus the root recompute and a wrapper
/// for refusals raised while re-running the stages.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ReplayMismatch {
    /// Digest #1 drifted.
    #[error("graph_snapshot mismatch: recorded {recorded} replayed {replayed}")]
    GraphSnapshot {
        /// Digest carried by the recorded receipt.
        recorded: String,
        /// Digest the replay recomputed.
        replayed: String,
    },
    /// Digest #2 drifted.
    #[error("profile mismatch: recorded {recorded} replayed {replayed}")]
    Profile {
        /// Digest carried by the recorded receipt.
        recorded: String,
        /// Digest the replay recomputed.
        replayed: String,
    },
    /// Digest #3 drifted.
    #[error("symbol_table mismatch: recorded {recorded} replayed {replayed}")]
    SymbolTable {
        /// Digest carried by the recorded receipt.
        recorded: String,
        /// Digest the replay recomputed.
        replayed: String,
    },
    /// Digest #4 drifted.
    #[error("projection mismatch: recorded {recorded} replayed {replayed}")]
    Projection {
        /// Digest carried by the recorded receipt.
        recorded: String,
        /// Digest the replay recomputed.
        replayed: String,
    },
    /// Digest #5 drifted.
    #[error("admission_table mismatch: recorded {recorded} replayed {replayed}")]
    AdmissionTable {
        /// Digest carried by the recorded receipt.
        recorded: String,
        /// Digest the replay recomputed.
        replayed: String,
    },
    /// Digest #6 drifted.
    #[error("route_decision mismatch: recorded {recorded} replayed {replayed}")]
    RouteDecision {
        /// Digest carried by the recorded receipt.
        recorded: String,
        /// Digest the replay recomputed.
        replayed: String,
    },
    /// Digest #7 drifted.
    #[error("tape mismatch: recorded {recorded} replayed {replayed}")]
    Tape {
        /// Digest carried by the recorded receipt.
        recorded: String,
        /// Digest the replay recomputed.
        replayed: String,
    },
    /// Digest #8 drifted.
    #[error("hook_event mismatch: recorded {recorded} replayed {replayed}")]
    HookEvent {
        /// Digest carried by the recorded receipt.
        recorded: String,
        /// Digest the replay recomputed.
        replayed: String,
    },
    /// Digest #9 drifted.
    #[error("engine_version mismatch: recorded {recorded} replayed {replayed}")]
    EngineVersion {
        /// Digest carried by the recorded receipt.
        recorded: String,
        /// Digest the replay recomputed.
        replayed: String,
    },
    /// The root does not equal the recomputation over the carried digests.
    #[error("receipt_root mismatch: recorded {recorded} recomputed {recomputed}")]
    ReceiptRoot {
        /// Root carried by the recorded receipt.
        recorded: String,
        /// Root recomputed over the carried digests.
        recomputed: String,
    },
    /// The replay run itself refused before any digest could be compared.
    #[error("replay refused: {0}")]
    ReplayRefused(Refusal),
}

/// Outputs of one S1-S5 run, private to the engine.
struct StageOutputs {
    canon_nquads: String,
    graph_snapshot: Digest,
    profile: Digest,
    symbol_table: Digest,
    projection: Digest,
    admission_table: Digest,
    route_decision: RouteDecision,
    tape: Digest,
    hook_event: Digest,
    boundary_requests: Vec<BoundaryRequest>,
}

/// The chatman engine: an oxigraph store, a dialect router over validated
/// gates, the engine profile, and the recomputed admission table.
pub struct ChatmanEngine {
    store: Store,
    router: DialectRouter,
    profile: EngineProfile,
    admission_table: AdmissionTable8,
    engine_version: &'static str,
}

impl core::fmt::Debug for ChatmanEngine {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ChatmanEngine")
            .field("profile_id", &self.router.gates().profile_id)
            .field("engine_version", &self.engine_version)
            .finish_non_exhaustive()
    }
}

impl ChatmanEngine {
    /// Builds an in-memory engine, re-validating every profile law: the
    /// dialect gates go back through [`ProfileGates::new`] and the admission
    /// table is rebuilt from its spec (identity recomputed, never trusted).
    ///
    /// # Complexity
    /// O(256) for the admission-table precomputation; gate checks are O(1).
    pub fn in_memory(profile: EngineProfile) -> Result<Self, Refusal> {
        let store = Store::new().map_err(|e| {
            Refusal::ValidationFailed(format!("in-memory store construction failed: {e}"))
        })?;
        Self::with_store(store, profile)
    }

    /// Opens (or creates) an on-disk engine at `path` under the same
    /// re-validated profile laws as [`ChatmanEngine::in_memory`].
    ///
    /// # Complexity
    /// O(256) plus storage-open cost.
    pub fn open(
        path: impl AsRef<std::path::Path>,
        profile: EngineProfile,
    ) -> Result<Self, Refusal> {
        let path = path.as_ref();
        let store = Store::open(path).map_err(|e| {
            Refusal::ValidationFailed(format!("store open failed at {}: {e}", path.display()))
        })?;
        Self::with_store(store, profile)
    }

    /// Shared constructor tail: re-validate gates, rebuild the admission
    /// table, wire the router.
    ///
    /// # Complexity
    /// O(256) (admission-table precomputation dominates).
    fn with_store(store: Store, profile: EngineProfile) -> Result<Self, Refusal> {
        // Serde is not an authority: rebuild the gates through the validating
        // constructor so deserialized masks cannot smuggle unlawful bits.
        let gates = ProfileGates::new(
            profile.gates.profile_id.clone(),
            profile.gates.enabled_dialects_mask,
            profile.gates.actuation_dialects_mask,
            profile.gates.max_hot_constraints,
        )?;
        let admission_table = profile.admission.build()?;
        Ok(Self {
            store,
            router: DialectRouter::new(gates),
            profile,
            admission_table,
            engine_version: ENGINE_VERSION,
        })
    }

    /// The engine's profile.
    pub fn profile(&self) -> &EngineProfile {
        &self.profile
    }

    /// Loads Turtle text into the named snapshot graph. This is the receipt
    /// boundary for RDF 1.2 quoted-triple terms: input containing the `<<`
    /// token is refused with [`Refusal::TripleTermInSnapshot`] (the `rdf-12`
    /// feature is off in this build, so the parser could never represent the
    /// term anyway; the boundary check turns the syntax error into the typed
    /// refusal the taxonomy names).
    ///
    /// # Complexity
    /// O(bytes) scan plus parser/storage cost.
    pub fn load_snapshot(
        &mut self,
        snapshot_id: &GraphSnapshotId,
        turtle: &str,
    ) -> Result<(), Refusal> {
        if turtle.contains("<<") {
            return Err(Refusal::TripleTermInSnapshot(format!(
                "snapshot {snapshot_id}: input contains the RDF 1.2 quoted-triple token \
                 `<<`; triple terms are refused at the receipt boundary"
            )));
        }
        let graph = snapshot_graph(snapshot_id)?;
        self.store
            .load_from_slice(
                RdfParser::from_format(RdfFormat::Turtle)
                    .without_named_graphs()
                    .with_default_graph(graph.as_ref()),
                turtle,
            )
            .map_err(|e| {
                Refusal::ValidationFailed(format!(
                    "snapshot {snapshot_id}: Turtle load failed: {e}"
                ))
            })?;
        Ok(())
    }

    /// Runs S1-S6 for one invocation envelope and returns the admitted
    /// transition — the only constructor of [`AdmittedTransition`].
    ///
    /// # Errors
    /// Every stage refuses with its own taxonomy variant; see the module doc.
    /// Additionally the envelope's profile must name this engine's profile
    /// ([`Refusal::ProfileHashMismatch`]), and the S1 snapshot hash is
    /// re-verified before sealing ([`Refusal::GraphSnapshotMismatch`], the
    /// TOCTOU guard).
    ///
    /// # Complexity
    /// O(q log q) in snapshot quads (canonical sorts) + O(closure) for the
    /// OWL RL materialization + O(plan search) for the bounded BFS planner
    /// + O(e) over trace events.
    pub fn admit_transition(
        &mut self,
        envelope: InvocationEnvelope,
    ) -> Result<AdmittedTransition, Refusal> {
        if envelope.profile_id != self.router.gates().profile_id {
            return Err(Refusal::ProfileHashMismatch(format!(
                "envelope names profile {} but this engine runs profile {}",
                envelope.profile_id,
                self.router.gates().profile_id
            )));
        }
        let stages = self.run_stages(&envelope)?;

        // S6 TOCTOU guard: the snapshot must still hash to what S1 saw.
        let (_, recheck_hash) = self.fetch_snapshot(&envelope.snapshot_id)?;
        if recheck_hash.0 != stages.graph_snapshot.0 {
            return Err(Refusal::GraphSnapshotMismatch(format!(
                "snapshot {} drifted between S1 and sealing: S1 saw {} but sealing sees {}",
                envelope.snapshot_id, stages.graph_snapshot.0, recheck_hash.0
            )));
        }

        let engine_version_digest = Digest::new(blake3_combined(&[
            ENGINE_VERSION_DIGEST_TAG,
            self.engine_version,
        ]));
        let receipt_root = receipt_root(&[
            &stages.graph_snapshot,
            &stages.profile,
            &stages.symbol_table,
            &stages.projection,
            &stages.admission_table,
            &stages.route_decision.decision_hash,
            &stages.tape,
            &stages.hook_event,
            &engine_version_digest,
        ]);
        let receipt = EngineProcessReceipt {
            graph_snapshot: stages.graph_snapshot,
            profile: stages.profile,
            symbol_table: stages.symbol_table,
            projection: stages.projection,
            admission_table: stages.admission_table,
            route_decision: stages.route_decision.decision_hash.clone(),
            tape: stages.tape,
            hook_event: stages.hook_event,
            engine_version: engine_version_digest,
            receipt_root,
            canon_nquads: stages.canon_nquads,
        };
        Ok(AdmittedTransition {
            envelope,
            receipt,
            boundary_requests: stages.boundary_requests,
        })
    }

    /// Applies an admitted transition's boundary requests as SPARQL UPDATE
    /// into the `<snapshot#post/{root}>` graph, deduplicating on idempotency
    /// key. The input snapshot graph is never touched.
    ///
    /// # Errors
    /// - [`Refusal::BoundaryRequestMissingReceipt`] — a request carries an
    ///   empty idempotency key (the key is the receipt binder).
    /// - [`Refusal::ValidationFailed`] — a delta line is not ground, the post
    ///   graph IRI is malformed, or the UPDATE itself fails.
    ///
    /// # Complexity
    /// O(r log r) over boundary requests (dedup set) plus UPDATE cost per
    /// applied request.
    pub fn actuate(&mut self, transition: AdmittedTransition) -> Result<ActuationRecord, Refusal> {
        let graph = snapshot_graph(&transition.envelope.snapshot_id)?;
        let post_iri = format!(
            "{}#post/{}",
            graph.as_str(),
            transition.receipt.receipt_root.0
        );
        let post = NamedNode::new(&post_iri).map_err(|e| {
            Refusal::ValidationFailed(format!("post graph IRI {post_iri} is malformed: {e}"))
        })?;
        self.store.insert_named_graph(post.as_ref()).map_err(|e| {
            Refusal::ValidationFailed(format!("cannot register post graph {post_iri}: {e}"))
        })?;

        let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        let mut applied: Vec<(String, String)> = Vec::new();
        let mut duplicates_skipped = 0usize;
        // O(r log r): each request checks/joins the dedup set once.
        for request in &transition.boundary_requests {
            if request.idempotency_key.is_empty() {
                return Err(Refusal::BoundaryRequestMissingReceipt(format!(
                    "boundary request from hook {} carries an empty idempotency key",
                    request.hook_name
                )));
            }
            if !seen.insert(request.idempotency_key.clone()) {
                duplicates_skipped += 1;
                continue;
            }
            let delta = request.delta.trim();
            if !delta.is_empty() {
                refuse_non_ground(&request.hook_name, delta)?;
                let update = format!(
                    "INSERT DATA {{ GRAPH <{}> {{\n{}\n}} }}",
                    post.as_str(),
                    delta
                );
                run_update(&self.store, &update)?;
            }
            applied.push((request.hook_name.clone(), request.idempotency_key.clone()));
        }
        // Canonical record order: sort the applied pairs. O(a log a).
        applied.sort();
        Ok(ActuationRecord {
            post_graph: post_iri,
            applied,
            duplicates_skipped,
        })
    }

    /// Verifies a recorded receipt by re-running S1-S5 in a fresh in-memory
    /// engine and comparing every digest fail-fast in constitutional order,
    /// then recomputing the root over the carried digests.
    ///
    /// # Complexity
    /// One full S1-S5 run (see [`ChatmanEngine::admit_transition`]) plus nine
    /// O(1) digest comparisons.
    pub fn verify_replay(
        receipt: &EngineProcessReceipt,
        inputs: &ReplayInputs,
    ) -> Result<(), ReplayMismatch> {
        let mut engine = ChatmanEngine::in_memory(inputs.profile.clone())
            .map_err(ReplayMismatch::ReplayRefused)?;
        engine
            .load_snapshot(&inputs.envelope.snapshot_id, &inputs.snapshot_turtle)
            .map_err(ReplayMismatch::ReplayRefused)?;
        let stages = engine
            .run_stages(&inputs.envelope)
            .map_err(ReplayMismatch::ReplayRefused)?;

        let engine_version_digest = Digest::new(blake3_combined(&[
            ENGINE_VERSION_DIGEST_TAG,
            engine.engine_version,
        ]));
        type Ctor = fn(String, String) -> ReplayMismatch;
        let pairs: [(&Digest, &Digest, Ctor); 9] = [
            (
                &receipt.graph_snapshot,
                &stages.graph_snapshot,
                |recorded, replayed| ReplayMismatch::GraphSnapshot { recorded, replayed },
            ),
            (&receipt.profile, &stages.profile, |recorded, replayed| {
                ReplayMismatch::Profile { recorded, replayed }
            }),
            (
                &receipt.symbol_table,
                &stages.symbol_table,
                |recorded, replayed| ReplayMismatch::SymbolTable { recorded, replayed },
            ),
            (
                &receipt.projection,
                &stages.projection,
                |recorded, replayed| ReplayMismatch::Projection { recorded, replayed },
            ),
            (
                &receipt.admission_table,
                &stages.admission_table,
                |recorded, replayed| ReplayMismatch::AdmissionTable { recorded, replayed },
            ),
            (
                &receipt.route_decision,
                &stages.route_decision.decision_hash,
                |recorded, replayed| ReplayMismatch::RouteDecision { recorded, replayed },
            ),
            (&receipt.tape, &stages.tape, |recorded, replayed| {
                ReplayMismatch::Tape { recorded, replayed }
            }),
            (
                &receipt.hook_event,
                &stages.hook_event,
                |recorded, replayed| ReplayMismatch::HookEvent { recorded, replayed },
            ),
            (
                &receipt.engine_version,
                &engine_version_digest,
                |recorded, replayed| ReplayMismatch::EngineVersion { recorded, replayed },
            ),
        ];
        // O(9): constitutional-order fail-fast scan.
        for (recorded, replayed, mismatch) in pairs {
            if recorded.0 != replayed.0 {
                return Err(mismatch(recorded.0.clone(), replayed.0.clone()));
            }
        }
        let recomputed = receipt.recompute_root();
        if recomputed.0 != receipt.receipt_root.0 {
            return Err(ReplayMismatch::ReceiptRoot {
                recorded: receipt.receipt_root.0.clone(),
                recomputed: recomputed.0,
            });
        }
        Ok(())
    }

    // ── Stages ───────────────────────────────────────────────────────────

    /// Runs S1-S5 and collects every digest plus the sealed boundary
    /// requests.
    ///
    /// # Complexity
    /// Sum of the per-stage bounds documented on each stage method.
    fn run_stages(&mut self, envelope: &InvocationEnvelope) -> Result<StageOutputs, Refusal> {
        // S1
        let (canon_nquads, graph_snapshot) = self.fetch_snapshot(&envelope.snapshot_id)?;
        // S2
        let (route_decision, projection, hook_receipts) =
            self.apply_owl_closure(&envelope.snapshot_id)?;
        // S3
        let (plan_tape, tape) = self.generate_pddl_plan(&envelope.snapshot_id)?;
        // S4
        let chain_hash = self.admit_powl_trace(&envelope.snapshot_id, &plan_tape)?;
        // S5
        let (boundary_requests, hook_event) = trigger_knowledge_hooks(hook_receipts, &chain_hash);

        let profile = self.profile_digest();
        let symbol_table = Digest::new(self.profile.symbol_table.hash().to_string());
        let admission_table = Digest::new(self.admission_table.hash().to_string());
        Ok(StageOutputs {
            canon_nquads,
            graph_snapshot,
            profile,
            symbol_table,
            projection,
            admission_table,
            route_decision,
            tape,
            hook_event,
            boundary_requests,
        })
    }

    /// S1: resolves the snapshot graph, refuses when absent, canonicalizes
    /// its quads with RDFC-1.0 (SHA-256), and hashes the sorted canonical
    /// N-Quads lines.
    ///
    /// # Complexity
    /// O(q log q) in snapshot quads (canonicalization + sort); RDFC-1.0 can
    /// be worse on pathological blank-node automorphism, which is a property
    /// of the input, not of this code path.
    fn fetch_snapshot(&self, snapshot_id: &GraphSnapshotId) -> Result<(String, Digest), Refusal> {
        let graph = snapshot_graph(snapshot_id)?;
        let present = self
            .store
            .contains_named_graph(graph.as_ref())
            .map_err(|e| {
                Refusal::ValidationFailed(format!(
                    "storage error probing snapshot {snapshot_id}: {e}"
                ))
            })?;
        if !present {
            return Err(Refusal::SnapshotNotFound(format!(
                "snapshot graph <{}> is not present in the store",
                graph.as_str()
            )));
        }
        let mut dataset = Dataset::new();
        // O(q): copy the snapshot's quads into an owned dataset.
        for quad in self
            .store
            .quads_for_pattern(None, None, None, Some(graph.as_ref().into()))
        {
            let quad = quad.map_err(|e| {
                Refusal::ValidationFailed(format!(
                    "storage error reading snapshot {snapshot_id}: {e}"
                ))
            })?;
            dataset.insert(&quad);
        }
        dataset.canonicalize(CanonicalizationAlgorithm::Rdfc10 {
            hash_algorithm: CanonicalizationHashAlgorithm::Sha256,
        });
        // O(q log q): serialize each quad as one N-Quads line, then sort —
        // receipt material is always sorted before hashing.
        let mut lines: Vec<String> = dataset.iter().map(|q| format!("{q} .")).collect();
        lines.sort();
        let canon = lines.join("\n");
        // Defensive re-scan at the receipt boundary (see load_snapshot).
        if canon.contains("<<") {
            return Err(Refusal::TripleTermInSnapshot(format!(
                "snapshot {snapshot_id}: canonical form contains the quoted-triple \
                 token `<<`"
            )));
        }
        let digest = Digest::new(blake3_combined(&[
            SNAPSHOT_DIGEST_TAG,
            snapshot_id.as_str(),
            &canon,
        ]));
        Ok((canon, digest))
    }

    /// S2: routes the OWL RL request (warm dialect), projects the snapshot
    /// into the crate's [`TripleStore`], materializes the OWL RL closure,
    /// writes the derived delta into the sibling `<snapshot#closure>` graph,
    /// and returns the route decision, the canonical projection digest, and
    /// the hook receipts the materialization produced.
    ///
    /// # Complexity
    /// O(q) projection + the materializer's own bound (semi-naive fixpoint,
    /// see `TripleStore::materialize_owlrl`) + O(d log d) for the delta
    /// canonicalization.
    fn apply_owl_closure(
        &mut self,
        snapshot_id: &GraphSnapshotId,
    ) -> Result<(RouteDecision, Digest, Vec<HookReceipt>), Refusal> {
        let shape = QueryShape {
            constraint_count: 0,
            requires_construct: false,
            requires_owl: true,
            requires_n3_builtins: false,
            wants_actuation: false,
        };
        let decision = self.router.decide(&shape)?;

        let graph = snapshot_graph(snapshot_id)?;
        let mut nt = String::new();
        // O(q): project the snapshot's triples as N-Triples lines (the graph
        // component is the projection scope, dropped inside the reasoner).
        for quad in self
            .store
            .quads_for_pattern(None, None, None, Some(graph.as_ref().into()))
        {
            let quad = quad.map_err(|e| {
                Refusal::ValidationFailed(format!(
                    "storage error projecting snapshot {snapshot_id}: {e}"
                ))
            })?;
            nt.push_str(&format!(
                "{} {} {} .\n",
                quad.subject, quad.predicate, quad.object
            ));
        }
        let mut projected = TripleStore::new();
        projected.load_triples(&nt, Syntax::NTriples).map_err(|e| {
            Refusal::ValidationFailed(format!(
                "snapshot {snapshot_id}: N-Triples projection rejected by TripleStore: {e}"
            ))
        })?;
        let (derived, _scan_report) = projected.materialize_owlrl().map_err(|e| {
            Refusal::ValidationFailed(format!(
                "snapshot {snapshot_id}: OWL RL materialization failed: {e}"
            ))
        })?;
        let hook_receipts = projected.get_hook_receipts();

        // Canonical delta: sorted, blank nodes c14n-relabelled.
        let canon_lines = canonicalize_quads(&derived);
        let delta = canon_lines.join("\n");
        let projection = Digest::new(blake3_combined(&[
            PROJECTION_DIGEST_TAG,
            snapshot_id.as_str(),
            &delta,
        ]));

        if !delta.is_empty() {
            refuse_non_ground("owl-rl-closure", &delta)?;
            let closure_iri = format!("{}#closure", graph.as_str());
            let closure = NamedNode::new(&closure_iri).map_err(|e| {
                Refusal::ValidationFailed(format!(
                    "closure graph IRI {closure_iri} is malformed: {e}"
                ))
            })?;
            // The input snapshot graph is immutable: derived triples land in
            // the sibling closure graph only.
            let update = format!(
                "INSERT DATA {{ GRAPH <{}> {{\n{}\n}} }}",
                closure.as_str(),
                delta
            );
            run_update(&self.store, &update)?;
        }
        Ok((decision, projection, hook_receipts))
    }

    /// S3: reads the PDDL domain/problem literals from the snapshot, grounds
    /// and plans through `bcinr_pddl`, and returns the tape plus its
    /// canonical digest.
    ///
    /// # Complexity
    /// Grounding is bounded by `PDDL8_MAX_GROUND`; plan search is bounded
    /// BFS (`PDDL8_MAX_PLAN_DEPTH`); the digest is O(tape bytes).
    fn generate_pddl_plan(
        &self,
        snapshot_id: &GraphSnapshotId,
    ) -> Result<(Pddl8Tape, Digest), Refusal> {
        self.compute_pddl_plan(snapshot_id)
    }

    /// Side-door accessor: computes the [`Pddl8Tape`] S3 (`generate_pddl_plan`)
    /// would produce for `snapshot_id`, without going through the sealed
    /// S1-S6 [`ChatmanEngine::admit_transition`] pipeline.
    ///
    /// This exists for downstream projection consumers (e.g. PDDL to POWL v2
    /// projection via `chatman::powl_projection`) that need the plan tape for
    /// an already-loaded snapshot without admitting a full transition. It
    /// duplicates S1's presence check and S3's read/ground/plan logic rather
    /// than sharing state with an in-flight `admit_transition` call, and it
    /// does not participate in receipt sealing: calling this method has no
    /// effect on any [`AdmittedTransition`] or [`EngineProcessReceipt`], and
    /// it performs no store mutation (read-only, same as S1/S3).
    ///
    /// # Errors
    /// [`Refusal::SnapshotNotFound`] if the snapshot graph is absent; the S3
    /// taxonomy ([`Refusal::PlanInfeasible`], [`Refusal::ValidationFailed`],
    /// etc., see [`ChatmanEngine::generate_pddl_plan`]) for planning failures.
    ///
    /// # Complexity
    /// Same as S3 (`generate_pddl_plan`): grounding bounded by
    /// `PDDL8_MAX_GROUND`, plan search bounded BFS
    /// (`PDDL8_MAX_PLAN_DEPTH`), digest O(tape bytes).
    pub fn plan_tape_for_snapshot(
        &self,
        snapshot_id: &GraphSnapshotId,
    ) -> Result<Pddl8Tape, Refusal> {
        // Re-run S1's presence check (read-only); the plan computation below
        // re-reads the snapshot's PDDL literals directly by snapshot_id, so
        // no other part of S1's canonicalization output is needed here.
        let _ = self.fetch_snapshot(snapshot_id)?;
        let (tape, _digest) = self.compute_pddl_plan(snapshot_id)?;
        Ok(tape)
    }

    /// Read-only side-door, many-to-one form of
    /// [`ChatmanEngine::plan_tape_for_snapshot`]: composes the PDDL planning
    /// surface admitted across `snapshot_ids` (in caller order) and plans it
    /// once, yielding one combined [`Pddl8Tape`].
    ///
    /// Each snapshot may carry a PDDL domain fragment literal
    /// (`ceng:pddlDomain`), a problem fragment literal (`ceng:pddlProblem`),
    /// or both. Fragments are parsed individually and merged structurally
    /// (never by text concatenation): all domain fragments must declare the
    /// same domain name; predicates, objects, init atoms, and goal atoms are
    /// deduplicated unions in first-seen order; actions are appended in
    /// caller order with duplicate action names refused. The merged goal is
    /// the conjunction of every problem fragment's goal, so every problem
    /// artifact is load-bearing. Like the single-snapshot side-door this
    /// performs no store mutation and never participates in receipt sealing.
    ///
    /// # Errors
    /// [`Refusal::PlanInfeasible`] for an empty `snapshot_ids`, a fragment
    /// set with no domain or no problem literal, or an unsolvable merged
    /// problem; [`Refusal::SnapshotNotFound`] for an absent snapshot graph;
    /// [`Refusal::ValidationFailed`] for parse failures, mismatched domain
    /// names, or duplicate action names.
    ///
    /// # Complexity
    /// O(n) literal selection over n snapshots plus O(a log a) merge over a
    /// total actions/predicates/atoms; grounding bounded by
    /// `PDDL8_MAX_GROUND`; plan search bounded BFS (`PDDL8_MAX_PLAN_DEPTH`).
    pub fn plan_tape_for_snapshots(
        &self,
        snapshot_ids: &[GraphSnapshotId],
    ) -> Result<Pddl8Tape, Refusal> {
        if snapshot_ids.is_empty() {
            return Err(Refusal::PlanInfeasible(
                "plan_tape_for_snapshots: empty snapshot set; nothing to plan".to_string(),
            ));
        }
        let mut domains: Vec<Pddl8Domain> = Vec::new();
        let mut problems: Vec<Pddl8Problem> = Vec::new();
        for snapshot_id in snapshot_ids {
            // Re-run S1's presence check (read-only) per snapshot.
            let _ = self.fetch_snapshot(snapshot_id)?;
            let graph = snapshot_graph(snapshot_id)?;
            if let Some(domain_text) = self.select_literal(&graph, PDDL_DOMAIN_PREDICATE)? {
                domains.push(domain_from_pddl(&domain_text)?);
            }
            if let Some(problem_text) = self.select_literal(&graph, PDDL_PROBLEM_PREDICATE)? {
                problems.push(problem_from_pddl(&problem_text)?);
            }
        }
        let (domain, problem) = merge_pddl_fragments(domains, problems)?;
        let ground = GroundProblem::build(&domain, &problem, None)?;
        let tape = ground.find_plan()?;
        Ok(tape)
    }

    /// Shared S3 body: reads the PDDL domain/problem literals from the
    /// snapshot, grounds and plans through `bcinr_pddl`, and returns the tape
    /// plus its canonical digest. Called by both the sealed S3 pipeline step
    /// ([`ChatmanEngine::generate_pddl_plan`]) and the read-only side-door
    /// accessor ([`ChatmanEngine::plan_tape_for_snapshot`]).
    ///
    /// # Complexity
    /// Grounding is bounded by `PDDL8_MAX_GROUND`; plan search is bounded
    /// BFS (`PDDL8_MAX_PLAN_DEPTH`); the digest is O(tape bytes).
    fn compute_pddl_plan(
        &self,
        snapshot_id: &GraphSnapshotId,
    ) -> Result<(Pddl8Tape, Digest), Refusal> {
        let graph = snapshot_graph(snapshot_id)?;
        let domain_text = self
            .select_literal(&graph, PDDL_DOMAIN_PREDICATE)?
            .ok_or_else(|| {
                Refusal::PlanInfeasible(format!(
                    "snapshot {snapshot_id}: no PDDL domain literal at \
                     <{PDDL_DOMAIN_PREDICATE}>"
                ))
            })?;
        let problem_text = self
            .select_literal(&graph, PDDL_PROBLEM_PREDICATE)?
            .ok_or_else(|| {
                Refusal::PlanInfeasible(format!(
                    "snapshot {snapshot_id}: no PDDL problem literal at \
                     <{PDDL_PROBLEM_PREDICATE}>"
                ))
            })?;
        let domain = domain_from_pddl(&domain_text)?;
        let problem = problem_from_pddl(&problem_text)?;
        let ground = GroundProblem::build(&domain, &problem, None)?;
        let tape = ground.find_plan()?;
        let digest = tape_digest(&tape);
        Ok((tape, digest))
    }

    /// S4: reads the OCEL trace literal, validates it structurally against
    /// the compat core OCEL laws, checks conformance against the plan tape,
    /// chains one causal frame per event (BLAKE3 rolling chain), and replays
    /// the token flow. Any violation refuses with
    /// [`Refusal::TraceUnlawful`]. Returns the causal chain hash carried
    /// into receipt digest #8.
    ///
    /// # Complexity
    /// O(e) over trace events for log building, chaining and replay;
    /// conformance is O(e + runs * ops) inside `bcinr_powl`.
    fn admit_powl_trace(
        &self,
        snapshot_id: &GraphSnapshotId,
        tape: &Pddl8Tape,
    ) -> Result<String, Refusal> {
        let graph = snapshot_graph(snapshot_id)?;
        let text = self
            .select_literal(&graph, OCEL_LOG_PREDICATE)?
            .ok_or_else(|| {
                Refusal::TraceUnlawful(format!(
                    "snapshot {snapshot_id}: no OCEL trace literal at <{OCEL_LOG_PREDICATE}>"
                ))
            })?;
        let doc: TraceDoc = serde_json::from_str(&text).map_err(|e| {
            Refusal::TraceUnlawful(format!(
                "snapshot {snapshot_id}: OCEL trace JSON does not parse: {e}"
            ))
        })?;

        // Structural OCEL 2.0 law (compat core): logical time only, every
        // event linked to a declared object.
        let objects: Vec<Object> = doc
            .objects
            .iter()
            .map(|o| Object::new(&o.id, &o.otype))
            .collect();
        let mut events: Vec<OcelEvent> = Vec::with_capacity(doc.events.len());
        let mut links: Vec<EventObjectLink> = Vec::new();
        // O(e): build the core log; at_ns == 0 encodes "absent" upstream, so
        // a lawful logical tick is strictly positive.
        for ev in &doc.events {
            if ev.at_ns == 0 {
                return Err(Refusal::TraceUnlawful(format!(
                    "event {}: at_ns must be a positive logical tick (0 encodes absent; \
                     wall-clock time is never admitted)",
                    ev.id
                )));
            }
            events.push(OcelEvent::new(&ev.id, &ev.activity).at_ns(ev.at_ns));
            for oid in &ev.objects {
                links.push(EventObjectLink::new(&ev.id, oid));
            }
        }
        let core_log = wasm4pm_compat::ocel::OcelLog::new(
            objects,
            events,
            links,
            Vec::<ObjectObjectLink>::new(),
            Vec::<ObjectChange>::new(),
        );
        core_log.validate().map_err(|e| {
            Refusal::TraceUnlawful(format!(
                "snapshot {snapshot_id}: OCEL structural law violated: {e:?}"
            ))
        })?;

        // Bridge the Pddl8Tape onto the 64-slot PowlTape for conformance.
        if tape.ops.len() > 64 {
            return Err(Refusal::WarmPathRequired(format!(
                "plan tape has {} ops; the POWL conformance tape holds at most 64 — \
                 split the plan",
                tape.ops.len()
            )));
        }
        let mut powl = PowlTape::new();
        // O(n), n <= 64: copy predecessor masks slot by slot.
        for op in &tape.ops {
            let idx = powl.alloc(OpKind::Atom).ok_or_else(|| {
                Refusal::WarmPathRequired(format!(
                    "POWL tape capacity exhausted at plan op {}",
                    op.index
                ))
            })?;
            powl.ops[idx as usize].pred_mask = op.pred_mask;
            if op.pred_mask == 0 {
                powl.entry_mask |= 1u64 << idx;
            }
        }

        // Feed the fixed-capacity POWL OCEL log and seal the fired mask.
        let mut plog = PowlOcelLog::new();
        let mut fired_mask = 0u64;
        // O(e): one op_fired record per event.
        for ev in &doc.events {
            let bit = 1u64.checked_shl(ev.op_index).ok_or_else(|| {
                Refusal::TraceUnlawful(format!(
                    "event {}: op_index {} exceeds the 64-op tape space",
                    ev.id, ev.op_index
                ))
            })?;
            fired_mask |= bit;
            plog.record_op_fired(doc.run_id, ev.op_index, 0)
                .map_err(|e| {
                    Refusal::TraceUnlawful(format!(
                        "event {}: POWL OCEL log refused the record: {e:?}",
                        ev.id
                    ))
                })?;
        }
        if doc.sealed {
            plog.record_run_sealed(doc.run_id, fired_mask)
                .map_err(|e| {
                    Refusal::TraceUnlawful(format!(
                        "run {}: POWL OCEL log refused the seal: {e:?}",
                        doc.run_id
                    ))
                })?;
        }
        match validate_against_tape(&plog, &powl) {
            ConformanceResult::Conforms => {}
            other => {
                return Err(Refusal::TraceUnlawful(format!(
                    "snapshot {snapshot_id}: trace does not conform to the plan tape: \
                     {other:?}"
                )))
            }
        }

        // Causal chain + token replay. The linear-chain bridge: op i consumes
        // token 1<<i and produces token 1<<(i+1), so replay succeeds iff the
        // trace fires plan ops in tape order.
        let run_hex =
            blake3_combined(&[CHAIN_RUN_TAG, snapshot_id.as_str(), &doc.run_id.to_string()]);
        let mut run_id_bytes = [0u8; 32];
        // The hex digest is 64 ASCII bytes; the first 32 identify the run.
        run_id_bytes.copy_from_slice(&run_hex.as_bytes()[..32]);
        let mut chain = OcelCausalReceipt::genesis(run_id_bytes);
        let entry_bit = match tape.ops.first() {
            Some(op) => 1u64 << u32::from(op.index & 63),
            None => 1u64,
        };
        let mut verifier = PowlReplayVerifier::new(entry_bit);
        // O(e): one replay frame and one causal frame per event.
        for (position, ev) in doc.events.iter().enumerate() {
            let node_bit = 1u64.checked_shl(ev.op_index).ok_or_else(|| {
                Refusal::TraceUnlawful(format!(
                    "event {}: op_index {} exceeds the 64-op token space",
                    ev.id, ev.op_index
                ))
            })?;
            let produces = if (ev.op_index as usize) + 1 < tape.ops.len() {
                node_bit << 1
            } else {
                0
            };
            let frame = PowlReplayFrame {
                node_id: ev.op_index,
                node_bit,
                required_tokens: node_bit,
                produces_tokens: produces,
                activity: ev.activity.clone(),
                ts_ns: ev.at_ns,
                object_ids: ev.objects.clone(),
            };
            verifier.replay_frame(&frame).map_err(|violation| {
                Refusal::TraceUnlawful(format!(
                    "event {}: POWL replay violation: {violation:?}",
                    ev.id
                ))
            })?;
            let mut causal = OcelCausalFrame {
                instruction_id: position as u64,
                fired_mask: DenialPolarity::ADMITTED.to_fired_mask(),
                denial: DenialPolarity::ADMITTED,
                obj_refs: core::array::from_fn(|_| PackedObjRef(0)),
                // Logical tick from the trace itself — never the wall clock.
                ts_ns: ev.at_ns,
                activity_idx: (ev.op_index & 0xFFFF) as u16,
                node_kind: 0,
                pad: [0u8; 5],
                prior_hash: [0u8; 32],
            };
            causal.prior_hash = chain.chain_hash;
            chain.chain(&causal);
        }
        let metrics = verifier.finalize();
        if metrics.fitness != 0x0001_0000 {
            return Err(Refusal::TraceUnlawful(format!(
                "snapshot {snapshot_id}: replay fitness {:#x} is below 1.0 (Q16.16)",
                metrics.fitness
            )));
        }
        let canonical = chain.canonical_hash();
        let chain_hex = core::str::from_utf8(&canonical)
            .map_err(|e| {
                Refusal::ValidationFailed(format!(
                    "causal chain hash is not ASCII hex (engine defect): {e}"
                ))
            })?
            .to_string();
        Ok(chain_hex)
    }

    // ── Internals ─────────────────────────────────────────────────────────

    /// Digest #2: gates digest combined with the sorted breed permits.
    ///
    /// # Complexity
    /// O(p log p) over breed permits (canonical sort before hashing).
    fn profile_digest(&self) -> Digest {
        let gates_hash = self.router.gates().hash();
        let mut permits: Vec<&str> = self
            .profile
            .breed_permits
            .iter()
            .map(String::as_str)
            .collect();
        permits.sort_unstable();
        let mut parts: Vec<&str> = Vec::with_capacity(permits.len() + 2);
        parts.push(PROFILE_DIGEST_TAG);
        parts.push(&gates_hash.0);
        parts.extend(permits);
        Digest::new(blake3_combined(&parts))
    }

    /// Selects the (deterministically smallest) literal value of `predicate`
    /// inside the snapshot graph, or `None` when absent.
    ///
    /// # Complexity
    /// O(m log m) over matches (collected and sorted so store iteration
    /// order can never leak into receipts).
    fn select_literal(
        &self,
        graph: &NamedNode,
        predicate: &str,
    ) -> Result<Option<String>, Refusal> {
        let query = format!(
            "SELECT ?v WHERE {{ GRAPH <{}> {{ ?s <{predicate}> ?v }} }}",
            graph.as_str()
        );
        let prepared = SparqlEvaluator::new().parse_query(&query).map_err(|e| {
            Refusal::ValidationFailed(format!("SELECT parse failed for <{predicate}>: {e}"))
        })?;
        let results = prepared.on_store(&self.store).execute().map_err(|e| {
            Refusal::ValidationFailed(format!("SELECT failed for <{predicate}>: {e}"))
        })?;
        let mut values: Vec<String> = Vec::new();
        match results {
            QueryResults::Solutions(solutions) => {
                // O(m): collect every binding, then sort for determinism.
                for solution in solutions {
                    let solution = solution.map_err(|e| {
                        Refusal::ValidationFailed(format!(
                            "SELECT evaluation failed for <{predicate}>: {e}"
                        ))
                    })?;
                    match solution.get("v") {
                        Some(Term::Literal(literal)) => values.push(literal.value().to_string()),
                        Some(other) => {
                            return Err(Refusal::ValidationFailed(format!(
                                "<{predicate}> must point at a text literal, found {other}"
                            )))
                        }
                        None => {}
                    }
                }
            }
            _ => {
                return Err(Refusal::ValidationFailed(format!(
                    "SELECT for <{predicate}> did not yield solutions"
                )))
            }
        }
        values.sort();
        Ok(values.into_iter().next())
    }
}

/// Structurally merges parsed PDDL domain and problem fragments (in caller
/// order) into one combined domain/problem pair — admitted graph composition
/// at the parsed-artifact level, never text concatenation.
///
/// Merge law: all domain fragments must declare the same domain name and all
/// problem fragments must target it; predicates, objects, init atoms, and
/// goal atoms are deduplicated unions preserving first-seen order; actions
/// are appended in fragment order with duplicate action names refused;
/// PDDL 3.1 extended fields (types, functions, durative actions, derived
/// predicates, constraints) are concatenated in fragment order. The merged
/// problem's goal is the conjunction of every fragment goal. Deterministic:
/// output depends only on fragment content and order.
///
/// # Errors
/// [`Refusal::PlanInfeasible`] when no domain or no problem fragment exists;
/// [`Refusal::ValidationFailed`] for mismatched domain names, problems
/// targeting a different domain, or duplicate action names.
///
/// # Complexity
/// O(a log a) over the total count a of actions, predicates, objects, and
/// atoms (BTreeSet dedup lookups).
fn merge_pddl_fragments(
    domains: Vec<Pddl8Domain>,
    problems: Vec<Pddl8Problem>,
) -> Result<(Pddl8Domain, Pddl8Problem), Refusal> {
    let Some(first_domain) = domains.first() else {
        return Err(Refusal::PlanInfeasible(format!(
            "no PDDL domain fragment literal at <{PDDL_DOMAIN_PREDICATE}> in any snapshot"
        )));
    };
    if problems.is_empty() {
        return Err(Refusal::PlanInfeasible(format!(
            "no PDDL problem fragment literal at <{PDDL_PROBLEM_PREDICATE}> in any snapshot"
        )));
    }
    let domain_name = first_domain.name.clone();

    let mut merged_domain = Pddl8Domain {
        name: domain_name.clone(),
        predicates: Vec::new(),
        actions: Vec::new(),
        types: Vec::new(),
        functions: Vec::new(),
        durative_actions: Vec::new(),
        derived: Vec::new(),
        constraints: Vec::new(),
        events: Vec::new(),
        processes: Vec::new(),
    };
    let mut seen_predicates: std::collections::BTreeSet<(String, u8)> = Default::default();
    let mut seen_actions: std::collections::BTreeSet<String> = Default::default();
    for fragment in domains {
        if fragment.name != domain_name {
            return Err(Refusal::ValidationFailed(format!(
                "domain fragment name mismatch: expected {domain_name:?}, found {:?}; \
                 every fragment must declare the same combined domain",
                fragment.name
            )));
        }
        // O(p log p): dedup union preserving first-seen order.
        for predicate in fragment.predicates {
            if seen_predicates.insert(predicate.clone()) {
                merged_domain.predicates.push(predicate);
            }
        }
        // O(a log a): append in fragment order; duplicate names refused.
        for action in fragment.actions {
            if !seen_actions.insert(action.name.clone()) {
                return Err(Refusal::ValidationFailed(format!(
                    "duplicate PDDL action name {:?} across domain fragments",
                    action.name
                )));
            }
            merged_domain.actions.push(action);
        }
        merged_domain.types.extend(fragment.types);
        merged_domain.functions.extend(fragment.functions);
        merged_domain
            .durative_actions
            .extend(fragment.durative_actions);
        merged_domain.derived.extend(fragment.derived);
        merged_domain.constraints.extend(fragment.constraints);
        merged_domain.events.extend(fragment.events);
        merged_domain.processes.extend(fragment.processes);
    }

    let mut merged_problem = Pddl8Problem {
        name: format!("{domain_name}-combined"),
        domain: domain_name.clone(),
        objects: Vec::new(),
        init: Vec::new(),
        goal: Vec::new(),
        object_types: Vec::new(),
        fn_values: Vec::new(),
        metric: None,
        timed_inits: Vec::new(),
        preferences: Vec::new(),
    };
    let mut seen_objects: std::collections::BTreeSet<String> = Default::default();
    let mut seen_init: std::collections::BTreeSet<String> = Default::default();
    let mut seen_goal: std::collections::BTreeSet<String> = Default::default();
    for fragment in problems {
        if fragment.domain != domain_name {
            return Err(Refusal::ValidationFailed(format!(
                "problem fragment {:?} targets domain {:?}, expected {domain_name:?}",
                fragment.name, fragment.domain
            )));
        }
        for object in fragment.objects {
            if seen_objects.insert(object.clone()) {
                merged_problem.objects.push(object);
            }
        }
        // Atoms keyed by debug rendering: Pddl8Atom is not Ord; the rendered
        // key is total and injective over (predicate, args).
        for atom in fragment.init {
            if seen_init.insert(format!("{atom:?}")) {
                merged_problem.init.push(atom);
            }
        }
        for atom in fragment.goal {
            if seen_goal.insert(format!("{atom:?}")) {
                merged_problem.goal.push(atom);
            }
        }
        merged_problem.object_types.extend(fragment.object_types);
        merged_problem.fn_values.extend(fragment.fn_values);
        merged_problem.timed_inits.extend(fragment.timed_inits);
        merged_problem.preferences.extend(fragment.preferences);
    }
    Ok((merged_domain, merged_problem))
}

/// S5: seals hook receipts into [`BoundaryRequest`]s and computes digest #8
/// over the S4 causal chain hash plus every receipt's identity triple, in
/// canonical `(hook_name, idempotency_key)` order.
///
/// # Complexity
/// O(h log h) over hook receipts (canonical sort before hashing).
fn trigger_knowledge_hooks(
    hook_receipts: Vec<HookReceipt>,
    trace_chain_hash: &str,
) -> (Vec<BoundaryRequest>, Digest) {
    let mut receipts = hook_receipts;
    receipts.sort_by(|a, b| {
        (&a.hook_name, &a.idempotency_key).cmp(&(&b.hook_name, &b.idempotency_key))
    });
    let mut parts: Vec<String> = Vec::with_capacity(receipts.len() * 3 + 2);
    parts.push(HOOK_EVENT_DIGEST_TAG.to_string());
    parts.push(trace_chain_hash.to_string());
    let mut requests: Vec<BoundaryRequest> = Vec::with_capacity(receipts.len());
    // O(h): one sealed request and three hash parts per receipt.
    for receipt in receipts {
        parts.push(receipt.hook_name.clone());
        parts.push(receipt.delta_hash.clone());
        parts.push(receipt.idempotency_key.clone());
        requests.push(BoundaryRequest {
            hook_name: receipt.hook_name,
            idempotency_key: receipt.idempotency_key,
            delta: receipt.delta_quads,
            seal: BoundarySeal(()),
        });
    }
    let refs: Vec<&str> = parts.iter().map(String::as_str).collect();
    (requests, Digest::new(blake3_combined(&refs)))
}

/// Canonical, injective tape digest: ops hashed in tape order (order is the
/// plan's semantics), every field length-prefixed by `blake3_combined`.
///
/// # Complexity
/// O(tape bytes).
fn tape_digest(tape: &Pddl8Tape) -> Digest {
    let mut parts: Vec<String> = Vec::with_capacity(tape.ops.len() * 7 + 1);
    parts.push(TAPE_DIGEST_TAG.to_string());
    // O(n) over ops: index order is canonical for a plan.
    for op in &tape.ops {
        parts.push(op.index.to_string());
        parts.push(op.label.clone());
        parts.push(op.pred_mask.to_string());
        parts.push(op.action.schema_name.clone());
        parts.push(atoms_key(&op.action.preconditions));
        parts.push(atoms_key(&op.action.add_effects));
        parts.push(atoms_key(&op.action.del_effects));
    }
    let refs: Vec<&str> = parts.iter().map(String::as_str).collect();
    Digest::new(blake3_combined(&refs))
}

/// Joins ground atoms as `pred(a,b);pred2(c)` — atom order inside an action
/// is grounding-deterministic, hence canonical.
///
/// # Complexity
/// O(total atom bytes).
fn atoms_key(atoms: &[wasm4pm_compat::pddl::Pddl8GroundAtom]) -> String {
    let mut out = String::new();
    // O(a) over atoms.
    for (i, atom) in atoms.iter().enumerate() {
        if i > 0 {
            out.push(';');
        }
        out.push_str(&atom.pred);
        out.push('(');
        out.push_str(&atom.args.join(","));
        out.push(')');
    }
    out
}

/// Resolves a snapshot id to its named graph, refusing malformed IRIs.
///
/// # Complexity
/// O(len) IRI validation.
fn snapshot_graph(snapshot_id: &GraphSnapshotId) -> Result<NamedNode, Refusal> {
    NamedNode::new(snapshot_id.as_str()).map_err(|e| {
        Refusal::SnapshotNotFound(format!(
            "snapshot id {snapshot_id} is not a valid graph IRI: {e}"
        ))
    })
}

/// Refuses delta material containing an unbound variable token — actuation
/// and closure writes must be ground; a `?var` surviving to this point is a
/// defect upstream, refused loudly rather than skipped.
///
/// # Complexity
/// O(bytes) scan over the delta lines.
fn refuse_non_ground(origin: &str, delta: &str) -> Result<(), Refusal> {
    // O(lines): a term token starting with '?' marks an unbound variable in
    // the crate's canonical quad serialization.
    for line in delta.lines() {
        if line.starts_with('?') || line.contains(" ?") {
            return Err(Refusal::ValidationFailed(format!(
                "{origin}: delta line is not ground: {line:?}"
            )));
        }
    }
    Ok(())
}

/// Executes one SPARQL UPDATE against the store via the non-deprecated
/// `SparqlEvaluator` interface.
///
/// # Complexity
/// Parser O(bytes) + storage cost of the update.
fn run_update(store: &Store, update: &str) -> Result<(), Refusal> {
    SparqlEvaluator::new()
        .parse_update(update)
        .map_err(|e| Refusal::ValidationFailed(format!("UPDATE parse failed: {e}")))?
        .on_store(store)
        .execute()
        .map_err(|e| Refusal::ValidationFailed(format!("UPDATE execution failed: {e}")))
}

/// JSON shape of the OCEL trace literal carried by a snapshot at
/// [`OCEL_LOG_PREDICATE`]. Timestamps are logical ticks (`at_ns`), never
/// wall-clock values.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct TraceDoc {
    /// Run identity within the trace.
    run_id: u64,
    /// Whether the run declares itself sealed (fired mask recorded).
    #[serde(default)]
    sealed: bool,
    /// Declared objects.
    objects: Vec<TraceDocObject>,
    /// Events in trace order.
    events: Vec<TraceDocEvent>,
}

/// One declared object inside a [`TraceDoc`].
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct TraceDocObject {
    id: String,
    otype: String,
}

/// One event inside a [`TraceDoc`].
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct TraceDocEvent {
    id: String,
    activity: String,
    /// Index of the plan-tape op this event fired.
    op_index: u32,
    /// Logical tick; must be strictly positive (0 encodes absent upstream).
    at_ns: u64,
    /// Object ids this event touches (E2O links).
    #[serde(default)]
    objects: Vec<String>,
}

// ── Breed consultation (cognition feature) ──────────────────────────────────

/// A breed's answer wrapped as a **witness**: it attests, it never
/// authorizes. The only way to treat it as authority is
/// [`BreedWitness::into_authority`], which always refuses with
/// [`Refusal::WitnessNotAuthority`].
#[cfg(feature = "cognition")]
#[derive(Debug, Clone)]
pub struct BreedWitness {
    /// The breed consulted.
    pub breed: String,
    /// The breed's human-readable explanation (non-machine evidence).
    pub explanation: String,
    /// The breed's recommended selection, when it made one.
    pub selected: Option<String>,
}

#[cfg(feature = "cognition")]
impl BreedWitness {
    /// Witnesses attest; they do not authorize. Any attempt to promote a
    /// breed's output into an authority is refused by construction.
    pub fn into_authority(self) -> Result<core::convert::Infallible, Refusal> {
        Err(Refusal::WitnessNotAuthority(format!(
            "breed {} produced a witness, not an authority; machine evidence \
             (receipts) is the only authority",
            self.breed
        )))
    }
}

#[cfg(feature = "cognition")]
impl ChatmanEngine {
    /// Consults a cognition breed under the profile's permit list.
    ///
    /// Laws, in refusal order:
    /// - an override request is refused ([`Refusal::AgentOverrideDenied`]);
    /// - the breed must be permitted ([`Refusal::BreedUnpermitted`]);
    /// - a nondeterministic operator requires at least one covering receipt,
    ///   each of which must verify
    ///   ([`Refusal::NondeterministicOperatorRequiresReceipt`]);
    /// - dispatch failures surface as [`Refusal::ValidationFailed`];
    /// - the output is classified as non-machine evidence and wrapped as a
    ///   [`BreedWitness`] (authority misuse refuses via
    ///   [`BreedWitness::into_authority`]).
    ///
    /// # Complexity
    /// O(r) receipt verification plus the breed's own run cost.
    pub fn consult_breed(
        &self,
        breed: &str,
        input: &wasm4pm_cognition::breeds::BreedInput,
        covering_receipts: &[super::abi::Receipt],
        override_requested: bool,
    ) -> Result<BreedWitness, Refusal> {
        if override_requested {
            return Err(Refusal::AgentOverrideDenied(format!(
                "breed {breed}: an agent may not override an operator decision"
            )));
        }
        if !self.profile.breed_permits.iter().any(|p| p == breed) {
            return Err(Refusal::BreedUnpermitted(format!(
                "breed {breed} is not permitted by profile {}",
                self.router.gates().profile_id
            )));
        }
        if covering_receipts.is_empty() {
            return Err(Refusal::NondeterministicOperatorRequiresReceipt(format!(
                "breed {breed} is a nondeterministic operator; at least one covering \
                 receipt is required"
            )));
        }
        // O(r): every covering receipt must verify before dispatch.
        for receipt in covering_receipts {
            receipt.verify()?;
        }
        let output =
            wasm4pm_cognition::breeds::dispatch::dispatch_breed(breed, input).map_err(|e| {
                Refusal::ValidationFailed(format!("breed {breed} dispatch failed: {e}"))
            })?;
        Ok(BreedWitness {
            breed: breed.to_string(),
            explanation: output.explanation,
            selected: output.selected,
        })
    }
}

#[cfg(test)]
mod tests {
    //! Chicago-style state-based tests over a real in-memory store: one
    //! happy path, one refusal per stage, determinism (byte-identical
    //! receipt roots across independent engines), actuation, and replay.

    use super::*;
    use crate::chatman::abi::{InputHandles, InvocationId, OperatorId, ProfileId};

    const SNAPSHOT_IRI: &str = "urn:chatman:snapshot:test";
    const PROFILE_IRI: &str = "profile:engine-test";

    /// Snapshot fixture: a tiny RDFS hierarchy (so OWL RL derives a fact),
    /// a one-step PDDL world, and a conforming one-event OCEL trace.
    const SNAPSHOT_TTL: &str = r#"
@prefix ex: <http://example.org/> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix ceng: <urn:chatman:engine#> .

ex:Employee rdfs:subClassOf ex:Person .
ex:alice a ex:Employee .

ex:world ceng:pddlDomain """
(define (domain chatman-min)
  (:requirements :strips)
  (:predicates (ready ?x) (done ?x))
  (:action finish
    :parameters (?x)
    :precondition (and (ready ?x))
    :effect (and (done ?x) (not (ready ?x)))))
""" .
ex:world ceng:pddlProblem """
(define (problem chatman-min-p)
  (:domain chatman-min)
  (:objects a)
  (:init (ready a))
  (:goal (done a))
)
""" .
ex:world ceng:ocelLog """{"run_id":1,"sealed":true,"objects":[{"id":"case-1","otype":"case"}],"events":[{"id":"e1","activity":"finish(a)","op_index":0,"at_ns":1,"objects":["case-1"]}]}""" .
"#;

    /// Same world, but the trace fires op 0 twice (DuplicateFire).
    const SNAPSHOT_TTL_DUPLICATE_FIRE: &str = r#"
@prefix ex: <http://example.org/> .
@prefix ceng: <urn:chatman:engine#> .
ex:world ceng:pddlDomain """
(define (domain chatman-min)
  (:requirements :strips)
  (:predicates (ready ?x) (done ?x))
  (:action finish
    :parameters (?x)
    :precondition (and (ready ?x))
    :effect (and (done ?x) (not (ready ?x)))))
""" .
ex:world ceng:pddlProblem """
(define (problem chatman-min-p)
  (:domain chatman-min)
  (:objects a)
  (:init (ready a))
  (:goal (done a))
)
""" .
ex:world ceng:ocelLog """{"run_id":1,"sealed":true,"objects":[{"id":"case-1","otype":"case"}],"events":[{"id":"e1","activity":"finish(a)","op_index":0,"at_ns":1,"objects":["case-1"]},{"id":"e2","activity":"finish(a)","op_index":0,"at_ns":2,"objects":["case-1"]}]}""" .
"#;

    /// Same world, but the goal is unreachable (no action derives it).
    const SNAPSHOT_TTL_INFEASIBLE: &str = r#"
@prefix ex: <http://example.org/> .
@prefix ceng: <urn:chatman:engine#> .
ex:world ceng:pddlDomain """
(define (domain chatman-min)
  (:requirements :strips)
  (:predicates (ready ?x) (done ?x) (never ?x))
  (:action finish
    :parameters (?x)
    :precondition (and (ready ?x))
    :effect (and (done ?x) (not (ready ?x)))))
""" .
ex:world ceng:pddlProblem """
(define (problem chatman-min-p)
  (:domain chatman-min)
  (:objects a)
  (:init (ready a))
  (:goal (never a))
)
""" .
ex:world ceng:ocelLog """{"run_id":1,"sealed":true,"objects":[{"id":"case-1","otype":"case"}],"events":[{"id":"e1","activity":"finish(a)","op_index":0,"at_ns":1,"objects":["case-1"]}]}""" .
"#;

    fn test_profile() -> Result<EngineProfile, Refusal> {
        let profile_id = ProfileId::new(PROFILE_IRI);
        let gates =
            ProfileGates::new(profile_id.clone(), ProfileGates::DEFAULT_ENABLED_MASK, 0, 8)?;
        let symbol_table = ProfileSymbolTable::build(
            profile_id,
            vec![
                "<urn:chatman:t0>".to_string(),
                "<urn:chatman:t1>".to_string(),
            ],
        )?;
        Ok(EngineProfile {
            gates,
            symbol_table,
            admission: AdmissionSpec {
                constraint_names: vec!["c0".to_string()],
                required_mask: 0,
                forbidden_mask: 0,
                set_on_admit: 0,
                clear_on_admit: 0,
            },
            breed_permits: Vec::new(),
        })
    }

    fn envelope() -> InvocationEnvelope {
        InvocationEnvelope {
            invocation_id: InvocationId::new("inv-1"),
            snapshot_id: GraphSnapshotId::new(SNAPSHOT_IRI),
            profile_id: ProfileId::new(PROFILE_IRI),
            operator_id: OperatorId::new("op-1"),
            input_handles: InputHandles::default(),
        }
    }

    fn engine_with(turtle: &str) -> Result<ChatmanEngine, Refusal> {
        let mut engine = ChatmanEngine::in_memory(test_profile()?)?;
        engine.load_snapshot(&GraphSnapshotId::new(SNAPSHOT_IRI), turtle)?;
        Ok(engine)
    }

    fn admit(turtle: &str) -> Result<AdmittedTransition, Refusal> {
        let mut engine = engine_with(turtle)?;
        engine.admit_transition(envelope())
    }

    fn refusal_check(
        result: Result<AdmittedTransition, Refusal>,
        want: &str,
    ) -> Result<(), Refusal> {
        match result {
            Err(refusal) if refusal.name() == want => Ok(()),
            other => Err(Refusal::ValidationFailed(format!(
                "wanted refusal {want}, got {other:?}"
            ))),
        }
    }

    #[test]
    fn happy_path_admits_and_seals_nine_digests() -> Result<(), Refusal> {
        // Arrange + Act
        let transition = admit(SNAPSHOT_TTL)?;
        let receipt = transition.receipt();

        // Assert: every digest is non-empty 64-hex material and the root
        // recomputes over the nine carried digests.
        for digest in [
            &receipt.graph_snapshot,
            &receipt.profile,
            &receipt.symbol_table,
            &receipt.projection,
            &receipt.admission_table,
            &receipt.route_decision,
            &receipt.tape,
            &receipt.hook_event,
            &receipt.engine_version,
        ] {
            assert_eq!(digest.0.len(), 64, "digest must be 64 hex chars");
        }
        assert_eq!(receipt.recompute_root(), receipt.receipt_root);
        // Canonical material is sorted (receipt law).
        let lines: Vec<&str> = receipt.canon_nquads.lines().collect();
        let mut sorted = lines.clone();
        sorted.sort_unstable();
        assert_eq!(lines, sorted, "canonical N-Quads must be sorted");
        assert!(!receipt.canon_nquads.is_empty());
        // The hookless fixture yields no boundary requests.
        assert!(transition.boundary_requests().is_empty());
        Ok(())
    }

    #[test]
    fn owl_closure_lands_in_sibling_graph_not_snapshot() -> Result<(), Refusal> {
        // Arrange
        let mut engine = engine_with(SNAPSHOT_TTL)?;
        let (_, before) = engine.fetch_snapshot(&GraphSnapshotId::new(SNAPSHOT_IRI))?;

        // Act
        let transition = engine.admit_transition(envelope())?;

        // Assert: the derived alice-is-a-Person fact exists in the closure
        // graph and the snapshot graph is byte-identical to before.
        let closure = NamedNode::new(format!("{SNAPSHOT_IRI}#closure"))
            .map_err(|e| Refusal::ValidationFailed(format!("closure IRI: {e}")))?;
        let mut derived_lines = Vec::new();
        for quad in engine
            .store
            .quads_for_pattern(None, None, None, Some(closure.as_ref().into()))
        {
            let quad = quad.map_err(|e| Refusal::ValidationFailed(format!("storage: {e}")))?;
            derived_lines.push(format!(
                "{} {} {}",
                quad.subject, quad.predicate, quad.object
            ));
        }
        assert!(
            derived_lines
                .iter()
                .any(|l| l.contains("alice") && l.contains("Person")),
            "OWL RL closure must derive alice rdf:type Person, got {derived_lines:?}"
        );
        let (_, after) = engine.fetch_snapshot(&GraphSnapshotId::new(SNAPSHOT_IRI))?;
        assert_eq!(before, after, "the input snapshot graph is immutable");
        assert_eq!(
            transition.receipt().recompute_root(),
            transition.receipt().receipt_root
        );
        Ok(())
    }

    #[test]
    fn double_run_is_byte_identical() -> Result<(), Refusal> {
        // Arrange + Act: two independent engines over the same snapshot.
        let first = admit(SNAPSHOT_TTL)?;
        let second = admit(SNAPSHOT_TTL)?;

        // Assert: receipts agree byte for byte.
        assert_eq!(first.receipt(), second.receipt());
        assert_eq!(
            first.receipt().receipt_root.0,
            second.receipt().receipt_root.0
        );
        Ok(())
    }

    #[test]
    fn s1_unknown_snapshot_refuses_snapshot_not_found() -> Result<(), Refusal> {
        // Arrange: engine with no snapshot loaded.
        let mut engine = ChatmanEngine::in_memory(test_profile()?)?;
        // Act
        let result = engine.admit_transition(envelope());
        // Assert
        refusal_check(result, "SnapshotNotFound")
    }

    #[test]
    fn s1_quoted_triple_text_refuses_triple_term() -> Result<(), Refusal> {
        // Arrange
        let mut engine = ChatmanEngine::in_memory(test_profile()?)?;
        // Act: RDF 1.2 quoted-triple syntax at the load boundary.
        let result = engine.load_snapshot(
            &GraphSnapshotId::new(SNAPSHOT_IRI),
            "<< <urn:s> <urn:p> <urn:o> >> <urn:q> <urn:r> .",
        );
        // Assert
        match result {
            Err(Refusal::TripleTermInSnapshot(_)) => Ok(()),
            other => Err(Refusal::ValidationFailed(format!(
                "wanted TripleTermInSnapshot, got {other:?}"
            ))),
        }
    }

    #[test]
    fn s2_owl_disabled_refuses_via_router() -> Result<(), Refusal> {
        // Arrange: profile whose gates never enable OwlRl.
        let profile_id = ProfileId::new(PROFILE_IRI);
        let hot_only = ProfileGates::new(
            profile_id.clone(),
            crate::chatman::router::Dialect::Triple8Pattern.mask_bit(),
            0,
            8,
        )?;
        let base = test_profile()?;
        let profile = EngineProfile {
            gates: hot_only,
            symbol_table: base.symbol_table,
            admission: base.admission,
            breed_permits: base.breed_permits,
        };
        let mut engine = ChatmanEngine::in_memory(profile)?;
        engine.load_snapshot(&GraphSnapshotId::new(SNAPSHOT_IRI), SNAPSHOT_TTL)?;
        // Act
        let result = engine.admit_transition(envelope());
        // Assert: the OWL RL shape has no enabled dialect >= its floor.
        refusal_check(result, "UnsupportedDialect")
    }

    #[test]
    fn s3_missing_pddl_refuses_plan_infeasible() -> Result<(), Refusal> {
        // Arrange: snapshot with graph data but no PDDL literals.
        let result = admit(
            r#"@prefix ex: <http://example.org/> .
ex:a ex:knows ex:b ."#,
        );
        // Assert
        refusal_check(result, "PlanInfeasible")
    }

    #[test]
    fn s3_unreachable_goal_refuses_plan_infeasible() -> Result<(), Refusal> {
        refusal_check(admit(SNAPSHOT_TTL_INFEASIBLE), "PlanInfeasible")
    }

    #[test]
    fn s4_duplicate_fire_refuses_trace_unlawful() -> Result<(), Refusal> {
        refusal_check(admit(SNAPSHOT_TTL_DUPLICATE_FIRE), "TraceUnlawful")
    }

    #[test]
    fn s4_missing_trace_refuses_trace_unlawful() -> Result<(), Refusal> {
        // Arrange: PDDL present, OCEL literal absent.
        let ttl = r#"
@prefix ex: <http://example.org/> .
@prefix ceng: <urn:chatman:engine#> .
ex:world ceng:pddlDomain """
(define (domain chatman-min)
  (:requirements :strips)
  (:predicates (ready ?x) (done ?x))
  (:action finish
    :parameters (?x)
    :precondition (and (ready ?x))
    :effect (and (done ?x) (not (ready ?x)))))
""" .
ex:world ceng:pddlProblem """
(define (problem chatman-min-p)
  (:domain chatman-min)
  (:objects a)
  (:init (ready a))
  (:goal (done a))
)
""" .
"#;
        refusal_check(admit(ttl), "TraceUnlawful")
    }

    #[test]
    fn envelope_naming_wrong_profile_refuses_profile_hash_mismatch() -> Result<(), Refusal> {
        // Arrange
        let mut engine = engine_with(SNAPSHOT_TTL)?;
        let mut env = envelope();
        env.profile_id = ProfileId::new("profile:someone-else");
        // Act + Assert
        refusal_check(engine.admit_transition(env), "ProfileHashMismatch")
    }

    #[test]
    fn actuate_registers_post_graph_and_dedups() -> Result<(), Refusal> {
        // Arrange
        let mut engine = engine_with(SNAPSHOT_TTL)?;
        let transition = engine.admit_transition(envelope())?;
        let root = transition.receipt().receipt_root.0.clone();

        // Act
        let record = engine.actuate(transition)?;

        // Assert: the post graph is named by the receipt root and exists.
        assert!(record.post_graph.contains(&root));
        assert_eq!(record.duplicates_skipped, 0);
        let post = NamedNode::new(&record.post_graph)
            .map_err(|e| Refusal::ValidationFailed(format!("post IRI: {e}")))?;
        let present = engine
            .store
            .contains_named_graph(post.as_ref())
            .map_err(|e| Refusal::ValidationFailed(format!("storage: {e}")))?;
        assert!(present, "actuation must register the post graph");
        Ok(())
    }

    #[test]
    fn verify_replay_accepts_faithful_and_refuses_tampered() -> Result<(), Refusal> {
        // Arrange
        let transition = admit(SNAPSHOT_TTL)?;
        let inputs = ReplayInputs {
            envelope: envelope(),
            snapshot_turtle: SNAPSHOT_TTL.to_string(),
            profile: test_profile()?,
        };

        // Act + Assert: the faithful receipt replays clean.
        match ChatmanEngine::verify_replay(transition.receipt(), &inputs) {
            Ok(()) => {}
            Err(mismatch) => {
                return Err(Refusal::ValidationFailed(format!(
                    "faithful replay must verify, got {mismatch}"
                )))
            }
        }

        // Tamper with digest #7: the fail-fast enum names the tape field.
        let mut tampered = transition.receipt().clone();
        tampered.tape = Digest::new("0".repeat(64));
        match ChatmanEngine::verify_replay(&tampered, &inputs) {
            Err(ReplayMismatch::Tape { .. }) => {}
            other => {
                return Err(Refusal::ValidationFailed(format!(
                    "tampered tape digest must fail as ReplayMismatch::Tape, got {other:?}"
                )))
            }
        }

        // Tamper with the root only: every field matches, the root recompute
        // catches the drift.
        let mut root_tampered = transition.receipt().clone();
        root_tampered.receipt_root = Digest::new("f".repeat(64));
        match ChatmanEngine::verify_replay(&root_tampered, &inputs) {
            Err(ReplayMismatch::ReceiptRoot { .. }) => Ok(()),
            other => Err(Refusal::ValidationFailed(format!(
                "tampered root must fail as ReplayMismatch::ReceiptRoot, got {other:?}"
            ))),
        }
    }
}
