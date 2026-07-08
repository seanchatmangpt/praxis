//! PR-14 acceptance tests: determinism, sensitivity, lawfulness,
//! boundedness, rationale completeness.

use std::collections::BTreeMap;

use praxis_proposer::domain::Account;
use praxis_proposer::{
    compute_fluents, ObjectiveFunction, Proposer, RevenueState, Stage, FLUENT_NAMES, MAX_PROPOSALS,
};

fn account(
    id: &str,
    stage: Stage,
    amount_cents: i64,
    legal: bool,
    sec: bool,
    exec: bool,
    days: u32,
) -> Account {
    Account {
        id: id.to_string(),
        stage,
        amount_cents,
        security_review_done: sec,
        legal_approved: legal,
        exec_sponsor: exec,
        days_in_stage: days,
    }
}

/// The 3-account fixture used across tests and the `rank_fixture` example.
fn fixture_state() -> RevenueState {
    RevenueState {
        accounts: vec![
            // Big deal ($25,000.00), fully evidenced, one step from close.
            account(
                "acct-1",
                Stage::Procurement,
                2_500_000,
                true,
                true,
                true,
                12,
            ),
            // Mid deal ($8,000.00), missing legal approval: may not be proposed past Proposal.
            account("acct-2", Stage::Qualified, 800_000, false, true, true, 45),
            // Small ($1,500.00), very stale early-stage deal, no evidence at all.
            account("acct-3", Stage::Lead, 150_000, false, false, false, 120),
        ],
    }
}

fn default_objective() -> ObjectiveFunction {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("revenue_objective.json");
    ObjectiveFunction::from_path(&path).expect("default objective loads")
}

fn weights(pairs: &[(&str, f64)]) -> ObjectiveFunction {
    let mut w = BTreeMap::new();
    for (k, v) in pairs {
        w.insert(k.to_string(), *v);
    }
    let obj = ObjectiveFunction {
        name: "test".into(),
        version: "1".into(),
        weights: w,
    };
    obj.validate().expect("test weights valid");
    obj
}

// ---------------------------------------------------------------------------
// Scoring determinism: same state + objective => byte-identical ranked list.
// ---------------------------------------------------------------------------

#[test]
fn scoring_is_deterministic_including_hashes() {
    let state = fixture_state();
    let objective = default_objective();

    let a = Proposer::new(objective.clone()).propose(&state);
    let b = Proposer::new(objective).propose(&state);

    assert!(!a.is_empty());
    assert_eq!(a, b, "ranked lists must be identical");
    for (pa, pb) in a.iter().zip(b.iter()) {
        assert_eq!(pa.proposal_hash, pb.proposal_hash);
        assert_eq!(pa.canonical_bytes(), pb.canonical_bytes(), "byte-identical");
        // Hash is recomputable from canonical bytes.
        assert_eq!(
            pa.proposal_hash,
            blake3::hash(&pa.canonical_bytes()).to_hex().to_string()
        );
    }
}

// ---------------------------------------------------------------------------
// Objective sensitivity: changing a weight reorders as expected.
// ---------------------------------------------------------------------------

#[test]
fn raising_time_penalty_weight_promotes_stale_account() {
    let state = fixture_state();

    // Revenue-only objective: acct-1's close (25,000_00 realized) dominates.
    let revenue_only = weights(&[("realized_revenue", 1.0)]);
    let ranked = Proposer::new(revenue_only).propose(&state);
    assert_eq!(ranked[0].target_account, "acct-1");
    assert_eq!(ranked[0].target_stage, Stage::ClosedWon);

    // Crank staleness weight far above any deal size: the 120-day-stale
    // acct-3 must now outrank acct-1's close. Direction of reorder is the
    // authored judgment "staleness matters most", faithfully applied.
    let staleness_heavy = weights(&[("realized_revenue", 1.0), ("time_penalty", 1_000_000.0)]);
    let ranked = Proposer::new(staleness_heavy).propose(&state);
    assert_eq!(ranked[0].target_account, "acct-3");
}

#[test]
fn flipping_weight_sign_flips_relative_order() {
    let state = RevenueState {
        accounts: vec![
            account("old", Stage::Lead, 1000, false, false, false, 100),
            account("new", Stage::Lead, 1000, false, false, false, 1),
        ],
    };
    let prefer_stale = weights(&[("time_penalty", 1.0)]);
    let ranked = Proposer::new(prefer_stale).propose(&state);
    assert_eq!(ranked[0].target_account, "old");

    let prefer_fresh = weights(&[("time_penalty", -1.0)]);
    let ranked = Proposer::new(prefer_fresh).propose(&state);
    assert_eq!(ranked[0].target_account, "new");
}

// ---------------------------------------------------------------------------
// Lawfulness pre-filter: missing evidence never yields over-reach.
// ---------------------------------------------------------------------------

#[test]
fn account_missing_legal_never_proposed_past_proposal() {
    let state = fixture_state();
    let ranked = Proposer::new(default_objective()).propose(&state);
    for p in ranked.iter().filter(|p| p.target_account == "acct-2") {
        assert!(
            p.target_stage <= Stage::Proposal,
            "acct-2 lacks legal_approved; got over-reaching proposal to {:?}",
            p.target_stage
        );
    }
    // And the pre-filter still leaves the lawful moves available.
    assert!(ranked.iter().any(|p| p.target_account == "acct-2"));
}

#[test]
fn closed_won_requires_all_three_evidence_flags() {
    // Missing only exec_sponsor: Procurement OK, ClosedWon never.
    let state = RevenueState {
        accounts: vec![account("a", Stage::Proposal, 1000, true, true, false, 1)],
    };
    let ranked = Proposer::new(default_objective()).propose(&state);
    assert!(ranked.iter().any(|p| p.target_stage == Stage::Procurement));
    assert!(ranked.iter().all(|p| p.target_stage != Stage::ClosedWon));
}

// ---------------------------------------------------------------------------
// Boundedness: enumeration is capped and documented.
// ---------------------------------------------------------------------------

#[test]
fn per_account_enumeration_bounded_by_forward_stages() {
    // Fully-evidenced Lead account: the structural maximum of 4 targets.
    let state = RevenueState {
        accounts: vec![account("a", Stage::Lead, 1000, true, true, true, 1)],
    };
    let ranked = Proposer::new(default_objective()).propose(&state);
    assert_eq!(ranked.len(), 4, "at most Stage::ALL.len()-1 per account");
}

#[test]
fn global_output_truncates_deterministically_at_max_proposals() {
    // 40 fully-evidenced Lead accounts => 160 raw candidates > MAX_PROPOSALS.
    let accounts: Vec<Account> = (0..40)
        .map(|i| {
            account(
                &format!("acct-{i:03}"),
                Stage::Lead,
                1000 + i,
                true,
                true,
                true,
                i as u32,
            )
        })
        .collect();
    let state = RevenueState { accounts };
    let objective = default_objective();
    let ranked = Proposer::new(objective.clone()).propose(&state);
    assert_eq!(ranked.len(), MAX_PROPOSALS, "hard cap enforced");
    // Truncation is deterministic: a second run keeps the identical survivors.
    let again = Proposer::new(objective).propose(&state);
    assert_eq!(ranked, again);
    // Truncation drops only the lowest-ranked: list is sorted descending.
    for w in ranked.windows(2) {
        assert!(w[0].score >= w[1].score);
    }
}

// ---------------------------------------------------------------------------
// Rationale completeness: every score explained by cited weights.
// ---------------------------------------------------------------------------

#[test]
fn every_nonzero_weight_is_cited_and_contributions_sum_to_score() {
    let state = fixture_state();
    let objective = default_objective();
    let ranked = Proposer::new(objective.clone()).propose(&state);
    assert!(!ranked.is_empty());

    for p in &ranked {
        let joined = p.rationale.join("\n");
        // The objective identity is cited.
        assert!(joined.contains(&objective.name));
        assert!(joined.contains(&objective.version));
        // Every fluent is cited by name (nonzero weights with value x weight
        // = contribution; zero weights explicitly marked ignored).
        for name in FLUENT_NAMES {
            assert!(joined.contains(name), "rationale missing fluent {name}");
        }
        // The score is reproducible from the cited weights and fluents.
        let acct = state
            .accounts
            .iter()
            .find(|a| a.id == p.target_account)
            .unwrap();
        let fluents = compute_fluents(acct, p.target_stage);
        let mut expected = 0.0f64;
        for (i, name) in FLUENT_NAMES.iter().enumerate() {
            expected += objective.weight(name) * fluents[i];
        }
        assert_eq!(
            expected.to_bits(),
            p.score.to_bits(),
            "score fully explained"
        );
        // And the total is stated in the rationale.
        assert!(joined.contains("total score ="));
    }
}

// ---------------------------------------------------------------------------
// PDDL goal emission format.
// ---------------------------------------------------------------------------

#[test]
fn pddl_goal_atom_format() {
    let state = RevenueState {
        accounts: vec![account(
            "acct-7",
            Stage::Proposal,
            1000,
            true,
            true,
            false,
            1,
        )],
    };
    let ranked = Proposer::new(default_objective()).propose(&state);
    let procurement = ranked
        .iter()
        .find(|p| p.target_stage == Stage::Procurement)
        .unwrap();
    assert_eq!(procurement.pddl_goal(), "(stage acct-7 procurement)");
}
