//! Maximum Reachable Revenue (MRR) — the revenue-physics upper bound.
//!
//! Given an observed [`RevenueState`], MRR answers a single question with a
//! single number: *if every account that could lawfully be closed were
//! closed, how much revenue would be realized?* It is the ceiling the
//! proposer's ranked candidates chase, expressed as a physical fact about the
//! pipeline rather than a scored preference.
//!
//! # Why this is objective-independent (Revenue *Physics*, not scoring)
//!
//! Realizable revenue is not a matter of taste, so it does not depend on the
//! authored [`ObjectiveFunction`] weights. It is defined by the same
//! `realized_revenue` fluent the objective *scores*
//! ([`crate::objective::compute_fluents`] index 0): an account realizes its
//! `amount_cents` exactly when it reaches [`Stage::ClosedWon`], and zero
//! otherwise. The objective decides *which* lawful move to prefer; MRR asks
//! only *whether* the closed-won state is lawfully reachable at all. Passing
//! an objective in therefore changes ranking downstream but never changes
//! MRR — a property [`maximum_reachable_revenue`] documents by not taking an
//! objective argument.
//!
//! # Boundedness argument (why a sum, not a search)
//!
//! The naive maximum is taken over the combinatorial space of *all* lawful
//! proposal combinations across accounts — one choice of forward target (or
//! "leave alone") per account, i.e. `∏ₐ (1 + |lawful_targets(a)|)` joint
//! states, exponential in the account count. It collapses to a linear sum
//! because **accounts are independent**:
//!
//! - The `realized_revenue` fluent of an account depends only on *that*
//!   account's `amount_cents` and target stage (see `compute_fluents`).
//! - The evidence gates ([`evidence_permits`]) that decide whether a target
//!   is lawful read only *that* account's own evidence flags.
//! - The hand-authored planning domain (`ontology/revenue.pddl`) advances one
//!   account at a time and shares no resource, budget, or mutex across
//!   accounts, so no account's move can preclude another's.
//!
//! With no cross-account coupling, the objective "total realized revenue"
//! separates additively, and `max` distributes over the independent sum:
//!
//! ```text
//! max_{joint plan}  Σₐ realized(a)  =  Σₐ  max_{lawful target of a} realized(a)
//! ```
//!
//! Each per-account maximum is itself over at most
//! `Stage::ALL.len() - 1 == 4` lawful forward targets ([`lawful_targets`] is
//! bounded by construction), so the whole computation is `O(accounts)` and
//! needs no enumeration of the joint space. That is the entire boundedness
//! claim: independence turns an exponential search into a bounded sum.

use serde::{Deserialize, Serialize};

use crate::domain::{evidence_permits, lawful_targets, Account, RevenueState, Stage};
use crate::objective::compute_fluents;

/// Per-account contribution to the [`MrrReport`], with the attribution a
/// reader needs to see *why* an account does or does not add to the ceiling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountRevenue {
    /// Account id, verbatim from the observed state.
    pub account_id: String,
    /// The account's deal size in integer cents.
    pub amount_cents: i64,
    /// The most revenue this account can lawfully realize: `amount_cents`
    /// if it is already closed-won or can lawfully reach closed-won, else `0`.
    pub max_realizable_cents: i64,
    /// Whether the account has already realized its revenue (stage is
    /// closed-won). Such accounts count toward MRR *and* actual-closed.
    pub already_closed: bool,
    /// Whether closed-won is a lawful forward target from the account's
    /// current stage (full evidence present). `false` for already-closed
    /// accounts (nothing forward of closed-won) — read `already_closed` too.
    pub closeable: bool,
    /// When `max_realizable_cents == 0`, the missing evidence that gates the
    /// account out of closed-won (`None` when the account does contribute).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_on: Option<Vec<String>>,
}

/// The three headline numbers plus per-account attribution.
///
/// All revenue figures are exact integer cents (the "chunk-sized numbers" the
/// pipeline hashes and receipts). [`MrrReport::revenue_utilization`] is the
/// only float and is confined to `[0.0, 1.0]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MrrReport {
    /// **Maximum Reachable Revenue**: `Σ max_realizable_cents` over all
    /// accounts. The lawful ceiling on realized revenue for this state.
    pub max_reachable_revenue_cents: i64,
    /// **Actual closed revenue**: `Σ amount_cents` over accounts already at
    /// closed-won. Always `≤ max_reachable_revenue_cents`.
    pub actual_closed_cents: i64,
    /// **Revenue Opportunity** (the gap): `MRR − actual_closed`. The revenue
    /// that is lawfully reachable but not yet realized.
    pub revenue_opportunity_cents: i64,
    /// **Revenue Utilization**: `actual_closed / MRR`, always in `[0.0, 1.0]`.
    /// Defined as `0.0` when `MRR == 0` (nothing reachable ⇒ nothing to
    /// utilize), documented rather than a division by zero.
    pub revenue_utilization: f64,
    /// The count of accounts considered (after any upstream constraint
    /// filtering the caller applied to the state).
    pub accounts_considered: usize,
    /// Per-account attribution, in the input account order.
    pub accounts: Vec<AccountRevenue>,
}

/// The evidence flags an account is missing for a lawful closed-won move,
/// as stable lower-snake strings matching the `Account` field names.
fn missing_close_evidence(a: &Account) -> Vec<String> {
    let mut missing = Vec::new();
    if !a.legal_approved {
        missing.push("legal_approved".to_string());
    }
    if !a.security_review_done {
        missing.push("security_review_done".to_string());
    }
    if !a.exec_sponsor {
        missing.push("exec_sponsor".to_string());
    }
    missing
}

/// The most revenue a single account can lawfully realize.
///
/// Reuses the shared `realized_revenue` fluent so this can never drift from
/// how the objective values a close: it is `amount_cents` iff the account is
/// already closed-won or has a lawful forward target of closed-won, else `0`.
fn account_max_realizable(a: &Account) -> i64 {
    if a.stage == Stage::ClosedWon {
        return a.amount_cents; // revenue already realized
    }
    // realized_revenue fluent (index 0) over each lawful forward target; it is
    // nonzero only for the closed-won target, so the max is `amount_cents`
    // exactly when closed-won is lawfully reachable.
    lawful_targets(a)
        .iter()
        .map(|&t| compute_fluents(a, t)[0] as i64)
        .max()
        .unwrap_or(0)
}

/// Compute the [`MrrReport`] for an observed revenue snapshot.
///
/// Objective-independent by construction (see the module docs): the ceiling
/// is a physical fact about lawful reachability, not a scored preference.
/// Linear in the number of accounts; no enumeration of joint plans.
pub fn maximum_reachable_revenue(state: &RevenueState) -> MrrReport {
    let mut accounts = Vec::with_capacity(state.accounts.len());
    let mut mrr: i64 = 0;
    let mut actual_closed: i64 = 0;

    for a in &state.accounts {
        let already_closed = a.stage == Stage::ClosedWon;
        let closeable = !already_closed && evidence_permits(a, Stage::ClosedWon);
        let max_realizable = account_max_realizable(a);
        mrr += max_realizable;
        if already_closed {
            actual_closed += a.amount_cents;
        }
        let blocked_on = if max_realizable == 0 {
            Some(missing_close_evidence(a))
        } else {
            None
        };
        accounts.push(AccountRevenue {
            account_id: a.id.clone(),
            amount_cents: a.amount_cents,
            max_realizable_cents: max_realizable,
            already_closed,
            closeable,
            blocked_on,
        });
    }

    let opportunity = mrr - actual_closed;
    let utilization = if mrr == 0 {
        0.0
    } else {
        actual_closed as f64 / mrr as f64
    };

    MrrReport {
        max_reachable_revenue_cents: mrr,
        actual_closed_cents: actual_closed,
        revenue_opportunity_cents: opportunity,
        revenue_utilization: utilization,
        accounts_considered: state.accounts.len(),
        accounts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn acct(id: &str, stage: Stage, amount: i64, legal: bool, sec: bool, exec: bool) -> Account {
        Account {
            id: id.into(),
            stage,
            amount_cents: amount,
            security_review_done: sec,
            legal_approved: legal,
            exec_sponsor: exec,
            days_in_stage: 0,
        }
    }

    /// The Day-2 fixture, mirrored from `src/revenue.rs::fixture_state`.
    fn fixture() -> RevenueState {
        RevenueState {
            accounts: vec![
                acct("acct-apex", Stage::Proposal, 5_000_000, true, true, true),
                acct(
                    "acct-legal-gap",
                    Stage::Qualified,
                    3_000_000,
                    false,
                    true,
                    true,
                ),
                acct("acct-fresh", Stage::Lead, 1_000_000, false, false, false),
                acct("acct-closed", Stage::ClosedWon, 500_000, true, true, true),
            ],
        }
    }

    #[test]
    fn fixture_mrr_is_the_sum_of_closeable_and_closed() {
        let r = maximum_reachable_revenue(&fixture());
        // apex (closeable, full evidence) + closed (already realized).
        // legal-gap (no legal) and fresh (no evidence) contribute 0.
        assert_eq!(r.max_reachable_revenue_cents, 5_000_000 + 500_000);
        assert_eq!(r.actual_closed_cents, 500_000);
        assert_eq!(r.revenue_opportunity_cents, 5_000_000);
        assert!((r.revenue_utilization - (500_000.0 / 5_500_000.0)).abs() < 1e-12);
    }

    #[test]
    fn mrr_is_invariant_to_account_ordering() {
        let base = fixture();
        let mut shuffled = base.clone();
        shuffled.accounts.reverse();
        let a = maximum_reachable_revenue(&base);
        let b = maximum_reachable_revenue(&shuffled);
        assert_eq!(a.max_reachable_revenue_cents, b.max_reachable_revenue_cents);
        assert_eq!(a.actual_closed_cents, b.actual_closed_cents);
        assert_eq!(a.revenue_opportunity_cents, b.revenue_opportunity_cents);
        assert_eq!(a.revenue_utilization, b.revenue_utilization);
    }

    #[test]
    fn removing_legal_lowers_mrr_by_exactly_that_accounts_contribution() {
        let base = fixture();
        let before = maximum_reachable_revenue(&base).max_reachable_revenue_cents;

        // Strip legal_approved from the one closeable, not-yet-closed account.
        let mut stripped = base.clone();
        let apex = stripped
            .accounts
            .iter_mut()
            .find(|a| a.id == "acct-apex")
            .expect("apex present");
        let apex_amount = apex.amount_cents;
        apex.legal_approved = false;

        let after = maximum_reachable_revenue(&stripped).max_reachable_revenue_cents;
        assert_eq!(
            before - after,
            apex_amount,
            "removing legal_approved must drop MRR by exactly apex's contribution"
        );
    }

    #[test]
    fn utilization_is_always_in_unit_interval() {
        // A spread of states, including the all-closed and all-blocked extremes.
        let states = [
            fixture(),
            RevenueState { accounts: vec![] },
            RevenueState {
                accounts: vec![acct("only-closed", Stage::ClosedWon, 42, true, true, true)],
            },
            RevenueState {
                accounts: vec![acct("only-blocked", Stage::Lead, 99, false, false, false)],
            },
        ];
        for s in &states {
            let u = maximum_reachable_revenue(s).revenue_utilization;
            assert!((0.0..=1.0).contains(&u), "utilization {u} out of [0,1]");
        }
    }

    #[test]
    fn already_closed_account_is_fully_utilized() {
        let s = RevenueState {
            accounts: vec![acct("c", Stage::ClosedWon, 1_000, true, true, true)],
        };
        let r = maximum_reachable_revenue(&s);
        assert_eq!(r.revenue_utilization, 1.0);
        assert_eq!(r.revenue_opportunity_cents, 0);
    }

    #[test]
    fn blocked_account_attributes_the_missing_evidence() {
        let r = maximum_reachable_revenue(&fixture());
        let gap = r
            .accounts
            .iter()
            .find(|a| a.account_id == "acct-legal-gap")
            .expect("legal-gap present");
        assert_eq!(gap.max_realizable_cents, 0);
        assert!(!gap.closeable);
        assert_eq!(
            gap.blocked_on.as_deref(),
            Some(&["legal_approved".to_string()][..])
        );
    }

    #[test]
    fn empty_state_is_zero_everywhere_not_nan() {
        let r = maximum_reachable_revenue(&RevenueState { accounts: vec![] });
        assert_eq!(r.max_reachable_revenue_cents, 0);
        assert_eq!(r.actual_closed_cents, 0);
        assert_eq!(r.revenue_opportunity_cents, 0);
        assert_eq!(r.revenue_utilization, 0.0);
        assert_eq!(r.accounts_considered, 0);
    }
}
