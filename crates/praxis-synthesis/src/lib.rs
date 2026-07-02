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

pub mod dag;
pub mod datalog;
pub mod pipeline;
pub mod sequence;
pub mod verify;

pub use dag::{Dag, DagReceipt, HashRunner, MemoCache, NodeReceipt, NodeRunner};
pub use datalog::{Atom, DlRule, Program, SaturationReceipt, Term};
pub use pipeline::{Synthesis, SynthesisReceipt};
pub use sequence::{
    BoundStep, BoundedCsp, Capability, SequencePlan, SequenceProblem, SolveReceipt, Solver,
};
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
    /// A structural invariant on inputs was violated (caller error, receipted).
    #[error("invalid input: {detail}")]
    InvalidInput {
        /// What was malformed.
        detail: String,
    },
    /// A verification refinement failed on the pipeline's own artifacts.
    #[error("verification failed: {failed:?}")]
    VerificationFailed {
        /// Names of the refinements that failed.
        failed: Vec<String>,
    },
}
