//! Genesis Day 6 phase 2 — **two institutions, one substrate.**
//!
//! This is the domain-independence proof, structured so a reader sees it
//! literally: below, two packs — revenue and church — are each turned into a
//! [`PackRun`] by calling the *identical* substrate functions
//! (`mission::run_pipeline::<P>`, `mission::ceiling::<P>`,
//! `mission::evidence_gate_agrees::<P>`, `mission::admit_advance::<P>`), the
//! **only** difference being the type parameter `P`, the authored objective,
//! and the observed state. Then **one loop** asserts the same invariants over
//! both:
//!
//! - the pipe runs green end to end (propose → goal → `plan solve` →
//!   `law judge`/`law admit` → `law receipt`) and is deterministic;
//! - the receipt chain is valid and binds back to the admitted proposal's
//!   `proposal_hash` (AR-9 closure);
//! - the evidence gate is enforced by the **same admission mechanism**
//!   (`ops::judge_payload`/`ops::admit_payload`): every plan action is
//!   admitted, the proposer pre-filter and the admission gate agree for every
//!   entity × stage, and a forced over-reach an entity lacks evidence for is
//!   *denied*;
//! - the Maximum Reachable objective (`mission::ceiling`) respects the gates in
//!   both packs — and for revenue reproduces the bespoke MRR numbers exactly.
//!
//! The substrate functions named in the loop are the same for both packs.
//! Only the ontology (`Pack` impl), the objective, and the state differ. That
//! is the whole claim.
#![cfg(feature = "proposer")]

use my_conforming_project::mission::{self, Pack};
use my_conforming_project::revenue;
use praxis_proposer::engine::Domain;
use praxis_proposer::{ChurchDomain, ChurchState, RevenueDomain, RevenueState};
use serde_json::{json, Value};

/// Fixed timestamp so every receipt chain hash in this file is reproducible.
const TS_NS: u64 = 1_751_328_000_000_000_000;

/// Under `--features law-signed`, the receipt step fails closed without a
/// signing key. Signing signs *over* the chain hash, so determinism is
/// unaffected — this just keeps the pipe green under `--all-features`.
fn ensure_signing_key() {
    #[cfg(feature = "law-signed")]
    {
        use std::sync::Once;
        static ONCE: Once = Once::new();
        ONCE.call_once(|| {
            std::env::set_var(
                "PRAXIS_SIGNING_KEY",
                "8bb5514c228cf4275a64aba09f3da77ef7de8b74a4424d670e71c26b0557e293",
            );
        });
    }
}

/// The type-erased result of driving one institution through the substrate.
/// Everything here was produced by a generic `mission::*::<P>` call; the
/// `Value`s let the single assertion loop treat both packs uniformly.
struct PackRun {
    pack: &'static str,
    /// `run_pipeline::<P>` transcript (steps 1–5 + closing chain hash).
    transcript: Value,
    /// A second `run_pipeline::<P>` transcript with identical inputs — for the
    /// determinism assertion.
    transcript_again: Value,
    /// `ceiling::<P>` — the pack's Maximum Reachable objective.
    ceiling: Value,
    /// Does the proposer pre-filter agree with the admission gate for *every*
    /// entity × stage? (`evidence_gate_agrees::<P>` swept.)
    gates_all_agree: bool,
    /// A forced over-reach the pipeline would never propose, pushed through the
    /// SAME `ops` admission gate — must be denied.
    denied_over_reach: Value,
}

/// Sweep the seam invariant across the whole state: proposer pre-filter
/// ([`Pack::evidence_permits`]) == admission gate for every entity × stage.
fn all_gates_agree<P: Pack>(state: &P::State) -> bool {
    P::entities(state).iter().all(|e| {
        P::all_stages()
            .iter()
            .all(|&t| mission::evidence_gate_agrees::<P>(e, t))
    })
}

/// Build the revenue [`PackRun`] — the substrate specialized to revenue.
fn revenue_run() -> PackRun {
    let state: RevenueState =
        serde_json::from_value(mission::revenue_fixture_state()).expect("revenue fixture");
    let objective = RevenueDomain::load_objective(revenue::REVENUE_OBJECTIVE).expect("rev objective");

    // Forced over-reach: acct-legal-gap lacks legal_approved, so it can never
    // lawfully reach closed-won — admission must deny it.
    let blocked = state
        .accounts
        .iter()
        .find(|a| a.id == "acct-legal-gap")
        .expect("legal-gap present");
    let denied_over_reach =
        mission::admit_advance::<RevenueDomain>(blocked, praxis_proposer::Stage::ClosedWon)
            .expect("forced admit returns Ok(domain-no)");

    PackRun {
        pack: "revenue",
        transcript: mission::run_pipeline::<RevenueDomain>(&state, &objective, "close-q3", TS_NS)
            .expect("revenue pipe green"),
        transcript_again: mission::run_pipeline::<RevenueDomain>(
            &state, &objective, "close-q3", TS_NS,
        )
        .expect("revenue pipe green (2nd)"),
        ceiling: mission::ceiling::<RevenueDomain>(&state),
        gates_all_agree: all_gates_agree::<RevenueDomain>(&state),
        denied_over_reach,
    }
}

/// Build the church [`PackRun`] — the SAME substrate, specialized to church.
/// Every `mission::*::<ChurchDomain>` call below is the revenue call with a
/// different type parameter.
fn church_run() -> PackRun {
    let state: ChurchState =
        serde_json::from_value(mission::church_fixture_state()).expect("church fixture");
    let objective = ChurchDomain::load_objective(mission::CHURCH_OBJECTIVE).expect("church objective");

    // Forced over-reach: visitor-fresh has no hospitality evidence, so it can
    // never lawfully reach leading — the SAME admission gate must deny it.
    let blocked = state
        .people
        .iter()
        .find(|p| p.id == "visitor-fresh")
        .expect("fresh present");
    let denied_over_reach =
        mission::admit_advance::<ChurchDomain>(blocked, praxis_proposer::church::Stage::Leading)
            .expect("forced admit returns Ok(domain-no)");

    PackRun {
        pack: "church",
        transcript: mission::run_pipeline::<ChurchDomain>(
            &state,
            &objective,
            "connect-newcomers",
            TS_NS,
        )
        .expect("church pipe green"),
        transcript_again: mission::run_pipeline::<ChurchDomain>(
            &state,
            &objective,
            "connect-newcomers",
            TS_NS,
        )
        .expect("church pipe green (2nd)"),
        ceiling: mission::ceiling::<ChurchDomain>(&state),
        gates_all_agree: all_gates_agree::<ChurchDomain>(&state),
        denied_over_reach,
    }
}

/// THE domain-independence proof: one loop, two packs. Both institutions were
/// driven through the identical substrate above; here the same invariants are
/// asserted over each. Nothing in this loop names an institution.
#[test]
fn two_institutions_one_substrate() {
    ensure_signing_key();

    let packs = [revenue_run(), church_run()];

    for run in &packs {
        let t = &run.transcript;

        // Self-identifies as the pack it was instantiated at.
        assert_eq!(t["pack"], json!(run.pack), "transcript pack tag");

        // Determinism: same P + objective + state + ts_ns ⇒ byte-identical.
        assert_eq!(t, &run.transcript_again, "[{}] pipe must be deterministic", run.pack);

        // Step 1: several ranked proposals, each with a 64-hex proposal_hash
        // and a cited rationale.
        let proposals = t["step_1_proposals"]["proposals"]
            .as_array()
            .expect("proposals array");
        assert!(proposals.len() >= 3, "[{}] expected ≥3 proposals", run.pack);
        let top = &proposals[0];
        assert_eq!(
            top["proposal_hash"].as_str().expect("hash").len(),
            64,
            "[{}] top proposal_hash is 64 hex",
            run.pack
        );
        assert!(
            top["rationale"].as_array().expect("rationale").len() >= 3,
            "[{}] rationale cites the objective + fluents",
            run.pack
        );

        // Step 2/3: the top proposal drives a multi-action, evidence-gated
        // plan (both fixtures' best move needs two gated steps).
        let plan = t["step_3_plan"]["plan"].as_array().expect("plan array");
        assert!(plan.len() >= 2, "[{}] top goal needs a multi-step plan: {plan:?}", run.pack);

        // Step 4: every plan action was validated AND admitted — via the SAME
        // ops::judge_payload/ops::admit_payload gate for both packs.
        let admissions = t["step_4_admissions"].as_array().expect("admissions array");
        assert!(!admissions.is_empty(), "[{}] plan produced admissions", run.pack);
        for adm in admissions {
            assert_eq!(adm["judge_verdict"], json!("validated"), "[{}] action judged", run.pack);
            assert_eq!(adm["admit_status"], json!("admitted"), "[{}] action admitted", run.pack);
        }

        // Step 5: the receipt binds the top proposal_hash and yields a valid
        // 64-hex chain hash (AR-9 closure), identically in both institutions.
        let bound = t["step_5_receipt"]["binds_proposal_hash"].as_str().expect("bound hash");
        assert_eq!(
            bound,
            top["proposal_hash"].as_str().expect("top hash"),
            "[{}] receipt binds the admitted top proposal",
            run.pack
        );
        assert_eq!(
            t["chain_hash"].as_str().expect("chain hash").len(),
            64,
            "[{}] valid chain hash",
            run.pack
        );

        // The seam invariant holds everywhere: proposer pre-filter == admission
        // gate for every entity × stage.
        assert!(
            run.gates_all_agree,
            "[{}] proposer pre-filter and admission gate must agree everywhere",
            run.pack
        );

        // Negative path, SAME admission mechanism: a forced over-reach the
        // entity lacks evidence for is denied (not admitted).
        assert_ne!(
            run.denied_over_reach["admit_status"],
            json!("admitted"),
            "[{}] over-reach missing evidence must be denied by the same admission gate",
            run.pack
        );

        // The Maximum Reachable objective respects the gates: opportunity is
        // never negative and utilization is a proper fraction in [0,1].
        let c = &run.ceiling;
        assert_eq!(c["status"], json!("computed"), "[{}] ceiling computed", run.pack);
        assert!(
            c["opportunity_value"].as_f64().expect("opportunity") >= 0.0,
            "[{}] reachable ≥ realized",
            run.pack
        );
        let util = c["utilization"].as_f64().expect("utilization");
        assert!((0.0..=1.0).contains(&util), "[{}] utilization {util} in [0,1]", run.pack);
    }

    // Both transcripts expose the IDENTICAL step vocabulary — the substrate
    // stages are the same functions, not parallel reimplementations.
    let keys = |v: &Value| {
        let mut ks: Vec<String> = v.as_object().unwrap().keys().cloned().collect();
        ks.sort();
        ks
    };
    assert_eq!(
        keys(&packs[0].transcript),
        keys(&packs[1].transcript),
        "both packs produce the same transcript shape from the same pipeline"
    );

    // Distinct institutions, distinct missions ⇒ distinct receipts.
    assert_ne!(
        packs[0].transcript["chain_hash"], packs[1].transcript["chain_hash"],
        "revenue and church receipts are distinct chains"
    );
}

/// The strongest single equivalence: the generic Maximum Reachable objective,
/// specialized to revenue, reproduces the bespoke `maximum_reachable_revenue`
/// headline numbers exactly. One ceiling algebra, proven against the concrete
/// one it generalizes.
#[test]
fn revenue_ceiling_equals_bespoke_mrr() {
    let state: RevenueState =
        serde_json::from_value(mission::revenue_fixture_state()).expect("revenue fixture");
    let mrr = praxis_proposer::maximum_reachable_revenue(&state);
    let c = mission::ceiling::<RevenueDomain>(&state);

    assert_eq!(
        c["max_reachable_value"].as_f64().unwrap() as i64,
        mrr.max_reachable_revenue_cents,
        "generic ceiling reproduces MRR"
    );
    assert_eq!(
        c["already_realized_value"].as_f64().unwrap() as i64,
        mrr.actual_closed_cents
    );
    assert_eq!(
        c["opportunity_value"].as_f64().unwrap() as i64,
        mrr.revenue_opportunity_cents
    );
    assert!((c["utilization"].as_f64().unwrap() - mrr.revenue_utilization).abs() < 1e-12);
}

/// The church Maximum Reachable objective respects the evidence gates: stripping
/// the hospitality evidence that unlocks deeper stages must lower the ceiling
/// of people the welcome team can lawfully connect. The church parallel of
/// "removing legal_approved lowers MRR".
#[test]
fn church_ceiling_respects_evidence_gates() {
    let state: ChurchState =
        serde_json::from_value(mission::church_fixture_state()).expect("church fixture");
    let before = mission::ceiling::<ChurchDomain>(&state)["max_reachable_value"]
        .as_f64()
        .unwrap();

    // Strip visitor-apex down to an untouched first-timer: it can no longer be
    // walked toward leading, so the reachable connection ceiling must drop.
    let mut stripped = state.clone();
    let apex = stripped
        .people
        .iter_mut()
        .find(|p| p.id == "visitor-apex")
        .expect("apex present");
    apex.stage = praxis_proposer::church::Stage::FirstTime;
    apex.followed_up = false;
    apex.in_small_group = false;
    apex.care_assigned = false;

    let after = mission::ceiling::<ChurchDomain>(&stripped)["max_reachable_value"]
        .as_f64()
        .unwrap();
    assert!(
        after < before,
        "removing hospitality evidence must lower the church ceiling ({after} !< {before})"
    );
}
