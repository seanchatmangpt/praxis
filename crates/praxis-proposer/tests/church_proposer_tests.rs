//! Genesis Day 6 acceptance tests for the church-operations domain pack.
//!
//! These mirror `proposer_tests.rs` (revenue) one-for-one: determinism,
//! objective sensitivity, the lawfulness gate, boundedness, and rationale
//! completeness. The parallelism is the point — the church proposer is
//! `engine::Proposer<ChurchDomain>`, the *same* substrate, and it passes the
//! *same* acceptance suite. A final test proves the generic substrate,
//! specialized to `RevenueDomain`, reproduces the concrete revenue proposer's
//! ranking: one engine, two domains, agreeing.

use std::collections::BTreeMap;

use praxis_proposer::church::{self, ChurchProposer, ChurchState, Person, Stage};
use praxis_proposer::engine::{Domain, Proposer as GenericProposer, MAX_PROPOSALS};
use praxis_proposer::{ObjectiveFunction, RevenueDomain};

fn person(
    id: &str,
    stage: Stage,
    welcomed: bool,
    followed_up: bool,
    in_small_group: bool,
    care_assigned: bool,
    days: u32,
) -> Person {
    Person {
        id: id.to_string(),
        stage,
        welcomed,
        followed_up,
        in_small_group,
        care_assigned,
        days_in_stage: days,
    }
}

/// The 3-person fixture, the church parallel of revenue's 3-account fixture:
/// a fully-evidenced deep person one step from Leading; a first-timer who was
/// welcomed but never followed up (capped at Returning); and a stale, no-touch
/// first-timer.
fn fixture_state() -> ChurchState {
    ChurchState {
        people: vec![
            // Fully evidenced, connected, one step from Leading — the "close".
            person("visitor-1", Stage::Connected, true, true, true, true, 12),
            // Welcomed but never followed up: may NOT be proposed past Returning.
            person("visitor-2", Stage::FirstTime, true, false, true, true, 45),
            // Very stale first-timer, no touch at all.
            person("visitor-3", Stage::FirstTime, false, false, false, false, 120),
        ],
    }
}

fn default_objective() -> ObjectiveFunction {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("church_objective.json");
    church::objective_from_path(&path).expect("default church objective loads")
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
    obj.validate_fluents(&church::FLUENT_NAMES)
        .expect("test weights valid against church vocabulary");
    obj
}

// ---------------------------------------------------------------------------
// Determinism: same state + objective => byte-identical ranked list + hashes.
// ---------------------------------------------------------------------------

#[test]
fn scoring_is_deterministic_including_hashes() {
    let state = fixture_state();
    let objective = default_objective();

    let a = ChurchProposer::new(objective.clone()).propose(&state);
    let b = ChurchProposer::new(objective).propose(&state);

    assert!(!a.is_empty());
    assert_eq!(a, b, "ranked lists must be identical");
    for (pa, pb) in a.iter().zip(b.iter()) {
        assert_eq!(pa.proposal_hash, pb.proposal_hash);
        assert_eq!(pa.canonical_bytes(), pb.canonical_bytes(), "byte-identical");
        assert_eq!(
            pa.proposal_hash,
            blake3::hash(&pa.canonical_bytes()).to_hex().to_string()
        );
        assert_eq!(pa.proposal_hash.len(), 64);
    }
}

#[test]
fn default_objective_top_proposal_is_the_deep_connection() {
    // Under the authored ZOE weights, walking the fully-evidenced person all
    // the way to Leading is the highest-value lawful move — the church analog
    // of "close the fully-evidenced deal".
    let ranked = ChurchProposer::new(default_objective()).propose(&fixture_state());
    assert_eq!(ranked[0].target_id, "visitor-1");
    assert_eq!(ranked[0].target_stage, Stage::Leading);
    assert_eq!(ranked[0].pddl_goal(), "(stage visitor-1 leading)");
}

// ---------------------------------------------------------------------------
// Objective sensitivity: changing a weight reorders as expected.
// ---------------------------------------------------------------------------

#[test]
fn raising_prompt_followup_weight_promotes_a_fresh_first_timer() {
    let state = ChurchState {
        people: vec![
            // Fully evidenced deep person: default weights love its Leading move.
            person("connected-1", Stage::Connected, true, true, true, true, 5),
            // A brand-new first-timer (1 day): the only lawful move is Returning,
            // and it is the *only* candidate that fires first_time_followup.
            person("fresh", Stage::FirstTime, false, false, false, false, 1),
        ],
    };

    // People-connected dominant: the deep Leading move wins.
    let connection_heavy = weights(&[("people_connected", 1000.0)]);
    let ranked = GenericProposer::<church::ChurchDomain>::new(connection_heavy).propose(&state);
    assert_eq!(ranked[0].target_id, "connected-1");
    assert_eq!(ranked[0].target_stage, Stage::Leading);

    // Crank prompt-follow-up far above any connection depth: catching the
    // brand-new first-timer within 48h must now outrank the deep move.
    // Direction of reorder is the authored judgment "don't lose the newcomer".
    let followup_heavy = weights(&[
        ("people_connected", 1000.0),
        ("first_time_followup_within_48h", 1_000_000.0),
    ]);
    let ranked = ChurchProposer::new(followup_heavy).propose(&state);
    assert_eq!(ranked[0].target_id, "fresh");
    assert_eq!(ranked[0].target_stage, Stage::Returning);
}

#[test]
fn flipping_volunteer_capacity_sign_flips_relative_order() {
    // Two people who can each be moved to Serving. volunteer_capacity_used
    // fires (=1.0) for a move into Serving. With people_connected fixed, the
    // sign on volunteer_capacity_used decides whether the Serving move beats a
    // shallower Connected move.
    let state = ChurchState {
        people: vec![person("candidate", Stage::Returning, true, true, true, false, 3)],
    };

    // Treat serving capacity as pure benefit: Serving (depth 3 + capacity) beats
    // Connected (depth 2).
    let capacity_is_good = weights(&[
        ("people_connected", 1.0),
        ("volunteer_capacity_used", 100.0),
    ]);
    let ranked = ChurchProposer::new(capacity_is_good).propose(&state);
    assert_eq!(ranked[0].target_stage, Stage::Serving);

    // Treat serving capacity as scarce/costly enough to overwhelm the extra
    // connection depth: the shallower Connected move now ranks first.
    let capacity_is_scarce = weights(&[
        ("people_connected", 1.0),
        ("volunteer_capacity_used", -100.0),
    ]);
    let ranked = ChurchProposer::new(capacity_is_scarce).propose(&state);
    assert_eq!(ranked[0].target_stage, Stage::Connected);
}

// ---------------------------------------------------------------------------
// Lawfulness pre-filter: missing evidence never yields over-reach.
// This is the SAME mechanism as revenue's evidence gate — reuse, proven.
// ---------------------------------------------------------------------------

#[test]
fn person_missing_followup_never_proposed_past_returning() {
    let state = fixture_state();
    let ranked = ChurchProposer::new(default_objective()).propose(&state);
    for p in ranked.iter().filter(|p| p.target_id == "visitor-2") {
        assert!(
            p.target_stage <= Stage::Returning,
            "visitor-2 was never followed up; got over-reaching proposal to {:?}",
            p.target_stage
        );
    }
    // The pre-filter still leaves the lawful move available (invite back).
    assert!(ranked
        .iter()
        .any(|p| p.target_id == "visitor-2" && p.target_stage == Stage::Returning));
}

#[test]
fn leading_requires_all_four_evidence_flags() {
    // Missing only care_assigned: Serving OK, Leading never.
    let state = ChurchState {
        people: vec![person("p", Stage::Connected, true, true, true, false, 1)],
    };
    let ranked = ChurchProposer::new(default_objective()).propose(&state);
    assert!(ranked.iter().any(|p| p.target_stage == Stage::Serving));
    assert!(ranked.iter().all(|p| p.target_stage != Stage::Leading));
}

// ---------------------------------------------------------------------------
// Boundedness: enumeration is capped and documented.
// ---------------------------------------------------------------------------

#[test]
fn per_person_enumeration_bounded_by_forward_stages() {
    // Fully-evidenced FirstTime person: the structural maximum of 4 targets.
    let state = ChurchState {
        people: vec![person("p", Stage::FirstTime, true, true, true, true, 1)],
    };
    let ranked = ChurchProposer::new(default_objective()).propose(&state);
    assert_eq!(ranked.len(), 4, "at most Stage::ALL.len()-1 per person");
}

#[test]
fn global_output_truncates_deterministically_at_max_proposals() {
    // 40 fully-evidenced FirstTime people => 160 raw candidates > MAX_PROPOSALS.
    let people: Vec<Person> = (0..40)
        .map(|i| person(&format!("p-{i:03}"), Stage::FirstTime, true, true, true, true, i))
        .collect();
    let state = ChurchState { people };
    let objective = default_objective();
    let ranked = ChurchProposer::new(objective.clone()).propose(&state);
    assert_eq!(ranked.len(), MAX_PROPOSALS, "hard cap enforced");
    let again = ChurchProposer::new(objective).propose(&state);
    assert_eq!(ranked, again);
    for w in ranked.windows(2) {
        assert!(w[0].score >= w[1].score);
    }
}

// ---------------------------------------------------------------------------
// Rationale completeness: every score explained by cited weights.
// ---------------------------------------------------------------------------

#[test]
fn every_church_fluent_is_cited_and_contributions_sum_to_score() {
    let state = fixture_state();
    let objective = default_objective();
    let ranked = ChurchProposer::new(objective.clone()).propose(&state);
    assert!(!ranked.is_empty());

    for p in &ranked {
        let joined = p.rationale.join("\n");
        assert!(joined.contains(&objective.name));
        assert!(joined.contains(&objective.version));
        for name in church::FLUENT_NAMES {
            assert!(joined.contains(name), "rationale missing fluent {name}");
        }
        // Score reproducible from the cited weights and fluents.
        let person = state
            .people
            .iter()
            .find(|a| a.id == p.target_id)
            .unwrap();
        let fluents = church::compute_fluents(person, p.target_stage);
        let mut expected = 0.0f64;
        for (i, name) in church::FLUENT_NAMES.iter().enumerate() {
            expected += objective.weight(name) * fluents[i];
        }
        assert_eq!(expected.to_bits(), p.score.to_bits(), "score fully explained");
        assert!(joined.contains("total score ="));
    }
}

// ---------------------------------------------------------------------------
// PDDL goal emission format.
// ---------------------------------------------------------------------------

#[test]
fn pddl_goal_atom_format() {
    let state = ChurchState {
        people: vec![person("visitor-7", Stage::Returning, true, true, false, false, 1)],
    };
    let ranked = ChurchProposer::new(default_objective()).propose(&state);
    let connected = ranked
        .iter()
        .find(|p| p.target_stage == Stage::Connected)
        .unwrap();
    assert_eq!(connected.pddl_goal(), "(stage visitor-7 connected)");
}

// ---------------------------------------------------------------------------
// Objective loading rejects a revenue fluent for the church vocabulary — the
// deny_unknown_fields discipline is reused, only the vocabulary differs.
// ---------------------------------------------------------------------------

#[test]
fn revenue_fluent_rejected_by_church_objective_loader() {
    let s = r#"{"name":"x","version":"1","weights":{"realized_revenue":1.0}}"#;
    assert!(church::objective_from_json_str(s).is_err());
    // And a genuine church fluent loads fine.
    let ok = r#"{"name":"x","version":"1","weights":{"people_connected":1.0}}"#;
    assert!(church::objective_from_json_str(ok).is_ok());
}

// ---------------------------------------------------------------------------
// Substrate identity: the GENERIC engine specialized to RevenueDomain
// reproduces the CONCRETE revenue proposer's ranking. One substrate, two
// domains, agreeing — the doctrine, proven at the type level.
// ---------------------------------------------------------------------------

#[test]
fn generic_engine_reproduces_concrete_revenue_ranking() {
    use praxis_proposer::{domain::Account, Proposer as ConcreteRevenueProposer, RevenueState};

    let state = RevenueState {
        accounts: vec![
            Account {
                id: "acct-1".into(),
                stage: praxis_proposer::Stage::Procurement,
                amount_cents: 2_500_000,
                security_review_done: true,
                legal_approved: true,
                exec_sponsor: true,
                days_in_stage: 12,
            },
            Account {
                id: "acct-2".into(),
                stage: praxis_proposer::Stage::Qualified,
                amount_cents: 800_000,
                security_review_done: true,
                legal_approved: false,
                exec_sponsor: true,
                days_in_stage: 45,
            },
            Account {
                id: "acct-3".into(),
                stage: praxis_proposer::Stage::Lead,
                amount_cents: 150_000,
                security_review_done: false,
                legal_approved: false,
                exec_sponsor: false,
                days_in_stage: 120,
            },
        ],
    };
    let objective = {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("revenue_objective.json");
        ObjectiveFunction::from_path(&path).expect("revenue objective loads")
    };

    let concrete = ConcreteRevenueProposer::new(objective.clone()).propose(&state);
    let generic = GenericProposer::<RevenueDomain>::new(objective).propose(&state);

    assert_eq!(concrete.len(), generic.len(), "same candidate count");
    for (c, g) in concrete.iter().zip(generic.iter()) {
        assert_eq!(c.target_account, g.target_id, "same ranked entity order");
        assert_eq!(c.target_stage, g.target_stage, "same ranked stage order");
        assert_eq!(c.score.to_bits(), g.score.to_bits(), "identical scores");
        assert_eq!(c.pddl_goal(), g.pddl_goal(), "identical goal atoms");
    }
    // Sanity: the shared top proposal is acct-1 -> closed-won either way.
    assert_eq!(generic[0].target_id, "acct-1");
    assert_eq!(generic[0].pddl_goal(), "(stage acct-1 closed-won)");
}

// The `Domain` trait is object-usable metadata: confirm the church pack
// self-describes correctly (naming that flows into hashes + PDDL).
#[test]
fn church_domain_self_description() {
    assert_eq!(church::ChurchDomain::pack_name(), "church");
    assert_eq!(church::ChurchDomain::goal_predicate(), "stage");
    assert_eq!(church::ChurchDomain::fluent_names().len(), 4);
    assert_eq!(church::ChurchDomain::stage_pddl_name(Stage::Leading), "leading");
}
