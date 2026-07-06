//! Tick budgets — the Chatman constant as enforced data.
//!
//! PORTED-FROM: knhk (workspace pkg "genesis" v1.2.0, unpublished)
//!   - /Users/sac/knhk/rust/genesis-mu-kernel/src/timing.rs:87 (`TickBudget`,
//!     branchless `consume`, `BudgetStatus`)
//!   - /Users/sac/knhk/rust/genesis-mu-kernel/src/lib.rs (`CHATMAN_CONSTANT`)
//!   - /Users/sac/knhk/rust/genesis-mu-kernel/src/timing.rs:11 (`TickCounter`
//!     — NOT ported: rdtsc/cntvct is unsafe + arch-specific; praxis budgets
//!     are DECLARED deterministic costs, not measured cycles)
//!
//! DELTAS: rdtsc → abstract [`Ticks`]; no `#[repr(C)]` (no FFI surface);
//! serde derives added (budgets travel inside receipts); unsafe removed
//! (`forbid(unsafe_code)` crate).
//!
//! SYNC: re-diff against the knhk paths above before claiming upstream parity.
//!
//! Path-dependency on knhk was refused with evidence: `genesis-mu-kernel`
//! builds standalone but ships `proptest`/`quickcheck` as regular
//! dependencies and uses `mem::transmute`; `genesis-runtime-primitives`
//! drags tokio/reqwest/opentelemetry via `genesis-otel`. Porting ~120 lines
//! costs less than importing that tree.

use serde::{Deserialize, Serialize};

/// The Chatman constant: the hot-path tick ceiling. The single most
/// load-bearing number in the lineage (CNS 8T → BitActor `TICK_BUDGET 8` →
/// knhk `CHATMAN_CONSTANT` → here).
pub const CHATMAN_CONSTANT: u64 = 8;

/// Abstract execution cost. One tick is one declared unit of bounded work —
/// deliberately NOT a measured CPU cycle (DIVERGES(knhk): `TickCounter`'s
/// rdtsc path assumed a hardcoded 4 GHz and knhk's own hot receipts carried
/// dummy tick values; a declared-cost model is the honest version for a
/// deterministic planner).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct Ticks(pub u64);

/// Result of consuming budget — two states, table-selected, branch-free.
// PORT(knhk): timing.rs `BudgetStatus` — verbatim semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum BudgetStatus {
    /// Within budget.
    Ok = 0,
    /// Budget exceeded.
    Exhausted = 1,
}

/// Tick budget with branchless accounting.
// PORT(knhk): timing.rs:87 `TickBudget` — arithmetic preserved exactly
// (saturating add; `(used > limit) as u8` table lookup).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TickBudget {
    /// Maximum allowed ticks.
    pub limit: u64,
    /// Ticks used so far.
    pub used: u64,
}

impl TickBudget {
    /// Budget fixed at the Chatman constant.
    #[inline(always)]
    #[must_use]
    pub const fn chatman() -> Self {
        Self {
            limit: CHATMAN_CONSTANT,
            used: 0,
        }
    }

    /// Custom budget.
    #[inline(always)]
    #[must_use]
    pub const fn new(limit: u64) -> Self {
        Self { limit, used: 0 }
    }

    /// Whether the budget is spent.
    #[inline(always)]
    #[must_use]
    pub const fn is_exhausted(&self) -> bool {
        self.used >= self.limit
    }

    /// Ticks left.
    #[inline(always)]
    #[must_use]
    pub const fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.used)
    }

    /// Consume ticks, branchlessly reporting the resulting status.
    #[inline(always)]
    pub fn consume(&mut self, ticks: Ticks) -> BudgetStatus {
        self.used = self.used.saturating_add(ticks.0);
        let exhausted = u8::from(self.used > self.limit);
        const STATUS_TABLE: [BudgetStatus; 2] = [BudgetStatus::Ok, BudgetStatus::Exhausted];
        STATUS_TABLE[usize::from(exhausted)]
    }

    /// Reset usage to zero.
    #[inline(always)]
    pub fn reset(&mut self) {
        self.used = 0;
    }
}

/// Compile-time Chatman bound: a type declaring its worst-case ticks proves
/// at compile time that it fits the hot path.
// PORT(knhk): genesis-mu-kernel/src/constitutional.rs `ChatmanBounded` —
// the const-assert pattern, generalized.
pub trait ChatmanBounded {
    /// Declared worst-case ticks.
    const WORST_CASE_TICKS: u64;
    /// Evaluating this constant fails compilation when the bound is broken.
    const CHATMAN_SATISFIED: () = assert!(
        Self::WORST_CASE_TICKS <= CHATMAN_CONSTANT,
        "type exceeds the Chatman constant (8 ticks)"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chatman_budget_exhausts_at_nine_not_eight() {
        // PORT semantics: `used > limit` (strict) — spending exactly the
        // limit is Ok; the ninth tick exhausts.
        let mut b = TickBudget::chatman();
        for _ in 0..8 {
            assert_eq!(b.consume(Ticks(1)), BudgetStatus::Ok);
        }
        assert!(
            b.is_exhausted(),
            "used == limit counts as exhausted for is_exhausted"
        );
        assert_eq!(b.remaining(), 0);
        assert_eq!(b.consume(Ticks(1)), BudgetStatus::Exhausted);
    }

    #[test]
    fn consume_saturates_never_wraps() {
        let mut b = TickBudget::new(4);
        assert_eq!(b.consume(Ticks(u64::MAX)), BudgetStatus::Exhausted);
        assert_eq!(b.consume(Ticks(u64::MAX)), BudgetStatus::Exhausted);
        assert_eq!(b.used, u64::MAX);
        assert_eq!(b.remaining(), 0);
    }

    /// Equivalence with the knhk reference arithmetic over a sweep of the
    /// u64 domain (boundary-heavy sample; property test in spirit without a
    /// proptest dependency).
    #[test]
    fn consume_matches_reference_semantics() {
        let samples: [u64; 9] = [0, 1, 7, 8, 9, 255, u64::MAX / 2, u64::MAX - 1, u64::MAX];
        for &limit in &samples {
            for &spend in &samples {
                let mut b = TickBudget::new(limit);
                let status = b.consume(Ticks(spend));
                let ref_used = spend; // saturating from 0
                let ref_status = if ref_used > limit {
                    BudgetStatus::Exhausted
                } else {
                    BudgetStatus::Ok
                };
                assert_eq!(status, ref_status, "limit={limit} spend={spend}");
                assert_eq!(b.used, ref_used);
            }
        }
    }

    #[test]
    fn reset_restores_full_budget() {
        let mut b = TickBudget::chatman();
        b.consume(Ticks(8));
        b.reset();
        assert_eq!(b.remaining(), 8);
        assert!(!b.is_exhausted());
    }

    struct HotOp;
    impl ChatmanBounded for HotOp {
        const WORST_CASE_TICKS: u64 = 7;
    }

    #[test]
    fn chatman_bounded_compiles_for_conforming_types() {
        // Evaluating the const proves the bound at compile time.
        #[allow(clippy::let_unit_value)]
        let () = HotOp::CHATMAN_SATISFIED;
    }
}
