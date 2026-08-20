//! RevTAC v0 — the mission language above the revenue substrate.
//!
//! Revenue operators do not author PDDL, objectives, or proposer invocations
//! by hand. They author **missions**: one small, declarative document that
//! names an intent (`close-q3`), the lawful scope it applies to
//! (`constraints`), and the domain-authored objective to rank under. RevTAC
//! *compiles* that mission down to the substrate the prior phase built — a
//! proposer invocation over a constrained [`RevenueState`] plus a planner goal
//! atom ready for `plan solve` — in the same spirit as ORTAC+ sits one layer
//! above a task/action grammar.
//!
//! # The format (JSON or TOML)
//!
//! ```json
//! {
//!   "mission": "close-q3",
//!   "constraints": {
//!     "min_evidence": ["legal_approved", "security_review_done"],
//!     "exclude_accounts": ["acct-legal-gap"]
//!   },
//!   "objective": "crates/praxis-proposer/revenue_objective.json"
//! }
//! ```
//!
//! - `mission` — a free-form intent name, echoed into the compiled output so a
//!   receipt can bind which mission produced a proposal.
//! - `constraints.min_evidence` — accounts are considered only if they carry
//!   **all** of these evidence flags. Names must be real `Account` evidence
//!   fields (`legal_approved`, `security_review_done`, `exec_sponsor`); an
//!   unknown name is a hard error, never silently ignored.
//! - `constraints.exclude_accounts` — account ids dropped from scope before
//!   proposing.
//! - `objective` — either a path string to a domain-authored objective JSON
//!   file, or an inline objective object. RevTAC never invents the objective
//!   (Non-goal 1); a mission with no objective is a hard error.
//!
//! # What "compile" means (AR-9: still only observation)
//!
//! Compiling a mission produces **proposals (O), not authority (O\*)**. The
//! output carries the ranked proposals, the top proposal's `planner_goal`
//! atom, and the [`MrrReport`] ceiling for the constrained scope — all of
//! which must still pass `law judge`/`law admit` downstream before any
//! effect. RevTAC adds vocabulary, not power.

use praxis_proposer::{
    maximum_reachable_revenue, Account, MrrReport, ObjectiveFunction, Proposal, Proposer,
    RevenueState,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::ops;

/// The evidence flag names an operator may name in `min_evidence`, mapped to
/// the `Account` field each reads. Keeping this list closed is what lets an
/// unknown evidence name be a hard error.
const KNOWN_EVIDENCE: [&str; 3] = ["legal_approved", "security_review_done", "exec_sponsor"];

/// Does `account` carry the named evidence flag? `None` if the name is not a
/// known evidence field.
fn account_has_evidence(account: &Account, name: &str) -> Option<bool> {
    match name {
        "legal_approved" => Some(account.legal_approved),
        "security_review_done" => Some(account.security_review_done),
        "exec_sponsor" => Some(account.exec_sponsor),
        _ => None,
    }
}

/// Lawful-scope constraints an operator attaches to a mission.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MissionConstraints {
    /// Accounts must carry **all** of these evidence flags to be in scope.
    #[serde(default)]
    pub min_evidence: Vec<String>,
    /// Account ids removed from scope before proposing.
    #[serde(default)]
    pub exclude_accounts: Vec<String>,
}

/// The objective a mission ranks under: a path to an authored JSON file, or an
/// inline authored objective object. Never a default — the system does not
/// invent objectives (Non-goal 1).
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ObjectiveSource {
    /// Path to a domain-authored objective JSON file.
    Path(String),
    /// Inline domain-authored objective object.
    Inline(Value),
}

/// A RevTAC mission: the whole authored document.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mission {
    /// Free-form intent name (e.g. `close-q3`).
    pub mission: String,
    /// Lawful scope; empty (accept every account) if omitted.
    #[serde(default)]
    pub constraints: MissionConstraints,
    /// Domain-authored objective source (path or inline).
    pub objective: ObjectiveSource,
}

impl Mission {
    /// Parse a mission from JSON or TOML text, chosen by `format`
    /// (`"json"`, `"toml"`, or `"auto"` — auto treats a leading `{` as JSON,
    /// otherwise TOML).
    pub fn parse(text: &str, format: &str) -> Result<Mission, String> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err("empty mission document".to_string());
        }
        let is_json = match format {
            "json" => true,
            "toml" => false,
            "auto" | "" => trimmed.starts_with('{'),
            other => return Err(format!("unknown mission format '{other}'")),
        };
        if is_json {
            serde_json::from_str(trimmed).map_err(|e| format!("invalid mission JSON: {e}"))
        } else {
            toml::from_str(trimmed).map_err(|e| format!("invalid mission TOML: {e}"))
        }
    }

    /// Resolve the authored objective from this mission's [`ObjectiveSource`].
    fn resolve_objective(&self) -> Result<ObjectiveFunction, String> {
        match &self.objective {
            ObjectiveSource::Path(p) => {
                ObjectiveFunction::from_path(std::path::Path::new(p)).map_err(|e| e.to_string())
            }
            ObjectiveSource::Inline(v) => {
                ObjectiveFunction::from_json_str(&v.to_string()).map_err(|e| e.to_string())
            }
        }
    }

    /// Validate that every `min_evidence` name is a real evidence field.
    fn validate_evidence_names(&self) -> Result<(), String> {
        for name in &self.constraints.min_evidence {
            if !KNOWN_EVIDENCE.contains(&name.as_str()) {
                return Err(format!(
                    "unknown evidence flag '{name}' in min_evidence (known: {KNOWN_EVIDENCE:?})"
                ));
            }
        }
        Ok(())
    }
}

/// Why an account was dropped from a mission's scope.
fn drop_reason(account: &Account, c: &MissionConstraints) -> Option<String> {
    if c.exclude_accounts.iter().any(|id| id == &account.id) {
        return Some("excluded_by_mission".to_string());
    }
    let missing: Vec<&String> = c
        .min_evidence
        .iter()
        .filter(|e| account_has_evidence(account, e) == Some(false))
        .collect();
    if !missing.is_empty() {
        return Some(format!("missing_min_evidence: {missing:?}"));
    }
    None
}

fn proposal_json(p: &Proposal) -> Value {
    let mut v = serde_json::to_value(p).unwrap_or(Value::Null);
    if let Value::Object(map) = &mut v {
        map.insert("pddl_goal".to_string(), json!(p.pddl_goal()));
    }
    v
}

/// Partition `state`'s accounts into in-scope vs dropped (with reasons) under
/// `constraints`. Shared by [`compile_mission`] and [`run_mission`] so the
/// scoping computed at compile time is exactly the scoping the plan/admission
/// step below actually solves and admits over — never a second, drifting copy.
fn scope_state(
    constraints: &MissionConstraints,
    state: &RevenueState,
) -> (RevenueState, Vec<Value>) {
    let mut in_scope = Vec::new();
    let mut dropped = Vec::new();
    for a in &state.accounts {
        match drop_reason(a, constraints) {
            Some(reason) => dropped.push(json!({ "id": a.id, "reason": reason })),
            None => in_scope.push(a.clone()),
        }
    }
    (RevenueState { accounts: in_scope }, dropped)
}

/// Compile a parsed [`Mission`] against an observed [`RevenueState`] into the
/// substrate invocation: a filtered proposer run plus a planner goal atom and
/// the reachable-revenue ceiling for the constrained scope.
///
/// Output is observation (O), not authority (O\*): the `planner_goal` and
/// proposals must still be admitted downstream (see [`run_mission`]).
pub fn compile_mission(mission: &Mission, state: &RevenueState) -> Result<Value, String> {
    mission.validate_evidence_names()?;
    let objective = mission.resolve_objective()?;

    let (scoped_state, dropped) = scope_state(&mission.constraints, state);

    // The compiled proposer invocation over the constrained scope.
    let proposer = Proposer::new(objective.clone());
    let proposals = proposer.propose(&scoped_state);

    // The reachable-revenue ceiling for exactly this scope.
    let mrr: MrrReport = maximum_reachable_revenue(&scoped_state);

    let (status, planner_goal, top_hash) = match proposals.first() {
        Some(top) => ("compiled", json!(top.pddl_goal()), json!(top.proposal_hash)),
        None => ("no_lawful_candidates", Value::Null, Value::Null),
    };

    Ok(json!({
        "status": status,
        "mission": mission.mission,
        "objective": { "name": objective.name, "version": objective.version },
        "constraints": {
            "min_evidence": mission.constraints.min_evidence,
            "exclude_accounts": mission.constraints.exclude_accounts,
        },
        "accounts_considered": scoped_state.accounts.len(),
        "accounts_dropped": dropped,
        // The compiled planner goal: the top proposal's PDDL atom, ready to
        // splice into a `plan solve` problem `(:goal ...)` block.
        "planner_goal": planner_goal,
        "top_proposal_hash": top_hash,
        "proposals": proposals.iter().map(proposal_json).collect::<Vec<_>>(),
        "mrr": mrr,
    }))
}

/// Compile `mission` and, if it yields a lawful top proposal, carry it all the
/// way through goal → plan → admission → receipt over the RevTAC-scoped
/// state — closing the gap between RevTAC's scoping/ranking layer and the
/// generic plan/admit/receipt pipe (`crate::revenue::plan_and_admit` +
/// `crate::ops::receipt_payload`), which never touched each other before.
///
/// `ts_ns` fixes the receipt timestamp so the returned `chain_hash` is stable
/// across runs with identical inputs. If the mission compiles to
/// `"no_lawful_candidates"` (empty scope, or no admissible proposal), this
/// returns that compiled observation unchanged — there is nothing to plan or
/// admit, and that is not an error, only the absence of a lawful candidate.
/// A solvable-but-unadmittable plan, or a goal the planner cannot reach, is a
/// hard `Err` — this pipe is only "real" if it stays green end to end.
pub fn run_mission(mission: &Mission, state: &RevenueState, ts_ns: u64) -> Result<Value, String> {
    let compiled = compile_mission(mission, state)?;
    if compiled["status"] != json!("compiled") {
        return Ok(json!({
            "mission": mission.mission,
            "status": "no_lawful_candidates",
            "compiled": compiled,
        }));
    }
    let goal_atom = compiled["planner_goal"]
        .as_str()
        .ok_or_else(|| "compiled mission missing planner_goal".to_string())?
        .to_string();

    // Re-derive the exact scoped state the compiled proposal was ranked
    // over, so the plan is solved and admitted against the same accounts —
    // never a raw, unscoped state.
    let (scoped_state, _dropped) = scope_state(&mission.constraints, state);
    let (plan_labels, admissions) = crate::revenue::plan_and_admit(&scoped_state, &goal_atom)?;

    let receipt_value = json!({
        "mission": mission.mission,
        "proposal_hash": compiled["top_proposal_hash"],
        "goal": goal_atom,
        "admitted_plan": plan_labels,
    });
    let receipt_input = json!({
        "value": receipt_value,
        "instruction_id": 1,
        "ts_ns": ts_ns,
    });
    let receipt = ops::receipt_payload(&receipt_input.to_string())?;
    if receipt["status"] != json!("receipted") {
        return Err(format!(
            "mission receipt was not issued: {}",
            receipt["status"]
        ));
    }

    Ok(json!({
        "mission": mission.mission,
        "status": "receipted",
        "ts_ns": ts_ns,
        "compiled": compiled,
        "plan": plan_labels,
        "admissions": admissions,
        "chain_hash": receipt["chain_hash"],
        "binds_proposal_hash": compiled["top_proposal_hash"],
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const OBJECTIVE: &str = include_str!("../crates/praxis-proposer/revenue_objective.json");

    fn fixture_state() -> Result<RevenueState, crate::AppError> {
        serde_json::from_value(json!({
            "accounts": [
                { "id": "acct-apex", "stage": "proposal", "amount_cents": 5_000_000,
                  "security_review_done": true, "legal_approved": true, "exec_sponsor": true, "days_in_stage": 20 },
                { "id": "acct-legal-gap", "stage": "qualified", "amount_cents": 3_000_000,
                  "security_review_done": true, "legal_approved": false, "exec_sponsor": true, "days_in_stage": 40 },
                { "id": "acct-fresh", "stage": "lead", "amount_cents": 1_000_000,
                  "security_review_done": false, "legal_approved": false, "exec_sponsor": false, "days_in_stage": 90 }
            ]
        }))
        .map_err(|e| crate::AppError::Other(e.to_string()))
    }

    fn mission_json() -> Result<String, crate::AppError> {
        Ok(json!({
            "mission": "close-q3",
            "constraints": {
                "min_evidence": ["legal_approved", "security_review_done"],
                "exclude_accounts": []
            },
            "objective": serde_json::from_str::<Value>(OBJECTIVE).map_err(|e| crate::AppError::Other(e.to_string()))?,
        })
        .to_string())
    }

    #[test]
    fn compiles_mission_to_planner_goal() -> Result<(), crate::AppError> {
        let mission = Mission::parse(&mission_json()?, "auto")
            .map_err(|e| crate::AppError::Other(e.to_string()))?;
        let out = compile_mission(&mission, &fixture_state()?)
            .map_err(|e| crate::AppError::Other(e.to_string()))?;
        assert_eq!(out["status"], json!("compiled"));
        assert_eq!(out["mission"], json!("close-q3"));
        // min_evidence [legal, security] drops legal-gap and fresh; only apex is in scope.
        assert_eq!(out["accounts_considered"], json!(1));
        assert_eq!(out["planner_goal"], json!("(stage acct-apex closed-won)"));
        assert_eq!(
            out["top_proposal_hash"]
                .as_str()
                .ok_or_else(|| crate::AppError::Other("missing hash".into()))?
                .len(),
            64
        );
        // The scoped MRR is apex's full amount (it is closeable).
        assert_eq!(out["mrr"]["max_reachable_revenue_cents"], json!(5_000_000));
        Ok(())
    }

    #[test]
    fn toml_and_json_missions_compile_identically() -> Result<(), crate::AppError> {
        let toml_text = r#"
mission = "close-q3"
objective = "crates/praxis-proposer/revenue_objective.json"

[constraints]
min_evidence = ["legal_approved", "security_review_done"]
exclude_accounts = []
"#;
        let json_text = json!({
            "mission": "close-q3",
            "constraints": { "min_evidence": ["legal_approved", "security_review_done"], "exclude_accounts": [] },
            "objective": "crates/praxis-proposer/revenue_objective.json",
        })
        .to_string();

        let m_toml =
            Mission::parse(toml_text, "toml").map_err(|e| crate::AppError::Other(e.to_string()))?;
        let m_json = Mission::parse(&json_text, "json")
            .map_err(|e| crate::AppError::Other(e.to_string()))?;
        let state = fixture_state()?;
        let a =
            compile_mission(&m_toml, &state).map_err(|e| crate::AppError::Other(e.to_string()))?;
        let b =
            compile_mission(&m_json, &state).map_err(|e| crate::AppError::Other(e.to_string()))?;
        assert_eq!(
            a, b,
            "TOML and JSON missions must compile to identical output"
        );
        Ok(())
    }

    #[test]
    fn exclude_accounts_removes_from_scope_with_reason() -> Result<(), crate::AppError> {
        let mission = Mission::parse(
            &json!({
                "mission": "exclude-apex",
                "constraints": { "exclude_accounts": ["acct-apex"] },
                "objective": serde_json::from_str::<Value>(OBJECTIVE).map_err(|e| crate::AppError::Other(e.to_string()))?,
            })
            .to_string(),
            "json",
        )
        .map_err(|e| crate::AppError::Other(e.to_string()))?;
        let out = compile_mission(&mission, &fixture_state()?)
            .map_err(|e| crate::AppError::Other(e.to_string()))?;
        let dropped = out["accounts_dropped"]
            .as_array()
            .ok_or_else(|| crate::AppError::Other("missing array".into()))?;
        assert!(dropped
            .iter()
            .any(|d| d["id"] == json!("acct-apex") && d["reason"] == json!("excluded_by_mission")));
        Ok(())
    }

    #[test]
    fn unknown_evidence_flag_is_hard_error() -> Result<(), crate::AppError> {
        let mission = Mission::parse(
            &json!({
                "mission": "typo",
                "constraints": { "min_evidence": ["legel_aproved"] },
                "objective": serde_json::from_str::<Value>(OBJECTIVE).map_err(|e| crate::AppError::Other(e.to_string()))?,
            })
            .to_string(),
            "json",
        )
        .map_err(|e| crate::AppError::Other(e.to_string()))?;
        let res = compile_mission(&mission, &fixture_state()?);
        match res {
            Err(err) => {
                assert!(err.contains("unknown evidence flag"), "got: {err}");
            }
            Ok(_) => return Err(crate::AppError::Other("expected an error".into())),
        }
        Ok(())
    }

    #[test]
    fn missing_objective_is_hard_error() -> Result<(), crate::AppError> {
        // No `objective` key at all: deny_unknown_fields + required field.
        let res = Mission::parse(
            &json!({ "mission": "no-obj", "constraints": {} }).to_string(),
            "json",
        );
        match res {
            Err(err) => {
                assert!(
                    err.contains("objective") || err.contains("missing"),
                    "got: {err}"
                );
            }
            Ok(_) => return Err(crate::AppError::Other("expected an error".into())),
        }
        Ok(())
    }

    #[test]
    fn over_constrained_mission_is_domain_no_not_error() -> Result<(), crate::AppError> {
        // Require an evidence flag no in-scope account has → empty scope.
        let mission = Mission::parse(
            &json!({
                "mission": "impossible",
                "constraints": { "min_evidence": ["exec_sponsor"], "exclude_accounts": ["acct-apex", "acct-legal-gap"] },
                "objective": serde_json::from_str::<Value>(OBJECTIVE).map_err(|e| crate::AppError::Other(e.to_string()))?,
            })
            .to_string(),
            "json",
        )
        .map_err(|e| crate::AppError::Other(e.to_string()))?;
        let out = compile_mission(&mission, &fixture_state()?)
            .map_err(|e| crate::AppError::Other(e.to_string()))?;
        assert_eq!(out["status"], json!("no_lawful_candidates"));
        assert_eq!(out["planner_goal"], Value::Null);
        Ok(())
    }

    /// Under `--features law-signed`, the receipt step fails closed without a
    /// signing key; set a fixed test key once so this test stays green in
    /// that build too (mirrors `revenue::tests::ensure_signing_key`).
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

    /// The composition this module exists to close: a RevTAC mission that
    /// *scopes* the fixture (excludes `acct-legal-gap`, requires
    /// `legal_approved` + `security_review_done`) drives the real
    /// plan→admission→receipt pipe, not just a ranked-proposal observation.
    #[test]
    fn run_mission_produces_a_receipted_scoped_outcome() -> Result<(), crate::AppError> {
        ensure_signing_key();
        let mission = Mission::parse(
            &json!({
                "mission": "close-q3-scoped",
                "constraints": {
                    "min_evidence": ["legal_approved", "security_review_done"],
                    "exclude_accounts": ["acct-legal-gap"]
                },
                "objective": serde_json::from_str::<Value>(OBJECTIVE)
                    .map_err(|e| crate::AppError::Other(e.to_string()))?,
            })
            .to_string(),
            "json",
        )
        .map_err(|e| crate::AppError::Other(e.to_string()))?;
        let state = fixture_state()?;

        let out = run_mission(&mission, &state, 1_000)
            .map_err(|e| crate::AppError::Other(e.to_string()))?;

        assert_eq!(out["status"], json!("receipted"));
        // (a) every admitted plan action was actually admitted.
        let admissions = out["admissions"]
            .as_array()
            .ok_or_else(|| crate::AppError::Other("missing admissions array".into()))?;
        assert!(!admissions.is_empty(), "expected a non-empty admitted plan");
        for admission in admissions {
            assert_eq!(
                admission["admit_status"],
                json!("admitted"),
                "expected every plan action admitted, got: {admission}"
            );
        }
        // (b) the receipt's bound proposal_hash matches the compiled top hash.
        assert_eq!(
            out["binds_proposal_hash"],
            out["compiled"]["top_proposal_hash"]
        );
        assert_ne!(out["binds_proposal_hash"], Value::Null);

        // (c) determinism: same ts_ns -> identical chain_hash.
        let out2 = run_mission(&mission, &state, 1_000)
            .map_err(|e| crate::AppError::Other(e.to_string()))?;
        assert_eq!(
            out["chain_hash"], out2["chain_hash"],
            "same ts_ns must yield a byte-identical chain_hash"
        );
        Ok(())
    }
}
