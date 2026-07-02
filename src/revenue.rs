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
//! `pddl#` instance vocabulary — indexed `pdl:param`/`pdl:pre`/`pdl:add`/
//! `pdl:del` **atom lists** plus a `pdl:Problem` individual — and lowers that
//! STRIPS8 IR to text. `ontology/revenue.ttl` instead uses the `pdl#`
//! *formula-text* projection style: `pdl:precondition`/`pdl:effect` are opaque
//! PDDL strings, `pdl:parameter` points at Types (not indexed param nodes),
//! and there is no `pdl:Problem` individual at all. The two vocabularies do
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
    let admission_ok = stage_required_evidence(target).iter().all(|e| have.contains(e));
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

    // 3. Goal → plan (project state to a problem, solve over the shipped domain).
    let domain_text = revenue_domain_text();
    let problem_text = build_problem(&state, &goal_atom);
    let domain = domain_from_pddl(&domain_text).map_err(|e| format!("domain parse: {e}"))?;
    let problem = problem_from_pddl(&problem_text).map_err(|e| format!("problem parse: {e}"))?;
    let ground = GroundProblem::build(&domain, &problem, None)
        .map_err(|e| format!("grounding failed: {e}"))?;
    let tape = ground
        .find_plan()
        .map_err(|e| format!("no plan reaches proposed goal {goal_atom}: {e}"))?;
    if tape.is_empty() {
        return Err(format!("empty plan for proposed goal {goal_atom}"));
    }

    // 4. Plan → admission: every action passes judge + admit.
    let mut plan_admissions = Vec::with_capacity(tape.len());
    let mut plan_labels = Vec::with_capacity(tape.len());
    for op in &tape.ops {
        let (acct_id, target) = action_target(&op.action).ok_or_else(|| {
            format!("plan action {} has no (stage ?a ?to) add effect", op.action.label)
        })?;
        let account = account_by_id(&state, &acct_id)
            .ok_or_else(|| format!("plan action moves unknown account {acct_id}"))?;
        let summary = admit_action(account, target, &op.action.label)?;
        if summary["admit_status"] != json!("admitted") {
            return Err(format!(
                "plan action {} was not admitted: {}",
                op.action.label, summary["admit_status"]
            ));
        }
        plan_labels.push(op.action.label.clone());
        plan_admissions.push(summary);
    }

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
        return Err(format!("mission receipt was not issued: {}", receipt["status"]));
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
            "plan_len": tape.len(),
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

    fn state() -> RevenueState {
        serde_json::from_value(fixture_state()).expect("fixture is a RevenueState")
    }

    #[test]
    fn evidence_gate_agrees_for_every_account_and_stage() {
        for account in &state().accounts {
            for target in Stage::ALL {
                assert!(
                    evidence_gate_agrees(account, target),
                    "disagreement for {} -> {}",
                    account.id,
                    target.pddl_name()
                );
            }
        }
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

    #[test]
    fn demo_runs_green_and_is_deterministic() {
        ensure_signing_key();
        let a = run_demo(1_000).expect("pipe should run green");
        let b = run_demo(1_000).expect("pipe should run green");
        assert_eq!(a, b, "same ts_ns must yield a byte-identical transcript");
        assert_eq!(a["step_3_plan"]["plan_len"].as_u64().unwrap(), 2);
    }
}
