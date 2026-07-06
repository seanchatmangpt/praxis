//! `plan` verb dispatcher — route, solve, analyze, execute, lawobject.
//!
//! Thin CLI wrappers over `bcinr_pddl`'s capability router, PDDL8 grounder,
//! forward-search planner, schedule analyzer, and Prolog8-gated temporal
//! executor. Every verb takes a single JSON `payload` string (parsed by the
//! pure `*_payload` function below it) except `lawobject`, a no-payload
//! self-test.
//!
//! `CapabilityRouteReceipt`, `CostVector`, and `ScheduleAnalysis64` are not
//! `Serialize`, so their JSON is hand-built here; `Pddl8Tape`, `TemporalPlan`,
//! `TemporalExecutionReceipt`, and `OCEL` are `Serialize` and go through
//! `serde_json::to_value` directly. Non-finite `f64`s (e.g.
//! `CostVector::refused().human_attention_seconds == f64::INFINITY`) are
//! converted to JSON `null` rather than handed to `serde_json`, which cannot
//! represent them.
//!
//! Malformed input (empty/invalid JSON, an unreadable file path, PDDL text
//! that fails to parse, an out-of-range `case_id`) is a hard `Err`. Domain
//! *infeasibility* — bounded search exhausted without reaching the goal, an
//! empty grounding, or a Prolog8 policy-gate denial at execution time — is
//! `Ok(json)` with `"admitted": false` and a `refusal_reason`, matching the
//! rest of the CLI's "domain denial is `Ok`" convention.

use bcinr_pddl::{
    analyze_schedule, compute_plan_chain, domain_from_pddl, execute::execute_temporal_plan,
    problem_from_pddl, route_capability_plan, CapabilityRouteReceipt, CapabilityTask,
    CapacityDelta, CostVector, DesiredEffect, GroundTemporalProblem, Pddl8Error,
    ScheduleAnalysis64,
};
use clap_noun_verb::error::{NounVerbError, Result};
use clap_noun_verb_macros::verb;
#[cfg(feature = "ggen")]
use my_conforming_project::mfg;
// Shared PDDL helpers + the single `plan solve` implementation live in the
// library `ops` module so the CLI verb and the MCP `plan_solve` tool call one
// implementation (AR-2, no drift). route/analyze/execute reuse the helpers.
use my_conforming_project::ops::{
    self, finite_or_null, is_infeasible, parse_payload, refusal_json, resolve_pddl_source, to_json,
};
use serde::Deserialize;
use serde_json::{json, Value};

// ── CostVector / ScheduleAnalysis64 hand-serialization ────────────────────

fn cost_json(cost: &CostVector) -> Value {
    json!({
        "admitted": cost.admitted,
        "unreceipted_mutation_risk": cost.unreceipted_mutation_risk,
        "human_attention_seconds": finite_or_null(cost.human_attention_seconds),
        "token_cost": cost.token_cost,
        "latency_ms": cost.latency_ms,
        "context_switches": cost.context_switches,
    })
}

fn capacity_delta_json(cd: &CapacityDelta) -> Value {
    json!({
        "minus_one_makespan": cd.minus_one_makespan.map_or(Value::Null, finite_or_null),
        "baseline_makespan": finite_or_null(cd.baseline_makespan),
        "plus_one_makespan": cd.plus_one_makespan.map_or(Value::Null, finite_or_null),
    })
}

fn schedule_analysis_json(a: &ScheduleAnalysis64) -> Value {
    let critical_path_ops: Vec<usize> = (0..a.op_count)
        .filter(|i| (a.critical_path_mask >> i) & 1 == 1)
        .collect();
    let slack_by_op: Vec<Value> = a.slack_by_op[..a.op_count]
        .iter()
        .copied()
        .map(finite_or_null)
        .collect();
    json!({
        "makespan": finite_or_null(a.makespan),
        "critical_path_mask_hex": format!("{:016x}", a.critical_path_mask),
        "critical_path_ops": critical_path_ops,
        "max_parallelism": a.max_parallelism,
        "binding_resource_mask_hex": format!("{:016x}", a.binding_resource_mask),
        "slack_by_op": slack_by_op,
        "op_count": a.op_count,
        "capacity_delta": a.capacity_delta.as_ref().map(capacity_delta_json),
    })
}

// ── `plan route` ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct DesiredEffectInput {
    kind: String,
    file: String,
}

#[derive(Deserialize)]
struct RouteInput {
    desired_effects: Vec<DesiredEffectInput>,
    attention_capacity: u32,
}

/// Route a [`CapabilityTask`] to a schedulable, cost-ordered plan.
///
/// `desired_effects[].kind` is one of `"edited"`, `"form_filled"`,
/// `"drafted"`; an unrecognized kind is malformed input (`Err`). Routing
/// infeasibility under the given `attention_capacity` is `route_capability_plan`'s
/// own `Ok(receipt.admitted == false)` path — passed straight through.
fn route_payload(payload: &str) -> std::result::Result<Value, String> {
    let input: RouteInput = parse_payload(payload)?;
    let mut effects = Vec::with_capacity(input.desired_effects.len());
    for e in input.desired_effects {
        let effect = match e.kind.as_str() {
            "edited" => DesiredEffect::Edited(e.file),
            "form_filled" => DesiredEffect::FormFilled(e.file),
            "drafted" => DesiredEffect::Drafted(e.file),
            other => {
                return Err(format!(
                    "unknown desired_effect kind `{other}` (expected edited|form_filled|drafted)"
                ))
            }
        };
        effects.push(effect);
    }
    let task = CapabilityTask {
        desired_effects: effects,
        attention_capacity: input.attention_capacity,
    };
    let receipt: CapabilityRouteReceipt =
        route_capability_plan(&task).map_err(|e| e.to_string())?;

    Ok(json!({
        "admitted": receipt.admitted,
        "refusal_reason": receipt.refusal_reason,
        "plan": to_json(&receipt.plan),
        "analysis": receipt.analysis.as_ref().map(schedule_analysis_json),
        "cost": cost_json(&receipt.cost),
        "route_chain": receipt.route_chain,
    }))
}

// ── `plan solve` ──────────────────────────────────────────────────────────

/// Solve a classical or temporal PDDL8 problem — thin call-through to the
/// single implementation in [`my_conforming_project::ops::plan_solve_payload`],
/// which the MCP `plan_solve` tool also calls (AR-2: one implementation, no
/// drift). `pub(crate)` so `verbs::propose`'s pipe test can drive the real
/// solve path with a proposer-emitted goal.
pub(crate) fn solve_payload(payload: &str) -> std::result::Result<Value, String> {
    ops::plan_solve_payload(payload)
}

// ── `plan analyze` ────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct AnalyzeInput {
    domain: Option<String>,
    problem: Option<String>,
    domain_file: Option<String>,
    problem_file: Option<String>,
    #[serde(default)]
    resource_keys: Vec<String>,
}

/// Ground a temporal problem and run `analyze_schedule`: makespan, critical
/// path (as both a hex bitmask and a decoded op-index list), max
/// parallelism, and ±1 capacity sensitivity for `resource_keys[0]`.
fn analyze_payload(payload: &str) -> std::result::Result<Value, String> {
    let input: AnalyzeInput = parse_payload(payload)?;
    let (domain_text, problem_text) = resolve_pddl_source(
        input.domain,
        input.problem,
        input.domain_file,
        input.problem_file,
    )?;
    let domain = domain_from_pddl(&domain_text).map_err(|e| e.to_string())?;
    let problem = problem_from_pddl(&problem_text).map_err(|e| e.to_string())?;
    let gtp = match GroundTemporalProblem::build(&domain, &problem) {
        Ok(g) => g,
        Err(e) if is_infeasible(&e) => return Ok(refusal_json("temporal", &e)),
        Err(e) => return Err(e.to_string()),
    };
    match analyze_schedule(&gtp, &input.resource_keys) {
        Ok(analysis) => Ok(json!({
            "admitted": true,
            "analysis": schedule_analysis_json(&analysis),
        })),
        Err(e) if is_infeasible(&e) => Ok(refusal_json("temporal", &e)),
        Err(e) => Err(e.to_string()),
    }
}

// ── `plan execute` ────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct PolicyRuleInput {
    head: String,
    #[serde(default)]
    body: Vec<String>,
}

#[derive(Deserialize)]
struct ExecuteInput {
    domain: Option<String>,
    problem: Option<String>,
    domain_file: Option<String>,
    problem_file: Option<String>,
    case_id: String,
    #[serde(default)]
    policy_rules: Vec<PolicyRuleInput>,
}

/// Ground + solve a temporal problem, then run `execute_temporal_plan`
/// through the Prolog8 `may_fire` admission gate. `policy_rules` are
/// `(head, body[])` May-fire Horn rules; empty (the default) is permissive —
/// every distinct action name in the plan is pre-admitted.
///
/// A Prolog8 denial (`Pddl8Error::StepDenied`) is domain infeasibility, not
/// an error: it comes back as `Ok({"admitted": false, ...})`. An invalid
/// `case_id` is malformed input (`Err`).
fn execute_payload(payload: &str) -> std::result::Result<Value, String> {
    let input: ExecuteInput = parse_payload(payload)?;
    let (domain_text, problem_text) = resolve_pddl_source(
        input.domain,
        input.problem,
        input.domain_file,
        input.problem_file,
    )?;
    let domain = domain_from_pddl(&domain_text).map_err(|e| e.to_string())?;
    let problem = problem_from_pddl(&problem_text).map_err(|e| e.to_string())?;
    let gtp = match GroundTemporalProblem::build(&domain, &problem) {
        Ok(g) => g,
        Err(e) if is_infeasible(&e) => return Ok(refusal_json("temporal", &e)),
        Err(e) => return Err(e.to_string()),
    };
    let plan = match gtp.find_temporal_plan() {
        Ok(p) => p,
        Err(e) if is_infeasible(&e) => return Ok(refusal_json("temporal", &e)),
        Err(e) => return Err(e.to_string()),
    };

    let policy_owned: Vec<(String, Vec<String>)> = input
        .policy_rules
        .into_iter()
        .map(|r| (r.head, r.body))
        .collect();
    let policy_refs: Vec<(&str, Vec<&str>)> = policy_owned
        .iter()
        .map(|(h, b)| (h.as_str(), b.iter().map(String::as_str).collect()))
        .collect();

    match execute_temporal_plan(&plan, &domain, &problem, &input.case_id, &policy_refs) {
        Ok((receipt, ocel)) => {
            let plan_chain = compute_plan_chain(&plan.steps);
            Ok(json!({
                "admitted": true,
                "case_id": input.case_id,
                "receipt": to_json(&receipt),
                "ocel": to_json(&ocel),
                "plan_chain": plan_chain,
            }))
        }
        Err(e @ Pddl8Error::StepDenied { .. }) => Ok(refusal_json("temporal", &e)),
        Err(Pddl8Error::InvalidCaseId(msg)) => Err(format!("invalid case_id: {msg}")),
        Err(e) => Err(e.to_string()),
    }
}

// ── `plan lawobject` ──────────────────────────────────────────────────────

/// The golden action sequence the mfg lane's `ontology/lawobject.ttl`
/// manufactures and solves — see `src/mfg.rs`'s
/// `manufacture_lawobject_golden_solves` test for the same assertion made
/// against the manufacturing pipeline directly.
#[cfg(feature = "ggen")]
const LAWOBJECT_GOLDEN_PLAN: [&str; 5] = [
    "supply-evidence",
    "clear-obligations",
    "judge",
    "admit",
    "receipt",
];

/// Self-test: manufacture `ontology/lawobject.ttl` into PDDL8 domain/problem
/// text, solve it, and assert the golden 5-action plan. Also reports (but
/// never asserts on, and never treats as a planner input) whether the ADL
/// exemplar in `docs/lawobject-capability.pddl` round-trips through
/// `domain_from_pddl`/`problem_from_pddl` — that file uses `forall`/`implies`
/// in preconditions, which the STRIPS8 grounder cannot ground even where the
/// parser accepts them, so it is informational provenance only, not a
/// contract this verb enforces (empirically, `bcinr-pddl`'s current parser
/// rejects the file's `judge` action; this is expected and not a bug in this
/// crate — see the `adl_exemplar` field for the live result).
///
/// A mismatch against the golden *manufactured* plan is a hard `Err`: this
/// verb exists to assert that fixed invariant, not to report a domain-level
/// "no".
#[cfg(feature = "ggen")]
fn lawobject_payload() -> std::result::Result<Value, String> {
    const ONTOLOGY: &str = include_str!("../../ontology/lawobject.ttl");
    const ADL_EXEMPLAR: &str = include_str!("../../docs/lawobject-capability.pddl");

    let manufactured =
        mfg::manufacture(ONTOLOGY, "ontology/lawobject.ttl").map_err(|e| e.to_string())?;
    let report = mfg::validate(&manufactured.domain_text, &manufactured.problem_text);
    if !report.solvable {
        return Err(format!(
            "lawobject self-test: golden plan not found: {:?}",
            report.error
        ));
    }
    if report.plan_steps != LAWOBJECT_GOLDEN_PLAN {
        return Err(format!(
            "lawobject self-test: expected plan {LAWOBJECT_GOLDEN_PLAN:?}, got {:?}",
            report.plan_steps
        ));
    }

    let (adl_domain_text, adl_problem_text) = ops::split_combined(ADL_EXEMPLAR)?;
    let adl_domain_parses = domain_from_pddl(&adl_domain_text).is_ok();
    let adl_problem_parses = problem_from_pddl(&adl_problem_text).is_ok();

    Ok(json!({
        "admitted": true,
        "graph_hash": manufactured.graph_hash_hex,
        "plan_steps": report.plan_steps,
        "plan_len": report.plan_len,
        "grounded_actions": report.grounded_actions,
        "adl_exemplar": {
            "note": "informational only: ADL uses forall/implies in preconditions, \
                     never solved and not asserted on by this verb",
            "domain_parses": adl_domain_parses,
            "problem_parses": adl_problem_parses,
        },
    }))
}

// ── Verb wrappers ─────────────────────────────────────────────────────────

/// Route a capability task (`desired_effects` + `attention_capacity`) to a
/// schedulable, cost-ordered plan over the fixed capability set.
#[verb]
pub fn route(payload: String) -> Result<Value> {
    route_payload(&payload).map_err(NounVerbError::argument_error)
}

/// Solve a classical or temporal PDDL8 problem.
#[verb]
pub fn solve(payload: String) -> Result<Value> {
    solve_payload(&payload).map_err(NounVerbError::argument_error)
}

/// Analyze the schedule of a temporal PDDL8 problem's found plan.
#[verb]
pub fn analyze(payload: String) -> Result<Value> {
    analyze_payload(&payload).map_err(NounVerbError::argument_error)
}

/// Solve and execute a temporal PDDL8 problem through the Prolog8 admission
/// gate, producing a BLAKE3-chained receipt and an OCEL export.
#[verb]
pub fn execute(payload: String) -> Result<Value> {
    execute_payload(&payload).map_err(NounVerbError::argument_error)
}

/// Self-test: manufacture + solve the mfg lane's `ontology/lawobject.ttl`
/// and assert the golden 5-action plan.
#[cfg(feature = "ggen")]
#[verb]
pub fn lawobject() -> Result<Value> {
    lawobject_payload().map_err(NounVerbError::argument_error)
}

/// Run the full planner vertical slice: goal graph (`pdl:` Turtle) ->
/// manufactured PDDL8 -> classical solve -> POWL sequence -> receipted
/// bcinr execution -> artifact write behind the solvability verifier ->
/// ledger receipt. See `src/plan_run.rs`.
#[cfg(feature = "ggen")]
#[verb]
pub fn run(
    #[arg(help = "Path to the pdl: Turtle goal ontology (domain + problem facts)")] goal: String,
    #[arg(
        default_value = "target/plan_run",
        help = "Directory the manufactured artifact is written to"
    )]
    out_dir: String,
    #[arg(
        default_value = "",
        help = "Receipts ledger directory (defaults to the configured receipts.dir)"
    )]
    receipts_dir: String,
) -> Result<Value> {
    let dir = if receipts_dir.is_empty() {
        my_conforming_project::config::config()
            .map(|admitted| admitted.value().receipts.dir.clone())
            .unwrap_or_else(|_| "receipts".to_string())
    } else {
        receipts_dir
    };
    my_conforming_project::plan_run::plan_run_payload(&goal, &out_dir, &dir)
        .map_err(NounVerbError::argument_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLASSICAL_DOMAIN: &str = r#"
(define (domain test-classical)
  (:requirements :strips)
  (:predicates (done))
  (:action finish
    :parameters ()
    :precondition ()
    :effect (done)))
"#;
    const CLASSICAL_PROBLEM: &str = r#"
(define (problem test-classical-case)
  (:domain test-classical)
  (:objects)
  (:init)
  (:goal (done)))
"#;

    const TEMPORAL_DOMAIN: &str = r#"
(define (domain test-temporal)
  (:requirements :durative-actions :typing)
  (:types item)
  (:predicates (done ?i - item))
  (:durative-action finish
    :parameters (?i - item)
    :duration (= ?duration 1)
    :condition (at start (not (done ?i)))
    :effect (at end (done ?i))))
"#;
    const TEMPORAL_PROBLEM: &str = r#"
(define (problem test-temporal-case)
  (:domain test-temporal)
  (:objects i1 - item)
  (:init)
  (:goal (done i1)))
"#;

    fn solve_payload_json(domain: &str, problem: &str, mode: &str) -> Value {
        let payload = json!({ "domain": domain, "problem": problem, "mode": mode }).to_string();
        solve_payload(&payload).expect("solve_payload should not hard-error")
    }

    // ── lawobject golden ───────────────────────────────────────────────

    #[cfg(feature = "ggen")]
    #[test]
    fn lawobject_golden_plan() {
        let result = lawobject_payload().expect("lawobject self-test should pass");
        assert_eq!(result["admitted"], json!(true));
        assert_eq!(
            result["plan_steps"],
            json!([
                "supply-evidence",
                "clear-obligations",
                "judge",
                "admit",
                "receipt"
            ])
        );
        // The ADL exemplar's parse outcome is informational only (see
        // `lawobject_payload`'s doc comment) — assert it's reported as a
        // plain boolean, not a specific pass/fail.
        assert!(result["adl_exemplar"]["domain_parses"].is_boolean());
        assert!(result["adl_exemplar"]["problem_parses"].is_boolean());
    }

    // ── refusal determinism ────────────────────────────────────────────

    #[test]
    fn route_refusal_is_deterministic() {
        let payload = json!({
            "desired_effects": [{"kind": "edited", "file": "f1"}],
            "attention_capacity": 0,
        })
        .to_string();
        let a = route_payload(&payload).expect("route_payload should not hard-error");
        let b = route_payload(&payload).expect("route_payload should not hard-error");
        assert_eq!(a["admitted"], json!(false));
        assert_eq!(a["refusal_reason"], b["refusal_reason"]);
        assert_eq!(a["route_chain"], b["route_chain"]);
        assert!(a["refusal_reason"].as_str().is_some());
    }

    #[test]
    fn solve_refusal_is_deterministic() {
        // Unsatisfiable classical goal: `finish` unconditionally adds
        // `(done)`, but the goal also demands `(never-true)`, which no
        // action in this domain ever adds.
        let domain = r#"
(define (domain test-unsat)
  (:requirements :strips)
  (:predicates (done) (never-true))
  (:action finish
    :parameters ()
    :precondition ()
    :effect (done)))
"#;
        let problem = r#"
(define (problem test-unsat-case)
  (:domain test-unsat)
  (:objects)
  (:init)
  (:goal (and (done) (never-true))))
"#;
        let a = solve_payload_json(domain, problem, "classical");
        let b = solve_payload_json(domain, problem, "classical");
        assert_eq!(a["admitted"], json!(false));
        assert_eq!(a["refusal_reason"], b["refusal_reason"]);
    }

    // ── plan-chain determinism ─────────────────────────────────────────

    #[test]
    fn temporal_plan_chain_is_deterministic() {
        let a = solve_payload_json(TEMPORAL_DOMAIN, TEMPORAL_PROBLEM, "temporal");
        let b = solve_payload_json(TEMPORAL_DOMAIN, TEMPORAL_PROBLEM, "temporal");
        assert_eq!(a["admitted"], json!(true));
        assert_eq!(a["plan_chain"], b["plan_chain"]);
        assert!(a["plan_chain"].as_str().is_some_and(|s| s.len() == 64));
    }

    #[test]
    fn execute_plan_chain_matches_solve_plan_chain() {
        let solved = solve_payload_json(TEMPORAL_DOMAIN, TEMPORAL_PROBLEM, "temporal");
        let payload = json!({
            "domain": TEMPORAL_DOMAIN,
            "problem": TEMPORAL_PROBLEM,
            "case_id": "case-1",
        })
        .to_string();
        let executed = execute_payload(&payload).expect("execute_payload should not hard-error");
        assert_eq!(executed["admitted"], json!(true));
        assert_eq!(executed["plan_chain"], solved["plan_chain"]);
        assert_eq!(executed["receipt"]["goal_reached"], json!(true));
    }

    // ── classical solve golden ──────────────────────────────────────────

    #[test]
    fn classical_solve_finds_single_step_plan() {
        let result = solve_payload_json(CLASSICAL_DOMAIN, CLASSICAL_PROBLEM, "classical");
        assert_eq!(result["admitted"], json!(true));
        assert_eq!(result["plan_len"], json!(1));
    }

    #[test]
    fn analyze_reports_makespan_and_critical_path() {
        let payload = json!({
            "domain": TEMPORAL_DOMAIN,
            "problem": TEMPORAL_PROBLEM,
            "resource_keys": [],
        })
        .to_string();
        let result = analyze_payload(&payload).expect("analyze_payload should not hard-error");
        assert_eq!(result["admitted"], json!(true));
        assert_eq!(result["analysis"]["makespan"], json!(1.0));
        assert_eq!(result["analysis"]["op_count"], json!(1));
    }

    // ── hard-error cases ────────────────────────────────────────────────

    #[test]
    fn empty_payload_is_hard_error() {
        assert!(route_payload("").is_err());
        assert!(solve_payload("").is_err());
        assert!(analyze_payload("").is_err());
        assert!(execute_payload("").is_err());
    }

    #[test]
    fn nonexistent_file_is_hard_error() {
        let payload = json!({
            "domain_file": "/nonexistent/path/does-not-exist.pddl",
            "problem": "(define (problem p) (:domain d) (:objects) (:init) (:goal (done)))",
        })
        .to_string();
        assert!(solve_payload(&payload).is_err());
    }

    #[test]
    fn unparseable_pddl_text_is_hard_error() {
        let payload = json!({
            "domain": "not valid pddl at all",
            "problem": "also not valid pddl",
        })
        .to_string();
        assert!(solve_payload(&payload).is_err());
    }

    #[test]
    fn missing_source_is_hard_error() {
        let payload = json!({ "mode": "classical" }).to_string();
        assert!(solve_payload(&payload).is_err());
    }

    #[test]
    fn split_combined_finds_last_marker() {
        let text = "(define (domain d) (:requirements :strips))\n(define (problem p) (:domain d))";
        let (domain, problem) = ops::split_combined(text).expect("should split");
        assert!(domain.contains("(define (domain d)"));
        assert!(problem.starts_with("(define (problem p)"));
    }

    #[test]
    fn split_combined_errors_without_marker() {
        assert!(ops::split_combined("(define (domain d))").is_err());
    }
}
