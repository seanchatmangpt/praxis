//! Revenue-operations domain model as **data**, not code opinions.
//!
//! Everything in this module is a plain description of the world an operator
//! already recognizes: accounts, pipeline stages, dollar amounts (integer
//! cents for determinism), evidence flags, and staleness. No value judgments
//! live here — the [`crate::objective::ObjectiveFunction`] (domain-authored
//! data) supplies those.
//!
//! # Integration notes
//!
//! INTEGRATED (Genesis Day 1): the admitted-observation path exists as
//! [`RevenueState::from_admitted`], which deserializes the JSON payload of a
//! `LawObject<serde_json::Value, Admitted, _>` — the payload type
//! praxis-core's `DefaultLaw` actually judges and admits. The shape of the
//! types stayed as-is; direct construction remains for tests and
//! pre-lifecycle callers.

use serde::{Deserialize, Serialize};

/// Ordered pipeline stage for a revenue account.
///
/// The ordering is load-bearing: candidate goals may only move an account
/// *forward* (strictly greater stage), and evidence gates key off the target
/// stage. Discriminants are explicit so the order survives refactoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Lead = 0,
    Qualified = 1,
    Proposal = 2,
    Procurement = 3,
    ClosedWon = 4,
}

impl Stage {
    /// All stages in pipeline order.
    pub const ALL: [Stage; 5] = [
        Stage::Lead,
        Stage::Qualified,
        Stage::Proposal,
        Stage::Procurement,
        Stage::ClosedWon,
    ];

    /// Zero-based pipeline index (Lead = 0 .. ClosedWon = 4).
    pub fn index(self) -> u8 {
        self as u8
    }

    /// Lower-kebab name used in PDDL goal atoms and the Turtle ontology
    /// (`ontology/revenue.ttl`). Must stay byte-stable: emitted text is
    /// hashed into proposal hashes.
    pub fn pddl_name(self) -> &'static str {
        match self {
            Stage::Lead => "lead",
            Stage::Qualified => "qualified",
            Stage::Proposal => "proposal",
            Stage::Procurement => "procurement",
            Stage::ClosedWon => "closed-won",
        }
    }
}

/// A single revenue account: the unit the proposer reasons over.
///
/// `amount_cents` is an integer (i64 cents) rather than a float so that
/// state snapshots are exact and hashing/replay is deterministic. Fluent
/// arithmetic converts to `f64` only inside scoring, in a fixed order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Account {
    /// Stable identifier; appears verbatim in PDDL goal atoms
    /// (e.g. `acct-7` -> `(stage acct-7 procurement)`).
    pub id: String,
    /// Current pipeline stage.
    pub stage: Stage,
    /// Deal size in integer cents (numeric fluent source).
    pub amount_cents: i64,
    /// Evidence flag: security review completed.
    pub security_review_done: bool,
    /// Evidence flag: legal approved the paper.
    pub legal_approved: bool,
    /// Evidence flag: an executive sponsor is attached.
    pub exec_sponsor: bool,
    /// Days the account has sat in its current stage (numeric fluent source).
    pub days_in_stage: u32,
}

/// Snapshot of the whole revenue pipeline the proposer observes.
///
/// INTEGRATED (was: local stand-in): the type stays local — praxis-core has
/// no revenue vocabulary and should not grow one — but the admitted-only
/// observation path now exists as [`RevenueState::from_admitted`], which
/// deserializes the snapshot out of a
/// `LawObject<serde_json::Value, Admitted, _>`. Direct struct construction
/// remains available for tests and for callers that have not yet adopted
/// the law-object lifecycle; the `propose` verbs document which path they
/// use.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevenueState {
    pub accounts: Vec<Account>,
}

impl RevenueState {
    /// Observe an **admitted** revenue snapshot (the AR-9-preferred path).
    ///
    /// Takes a [`praxis_core::LawObject`] in the `Admitted` stage carrying a
    /// JSON payload (the payload type praxis-core's `DefaultLaw` judges and
    /// admits) and deserializes it into a `RevenueState`. Because the
    /// `Admitted` stage can only be reached through `Judge`/`Admit` — the
    /// stage traits are sealed and the stage-transition helper is
    /// crate-private in praxis-core, so no external code can forge the stage
    /// — a snapshot obtained through this constructor is guaranteed to have
    /// passed obligation judgment: the proposer observes admitted reality,
    /// never raw input.
    ///
    /// A payload that is admitted but not shaped like a `RevenueState`
    /// (unknown fields included — `deny_unknown_fields` applies) is an
    /// `Err`: admission and schema conformance are separate obligations.
    pub fn from_admitted<Law>(
        admitted: &praxis_core::LawObject<
            serde_json::Value,
            praxis_core::lifecycle::Admitted,
            Law,
        >,
    ) -> Result<RevenueState, serde_json::Error> {
        serde_json::from_value(admitted.payload().clone())
    }
}

/// Evidence gate: may `account` lawfully be *proposed* into `target`?
///
/// This mirrors (does not replace) the admission rules that will gate any
/// proposal downstream. The proposer pre-filters so it never emits a
/// proposal that admission would trivially refuse:
///
/// | target       | required evidence                                        |
/// |--------------|----------------------------------------------------------|
/// | Lead         | (never a forward target)                                 |
/// | Qualified    | none                                                     |
/// | Proposal     | none                                                     |
/// | Procurement  | `legal_approved` AND `security_review_done`              |
/// | ClosedWon    | `legal_approved` AND `security_review_done` AND `exec_sponsor` |
///
/// In particular: an account missing `legal_approved` can never be proposed
/// past `Proposal`.
pub fn evidence_permits(account: &Account, target: Stage) -> bool {
    match target {
        Stage::Lead | Stage::Qualified | Stage::Proposal => true,
        Stage::Procurement => account.legal_approved && account.security_review_done,
        Stage::ClosedWon => {
            account.legal_approved && account.security_review_done && account.exec_sponsor
        }
    }
}

/// Lawful forward targets for an account: every stage strictly ahead of the
/// current one whose evidence gate passes. Bounded by construction: at most
/// `Stage::ALL.len() - 1 == 4` targets per account.
pub fn lawful_targets(account: &Account) -> Vec<Stage> {
    Stage::ALL
        .iter()
        .copied()
        .filter(|t| t.index() > account.stage.index())
        .filter(|t| evidence_permits(account, *t))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn acct(stage: Stage, legal: bool, sec: bool, exec: bool) -> Account {
        Account {
            id: "a".into(),
            stage,
            amount_cents: 100,
            security_review_done: sec,
            legal_approved: legal,
            exec_sponsor: exec,
            days_in_stage: 0,
        }
    }

    #[test]
    fn stage_order_is_pipeline_order() {
        assert!(Stage::Lead < Stage::Qualified);
        assert!(Stage::Proposal < Stage::Procurement);
        assert!(Stage::Procurement < Stage::ClosedWon);
    }

    #[test]
    fn missing_legal_never_passes_proposal() {
        let a = acct(Stage::Lead, false, true, true);
        let targets = lawful_targets(&a);
        assert!(targets.iter().all(|t| *t <= Stage::Proposal));
    }

    #[test]
    fn full_evidence_unlocks_closed_won() {
        let a = acct(Stage::Procurement, true, true, true);
        assert_eq!(lawful_targets(&a), vec![Stage::ClosedWon]);
    }

    #[test]
    fn closed_won_has_no_forward_targets() {
        let a = acct(Stage::ClosedWon, true, true, true);
        assert!(lawful_targets(&a).is_empty());
    }

    #[test]
    fn from_admitted_observes_a_judged_and_admitted_snapshot() {
        use praxis_core::{Admit, DefaultLaw, Judge, LawObject};

        let payload = serde_json::json!({
            "accounts": [{
                "id": "acct-1",
                "stage": "lead",
                "amount_cents": 100,
                "security_review_done": true,
                "legal_approved": true,
                "exec_sponsor": false,
                "days_in_stage": 3
            }]
        });
        let raw = LawObject::<serde_json::Value, praxis_core::lifecycle::Raw, DefaultLaw>::new(
            payload,
            vec![],
        );
        let Ok(validated) = DefaultLaw::judge(raw) else {
            panic!("no obligations => validated");
        };
        let admitted = DefaultLaw::admit(validated).expect("green andon => admitted");

        let state = RevenueState::from_admitted(&admitted)
            .expect("admitted payload deserializes into RevenueState");
        assert_eq!(state.accounts.len(), 1);
        assert_eq!(state.accounts[0].id, "acct-1");
        assert_eq!(state.accounts[0].stage, Stage::Lead);
    }

    #[test]
    fn from_admitted_rejects_non_revenue_payload() {
        use praxis_core::{Admit, DefaultLaw, Judge, LawObject};

        let payload = serde_json::json!({"accounts": [], "vibes": true});
        let raw = LawObject::<serde_json::Value, praxis_core::lifecycle::Raw, DefaultLaw>::new(
            payload,
            vec![],
        );
        let Ok(validated) = DefaultLaw::judge(raw) else {
            panic!("no obligations => validated");
        };
        let admitted = DefaultLaw::admit(validated).expect("green andon => admitted");

        // deny_unknown_fields: admission does not imply schema conformance.
        assert!(RevenueState::from_admitted(&admitted).is_err());
    }
}
