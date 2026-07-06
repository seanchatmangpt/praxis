//! Runtime classes and per-class failure actions — knhk's failure
//! classification, wired to praxis's certified refusals.
//!
//! PORTED-FROM: knhk (workspace pkg "genesis" v1.2.0, unpublished)
//!   - /Users/sac/knhk/rust/genesis-etl/src/runtime_class.rs (`RuntimeClass`,
//!     `RuntimeClassMetadata`, `classify_operation` incl. the R1→W1 overflow
//!     rule for data_size > 8)
//!   - /Users/sac/knhk/rust/genesis-etl/src/failure_actions.rs
//!     (`handle_{r1,w1,c1}_failure` decision semantics: R1 park+escalate,
//!     W1 retry-then-degrade, C1 async-finalize-never-block)
//!
//! DELTAS: genesis-etl's `LoadResult`/`Receipt`/OTel plumbing dropped — the
//! decision functions are PURE here, returning a [`FailureAction`] the
//! executor actuates; a budget breach becomes a *certified*
//! [`crate::Refusal`], which knhk lacked (its escalation was a metric, not a
//! proof). Path-dep refused: genesis-etl drags rdkafka/oxigraph/otel.
//!
//! SYNC: re-diff against the knhk paths above before claiming upstream parity.

use serde::{Deserialize, Serialize};

use crate::budget::CHATMAN_CONSTANT;

/// Runtime class: which budget tier an operation belongs to.
// PORT(knhk): runtime_class.rs `RuntimeClass` — R1/W1/C1 tiers verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RuntimeClass {
    /// Hot: bounded lookups/checks, ≤ 8 items, ≤ 8 ticks.
    R1,
    /// Warm: manufacture/transform, ≤ 500 µs budget.
    W1,
    /// Cold: full search/analytics, ≤ 200 ms budget.
    C1,
}

/// Per-class budget metadata.
// PORT(knhk): runtime_class.rs `RuntimeClassMetadata` — same numbers;
// `operation_type: String` dropped (allocation-free).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassBudget {
    /// The class this budget describes.
    pub class: RuntimeClass,
    /// Tick budget for R1 (the Chatman constant) — `None` for time-tier classes.
    pub ticks: Option<u64>,
    /// Wall budget in nanoseconds.
    pub budget_ns: u64,
    /// p99 SLO in nanoseconds.
    pub slo_p99_ns: u64,
}

impl RuntimeClass {
    /// The class's budget contract.
    #[must_use]
    pub const fn budget(self) -> ClassBudget {
        match self {
            RuntimeClass::R1 => ClassBudget {
                class: RuntimeClass::R1,
                ticks: Some(CHATMAN_CONSTANT),
                budget_ns: 8,
                slo_p99_ns: 2,
            },
            RuntimeClass::W1 => ClassBudget {
                class: RuntimeClass::W1,
                ticks: None,
                budget_ns: 500_000,
                slo_p99_ns: 1_000_000,
            },
            RuntimeClass::C1 => ClassBudget {
                class: RuntimeClass::C1,
                ticks: None,
                budget_ns: 200_000_000,
                slo_p99_ns: 500_000_000,
            },
        }
    }
}

/// What a plan node *does*, for classification purposes. praxis's analogue
/// of knhk's operation-type strings, as a closed enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    /// Membership/threshold check against known facts.
    Check,
    /// Bounded manufacture: apply a capability's effects.
    Apply,
    /// Search: solver/planning work.
    Search,
    /// Aggregation/analytics over many tuples.
    Analyze,
}

/// Classify a node by kind and data size.
// PORT(knhk): runtime_class.rs `classify_operation` — including the
// R1→W1 overflow rule: hot-kind work over more than 8 items is warm
// (MAX_RUN_LEN ≤ 8 guard preserved).
#[must_use]
pub fn classify(kind: NodeKind, data_size: usize) -> RuntimeClass {
    match kind {
        NodeKind::Check | NodeKind::Apply if data_size <= 8 => RuntimeClass::R1,
        NodeKind::Check | NodeKind::Apply => RuntimeClass::W1,
        NodeKind::Search => RuntimeClass::W1,
        NodeKind::Analyze => RuntimeClass::C1,
    }
}

/// The lawful actions an executor may take on a classified failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureAction {
    /// Try again (W1 discipline), up to the stated remaining attempts.
    Retry {
        /// Attempts remaining after this decision.
        remaining: u8,
    },
    /// Park the work for later re-admission (R1 discipline: never burn the
    /// hot path on a struggling item).
    Park,
    /// Refuse with a certificate: the budget/authority breach is proven,
    /// not just counted. (Praxis upgrade — knhk emitted a metric here.)
    Refuse {
        /// Rendered reason head for the refusal register.
        reason: String,
    },
    /// Hand upward: outside this executor's lawful responses.
    Escalate,
    /// Degrade to a cached prior answer (W1 exhausted retries with a memo
    /// hit available).
    Degrade,
    /// Finalize asynchronously; never block the hot path (C1 discipline).
    AsyncFinalize,
}

/// R1 failure decision: park, and escalate-with-certificate on budget breach.
// PORT(knhk): failure_actions.rs `handle_r1_failure` — park-always
// semantics preserved; escalation upgraded from OTel metric to refusal.
#[must_use]
pub fn handle_r1_failure(budget_exceeded: bool) -> FailureAction {
    if budget_exceeded {
        FailureAction::Refuse {
            reason: format!("R1 budget breach: exceeded {CHATMAN_CONSTANT} ticks on the hot path"),
        }
    } else {
        FailureAction::Park
    }
}

/// W1 failure decision: retry up to `max_retries`, then degrade if a cached
/// answer exists, else park.
// PORT(knhk): failure_actions.rs `handle_w1_failure` — retry×N-then-degrade.
#[must_use]
pub fn handle_w1_failure(retry_count: u8, max_retries: u8, cache_available: bool) -> FailureAction {
    if retry_count < max_retries {
        FailureAction::Retry {
            remaining: max_retries - retry_count - 1,
        }
    } else if cache_available {
        FailureAction::Degrade
    } else {
        FailureAction::Park
    }
}

/// C1 failure decision: always asynchronous, never block.
// PORT(knhk): failure_actions.rs `handle_c1_failure`.
#[must_use]
pub fn handle_c1_failure() -> FailureAction {
    FailureAction::AsyncFinalize
}

/// Total dispatch: one decision function per class.
#[must_use]
pub fn decide(
    class: RuntimeClass,
    budget_exceeded: bool,
    retry_count: u8,
    max_retries: u8,
    cache_available: bool,
) -> FailureAction {
    match class {
        RuntimeClass::R1 => handle_r1_failure(budget_exceeded),
        RuntimeClass::W1 => handle_w1_failure(retry_count, max_retries, cache_available),
        RuntimeClass::C1 => handle_c1_failure(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classification_is_total_and_respects_the_overflow_rule() {
        // Hot kinds stay hot at ≤ 8 items…
        assert_eq!(classify(NodeKind::Check, 8), RuntimeClass::R1);
        assert_eq!(classify(NodeKind::Apply, 1), RuntimeClass::R1);
        // …and overflow to warm past 8 (MAX_RUN_LEN guard).
        assert_eq!(classify(NodeKind::Check, 9), RuntimeClass::W1);
        assert_eq!(classify(NodeKind::Apply, 10_000), RuntimeClass::W1);
        // Search is warm; analytics are cold, regardless of size.
        assert_eq!(classify(NodeKind::Search, 1), RuntimeClass::W1);
        assert_eq!(classify(NodeKind::Analyze, 1), RuntimeClass::C1);
    }

    #[test]
    fn budgets_carry_the_knhk_numbers() {
        assert_eq!(RuntimeClass::R1.budget().ticks, Some(8));
        assert_eq!(RuntimeClass::W1.budget().budget_ns, 500_000);
        assert_eq!(RuntimeClass::C1.budget().slo_p99_ns, 500_000_000);
    }

    #[test]
    fn r1_parks_normally_and_refuses_with_reason_on_budget_breach() {
        assert_eq!(handle_r1_failure(false), FailureAction::Park);
        match handle_r1_failure(true) {
            FailureAction::Refuse { reason } => assert!(reason.contains("8 ticks")),
            other => panic!("expected certified refusal, got {other:?}"),
        }
    }

    #[test]
    fn w1_retries_then_degrades_then_parks() {
        assert_eq!(
            handle_w1_failure(0, 3, false),
            FailureAction::Retry { remaining: 2 }
        );
        assert_eq!(
            handle_w1_failure(2, 3, false),
            FailureAction::Retry { remaining: 0 }
        );
        assert_eq!(handle_w1_failure(3, 3, true), FailureAction::Degrade);
        assert_eq!(handle_w1_failure(3, 3, false), FailureAction::Park);
    }

    #[test]
    fn c1_never_blocks() {
        assert_eq!(handle_c1_failure(), FailureAction::AsyncFinalize);
    }
}
