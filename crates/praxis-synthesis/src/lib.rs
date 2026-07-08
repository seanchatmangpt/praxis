//! # praxis-synthesis — the deep-research stack as one bounded, receipted pipeline
//!
//! Prototype crate combining four research findings into a single lawful
//! pipeline, each layer riding on substrates praxis already owns:
//!
//! | layer | module | research lesson | what it adds |
//! |-------|--------|-----------------|--------------|
//! | 1 | [`datalog`] | Nemo (scalable Datalog) | semi-naive forward saturation over `pddl-index`'s interned ID space |
//! | 2 | [`sequence`] | SMT capability sequencing | declared capabilities → solver-discovered order + parameter bindings (no hand-authored PDDL) |
//! | 3 | [`dag`] | OxyMake (content-addressable workflows) | BLAKE3 per-node output hashes, memoized replay, order-independent root hash |
//! | 4 | [`verify`] | Flux (refinement types) | machine-checkable refinements over the pipeline's own receipts |
//!
//! [`pipeline`] composes them: facts → saturate → sequence → dag-execute →
//! verify → one [`pipeline::SynthesisReceipt`].
//!
//! ## Doctrine
//!
//! Everything is bounded (hard caps, deterministic iteration order) and every
//! bound violation is a [`Refusal`] carrying a reason and salvage data — never
//! a panic, never a silent truncation. The sequencing layer is deliberately a
//! bounded in-crate CSP solver, not a native SMT binding: the [`sequence::Solver`]
//! trait is the seam where a real theory solver plugs in later.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod agent_registry;
pub mod boundary;
pub mod breeds;
pub mod budget;
pub mod cell;
pub mod cell_supervise;
pub mod dag;
pub mod datalog;
pub mod delta;
pub mod envelope;
pub mod fault;
pub mod firing;
pub mod fleet;
pub mod gen;
pub mod geometry;
pub mod glue;
pub mod graph;
pub mod ground;
pub mod handlers;
pub mod hooks;
pub mod kernel;
pub mod life;
pub mod livelock;
pub mod park;
pub mod pipeline;
pub mod quarantine;
pub mod reality;
pub mod rel;
pub mod sequence;
pub mod solver8;
pub mod supervise;
pub mod verify;
pub mod wal;

pub use agent_registry::{
    agent_canonical_form, agent_registry_hash, extract_agents, spawn_depth_law, AgentProfile,
};
pub use boundary::{
    execute_emit_delta, get_delta_template, project_delta_template, BoundaryRequest,
};
pub use dag::{Dag, DagReceipt, HashRunner, MemoCache, NodeReceipt, NodeRunner};
pub use datalog::{Atom, DlRule, Program, SaturationReceipt, Term};
pub use delta::GraphDelta;
pub use envelope::{
    verify_envelope_chain, wrap_firing_receipt, wrap_workflow_receipt, PayloadRef, ReceiptEnvelope,
};
pub use firing::{
    fire_hooks, idempotency_key, replay_firing, to_ocel_event, window_history_hash, FiringOutcome,
    HookFiringReceipt,
};
pub use glue::{compose_workflows, execute_composed, ComposedGraph, ComposedWorkflowReceipt};
#[allow(deprecated)]
#[deprecated(since = "26.7.2", note = "use RiceQuarantine and Admission instead")]
pub use graph::execute_workflow;
#[allow(deprecated)]
#[deprecated(since = "26.7.2", note = "use RiceQuarantine and Admission instead")]
pub use graph::execute_workflow_with;
pub use graph::{replay_workflow, WorkflowIr, WorkflowReceipt};
pub use ground::{capability_task_spec, ground_fired_action, CapabilityTaskSpec};
pub use handlers::{Delegability, HandlerBinding, HandlerRegistry};
pub use hooks::{
    evaluate_hooks, extract_hooks, hook_hash, load_hook_pack, schedule_hooks, DiagnosticDetail,
    HookCondition, HookPack, HookVerdict, HookVerdictRecord, KnowledgeHook, TriggerDiagnostic,
};
pub use kernel::{extract_kernel, kernel_hash, PrayerClause};
pub use livelock::{
    detect, detection_program, rehearsal_exceeded, LivelockClass, ALL_CLASSES, STEPS,
};
pub use pipeline::{Synthesis, SynthesisReceipt};
pub use quarantine::{
    Admission, AdmissionRecord, AdmittedEvent, MeaningSource, Origin, Reference, RiceQuarantine,
};
pub use reality::{RealityAddressRecord, PROVENANCE_PREDICATE, SPACE_PREDICATE, TIME_PREDICATE};
pub use sequence::{
    BoundStep, BoundedCsp, Capability, Constraint, SequencePlan, SequenceProblem, SolveReceipt,
    Solver,
};
pub use solver8::{CoreCache, Solver8};
pub use verify::{CheckOutcome, Verdict};

use serde::{Deserialize, Serialize};

/// A first-class, receipted refusal. The only lawful way any layer declines
/// work: reason stated, salvage data attached, nothing silent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum Refusal {
    /// A search or saturation budget was exhausted before completion.
    #[error("budget exceeded: {what} spent {spent} of {budget} ({salvage})")]
    BudgetExceeded {
        /// Which budget was exhausted (e.g. `search_nodes`, `iterations`).
        what: String,
        /// The configured cap.
        budget: u64,
        /// How much was actually spent when the cap fired.
        spent: u64,
        /// What survives: partial-progress data a caller can act on.
        salvage: String,
    },
    /// The derived-tuple cap fired during saturation.
    #[error("tuple cap exceeded: {derived} derived of max {cap} at iteration {iteration}")]
    TupleCapExceeded {
        /// Tuples derived when the cap fired.
        derived: u64,
        /// The configured cap.
        cap: u64,
        /// Saturation iteration at which the cap fired.
        iteration: u64,
    },
    /// The rule program has no stratification (negation cycle).
    #[error("unstratifiable program: {detail}")]
    Unstratifiable {
        /// Which predicate/rule participates in the negation cycle.
        detail: String,
    },
    /// The sequencing goal is unreachable within the horizon.
    #[error("unsatisfiable: {detail} (explored {nodes_explored} nodes)")]
    Unsatisfiable {
        /// Why the goal is unreachable, as far as the solver can state it.
        detail: String,
        /// Search nodes explored before concluding.
        nodes_explored: u64,
    },
    /// Unsatisfiability *certified before search*: the refusal carries a
    /// minimal conflicting constraint core (a bounded MUS). A second agent
    /// can verify the impossibility by re-propagating the core alone —
    /// no search required. Proofs of impossibility with named culprits are
    /// the fleet's dead-end-sharing currency.
    #[error("unsat (certified): {detail}; core = {core:?}")]
    UnsatProof {
        /// What became impossible (which mandatory capability's window emptied).
        detail: String,
        /// Minimal unsatisfiable core: rendered constraints whose conjunction
        /// alone reproduces the emptiness.
        core: Vec<String>,
        /// Whether this proof was replayed from a shared core cache rather
        /// than re-derived.
        replayed: bool,
    },
    /// A structural invariant on inputs was violated (caller error, receipted).
    #[error("invalid input: {detail}")]
    InvalidInput {
        /// What was malformed.
        detail: String,
    },
    /// The workflow graph document violates the bounded Turtle subset:
    /// lexical error, grammar error, or a refused construct (blank node,
    /// collection, language tag, datatype, `@base`, non-integer numeric).
    #[error("graph malformed at {line}:{column}: {detail}")]
    GraphMalformed {
        /// 1-based line of the culprit byte.
        line: usize,
        /// 1-based column of the culprit byte.
        column: usize,
        /// Which construct or rule was violated.
        detail: String,
    },
    /// A `wf:` predicate outside the closed-world vocabulary table.
    #[error("unknown predicate {predicate} on {subject}")]
    UnknownPredicate {
        /// The offending fully-expanded predicate IRI.
        predicate: String,
        /// The subject IRI it appeared on.
        subject: String,
    },
    /// The graph parsed but does not describe a well-formed workflow
    /// (missing required fields, argument gaps, duplicate names, wrong
    /// cardinality of `wf:Workflow`, unknown constraint kind).
    #[error("workflow ill-formed at {subject}: {detail}")]
    WorkflowIllFormed {
        /// The subject IRI of the offending node.
        subject: String,
        /// What shape rule was violated.
        detail: String,
    },
    /// A hard input cap on the graph front end fired; nothing was truncated.
    #[error("graph cap exceeded: {what} {actual} of max {cap}")]
    GraphCapExceeded {
        /// Which cap fired: `ttl_bytes` | `triples` | `iri_len` | `lit_len` | `prefixes`.
        what: String,
        /// The configured cap.
        cap: u64,
        /// The observed value when the cap fired.
        actual: u64,
    },
    /// A verification refinement failed on the pipeline's own artifacts.
    #[error("verification failed: {failed:?}")]
    VerificationFailed {
        /// Names of the refinements that failed.
        failed: Vec<String>,
    },
    /// Two constituent graphs assert conflicting values for a functional
    /// wf: predicate on the same subject — the glue law is violated.
    #[error("glue conflict on <{subject}> {predicate}: values {values:?}")]
    GlueConflict {
        /// The shared subject IRI.
        subject: String,
        /// The functional predicate IRI.
        predicate: String,
        /// Every distinct canonical object rendering observed (≥ 2, byte-sorted).
        values: Vec<String>,
    },
    /// A delta failed the admission gate: retracting an absent triple, or a
    /// post-state that violates a decidable admission rule.
    #[error("admission refused for {subject}: {detail}")]
    AdmissionRefused {
        /// The offending triple rendering or subject IRI.
        subject: String,
        /// Which admission rule was violated.
        detail: String,
    },
    /// A hook declared a condition kind praxis has no bounded engine for.
    /// Refused by name — never faked — with the honest analog stated.
    #[error(
        "condition kind '{kind}' unsupported on {subject}; supported analog: {supported_analog}"
    )]
    ConditionUnsupported {
        /// The declared kind.
        kind: String,
        /// The hook node IRI.
        subject: String,
        /// The supported condition kind that honestly approximates it.
        supported_analog: String,
    },
    /// A `hook:` node violated the closed-world hook vocabulary or shape
    /// rules (cardinality, unknown predicate/class, registry bound).
    #[error("hook ill-formed at {subject}: {detail}")]
    HookIllFormed {
        /// The subject IRI of the offending node.
        subject: String,
        /// What shape rule was violated.
        detail: String,
    },
    /// A `prayer-kernel:` node violated the closed-world kernel vocabulary
    /// or coverage rules (unknown predicate/class, wrong cardinality,
    /// missing/extra/duplicate clause, unknown clause name or boundary).
    #[error("prayer kernel ill-formed at {subject}: {detail}")]
    KernelIllFormed {
        /// The subject IRI of the offending node (or `(kernel)`).
        subject: String,
        /// What coverage or shape rule was violated.
        detail: String,
    },
    /// A graph-declared handler IRI is not in the closed registry.
    /// Refused BEFORE solving; the known table is named.
    #[error("unknown handler {handler} on capability '{capability}' (known: {known:?})")]
    UnknownHandler {
        /// The capability whose binding named the unknown handler.
        capability: String,
        /// The declared handler IRI.
        handler: String,
        /// The registry's exact key set.
        known: Vec<String>,
    },
    /// An automated runner was asked to execute a capability whose
    /// delegability grade reserves the act for the human.
    #[error("delegability violation on '{capability}': declared {declared}, requires {required}")]
    DelegabilityViolation {
        /// The capability.
        capability: String,
        /// The minimum grade an automated runner requires.
        required: String,
        /// The grade the graph declares.
        declared: String,
    },
    /// The surrender boundary of the prayer kernel was violated: a clause
    /// whose boundary is `god-receives-unbounded` was routed toward
    /// computation (its action is not a refuse-effect hook, or a
    /// non-refusing hook watches a surrendered predicate). The unbounded is
    /// surrendered, never computed — enforced at firing time, not by TTL
    /// convention.
    #[error("surrender boundary violated at {subject}: {detail}")]
    BoundaryViolation {
        /// The clause or hook IRI that violates the boundary.
        subject: String,
        /// Which boundary rule was violated.
        detail: String,
    },
    /// A cross-domain receipt envelope chain does not link: some envelope's
    /// `previous_envelope_hash` does not equal the prior envelope's
    /// `envelope_hash` exactly (or a non-genesis envelope declares `None`).
    #[error("envelope chain broken at index {index}: {detail}")]
    EnvelopeChainBroken {
        /// The 0-based index of the first envelope whose link fails.
        index: usize,
        /// What was expected vs found.
        detail: String,
    },
    /// An `agent:` node violated the closed-world agent vocabulary or
    /// shape rules (unknown predicate/class, cardinality, registry bound,
    /// out-of-range layer depth), or violated the depth-5 spawn law (a
    /// terminal agent declaring a non-empty spawn set).
    #[error("agent ill-formed at {subject}: {detail}")]
    AgentIllFormed {
        /// The subject IRI of the offending node (or `(registry)`).
        subject: String,
        /// What shape or spawn-law rule was violated.
        detail: String,
    },
    /// A subject has none of the three public-ontology anchors (OWL-Time,
    /// GeoSPARQL, PROV-O) and is therefore not a reality address — refused
    /// rather than returned as an empty-but-valid-looking record.
    #[error("reality address ill-formed at {subject}: {detail}")]
    RealityAddressIllFormed {
        /// The subject IRI that lacks any anchor.
        subject: String,
        /// Which anchors were checked and found absent.
        detail: String,
    },
}
