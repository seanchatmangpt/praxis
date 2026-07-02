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
use clap_noun_verb_macros::{arg, verb};
use praxis_proposer::{ObjectiveFunction, Proposal, Proposer, RevenueState};
use serde::Deserialize;
use serde_json::{json, Value};

// ── Parsing helpers ───────────────────────────────────────────────────────

/// Wire schema shared by `propose revenue` and `propose goal`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProposeInput {
    /// The observed revenue pipeline snapshot. Prefer feeding the payload of
    /// an *admitted* law object here (see
    /// `praxis_proposer::RevenueState::from_admitted`); the verb accepts any
    /// well-formed snapshot because the proposal it emits is untrusted
    /// either way (AR-9) — admission happens downstream, on the proposal.
    state: RevenueState,
    /// Inline domain-authored objective (mutually exclusive with
    /// `objective_file` and the `--objective` argument).
    #[serde(default)]
    objective: Option<Value>,
    /// Path to a domain-authored objective JSON file (mutually exclusive
    /// with `objective` and the `--objective` argument).
    #[serde(default)]
    objective_file: Option<String>,
}

/// Parse a payload string into `T`. Empty or invalid JSON is a hard error.
fn parse_payload<T: for<'de> Deserialize<'de>>(payload: &str) -> std::result::Result<T, String> {
    if payload.trim().is_empty() {
        return Err("empty payload".to_string());
    }
    serde_json::from_str(payload).map_err(|e| format!("invalid JSON: {e}"))
}

/// Resolve the authored objective from exactly one of: the `--objective`
/// path argument, the payload's `objective_file` path, or the payload's
/// inline `objective` object. None ⇒ hard error (Non-goal 1: the system
/// never invents values); more than one ⇒ hard error (ambiguous authorship).
fn resolve_objective(
    arg_path: &str,
    input: &ProposeInput,
) -> std::result::Result<ObjectiveFunction, String> {
    let mut sources = 0;
    if !arg_path.is_empty() {
        sources += 1;
    }
    if input.objective.is_some() {
        sources += 1;
    }
    if input.objective_file.is_some() {
        sources += 1;
    }
    match sources {
        0 => {
            return Err("no objective supplied: pass --objective <path>, or put `objective` \
                        (inline) or `objective_file` (path) in the payload — the objective \
                        function is domain-authored data the system never invents (Non-goal 1)"
                .to_string())
        }
        1 => {}
        _ => {
            return Err("multiple objective sources supplied: use exactly one of \
                        --objective, payload `objective`, payload `objective_file`"
                .to_string())
        }
    }
    if let Some(inline) = &input.objective {
        return ObjectiveFunction::from_json_str(&inline.to_string()).map_err(|e| e.to_string());
    }
    let path = if !arg_path.is_empty() {
        arg_path
    } else {
        input.objective_file.as_deref().unwrap_or_default()
    };
    ObjectiveFunction::from_path(std::path::Path::new(path)).map_err(|e| e.to_string())
}

/// A [`Proposal`] as JSON, with the derived `pddl_goal` atom attached so a
/// caller can splice it into a PDDL problem `(:goal ...)` block (e.g. the
/// one shipped in `ontology/revenue.pddl`) without re-deriving it.
fn proposal_json(p: &Proposal) -> Value {
    let mut v = serde_json::to_value(p).unwrap_or(Value::Null);
    if let Value::Object(map) = &mut v {
        map.insert("pddl_goal".to_string(), json!(p.pddl_goal()));
    }
    v
}

fn objective_summary(obj: &ObjectiveFunction) -> Value {
    json!({ "name": obj.name, "version": obj.version })
}

// ── `propose revenue` ─────────────────────────────────────────────────────

/// Enumerate, score, and rank candidate goal states for a revenue snapshot.
///
/// Returns the full ranked proposal list: for each candidate, the goal
/// description, target account/stage, authored-objective score, the
/// line-by-line rationale explaining every score contribution, the blake3
/// `proposal_hash`, and the `pddl_goal` atom. Output is observation (O),
/// never authority (O*) — see the module docs.
fn revenue_payload(payload: &str, objective_path: &str) -> std::result::Result<Value, String> {
    let input: ProposeInput = parse_payload(payload)?;
    let objective = resolve_objective(objective_path, &input)?;
    let proposer = Proposer::new(objective);
    let proposals = proposer.propose(&input.state);
    let status = if proposals.is_empty() { "no_lawful_candidates" } else { "proposed" };
    Ok(json!({
        "status": status,
        "objective": objective_summary(proposer.objective()),
        "count": proposals.len(),
        "proposals": proposals.iter().map(proposal_json).collect::<Vec<_>>(),
    }))
}

// ── `propose goal` ────────────────────────────────────────────────────────

/// Emit only the top-ranked proposal's PDDL goal atom (plus its hash and
/// rationale), ready to splice into a problem `(:goal ...)` block for
/// `plan solve` — e.g. over the shipped `ontology/revenue.pddl` domain.
fn goal_payload(payload: &str, objective_path: &str) -> std::result::Result<Value, String> {
    let input: ProposeInput = parse_payload(payload)?;
    let objective = resolve_objective(objective_path, &input)?;
    let proposer = Proposer::new(objective);
    let proposals = proposer.propose(&input.state);
    let Some(top) = proposals.first() else {
        return Ok(json!({
            "status": "no_lawful_candidates",
            "objective": objective_summary(proposer.objective()),
        }));
    };
    Ok(json!({
        "status": "proposed",
        "objective": objective_summary(proposer.objective()),
        "goal": top.pddl_goal(),
        "goal_description": top.goal_description,
        "target_account": top.target_account,
        "target_stage": top.target_stage,
        "score": top.score,
        "proposal_hash": top.proposal_hash,
        "rationale": top.rationale,
        "candidates_considered": proposals.len(),
    }))
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The default authored objective shipped beside the proposer crate.
    const OBJECTIVE: &str = include_str!("../../crates/praxis-proposer/revenue_objective.json");

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
        assert!(proposals.len() >= 2, "need at least two ranked proposals for this test");

        for proposal in &proposals[..2] {
            let goal_atom = proposal["pddl_goal"].as_str().expect("pddl_goal string");
            let spliced = REVENUE_PDDL.replace(FIXTURE_GOAL, goal_atom);
            assert!(spliced.contains(goal_atom));
            let solve_input =
                json!({ "domain": spliced, "mode": "classical" }).to_string();
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
        assert!(err.contains("never invents"), "error must cite Non-goal 1: {err}");
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
}
