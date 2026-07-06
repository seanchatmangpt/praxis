//! `propose` verb dispatcher — revenue, goal (PR-14, feature `proposer`).
//!
//! Thin CLI wrappers over the `praxis-proposer` crate: enumerate lawful
//! candidate goal states for a revenue pipeline, score them under a
//! **domain-authored** objective function, and return them ranked.
//!
//! # AR-9 — output is proposal (O), not authority (O*)
//!
//! Everything these verbs emit is an untrusted **observation**: a ranked
//! candidate with a rationale and a blake3 `proposal_hash`, sitting *outside*
//! the admission boundary. A proposal grants nothing and permits nothing —
//! it must pass Rice quarantine and `law judge`/`law admit` like any other
//! raw input before it can have any effect, and the eventual admission
//! receipt binds back to `proposal_hash` so "which proposal was admitted"
//! stays provable. The proposer may be heuristic or model-backed precisely
//! *because* it has no authority.
//!
//! # No value discovery (Non-goal 1)
//!
//! The objective function is authored data: it arrives as a file path (the
//! `--objective` argument or the payload's `objective_file`) or inline as
//! the payload's `objective` object. Supplying none of these is a hard
//! `Err` — the system never invents weights.
//!
//! Malformed input (bad JSON, an unreadable objective path, an objective
//! with unknown fluents or non-finite weights, zero or multiple objective
//! sources) is a hard `Err`. A state with *no lawful candidates* (every
//! account terminal or evidence-blocked) is a domain "no": `Ok(json)` with
//! `"status": "no_lawful_candidates"`, matching the CLI's "domain denial is
//! `Ok`" convention.

use clap_noun_verb::error::{NounVerbError, Result};
use clap_noun_verb_macros::verb;
// The `revenue`/`goal` ranking implementation lives in the library `ops`
// module so the CLI verbs and the MCP `propose_revenue`/`propose_goal` tools
// call one implementation (AR-2, no drift). `objective_summary`/`parse_payload`
// are the shared seams the church/mrr/mission verbs below also reuse.
use my_conforming_project::ops::{self, objective_summary, parse_payload};
use my_conforming_project::revtac::{compile_mission, Mission};
use praxis_proposer::{
    church, maximum_reachable_revenue, ChurchProposal, ChurchProposer, ChurchState,
    ObjectiveFunction, RevenueState,
};
use serde::Deserialize;
use serde_json::{json, Value};

// ── `propose revenue` / `propose goal` (thin call-throughs to `ops`) ───────

/// Rank candidate goal states for a revenue snapshot — thin call-through to
/// [`my_conforming_project::ops::propose_revenue_payload`], the single
/// implementation the MCP `propose_revenue` tool also calls.
fn revenue_payload(payload: &str, objective_path: &str) -> std::result::Result<Value, String> {
    ops::propose_revenue_payload(payload, objective_path)
}

/// Emit the top-ranked proposal's PDDL goal atom — thin call-through to
/// [`my_conforming_project::ops::propose_goal_payload`], the single
/// implementation the MCP `propose_goal` tool also calls.
fn goal_payload(payload: &str, objective_path: &str) -> std::result::Result<Value, String> {
    ops::propose_goal_payload(payload, objective_path)
}

// ── Verb wrappers ─────────────────────────────────────────────────────────

/// Rank candidate goal states for a revenue pipeline under a domain-authored
/// objective. Output is proposal (O), not authority (O*): every candidate
/// must still pass quarantine and admission before any effect (AR-9).
#[verb]
pub fn revenue(
    payload: String,
    #[arg(help = "Path to the domain-authored objective JSON file")] objective: Option<String>,
) -> Result<Value> {
    revenue_payload(&payload, objective.as_deref().unwrap_or(""))
        .map_err(NounVerbError::argument_error)
}

/// Emit the top-ranked proposal's PDDL goal atom for `plan solve`. Output is
/// proposal (O), not authority (O*): the goal asserts nothing about what may
/// happen — it is a scored, hash-committed observation (AR-9).
#[verb]
pub fn goal(
    payload: String,
    #[arg(help = "Path to the domain-authored objective JSON file")] objective: Option<String>,
) -> Result<Value> {
    goal_payload(&payload, objective.as_deref().unwrap_or(""))
        .map_err(NounVerbError::argument_error)
}

// ── `propose mrr` — Maximum Reachable Revenue ─────────────────────────────

/// Input for `propose mrr`: just the observed snapshot (MRR is
/// objective-independent — see `praxis_proposer::mrr`).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MrrInput {
    state: RevenueState,
}

/// Compute the Maximum Reachable Revenue, Revenue Utilization, and Revenue
/// Opportunity for an observed snapshot, with per-account attribution.
fn mrr_payload(payload: &str) -> std::result::Result<Value, String> {
    let input: MrrInput = parse_payload(payload)?;
    let report = maximum_reachable_revenue(&input.state);
    let mut v = serde_json::to_value(&report).map_err(|e| e.to_string())?;
    if let Value::Object(map) = &mut v {
        map.insert("status".to_string(), json!("computed"));
    }
    Ok(v)
}

// ── `propose mission` — RevTAC v0 mission compilation ─────────────────────

/// Input for `propose mission`: the observed snapshot plus a mission, either
/// inline (`mission` object) or as a file path (`mission_file`, `.toml`
/// parsed as TOML, otherwise JSON). Exactly one mission source is required.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MissionInput {
    state: RevenueState,
    #[serde(default)]
    mission: Option<Value>,
    #[serde(default)]
    mission_file: Option<String>,
}

/// Compile a RevTAC mission down to the proposer invocation + planner goal
/// (and the scoped MRR ceiling). Exactly one of `mission` / `mission_file`
/// must be supplied.
fn mission_payload(payload: &str) -> std::result::Result<Value, String> {
    let input: MissionInput = parse_payload(payload)?;
    let mission = match (&input.mission, &input.mission_file) {
        (Some(_), Some(_)) => {
            return Err(
                "supply exactly one of `mission` (inline) or `mission_file` (path), \
                        not both"
                    .to_string(),
            )
        }
        (None, None) => {
            return Err(
                "no mission supplied: put a `mission` object or a `mission_file` \
                        path in the payload"
                    .to_string(),
            )
        }
        (Some(inline), None) => Mission::parse(&inline.to_string(), "json")?,
        (None, Some(path)) => {
            let text = std::fs::read_to_string(path)
                .map_err(|e| format!("cannot read mission_file '{path}': {e}"))?;
            let fmt = if path.ends_with(".toml") {
                "toml"
            } else {
                "auto"
            };
            Mission::parse(&text, fmt)?
        }
    };
    compile_mission(&mission, &input.state)
}

/// Report the Maximum Reachable Revenue ceiling (MRR), Revenue Utilization,
/// and Revenue Opportunity for a snapshot. Output is observation (O): a
/// physical bound on lawful revenue, not an instruction to close anything.
#[verb]
pub fn mrr(payload: String) -> Result<Value> {
    mrr_payload(&payload).map_err(NounVerbError::argument_error)
}

/// Compile a RevTAC v0 mission (JSON or TOML) into a proposer invocation and
/// a planner goal atom. Output is proposal (O), not authority (O*): the
/// compiled goal must still pass admission (AR-9).
#[verb]
pub fn mission(payload: String) -> Result<Value> {
    mission_payload(&payload).map_err(NounVerbError::argument_error)
}

// ── `propose church` — church-operations domain pack (Genesis Day 6) ───────
//
// The domain-pack proof: the SAME substrate (enumerate → score → rank → hash),
// selected by verb (the verb name *is* the `--pack` selector), run over a
// church-operations ontology whose mission variables are NOT revenue —
// attendance/connection/care/service. Structurally identical to
// `propose revenue`; only the ontology (`ChurchState`) and the authored
// objective (church fluent vocabulary) differ.

/// Wire schema for `propose church`. Mirrors [`ProposeInput`] exactly, over a
/// church snapshot instead of a revenue one.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChurchProposeInput {
    /// The observed church-operations snapshot (people the welcome team is
    /// walking with). Prefer feeding an *admitted* law object's payload
    /// (`ChurchState::from_admitted`); the verb accepts any well-formed
    /// snapshot because the proposal it emits is untrusted either way (AR-9).
    state: ChurchState,
    #[serde(default)]
    objective: Option<Value>,
    #[serde(default)]
    objective_file: Option<String>,
}

/// Resolve the authored **church** objective from exactly one source, checked
/// against the church fluent vocabulary. Same one-source rule and Non-goal-1
/// discipline as [`resolve_objective`]; only the allowed fluent set differs.
fn resolve_church_objective(
    arg_path: &str,
    inline: &Option<Value>,
    objective_file: &Option<String>,
) -> std::result::Result<ObjectiveFunction, String> {
    let mut sources = 0;
    if !arg_path.is_empty() {
        sources += 1;
    }
    if inline.is_some() {
        sources += 1;
    }
    if objective_file.is_some() {
        sources += 1;
    }
    match sources {
        0 => {
            return Err(
                "no objective supplied: pass --objective <path>, or put `objective` \
                        (inline) or `objective_file` (path) in the payload — the objective \
                        function is domain-authored data the system never invents (Non-goal 1)"
                    .to_string(),
            )
        }
        1 => {}
        _ => {
            return Err("multiple objective sources supplied: use exactly one of \
                        --objective, payload `objective`, payload `objective_file`"
                .to_string())
        }
    }
    if let Some(inline) = inline {
        return church::objective_from_json_str(&inline.to_string()).map_err(|e| e.to_string());
    }
    let path = if !arg_path.is_empty() {
        arg_path
    } else {
        objective_file.as_deref().unwrap_or_default()
    };
    church::objective_from_path(std::path::Path::new(path)).map_err(|e| e.to_string())
}

/// A [`ChurchProposal`] as JSON. The generic proposal is not `Serialize`
/// (its stage type is domain-specific), so the wire shape is built by hand —
/// mirroring [`proposal_json`], with the `pddl_goal` atom attached.
fn church_proposal_json(p: &ChurchProposal) -> Value {
    json!({
        "goal_description": p.goal_description,
        "target_person": p.target_id,
        "target_stage": p.target_stage.pddl_name(),
        "score": p.score,
        "rationale": p.rationale,
        "proposal_hash": p.proposal_hash,
        "pddl_goal": p.pddl_goal(),
    })
}

/// Enumerate, score, and rank candidate goal states for a church snapshot.
fn church_payload(payload: &str, objective_path: &str) -> std::result::Result<Value, String> {
    let input: ChurchProposeInput = parse_payload(payload)?;
    let objective =
        resolve_church_objective(objective_path, &input.objective, &input.objective_file)?;
    let proposer = ChurchProposer::new(objective);
    let proposals = proposer.propose(&input.state);
    let status = if proposals.is_empty() {
        "no_lawful_candidates"
    } else {
        "proposed"
    };
    Ok(json!({
        "status": status,
        "pack": "church",
        "objective": objective_summary(proposer.objective()),
        "count": proposals.len(),
        "proposals": proposals.iter().map(church_proposal_json).collect::<Vec<_>>(),
    }))
}

/// Rank candidate goal states for a **church-operations** snapshot under a
/// domain-authored objective — Mission Physics beyond revenue. This is the
/// same substrate as `propose revenue`, selected by pack: proof that the
/// proposer/planner/admission/receipt machinery is domain-independent and only
/// the ontology + authored objective change. Output is proposal (O), not
/// authority (O*): a suggestion for the welcome team to weigh, never an
/// instruction and never authority over a person (AR-9).
#[verb]
pub fn church(
    payload: String,
    #[arg(help = "Path to the domain-authored church objective JSON file")] objective: Option<
        String,
    >,
) -> Result<Value> {
    church_payload(&payload, objective.as_deref().unwrap_or(""))
        .map_err(NounVerbError::argument_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The default authored objective shipped beside the proposer crate.
    const OBJECTIVE: &str = include_str!("../../crates/praxis-proposer/revenue_objective.json");

    /// The default authored CHURCH objective (Genesis Day 6).
    const CHURCH_OBJECTIVE: &str =
        include_str!("../../crates/praxis-proposer/church_objective.json");

    /// The shipped PDDL8-safe church domain + fixture problem.
    const CHURCH_PDDL: &str = include_str!("../../ontology/church.pddl");

    /// The church fixture problem's authored goal line (must stay in sync
    /// with `ontology/church.pddl`).
    const CHURCH_FIXTURE_GOAL: &str = "(stage visitor-1 leading)";

    /// The shipped PDDL8-safe revenue domain + fixture problem
    /// (`plan solve` accepts it as a combined file).
    const REVENUE_PDDL: &str = include_str!("../../ontology/revenue.pddl");

    /// The fixture problem's authored goal line (must stay in sync with
    /// `ontology/revenue.pddl`).
    const FIXTURE_GOAL: &str = "(stage acct-1 closed-won)";

    /// Mirror of `crates/praxis-proposer/examples/rank_fixture.rs`.
    fn fixture_state() -> Value {
        json!({
            "accounts": [
                {
                    "id": "acct-1", "stage": "procurement", "amount_cents": 2_500_000,
                    "security_review_done": true, "legal_approved": true,
                    "exec_sponsor": true, "days_in_stage": 12
                },
                {
                    "id": "acct-2", "stage": "qualified", "amount_cents": 800_000,
                    "security_review_done": true, "legal_approved": false,
                    "exec_sponsor": true, "days_in_stage": 45
                },
                {
                    "id": "acct-3", "stage": "lead", "amount_cents": 150_000,
                    "security_review_done": false, "legal_approved": false,
                    "exec_sponsor": false, "days_in_stage": 120
                }
            ]
        })
    }

    fn payload_with_inline_objective() -> String {
        json!({
            "state": fixture_state(),
            "objective": serde_json::from_str::<Value>(OBJECTIVE).unwrap(),
        })
        .to_string()
    }

    #[test]
    fn revenue_ranks_and_hashes_deterministically() {
        let payload = payload_with_inline_objective();
        let a = revenue_payload(&payload, "").expect("revenue_payload should not hard-error");
        let b = revenue_payload(&payload, "").expect("revenue_payload should not hard-error");
        assert_eq!(a, b, "same state + same objective must be byte-identical");
        assert_eq!(a["status"], json!("proposed"));
        assert!(a["count"].as_u64().unwrap() >= 3);
        let top = &a["proposals"][0];
        assert_eq!(top["pddl_goal"], json!(FIXTURE_GOAL));
        assert_eq!(top["proposal_hash"].as_str().unwrap().len(), 64);
        assert!(top["rationale"].as_array().unwrap().len() >= 3);
    }

    #[test]
    fn goal_emits_top_proposal_atom() {
        let out = goal_payload(&payload_with_inline_objective(), "")
            .expect("goal_payload should not hard-error");
        assert_eq!(out["status"], json!("proposed"));
        assert_eq!(out["goal"], json!(FIXTURE_GOAL));
        assert_eq!(out["proposal_hash"].as_str().unwrap().len(), 64);
    }

    /// End-to-end pipe (Genesis Day 1 → Day 2 seam): `propose goal` output
    /// splices into the shipped `ontology/revenue.pddl` problem's
    /// `(:goal ...)` block and `plan solve` (the real solve path, via
    /// `plan::solve_payload`) finds an admitted plan for it. Exercised for
    /// the top proposal AND a lower-ranked one, so the splice is proven on a
    /// goal that differs from the fixture's authored goal text.
    #[test]
    fn propose_goal_feeds_plan_solve() {
        assert!(
            REVENUE_PDDL.contains(FIXTURE_GOAL),
            "fixture goal line drifted from ontology/revenue.pddl"
        );

        let revenue =
            revenue_payload(&payload_with_inline_objective(), "").expect("revenue proposals");
        let proposals = revenue["proposals"].as_array().expect("proposals array");
        assert!(
            proposals.len() >= 2,
            "need at least two ranked proposals for this test"
        );

        for proposal in &proposals[..2] {
            let goal_atom = proposal["pddl_goal"].as_str().expect("pddl_goal string");
            let spliced = REVENUE_PDDL.replace(FIXTURE_GOAL, goal_atom);
            assert!(spliced.contains(goal_atom));
            let solve_input = json!({ "domain": spliced, "mode": "classical" }).to_string();
            let solved = super::super::plan::solve_payload(&solve_input)
                .expect("plan solve should not hard-error on a proposer goal");
            assert_eq!(
                solved["admitted"],
                json!(true),
                "proposer goal {goal_atom} must be reachable in ontology/revenue.pddl"
            );
        }
    }

    #[test]
    fn evidence_blocked_state_is_domain_no_not_error() {
        // A single closed-won account has no lawful forward targets.
        let payload = json!({
            "state": { "accounts": [{
                "id": "acct-9", "stage": "closed_won", "amount_cents": 1,
                "security_review_done": true, "legal_approved": true,
                "exec_sponsor": true, "days_in_stage": 1
            }]},
            "objective": serde_json::from_str::<Value>(OBJECTIVE).unwrap(),
        })
        .to_string();
        let out = goal_payload(&payload, "").expect("domain no must be Ok");
        assert_eq!(out["status"], json!("no_lawful_candidates"));
    }

    #[test]
    fn missing_objective_is_hard_error_never_invented() {
        let payload = json!({ "state": fixture_state() }).to_string();
        let err = revenue_payload(&payload, "").unwrap_err();
        assert!(
            err.contains("never invents"),
            "error must cite Non-goal 1: {err}"
        );
    }

    #[test]
    fn multiple_objective_sources_are_hard_error() {
        let payload = payload_with_inline_objective();
        assert!(revenue_payload(&payload, "/tmp/other.json").is_err());
    }

    #[test]
    fn empty_payload_is_hard_error() {
        assert!(revenue_payload("", "").is_err());
        assert!(goal_payload("", "").is_err());
    }

    #[test]
    fn unknown_fluent_in_objective_is_hard_error() {
        let payload = json!({
            "state": fixture_state(),
            "objective": {"name": "x", "version": "1", "weights": {"vibes": 1.0}},
        })
        .to_string();
        assert!(revenue_payload(&payload, "").is_err());
    }

    #[test]
    fn mrr_payload_reports_the_three_numbers() {
        let payload = json!({ "state": fixture_state() }).to_string();
        let out = mrr_payload(&payload).expect("mrr_payload should not hard-error");
        assert_eq!(out["status"], json!("computed"));
        // acct-1 (procurement, full evidence) is closeable → its amount is the MRR.
        assert_eq!(out["max_reachable_revenue_cents"], json!(2_500_000));
        assert_eq!(out["actual_closed_cents"], json!(0));
        assert_eq!(out["revenue_opportunity_cents"], json!(2_500_000));
        assert_eq!(out["revenue_utilization"], json!(0.0));
        assert_eq!(out["accounts"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn mission_payload_compiles_inline_mission_to_goal() {
        let payload = json!({
            "state": fixture_state(),
            "mission": {
                "mission": "close-q3",
                "constraints": { "min_evidence": ["legal_approved", "security_review_done"] },
                "objective": serde_json::from_str::<Value>(OBJECTIVE).unwrap(),
            }
        })
        .to_string();
        let out = mission_payload(&payload).expect("mission_payload should not hard-error");
        assert_eq!(out["status"], json!("compiled"));
        assert_eq!(out["mission"], json!("close-q3"));
        // Only acct-1 carries both legal + security in this fixture.
        assert_eq!(out["accounts_considered"], json!(1));
        assert_eq!(out["planner_goal"], json!("(stage acct-1 closed-won)"));
    }

    #[test]
    fn mission_payload_requires_exactly_one_source() {
        // Neither source.
        let neither = json!({ "state": fixture_state() }).to_string();
        assert!(mission_payload(&neither).is_err());
        // Both sources.
        let both = json!({
            "state": fixture_state(),
            "mission": { "mission": "m", "objective": "x.json" },
            "mission_file": "m.toml",
        })
        .to_string();
        assert!(mission_payload(&both).is_err());
    }

    // ── church pack (Genesis Day 6): the SAME verb tests, different domain ──

    /// Mirror of the church fixture in the proposer crate's church tests.
    fn church_fixture_state() -> Value {
        json!({
            "people": [
                {
                    "id": "visitor-1", "stage": "connected", "welcomed": true,
                    "followed_up": true, "in_small_group": true, "care_assigned": true,
                    "days_in_stage": 12
                },
                {
                    "id": "visitor-2", "stage": "first_time", "welcomed": true,
                    "followed_up": false, "in_small_group": true, "care_assigned": true,
                    "days_in_stage": 45
                },
                {
                    "id": "visitor-3", "stage": "first_time", "welcomed": false,
                    "followed_up": false, "in_small_group": false, "care_assigned": false,
                    "days_in_stage": 120
                }
            ]
        })
    }

    fn church_payload_with_inline_objective() -> String {
        json!({
            "state": church_fixture_state(),
            "objective": serde_json::from_str::<Value>(CHURCH_OBJECTIVE).unwrap(),
        })
        .to_string()
    }

    #[test]
    fn church_ranks_and_hashes_deterministically() {
        let payload = church_payload_with_inline_objective();
        let a = church_payload(&payload, "").expect("church_payload should not hard-error");
        let b = church_payload(&payload, "").expect("church_payload should not hard-error");
        assert_eq!(a, b, "same state + same objective must be byte-identical");
        assert_eq!(a["status"], json!("proposed"));
        assert_eq!(a["pack"], json!("church"));
        assert!(a["count"].as_u64().unwrap() >= 3);
        let top = &a["proposals"][0];
        assert_eq!(top["pddl_goal"], json!(CHURCH_FIXTURE_GOAL));
        assert_eq!(top["target_person"], json!("visitor-1"));
        assert_eq!(top["proposal_hash"].as_str().unwrap().len(), 64);
        assert!(top["rationale"].as_array().unwrap().len() >= 3);
    }

    /// The domain-pack proof end-to-end: `propose church` output splices into
    /// the shipped `ontology/church.pddl` problem's `(:goal ...)` block and
    /// the SAME `plan solve` path (`plan::solve_payload`) finds an admitted
    /// plan — the identical seam proven for revenue, now for church.
    #[test]
    fn propose_church_feeds_plan_solve() {
        assert!(
            CHURCH_PDDL.contains(CHURCH_FIXTURE_GOAL),
            "church fixture goal drifted from ontology/church.pddl"
        );
        let church =
            church_payload(&church_payload_with_inline_objective(), "").expect("church proposals");
        let proposals = church["proposals"].as_array().expect("proposals array");
        assert!(proposals.len() >= 2, "need at least two ranked proposals");

        for proposal in &proposals[..2] {
            let goal_atom = proposal["pddl_goal"].as_str().expect("pddl_goal string");
            let spliced = CHURCH_PDDL.replace(CHURCH_FIXTURE_GOAL, goal_atom);
            assert!(spliced.contains(goal_atom));
            let solve_input = json!({ "domain": spliced, "mode": "classical" }).to_string();
            let solved = super::super::plan::solve_payload(&solve_input)
                .expect("plan solve should not hard-error on a church proposer goal");
            assert_eq!(
                solved["admitted"],
                json!(true),
                "church goal {goal_atom} must be reachable in ontology/church.pddl"
            );
        }
    }

    #[test]
    fn church_missing_objective_is_hard_error_never_invented() {
        let payload = json!({ "state": church_fixture_state() }).to_string();
        let err = church_payload(&payload, "").unwrap_err();
        assert!(
            err.contains("never invents"),
            "error must cite Non-goal 1: {err}"
        );
    }

    #[test]
    fn church_rejects_revenue_fluent_in_objective() {
        // A revenue fluent is unknown to the church vocabulary — reused
        // deny_unknown_fields discipline, different vocabulary.
        let payload = json!({
            "state": church_fixture_state(),
            "objective": {"name": "x", "version": "1", "weights": {"realized_revenue": 1.0}},
        })
        .to_string();
        assert!(church_payload(&payload, "").is_err());
    }

    #[test]
    fn church_evidence_blocked_state_is_domain_no_not_error() {
        // A single leading person has no lawful forward targets.
        let payload = json!({
            "state": { "people": [{
                "id": "leader-1", "stage": "leading", "welcomed": true,
                "followed_up": true, "in_small_group": true, "care_assigned": true,
                "days_in_stage": 1
            }]},
            "objective": serde_json::from_str::<Value>(CHURCH_OBJECTIVE).unwrap(),
        })
        .to_string();
        let out = church_payload(&payload, "").expect("domain no must be Ok");
        assert_eq!(out["status"], json!("no_lawful_candidates"));
    }
}
