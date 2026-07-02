//! # praxis-proposer — PR-14 proposer layer (Vision 2030)
//!
//! Given an observed [`domain::RevenueState`] and a *domain-authored*
//! [`objective::ObjectiveFunction`], enumerate lawful candidate goal states,
//! score them, and return a ranked list of [`proposer::Proposal`]s with
//! auditable rationales and blake3 proposal hashes.
//!
//! ## Boundary position (AR-9)
//!
//! This crate sits **outside** the admission boundary. Its outputs are
//! untrusted observations (O, not O*): every proposal must pass Rice
//! quarantine and admission before any effect. Nothing here judges, admits,
//! receipts, or executes.
//!
//! ## No value discovery (Non-goal 1)
//!
//! The system never invents values. The objective function is data authored
//! by the domain owner (see `revenue_objective.json` and the schema in
//! [`objective`]); this crate contributes only enumeration and algebra.
//!
//! ## Integration state (workspace member since Genesis Day 1)
//!
//! This crate is a praxis workspace member and depends on `praxis-core`.
//! The two doc-marked swap points from its standalone phase resolved as:
//!
//! - [`domain::RevenueState`] — stays a local type (praxis-core has no
//!   revenue vocabulary), but gains [`domain::RevenueState::from_admitted`],
//!   the adapter that observes the payload of a
//!   `LawObject<RevenueState, Admitted, _>` — the proposer can now observe
//!   admitted reality rather than raw input.
//! - [`proposer::Proposal::pddl_goal`] — still emits plain goal-atom text;
//!   the bcinr-pddl splice lives in the root crate's `propose` verbs (see
//!   `src/verbs/propose.rs`), which substitute the atom into a PDDL problem
//!   `:goal` block consumable by `plan solve`. This crate deliberately does
//!   not depend on bcinr-pddl: the proposer emits observations, the planner
//!   adapter belongs to the caller that owns the planning surface.
//!
//! The domain vocabulary is mirrored in Turtle at `ontology/revenue.ttl`,
//! and a PDDL8-safe planning domain for the same vocabulary ships at
//! `ontology/revenue.pddl`.

pub mod domain;
pub mod objective;
pub mod proposer;

pub use domain::{evidence_permits, lawful_targets, Account, RevenueState, Stage};
pub use objective::{compute_fluents, ObjectiveError, ObjectiveFunction, FLUENT_NAMES};
pub use proposer::{Proposal, Proposer, MAX_PROPOSALS, PROPOSAL_HASH_DOMAIN};
