//! Family F21 -- "Parent-Child Closure" (atlas ticket V12-021).
//!
//! Survey verdict: **MIXED**. This module is a Wire-phase-1 pass over the survey's
//! `ALREADY_BUILT` / `HAND_WRITE_REQUIRED` breakdown, not a from-scratch implementation --
//! every type re-exported below is real, independently-tested code in
//! `praxis_graphlaw::chatman::closure` (verified this session by reading the source directly:
//! `crates/praxis-graphlaw/src/chatman/closure.rs`, 573 lines, plus its 666-line
//! `closure_test.rs`). `praxis-graphlaw` is already a real `path` dependency of this crate
//! (added by an earlier Wire pass for F06/F01/F05/F07/F09; see `Cargo.toml`), so no new
//! dependency edge was required for this family. Per `.claude/rules/no-overclaiming.md`,
//! everything under "What is ALIVE below" was verified this session by reading the cited code
//! and by this module's own passing tests; everything under "What is still not wired" is
//! disclosed as a gap, not dressed up as done.
//!
//! # What is ALIVE below (re-exports + one composition function, this session)
//!
//! 1. **Closure-law decision core** (`ALREADY_BUILT` per the survey) --
//!    [`ChildCompletionState`] (the `Open`/`Observed`/`Admitted` three-state admission gate,
//!    PRD §9 line 525: "a child completion signal SHALL be treated as observation until
//!    admitted"), [`ClosureLaw`] (all six PRD-named variants: `AllRequired`, `Quorum(u32)`,
//!    `AnySufficient`, `OrderedSubset(Vec<WorkflowSocketId>)`, `PolicyDecides`,
//!    `FirstConformant` -- confirmed by reading `closure.rs:96-136`), and
//!    [`RecursiveSocketClosure`] itself (`declare`/`observe`/`admit`/
//!    `promote_observed_to_admitted`/`record_policy_decision`/`is_closed`/`close`, keyed by a
//!    deterministic `BTreeMap<WorkflowSocketId, ChildCompletionState>` -- never `HashMap`,
//!    confirmed by `closure.rs:181` and exercised by its own
//!    `declared_child_order_is_deterministic_across_runs` test).
//! 2. **Real, non-decorative child-set sourcing** -- [`RecursiveSocketClosure::declare`] sources
//!    its child set exclusively from a caller-supplied
//!    [`ParentChildClosure::children_of`] (re-exported below from `powl2_decompose`), a real
//!    deterministic-DFS-built index (`powl2-decompose/src/powl.rs:387-476`), not a reinvented
//!    child-identity scheme.
//! 3. **Eight typed refusal variants**, all real and each independently tested end-to-end in
//!    `closure_test.rs` (confirmed by reading `abi.rs:474-536` and cross-checking each name
//!    appears there): [`Refusal::ClosureLawNoChildren`], [`Refusal::ClosureLawQuorumOutOfRange`],
//!    [`Refusal::ClosureLawUnknownChild`], [`Refusal::ClosureLawOrderedSubsetInvalid`],
//!    [`Refusal::ClosureLawPolicyNotDeclared`], [`Refusal::ChildConformanceRefused`],
//!    [`Refusal::ChildCompletionUnadmitted`], [`Refusal::ParentClosureUnsatisfied`]. There is no
//!    separate `ParentClosureRefused` type in the real code -- the family survey's requirements
//!    summary names one, but the actual, already-tested implementation expresses every one of
//!    these refusals as a variant of the shared `chatman::abi::Refusal` enum. This module
//!    re-exports that real enum rather than inventing a `ParentClosureRefused` wrapper type that
//!    would not correspond to anything the underlying code actually returns.
//! 4. [`admit_child_and_evaluate`] -- a genuine (not decorative) composition function added in
//!    this module: it chains the real [`RecursiveSocketClosure::observe`] (Child Observation
//!    Gate), [`RecursiveSocketClosure::promote_observed_to_admitted`] (Child Admission, gated on
//!    real SHACL [`ValidationReport`] conformance evidence), and
//!    [`RecursiveSocketClosure::is_closed`] (Closure Evaluator) into the single call the
//!    family's L1/L2 pipeline names as three consecutive steps. It reimplements none of their
//!    logic and adds no new refusal variant of its own -- every `Err` it can return is one of
//!    the real variants named in (3) above, propagated with `?`. This is a real new call path,
//!    exercised by this module's own `#[cfg(test)]` tests (positive close, non-conforming
//!    refusal, unknown-child refusal, and idempotent re-admission of an already-admitted child
//!    not double-counting toward `Quorum`) written and run this session -- but, like F06's
//!    `route_and_execute_n3`, it lives in `multifractal-workflow`, not inside
//!    `praxis-graphlaw::chatman::engine`'s own admission pipeline; see gap (a) below.
//!
//! # What is still not wired (disclosed gaps, not fixed by this pass)
//!
//! (a) **This module's composition function is not called from `ChatmanEngine`'s own production
//!     path.** The real production caller of the closure-law core is
//!     `ChatmanEngine::admit_child_completion` (`engine.rs:835-849`), which already exists,
//!     already re-runs the engine's real S1 snapshot-presence check, and is out of scope for
//!     this module to touch or duplicate. [`admit_child_and_evaluate`] below is a real,
//!     independently tested convenience composition over the same library calls, not a retrofit
//!     of the engine's own pipeline.
//! (b) **No graph-backed [`ClosureReceipt`]** -- `RecursiveSocketClosure::is_closed`/`close`
//!     return a plain `Result<bool, Refusal>` / `Result<(), Refusal>`; there is no BLAKE3
//!     receipt, no `ChildObservation -> ChildAdmissionReceipt -> ClosureLaw -> AdmittedChildSet
//!     -> ClosureDecision -> PolicyDecision -> ParentState -> ClosureReceipt` `prov:wasDerivedFrom`
//!     chain, and no replay-equivalence verification (F21-L6). This is real algorithmic and
//!     schema-design work -- deciding what the receipt covers and how it chains -- not
//!     mechanical scaffolding, so it is not attempted here. No function in this module claims to
//!     produce a `ClosureReceipt`; none exists.
//! (c) **No multi-level/cascading closure.** A child socket's own closure becoming an admitted
//!     consequence that feeds a grandparent's admitted-child-set is not implemented anywhere in
//!     `closure.rs`, and this module does not add it either -- confirmed this session by
//!     re-reading `closure.rs` end to end: every method operates on exactly one
//!     `RecursiveSocketClosure` and never constructs or consults another one. The
//!     `RECURSIVE_CHILD_CLOSURE_PROVEN` exit bar (F21-L1/L3/L8), read literally as a
//!     multi-level cascade proof, is not met by anything in this crate.
//! (d) **No L7 concurrency/chaos-recovery semantics.** Duplicate-event/process-restart/stale-
//!     result handling via an atomic idempotency+correlation gate and a durable receipt-head/
//!     replay-state is not implemented; `RecursiveSocketClosure` is pure in-memory,
//!     single-process state with no persistence. `observe`/`admit`/
//!     `promote_observed_to_admitted` are idempotent *within a single process* (re-admitting an
//!     already-`Admitted` child is a documented no-op) but nothing here survives a restart.
//!
//! # Survey-cited paths for F21
//! - /Users/sac/Downloads/v26.7.12_mermaid_atlas/families/F21_parent-child-closure.md
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/closure.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/closure_test.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/mod.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/abi.rs
//! - /Users/sac/praxis/crates/powl2-decompose/src/powl.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/engine.rs
//! - /Users/sac/praxis/docs/jira/v26.7.11/PATH_TO_100.md
//! - /Users/sac/praxis/crates/cng/src/bench/dispatch.rs
//! - /Users/sac/praxis/Cargo.toml

// ---- Closure-law decision core: ALREADY_BUILT, re-exported (not reimplemented). ----
pub use praxis_graphlaw::chatman::closure::{
    ChildCompletionState, ClosureLaw, RecursiveSocketClosure,
};

// ---- Cross-cutting ABI/graph types the closure core is expressed in terms of. ----
pub use praxis_graphlaw::chatman::abi::{OperatorId, Refusal};
pub use praxis_graphlaw::shacl::ValidationReport;

// ---- Real child-identity/child-set source: re-exported (never reinvented locally). ----
pub use powl2_decompose::{ParentChildClosure, Powl, SocketKind, SocketPath, WorkflowSocketId};

/// Admits `child` under `rsc` and reports whether the parent recursive socket closes as a
/// result -- the family's Child Observation Gate, Child Admission, and Closure Evaluator steps
/// (F21-L1/L2) chained into one call.
///
/// Calls, in order: [`RecursiveSocketClosure::observe`] (idempotent; a completion signal is an
/// observation, never itself a closure event -- PRD §9 line 525), then
/// [`RecursiveSocketClosure::promote_observed_to_admitted`] (gated on `evidence.conforms`; an
/// already-`Admitted` child is a documented no-op, so re-admitting the same child twice never
/// double-counts toward a [`ClosureLaw::Quorum`]), then
/// [`RecursiveSocketClosure::is_closed`]. A `false` result is a legitimate open parent, not an
/// error -- consistent with [`RecursiveSocketClosure::is_closed`]'s own contract.
///
/// This function adds no new refusal semantics of its own: every `Err` variant it can return is
/// one of the eight real [`Refusal`] variants named in this module's doc comment, propagated
/// unchanged from the calls above.
///
/// # Errors
/// - [`Refusal::ClosureLawUnknownChild`] if `child` is not part of `rsc`'s declared closure.
/// - [`Refusal::ChildConformanceRefused`] if `child` is `Observed` and `evidence.conforms` is
///   `false` -- `child` stays `Observed`, never silently promoted.
///
/// # Complexity
/// O(log c) for the `observe`/`promote_observed_to_admitted` lookups plus O(c) (or O(k log c)
/// for [`ClosureLaw::OrderedSubset`]) for [`RecursiveSocketClosure::is_closed`], c = declared
/// child count -- identical bounds to the three calls it composes, since this function adds only
/// O(1) glue around them.
pub fn admit_child_and_evaluate(
    rsc: &mut RecursiveSocketClosure,
    child: &WorkflowSocketId,
    evidence: &ValidationReport,
) -> Result<bool, Refusal> {
    rsc.observe(child)?;
    rsc.promote_observed_to_admitted(child, evidence)?;
    rsc.is_closed()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use praxis_graphlaw::shacl::ValidationResult;
    use praxis_graphlaw::term::Term;

    /// A `ValidationReport` with zero results -- real SHACL Core conformance
    /// (`conforms` iff `results.is_empty()`), mirroring
    /// `praxis-graphlaw`'s own `closure_test.rs::conforming_evidence` helper.
    fn conforming_evidence() -> ValidationReport {
        ValidationReport {
            conforms: true,
            results: Vec::new(),
        }
    }

    /// A `ValidationReport` carrying one real violation -- genuinely non-conforming evidence.
    fn nonconforming_evidence() -> ValidationReport {
        ValidationReport {
            conforms: false,
            results: vec![ValidationResult {
                focus_node: Term::parse("<urn:test:focus-node>".to_string()),
                result_path: None,
                value: None,
                source_constraint_component: Term::parse(
                    "<http://www.w3.org/ns/shacl#MinCountConstraintComponent>".to_string(),
                ),
                source_shape: Term::parse("<urn:test:shape>".to_string()),
                severity: Term::parse("<http://www.w3.org/ns/shacl#Violation>".to_string()),
                message: Some("f21 test-injected violation".to_string()),
            }],
        }
    }

    fn root_partial_order_over(n: usize) -> Powl {
        let children = (0..n)
            .map(|i| Powl::Leaf(Some(format!("leaf-{i}"))))
            .collect();
        Powl::PartialOrder {
            children,
            order: BTreeSet::new(),
        }
    }

    fn root_socket() -> WorkflowSocketId {
        WorkflowSocketId {
            path: SocketPath::root(),
            kind: SocketKind::PartialOrder,
        }
    }

    fn leaf_socket(i: usize) -> WorkflowSocketId {
        WorkflowSocketId {
            path: SocketPath::root().child(i),
            kind: SocketKind::Leaf,
        }
    }

    #[test]
    fn admit_child_and_evaluate_closes_a_single_child_all_required_parent() {
        let model = root_partial_order_over(1);
        let pcc = ParentChildClosure::from_model(&model);
        let mut rsc = RecursiveSocketClosure::declare(&pcc, root_socket(), ClosureLaw::AllRequired)
            .expect("one declared leaf child");

        let closed = admit_child_and_evaluate(&mut rsc, &leaf_socket(0), &conforming_evidence())
            .expect("known child, conforming evidence");

        assert!(closed);
        assert_eq!(
            rsc.children().get(&leaf_socket(0)),
            Some(&ChildCompletionState::Admitted)
        );
    }

    #[test]
    fn admit_child_and_evaluate_refuses_nonconforming_evidence_and_leaves_child_observed() {
        let model = root_partial_order_over(1);
        let pcc = ParentChildClosure::from_model(&model);
        let mut rsc = RecursiveSocketClosure::declare(&pcc, root_socket(), ClosureLaw::AllRequired)
            .expect("one declared leaf child");

        let err = admit_child_and_evaluate(&mut rsc, &leaf_socket(0), &nonconforming_evidence())
            .expect_err("nonconforming evidence must refuse, never silently admit");

        assert!(matches!(err, Refusal::ChildConformanceRefused(_)));
        assert_eq!(
            rsc.children().get(&leaf_socket(0)),
            Some(&ChildCompletionState::Observed),
            "a refused promotion must not advance the child past Observed"
        );
    }

    #[test]
    fn admit_child_and_evaluate_refuses_an_unknown_child() {
        let model = root_partial_order_over(1);
        let pcc = ParentChildClosure::from_model(&model);
        let mut rsc = RecursiveSocketClosure::declare(&pcc, root_socket(), ClosureLaw::AllRequired)
            .expect("one declared leaf child");
        let stranger = WorkflowSocketId {
            path: SocketPath::root().child(99),
            kind: SocketKind::Leaf,
        };

        let err = admit_child_and_evaluate(&mut rsc, &stranger, &conforming_evidence())
            .expect_err("a child never declared under this socket must refuse");

        assert!(matches!(err, Refusal::ClosureLawUnknownChild(_)));
    }

    #[test]
    fn admit_child_and_evaluate_is_idempotent_and_never_double_counts_toward_quorum() {
        // Quorum(1) over 2 children: only one admitted child is required to close, and
        // re-admitting the SAME child twice must not fabricate a second admission.
        let model = root_partial_order_over(2);
        let pcc = ParentChildClosure::from_model(&model);
        let mut rsc = RecursiveSocketClosure::declare(&pcc, root_socket(), ClosureLaw::Quorum(2))
            .expect("two declared leaf children, quorum 2 is in range");

        let first = admit_child_and_evaluate(&mut rsc, &leaf_socket(0), &conforming_evidence())
            .expect("known child, conforming evidence");
        assert!(!first, "quorum(2) must stay open after only one admission");

        let second = admit_child_and_evaluate(&mut rsc, &leaf_socket(0), &conforming_evidence())
            .expect("re-admitting an already-admitted child is a documented no-op, not an error");
        assert!(
            !second,
            "re-admitting the same child must not silently satisfy quorum(2) on its own"
        );

        let third = admit_child_and_evaluate(&mut rsc, &leaf_socket(1), &conforming_evidence())
            .expect("second distinct child, conforming evidence");
        assert!(
            third,
            "quorum(2) closes once two distinct children are admitted"
        );
    }
}
