//! Revenue Physics end-to-end pipe (Genesis Day 2 — PR-14 / AR-9).
//!
//! This module wires the revenue lanes into one auditable chain that runs
//! entirely in-process over the shared `*_payload` ops functions:
//!
//! 1. **Observe → propose.** A [`RevenueState`] fixture (mixed evidence
//!    flags) is scored by [`praxis_proposer::Proposer`] under a
//!    *domain-authored* objective; the result is a ranked list of proposals,
//!    each carrying a rationale and a blake3 `proposal_hash`. Output is
//!    observation (O), never authority (O*).
//! 2. **Propose → goal.** The top proposal's [`Proposal::pddl_goal`] atom
//!    becomes the planner goal.
//! 3. **Goal → plan.** The observed state projects to a PDDL8 *problem* over
//!    the hand-authored [`REVENUE_PDDL`] *domain* (evidence-gated stage
//!    advances), and `bcinr-pddl`'s classical solver finds an action sequence
//!    reaching the goal.
//! 4. **Plan → admission.** Every plan action is re-derived into an evidence
//!    obligation set and run through `law judge` / `law admit`. The obligation
//!    set is computed from the **same** evidence vocabulary the proposer's
//!    lawfulness pre-filter uses (see [`stage_required_evidence`] and the
//!    [`evidence_gate_agrees`] invariant), so a proposal admission can never
//!    disagree with the pre-filter that emitted it.
//! 5. **Admission → receipt.** A single `law receipt` binds the whole mission,
//!    with the top proposal's `proposal_hash` embedded in the receipt payload
//!    so the blake3 chain hash provably binds back to *which* proposal was
//!    admitted (AR-9 closure).
//!
//! # Why the domain is hand-authored, not manufactured (receipted gap)
//!
//! Genesis Day 2 asks whether the PDDL domain can be manufactured from
//! `ontology/revenue.ttl` via the `mfg` lane. It cannot, and this is the
//! receipt of that refusal: the `mfg` lane ([`crate::mfg`]) consumes the
//! `pddl#` instance vocabulary — indexed `pddl:param`/`pddl:pre`/`pddl:add`/
//! `pddl:del` **atom lists** plus a `pddl:Problem` individual — and lowers that
//! STRIPS8 IR to text. `ontology/revenue.ttl` instead uses the `pdl#`
//! *formula-text* projection style: `pddl:precondition`/`pddl:effect` are opaque
//! PDDL strings, `pddl:parameter` points at Types (not indexed param nodes),
//! and there is no `pddl:Problem` individual at all. The two vocabularies do
//! not overlap, so the `mfg` extractor would find zero actions. Rather than
//! grow a second manufacturing path speculatively, the domain is hand-authored
//! at `ontology/revenue.pddl` (PDDL8-safe: `:strips :typing`, add-only
//! monotone effects, evidence gates as separate action schemas). Its
//! parser/planner round-trip is exercised by [`crate::verbs`]' propose tests
//! and by [`run_demo`] here.

use bcinr_pddl::{domain_from_pddl, problem_from_pddl, GroundProblem};
use praxis_proposer::{
    evidence_permits, Account, ObjectiveFunction, Proposal, Proposer, RevenueState, Stage,
};
use serde_json::{json, Value};

use crate::ops;

/// The hand-authored PDDL8-safe revenue domain (+ an illustrative problem the
/// demo does not use — see [`revenue_domain_text`]).
pub const REVENUE_PDDL: &str = include_str!("../ontology/revenue.pddl");

/// The default domain-authored objective shipped beside the proposer crate.
/// Weights are authored data; the system never invents them (Non-goal 1).
pub const REVENUE_OBJECTIVE: &str =
    include_str!("../crates/praxis-proposer/revenue_objective.json");

// ── Evidence vocabulary: the single seam shared by proposer + admission ─────

/// Evidence type name: legal signed off on the paper. Matches
/// `Account::legal_approved` and the `(legal-approved ?a)` PDDL predicate.
pub const EV_LEGAL: &str = "legal_approved";
/// Evidence type name: security review completed. Matches
/// `Account::security_review_done` and `(security-reviewed ?a)`.
pub const EV_SECURITY: &str = "security_review_done";
/// Evidence type name: an executive sponsor is attached. Matches
/// `Account::exec_sponsor` and `(exec-sponsored ?a)`.
pub const EV_EXEC: &str = "exec_sponsor";

/// The evidence a stage advance into `target` requires, as `law`
/// `EvidenceRequired` obligation types.
///
/// This is the admission-side statement of the same gate the proposer's
/// [`evidence_permits`] pre-filter enforces. Keeping both derived from this
/// one table is what makes the two provably agree ([`evidence_gate_agrees`]).
pub fn stage_required_evidence(target: Stage) -> Vec<&'static str> {
    match target {
        Stage::Lead | Stage::Qualified | Stage::Proposal => vec![],
        Stage::Procurement => vec![EV_LEGAL, EV_SECURITY],
        Stage::ClosedWon => vec![EV_LEGAL, EV_SECURITY, EV_EXEC],
    }
}

/// The evidence an account actually carries, as `law` evidence strings — the
/// exact set the admission gate checks obligations against.
pub fn account_evidence(a: &Account) -> Vec<&'static str> {
    let mut v = Vec::new();
    if a.legal_approved {
        v.push(EV_LEGAL);
    }
    if a.security_review_done {
        v.push(EV_SECURITY);
    }
    if a.exec_sponsor {
        v.push(EV_EXEC);
    }
    v
}

/// The seam invariant: the proposer's lawfulness pre-filter
/// ([`evidence_permits`]) and the admission gate (obligations vs. supplied
/// evidence) agree for `(account, target)`.
///
/// `true` iff both say the same thing. The demo and the integration test
/// assert this holds for every account and every stage, so "the proposal
/// lawfulness pre-filter and admission gate must agree" is a checked property,
/// not a hope.
pub fn evidence_gate_agrees(account: &Account, target: Stage) -> bool {
    let have = account_evidence(account);
    let admission_ok = stage_required_evidence(target)
        .iter()
        .all(|e| have.contains(e));
    admission_ok == evidence_permits(account, target)
}

// ── Stage ↔ PDDL name helpers ───────────────────────────────────────────────

/// Resolve a lower-kebab PDDL stage token (e.g. `closed-won`) back to a
/// [`Stage`]. `None` for an unknown token.
fn stage_from_pddl(name: &str) -> Option<Stage> {
    Stage::ALL.into_iter().find(|s| s.pddl_name() == name)
}

// ── PDDL problem projection from an observed state ──────────────────────────

/// The domain half of [`REVENUE_PDDL`] (everything before the illustrative
/// `(define (problem …)` block), so the demo can pair the shipped, tested
/// domain with a problem projected from *its own* observed state rather than
/// the file's fixed example problem.
pub fn revenue_domain_text() -> String {
    match REVENUE_PDDL.find("(define (problem") {
        Some(idx) => REVENUE_PDDL[..idx].to_string(),
        None => REVENUE_PDDL.to_string(),
    }
}

/// Project an observed [`RevenueState`] and a proposed goal atom into a PDDL8
/// problem over the `revenue-pipeline` domain.
///
/// The static facts (`next`, `gate-free`, `needs-legal-security`,
/// `needs-full-evidence`) mirror the shipped domain's gate structure; each
/// account contributes its current `(stage …)` and one predicate per evidence
/// flag it carries, using the same predicate spellings the domain declares.
pub fn build_problem(state: &RevenueState, goal_atom: &str) -> String {
    let mut s = String::new();
    s.push_str("(define (problem revenue-observed)\n");
    s.push_str("  (:domain revenue-pipeline)\n");
    s.push_str("  (:objects\n");
    for a in &state.accounts {
        s.push_str(&format!("    {} - account\n", a.id));
    }
    s.push_str("    lead qualified proposal procurement closed-won - rstage)\n");
    s.push_str("  (:init\n");
    // Static pipeline order + gate classification (mirrors ontology/revenue.pddl).
    s.push_str("    (next lead qualified)\n");
    s.push_str("    (next qualified proposal)\n");
    s.push_str("    (next proposal procurement)\n");
    s.push_str("    (next procurement closed-won)\n");
    s.push_str("    (gate-free qualified)\n");
    s.push_str("    (gate-free proposal)\n");
    s.push_str("    (needs-legal-security procurement)\n");
    s.push_str("    (needs-full-evidence closed-won)\n");
    for a in &state.accounts {
        s.push_str(&format!("    (stage {} {})\n", a.id, a.stage.pddl_name()));
        if a.legal_approved {
            s.push_str(&format!("    (legal-approved {})\n", a.id));
        }
        if a.security_review_done {
            s.push_str(&format!("    (security-reviewed {})\n", a.id));
        }
        if a.exec_sponsor {
            s.push_str(&format!("    (exec-sponsored {})\n", a.id));
        }
    }
    s.push_str("  )\n");
    s.push_str(&format!("  (:goal {goal_atom}))\n"));
    s
}

// ── Admission bridge: one plan action → a law judge/admit round ─────────────

/// The `(account_id, target_stage)` a revenue plan action moves to, read from
/// its grounded `(stage ?a ?to)` add effect.
fn action_target(action: &bcinr_pddl::Pddl8GroundAction) -> Option<(String, Stage)> {
    let atom = action.add_effects.iter().find(|a| a.pred == "stage")?;
    let account = atom.args.first()?.clone();
    let stage = stage_from_pddl(atom.args.get(1)?)?;
    Some((account, stage))
}

/// Build the `law` payload that gates one plan action: the target stage's
/// required evidence as obligations, checked against the moving account's
/// actual evidence. Both sides come from the shared vocabulary above, so
/// admission agrees with the proposer's pre-filter by construction.
fn admission_payload(label: &str, account: &Account, target: Stage) -> Value {
    let obligations: Vec<Value> = stage_required_evidence(target)
        .iter()
        .map(|e| json!({ "type": "evidence_required", "evidence_type": e }))
        .collect();
    let evidence: Vec<&str> = account_evidence(account);
    json!({
        "value": {
            "action": label,
            "account": account.id,
            "target_stage": target.pddl_name(),
        },
        "obligations": obligations,
        "evidence": evidence,
    })
}

/// Run `law judge` + `law admit` on one plan action and summarize the outcome.
fn admit_action(account: &Account, target: Stage, label: &str) -> Result<Value, String> {
    let payload = admission_payload(label, account, target).to_string();
    let judged = ops::judge_payload(&payload, "default")?;
    let admitted = ops::admit_payload(&payload, "default")?;
    Ok(json!({
        "action": label,
        "account": account.id,
        "target_stage": target.pddl_name(),
        "required_evidence": stage_required_evidence(target),
        "supplied_evidence": account_evidence(account),
        "judge_verdict": judged["verdict"],
        "admit_status": admitted["status"],
    }))
}

/// Look up an account by id in the observed state.
fn account_by_id<'a>(state: &'a RevenueState, id: &str) -> Option<&'a Account> {
    state.accounts.iter().find(|a| a.id == id)
}

// ── The demo fixture ────────────────────────────────────────────────────────

/// A 4-account revenue snapshot with mixed evidence flags, decoupled from the
/// example problem inside `ontology/revenue.pddl` so the demo can drive its
/// own multi-step, evidence-gated plan:
///
/// - `acct-apex` — at `proposal`, full evidence, largest deal: its top-ranked
///   goal (`closed-won`) needs a two-action gated plan
///   (`advance-gated` → `close`).
/// - `acct-legal-gap` — at `qualified`, **missing** `legal_approved`: can never
///   be proposed past `proposal`, and is refused by `admit` if forced further
///   (the negative-path account).
/// - `acct-fresh` — at `lead`, no evidence: only the ungated early moves are
///   lawful.
/// - `acct-closed` — already `closed_won`: terminal, yields no proposals.
pub fn fixture_state() -> Value {
    json!({
        "accounts": [
            {
                "id": "acct-apex", "stage": "proposal", "amount_cents": 5_000_000,
                "security_review_done": true, "legal_approved": true,
                "exec_sponsor": true, "days_in_stage": 20
            },
            {
                "id": "acct-legal-gap", "stage": "qualified", "amount_cents": 3_000_000,
                "security_review_done": true, "legal_approved": false,
                "exec_sponsor": true, "days_in_stage": 40
            },
            {
                "id": "acct-fresh", "stage": "lead", "amount_cents": 1_000_000,
                "security_review_done": false, "legal_approved": false,
                "exec_sponsor": false, "days_in_stage": 90
            },
            {
                "id": "acct-closed", "stage": "closed_won", "amount_cents": 500_000,
                "security_review_done": true, "legal_approved": true,
                "exec_sponsor": true, "days_in_stage": 5
            }
        ]
    })
}

// ── Reusable goal → plan → admission bridge ─────────────────────────────────

/// Project `state` to a PDDL8 problem over the shipped `revenue-pipeline`
/// domain, solve for `goal_atom`, and run every resulting plan action through
/// `law judge` + `law admit`.
///
/// Shared by [`run_demo`] (unscoped fixture) and `revtac::run_mission`
/// (RevTAC-scoped state), so both callers exercise the exact same goal→plan→
/// admission machinery instead of two divergent copies of it.
///
/// A plan action failed `law admit` partway through an otherwise-solvable
/// plan. Carries every admission already computed for the steps that *did*
/// admit successfully, plus the exact failing step's index and label, so the
/// caller never has to re-derive or discard real admission evidence just
/// because a later step in the same plan was refused.
#[derive(Debug, Clone)]
pub struct PlanAdmissionFailure {
    /// Label of the plan action that failed admission.
    pub failed_action: String,
    /// Index into the plan (0-based) at which admission failed.
    pub failed_step_index: usize,
    /// The `admit_status` value `law admit` returned for the failing action.
    pub admit_status: Value,
    /// Admission summaries for every prior step in the plan that *did*
    /// admit successfully, in plan order. Never discarded on failure.
    pub partial_admissions: Vec<Value>,
}

impl std::fmt::Display for PlanAdmissionFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "plan action {} (step {}) was not admitted: {} ({} prior step(s) admitted)",
            self.failed_action,
            self.failed_step_index,
            self.admit_status,
            self.partial_admissions.len()
        )
    }
}

/// Typed failure surface for [`plan_and_admit`]. Replaces a lossy `String`
/// error with a variant that distinguishes "the planner could not reach the
/// goal at all" (`Solve`, no plan ever existed) from "a real plan existed and
/// partially admitted, but one step was refused" (`Admission`, which carries
/// every admission already computed).
#[derive(Debug, Clone)]
pub enum PlanAndAdmitError {
    /// Domain/problem parsing, grounding, or the classical solver itself
    /// failed to produce any plan for the proposed goal.
    Solve(String),
    /// A plan was found and executed, but one action's `law admit` call
    /// returned a non-`"admitted"` status.
    Admission(PlanAdmissionFailure),
}

impl std::fmt::Display for PlanAndAdmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanAndAdmitError::Solve(msg) => write!(f, "{msg}"),
            PlanAndAdmitError::Admission(failure) => write!(f, "{failure}"),
        }
    }
}

impl std::error::Error for PlanAndAdmitError {}

/// Existing call sites (`run_demo`, `revtac::run_mission`) return
/// `Result<_, String>`; this lets `plan_and_admit`'s typed error flow through
/// their `?` operator unchanged while callers that want the typed variant
/// (e.g. tests asserting on `partial_admissions`) call `plan_and_admit`
/// directly and match on `PlanAndAdmitError` before it is stringified.
impl From<PlanAndAdmitError> for String {
    fn from(err: PlanAndAdmitError) -> Self {
        err.to_string()
    }
}

/// Returns the admitted plan's action labels (in order) and the per-action
/// admission summaries. An unsolvable goal is [`PlanAndAdmitError::Solve`];
/// an unadmitted action is [`PlanAndAdmitError::Admission`], which carries
/// every admission computed for the plan's prior, successfully-admitted
/// steps rather than discarding them.
pub fn plan_and_admit(
    state: &RevenueState,
    goal_atom: &str,
) -> Result<(Vec<String>, Vec<Value>), PlanAndAdmitError> {
    let domain_text = revenue_domain_text();
    let problem_text = build_problem(state, goal_atom);
    let domain = domain_from_pddl(&domain_text)
        .map_err(|e| PlanAndAdmitError::Solve(format!("domain parse: {e}")))?;
    let problem = problem_from_pddl(&problem_text)
        .map_err(|e| PlanAndAdmitError::Solve(format!("problem parse: {e}")))?;
    let ground = GroundProblem::build(&domain, &problem, None)
        .map_err(|e| PlanAndAdmitError::Solve(format!("grounding failed: {e}")))?;
    let tape = ground.find_plan().into_result().map_err(|e| {
        PlanAndAdmitError::Solve(format!("no plan reaches proposed goal {goal_atom}: {e}"))
    })?;
    if tape.is_empty() {
        return Err(PlanAndAdmitError::Solve(format!(
            "empty plan for proposed goal {goal_atom}"
        )));
    }

    let mut steps = Vec::with_capacity(tape.len());
    for op in &tape.ops {
        let (acct_id, target) = action_target(&op.action).ok_or_else(|| {
            PlanAndAdmitError::Solve(format!(
                "plan action {} has no (stage ?a ?to) add effect",
                op.action.label
            ))
        })?;
        steps.push((acct_id, target, op.action.label.clone()));
    }
    admit_plan_steps(state, &steps)
}

/// Run `law judge`/`law admit` over an ordered sequence of
/// `(account_id, target_stage, action_label)` plan steps, stopping at the
/// first unadmitted step.
///
/// Split out of [`plan_and_admit`] so the admission-loop invariant this
/// ticket exists to fix — a failure partway through must carry every
/// admission already computed for the steps that *did* admit, never discard
/// it — is directly unit-testable against the real `law judge`/`law admit`
/// pipe, without needing a plan the classical solver can actually produce
/// (the shipped domain's preconditions are, by construction, kept in lock-
/// step with [`stage_required_evidence`] via [`evidence_gate_agrees`], so a
/// solver-found plan can never itself contain an unadmittable step).
fn admit_plan_steps(
    state: &RevenueState,
    steps: &[(String, Stage, String)],
) -> Result<(Vec<String>, Vec<Value>), PlanAndAdmitError> {
    let mut plan_admissions = Vec::with_capacity(steps.len());
    let mut plan_labels = Vec::with_capacity(steps.len());
    for (acct_id, target, label) in steps {
        let account = account_by_id(state, acct_id).ok_or_else(|| {
            PlanAndAdmitError::Solve(format!("plan action moves unknown account {acct_id}"))
        })?;
        let summary = admit_action(account, *target, label).map_err(PlanAndAdmitError::Solve)?;
        if summary["admit_status"] != json!("admitted") {
            return Err(PlanAndAdmitError::Admission(PlanAdmissionFailure {
                failed_action: label.clone(),
                failed_step_index: plan_labels.len(),
                admit_status: summary["admit_status"].clone(),
                partial_admissions: plan_admissions,
            }));
        }
        plan_labels.push(label.clone());
        plan_admissions.push(summary);
    }
    Ok((plan_labels, plan_admissions))
}

// ── The whole pipe ──────────────────────────────────────────────────────────

/// Run the full observation→proposal→plan→admission→receipt pipe over the
/// [`fixture_state`] and the default authored objective, deterministically.
///
/// `ts_ns` fixes the receipt timestamp so the returned `chain_hash` is stable
/// across runs. Returns a transcript `Value` with one section per stage plus
/// the closing receipt; every step's status is recorded so the caller can
/// print or assert on it. A broken seam (unsolvable proposed goal, a plan
/// action that fails to admit, an evidence-gate disagreement) is a hard
/// `Err` — the pipe is only "real" if it stays green end to end.
pub fn run_demo(ts_ns: u64) -> Result<Value, String> {
    // 1. Observe → propose.
    let state: RevenueState = serde_json::from_value(fixture_state())
        .map_err(|e| format!("fixture state is not a RevenueState: {e}"))?;
    let objective = ObjectiveFunction::from_json_str(REVENUE_OBJECTIVE)
        .map_err(|e| format!("authored objective failed to load: {e}"))?;
    let proposer = Proposer::new(objective.clone());
    let proposals = proposer.propose(&state);
    if proposals.is_empty() {
        return Err("no lawful proposals for the fixture state".to_string());
    }

    // Seam invariant: pre-filter and admission gate agree everywhere.
    for account in &state.accounts {
        for target in Stage::ALL {
            if !evidence_gate_agrees(account, target) {
                return Err(format!(
                    "evidence gate disagreement: {} -> {}",
                    account.id,
                    target.pddl_name()
                ));
            }
        }
    }

    let proposals_json: Vec<Value> = proposals
        .iter()
        .map(|p| {
            json!({
                "goal": p.pddl_goal(),
                "goal_description": p.goal_description,
                "target_account": p.target_account,
                "target_stage": p.target_stage.pddl_name(),
                "score": p.score,
                "proposal_hash": p.proposal_hash,
                "rationale": p.rationale,
            })
        })
        .collect();

    // 2. Propose → goal (top-ranked).
    let top: &Proposal = &proposals[0];
    let goal_atom = top.pddl_goal();

    // 3. Goal → plan → admission (project state to a problem, solve over the
    //    shipped domain, then run every action through judge + admit).
    let (plan_labels, plan_admissions) = plan_and_admit(&state, &goal_atom)?;

    // 5. Admission → receipt binding the proposal_hash (AR-9 closure).
    let receipt_value = json!({
        "mission": "revenue-physics-day2",
        "proposal_hash": top.proposal_hash,
        "goal": goal_atom,
        "target_account": top.target_account,
        "target_stage": top.target_stage.pddl_name(),
        "admitted_plan": plan_labels,
        "objective": { "name": objective.name, "version": objective.version },
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
        "mission": "revenue-physics-day2",
        "ts_ns": ts_ns,
        "objective": { "name": objective.name, "version": objective.version },
        "step_1_proposals": {
            "count": proposals.len(),
            "proposals": proposals_json,
        },
        "step_2_top_goal": {
            "goal": goal_atom,
            "proposal_hash": top.proposal_hash,
            "target_account": top.target_account,
            "target_stage": top.target_stage.pddl_name(),
            "score": top.score,
        },
        "step_3_plan": {
            "domain": "revenue-pipeline (ontology/revenue.pddl, hand-authored)",
            "plan_len": plan_labels.len(),
            "plan": plan_labels,
        },
        "step_4_admissions": plan_admissions,
        "step_5_receipt": {
            "chain_hash": receipt["chain_hash"],
            "payload_hash": receipt["payload_hash"],
            "prev_chain_hash": receipt["prev_chain_hash"],
            "binds_proposal_hash": top.proposal_hash,
        },
        "chain_hash": receipt["chain_hash"],
    }))
}

/// Force an admission for advancing `account` into `target` even though the
/// proposer would never propose it that far, and return the `law admit`
/// result. Used by the negative-path assertion: an account missing
/// `legal_approved` must be **refused** here, matching its exclusion from the
/// proposal set.
pub fn forced_admit(account: &Account, target: Stage) -> Result<Value, String> {
    let payload = admission_payload("forced-advance", account, target).to_string();
    ops::admit_payload(&payload, "default")
}

/// JSON-only entry point for the negative path: force an admission for the
/// [`fixture_state`] account `account_id` into the stage named by the
/// lower-kebab `target_pddl` token (e.g. `procurement`), and return the
/// `law admit` result. Lets callers (e.g. the integration test) exercise the
/// forced-refusal seam without naming the proposer's Rust types.
pub fn forced_admit_by_id(account_id: &str, target_pddl: &str) -> Result<Value, String> {
    let state: RevenueState = serde_json::from_value(fixture_state())
        .map_err(|e| format!("fixture state is not a RevenueState: {e}"))?;
    let account = account_by_id(&state, account_id)
        .ok_or_else(|| format!("no fixture account {account_id}"))?;
    let target =
        stage_from_pddl(target_pddl).ok_or_else(|| format!("unknown stage {target_pddl}"))?;
    forced_admit(account, target)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> Result<RevenueState, crate::AppError> {
        Ok(serde_json::from_value(fixture_state())?)
    }

    #[test]
    fn evidence_gate_agrees_for_every_account_and_stage() -> Result<(), crate::AppError> {
        for account in &state()?.accounts {
            for target in Stage::ALL {
                assert!(
                    evidence_gate_agrees(account, target),
                    "disagreement for {} -> {}",
                    account.id,
                    target.pddl_name()
                );
            }
        }
        Ok(())
    }

    /// Under `--features law-signed`, the receipt step fails closed without a
    /// signing key; set a fixed test key once so this unit test stays green in
    /// that build too. Signing does not affect the transcript's `chain_hash`.
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

    /// The seam this ticket exists to fix: when a plan's steps are run
    /// through `law judge`/`law admit` in order and a later step is refused,
    /// every admission already computed for the prior, successfully-admitted
    /// steps must survive on the error — never be silently discarded.
    ///
    /// Drives the real `admit_plan_steps` helper (the same one
    /// `plan_and_admit` calls after solving) over a hand-built two-step
    /// sequence rather than a solver-found plan: the shipped domain's
    /// preconditions are kept in lock-step with `stage_required_evidence` via
    /// `evidence_gate_agrees` (see the test above), so a plan the classical
    /// solver actually finds can never itself contain an unadmittable step.
    /// The second step here uses `acct-legal-gap`, whose fixture evidence
    /// deliberately fails `procurement`'s gate — a real, non-mocked
    /// `law judge`/`law admit` refusal, run through the same admission
    /// machinery `plan_and_admit` uses.
    #[test]
    fn admit_plan_steps_preserves_partial_admissions_on_failure() -> Result<(), crate::AppError> {
        let s = state()?;
        let steps = vec![
            (
                "acct-apex".to_string(),
                Stage::Proposal,
                "step-0-admits".to_string(),
            ),
            (
                "acct-legal-gap".to_string(),
                Stage::Procurement,
                "step-1-refused".to_string(),
            ),
        ];

        let err = admit_plan_steps(&s, &steps)
            .expect_err("acct-legal-gap must fail procurement's evidence gate");
        let PlanAndAdmitError::Admission(failure) = err else {
            return Err(crate::AppError::Other(format!(
                "expected PlanAndAdmitError::Admission, got a Solve error: {err}"
            )));
        };

        assert_eq!(failure.failed_action, "step-1-refused");
        assert_eq!(failure.failed_step_index, 1);
        assert_ne!(failure.admit_status, json!("admitted"));
        assert_eq!(
            failure.partial_admissions.len(),
            1,
            "step 0's real admission must not be discarded when step 1 fails"
        );
        assert_eq!(
            failure.partial_admissions[0]["action"],
            json!("step-0-admits")
        );
        assert_eq!(
            failure.partial_admissions[0]["admit_status"],
            json!("admitted")
        );
        Ok(())
    }

    #[test]
    fn demo_runs_green_and_is_deterministic() -> Result<(), crate::AppError> {
        ensure_signing_key();
        let a = run_demo(1_000).map_err(|e| crate::AppError::Other(e.to_string()))?;
        let b = run_demo(1_000).map_err(|e| crate::AppError::Other(e.to_string()))?;
        assert_eq!(a, b, "same ts_ns must yield a byte-identical transcript");
        assert_eq!(
            a["step_3_plan"]["plan_len"]
                .as_u64()
                .ok_or_else(|| crate::AppError::Other("missing val".into()))?,
            2
        );
        Ok(())
    }
}
