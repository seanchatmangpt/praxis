//! Mission Physics — one mission language above the substrate, for every pack.
//!
//! Genesis Day 6 phase 2 proves the doctrine structurally: **two institutions
//! run on one substrate.** The revenue pipeline and the church-operations
//! pipeline are not two programs that resemble each other — they are the
//! *same* generic function ([`run_pipeline`]) instantiated at two
//! [`Pack`]s. The proposer, the scorer, the ranker, the hasher, the planner
//! adapter, the admission gate, and the receipt chain are authored **once,
//! here and in `praxis-proposer::engine`**, and reused verbatim. Only three
//! inputs differ between a revenue mission and a church mission:
//!
//! 1. the **ontology** — which stages/evidence exist and which moves are
//!    lawful (a [`Pack`] impl over a `praxis_proposer::engine::Domain`);
//! 2. the **authored objective** — the weights the domain owner wrote
//!    (Non-goal 1: the system never invents them); and
//! 3. the **observed state** — the snapshot fed in.
//!
//! Everything else — the *code path* — is identical. `tests/two_domains.rs`
//! makes that literal: one loop, two packs, the same substrate functions.
//!
//! # The mission language (RevTAC generalized)
//!
//! `mission run --pack <revenue|church> --objective <path> --state <path>`
//! compiles a mission down to the substrate invocation:
//! observe → propose → goal → `plan solve` → `law judge`/`law admit` →
//! `law receipt`. `mission ceiling --pack <p> --state <path>` computes the
//! pack's **Maximum Reachable objective** — MRR generalized: the ceiling of
//! realized mission value lawfully reachable under the pack's evidence gates
//! (see [`ceiling`]).
//!
//! # Boundary position (AR-9) — unchanged across packs
//!
//! Every proposal a mission emits is an observation (O), not authority (O\*):
//! it must pass `law judge`/`law admit` before any effect, and the receipt
//! binds back to the admitted proposal's `proposal_hash` so "which proposal
//! was admitted" stays provable — in every pack.

use bcinr_pddl::{domain_from_pddl, problem_from_pddl, GroundProblem, Pddl8GroundAction};
use praxis_proposer::{
    church::{self, ChurchDomain, ChurchState, Person},
    engine::{Domain, Proposal, Proposer},
    Account, ObjectiveFunction, RevenueDomain, RevenueState,
};
use serde_json::{json, Value};

use crate::{ops, revenue};

/// The default authored church objective shipped beside the proposer crate.
/// Weights are authored data; the system never invents them (Non-goal 1).
pub const CHURCH_OBJECTIVE: &str = include_str!("../crates/praxis-proposer/church_objective.json");

/// The hand-authored PDDL8-safe church-operations planning domain (+ an
/// illustrative problem the pipe does not use — see [`church_domain_text`]).
pub const CHURCH_PDDL: &str = include_str!("../ontology/church.pddl");

// ─────────────────────────────────────────────────────────────────────────
// The `Pack` trait: the *entire* per-institution surface above the proposer.
// ─────────────────────────────────────────────────────────────────────────

/// A mission pack: a `praxis_proposer::engine::Domain` (ontology + objective
/// vocabulary + lawfulness gate) extended with the planning and admission
/// surface the shared pipeline needs. Implementing this trait — plus authoring
/// an objective JSON and a PDDL8 domain — is the *whole* cost of running a new
/// institution on the substrate. No proposer/planner/admission/receipt code is
/// written per pack; [`run_pipeline`] and [`ceiling`] supply all of it.
pub trait Pack: Domain {
    /// Every evidence-flag name this pack recognizes (stable snake_case),
    /// used for missing-evidence attribution in [`ceiling`].
    fn evidence_vocabulary() -> &'static [&'static str];

    /// The subset of [`Domain::fluent_names`] that represents *realized
    /// mission value* — revenue realized, people connected + cared for. These
    /// are summed (unweighted, objective-independent physics) to form the
    /// Maximum Reachable objective in [`ceiling`]. Cost/process fluents
    /// (`time_penalty`, `volunteer_capacity_used`, follow-up timeliness) are
    /// deliberately excluded: the ceiling is a bound on value, not a score.
    fn ceiling_fluents() -> &'static [&'static str];

    /// All stages in pipeline order (for reverse PDDL-name lookup and the
    /// gate-agreement sweep).
    fn all_stages() -> &'static [Self::Stage];

    /// The hand-authored PDDL8 domain text (domain half only, no problem
    /// block), paired with a problem projected from the observed state.
    fn pddl_domain_text() -> String;

    /// Project an observed state and a proposed goal atom into a PDDL8 problem
    /// over this pack's domain.
    fn build_problem(state: &Self::State, goal_atom: &str) -> String;

    /// The evidence a stage advance into `target` requires, as `law`
    /// `EvidenceRequired` obligation types. This is the *admission-side*
    /// statement of the same gate [`Domain::lawful_targets`] enforces in the
    /// proposer; [`evidence_gate_agrees`] proves the two never disagree.
    fn stage_required_evidence(target: Self::Stage) -> Vec<&'static str>;

    /// The evidence a given entity actually carries, as `law` evidence
    /// strings — the exact set the admission gate checks obligations against.
    fn entity_evidence(entity: &Self::Entity) -> Vec<&'static str>;

    /// The proposer's evidence pre-filter for `(entity, target)` — the
    /// lawfulness gate the proposer crate authored. Kept alongside
    /// [`Pack::stage_required_evidence`] so [`evidence_gate_agrees`] can prove
    /// the two independent codepaths agree.
    fn evidence_permits(entity: &Self::Entity, target: Self::Stage) -> bool;

    // ── Provided helpers (identical for every pack) ───────────────────────

    /// Evidence flags this entity is missing (vocabulary minus carried).
    fn entity_missing_evidence(entity: &Self::Entity) -> Vec<String> {
        let have = Self::entity_evidence(entity);
        Self::evidence_vocabulary()
            .iter()
            .filter(|e| !have.contains(*e))
            .map(|e| (*e).to_string())
            .collect()
    }

    /// Reverse lookup: a lower-kebab PDDL stage token back to a stage.
    fn stage_from_pddl(name: &str) -> Option<Self::Stage> {
        Self::all_stages()
            .iter()
            .copied()
            .find(|s| Self::stage_pddl_name(*s) == name)
    }

    /// Look up an entity by id in the observed state.
    fn entity_by_id<'a>(state: &'a Self::State, id: &str) -> Option<&'a Self::Entity> {
        Self::entities(state)
            .iter()
            .find(|e| Self::entity_id(e) == id)
    }

    /// Load + validate an authored objective from JSON text against *this*
    /// pack's fluent vocabulary. The loader, the `deny_unknown_fields`
    /// discipline, and the finite-weight rule are identical across packs;
    /// only the allowed fluent set changes.
    fn load_objective(text: &str) -> Result<ObjectiveFunction, String> {
        ObjectiveFunction::from_json_str_for(text, Self::fluent_names()).map_err(|e| e.to_string())
    }

    /// Load + validate an authored objective from a JSON file on disk.
    fn load_objective_path(path: &std::path::Path) -> Result<ObjectiveFunction, String> {
        ObjectiveFunction::from_path_for(path, Self::fluent_names()).map_err(|e| e.to_string())
    }
}

// ─────────────────────────────────────────────────────────────────────────
// The one seam invariant, generic: proposer pre-filter == admission gate.
// ─────────────────────────────────────────────────────────────────────────

/// The seam invariant, for **any** pack: the proposer's lawfulness pre-filter
/// ([`Pack::evidence_permits`], authored in the proposer crate) and the
/// admission gate ([`Pack::stage_required_evidence`] vs
/// [`Pack::entity_evidence`], authored here) agree for `(entity, target)`.
///
/// `true` iff both say the same thing. [`run_pipeline`] asserts this holds for
/// every entity and every stage before it trusts a single proposal — so "the
/// proposal pre-filter and the admission gate must agree" is a checked
/// property in every institution, not a hope.
pub fn evidence_gate_agrees<P: Pack>(entity: &P::Entity, target: P::Stage) -> bool {
    let have = P::entity_evidence(entity);
    let admission_ok = P::stage_required_evidence(target)
        .iter()
        .all(|e| have.contains(e));
    admission_ok == P::evidence_permits(entity, target)
}

// ─────────────────────────────────────────────────────────────────────────
// The one pipeline, generic. propose → goal → plan → admit → receipt.
// ─────────────────────────────────────────────────────────────────────────

/// One plan action's `(entity_id, target_stage)`, read from its grounded
/// `(stage ?e ?to)` add effect. Generic over the pack's goal predicate and
/// stage vocabulary — the same reader serves every institution.
fn action_target<P: Pack>(action: &Pddl8GroundAction) -> Option<(String, P::Stage)> {
    let atom = action
        .add_effects
        .iter()
        .find(|a| a.pred == P::goal_predicate())?;
    let id = atom.args.first()?.clone();
    let stage = P::stage_from_pddl(atom.args.get(1)?)?;
    Some((id, stage))
}

/// Run `law judge` + `law admit` on one plan action and summarize the outcome.
/// Both the obligations and the supplied evidence come from the pack's shared
/// vocabulary, so admission agrees with the proposer pre-filter by
/// construction. This calls the **same** `ops::*_payload` functions in every
/// pack — the admission mechanism is not re-implemented per institution.
fn admit_action<P: Pack>(
    entity: &P::Entity,
    target: P::Stage,
    label: &str,
) -> Result<Value, String> {
    let required = P::stage_required_evidence(target);
    let obligations: Vec<Value> = required
        .iter()
        .map(|e| json!({ "type": "evidence_required", "evidence_type": e }))
        .collect();
    let supplied = P::entity_evidence(entity);
    let payload = json!({
        "value": {
            "action": label,
            "entity": P::entity_id(entity),
            "target_stage": P::stage_pddl_name(target),
        },
        "obligations": obligations,
        "evidence": supplied,
    })
    .to_string();
    let judged = ops::judge_payload(&payload, "default")?;
    let admitted = ops::admit_payload(&payload, "default")?;
    Ok(json!({
        "action": label,
        "entity": P::entity_id(entity),
        "target_stage": P::stage_pddl_name(target),
        "required_evidence": required,
        "supplied_evidence": supplied,
        "judge_verdict": judged["verdict"],
        "admit_status": admitted["status"],
    }))
}

/// Force an admission for advancing `entity` into `target` through the **same**
/// `ops::judge_payload`/`ops::admit_payload` gate the pipeline uses — the
/// negative-path probe. An entity that lacks the evidence `target` requires
/// must be *denied* here, proving the evidence gate is enforced by admission
/// (not merely by the proposer's pre-filter), identically in every pack. This
/// is the pack-independent generalization of `revenue::forced_admit`.
pub fn admit_advance<P: Pack>(entity: &P::Entity, target: P::Stage) -> Result<Value, String> {
    admit_action::<P>(entity, target, "forced-advance")
}

/// A ranked [`Proposal`] as JSON, with its `pddl_goal` atom attached. Built by
/// hand because the generic proposal's stage type is pack-specific (not
/// `Serialize`); one shape serves every pack.
fn proposal_json<P: Pack>(p: &Proposal<P>) -> Value {
    json!({
        "goal_description": p.goal_description,
        "target_id": p.target_id,
        "target_stage": P::stage_pddl_name(p.target_stage),
        "score": p.score,
        "proposal_hash": p.proposal_hash,
        "rationale": p.rationale,
        "pddl_goal": p.pddl_goal(),
    })
}

/// Run the full observation→proposal→plan→admission→receipt pipe for **any**
/// pack, deterministically under a fixed `ts_ns`.
///
/// This is the domain-independence proof in code: the body names no
/// institution. It calls `Proposer::<P>::propose`, the shared `plan solve`
/// path (`bcinr_pddl`), the shared `ops::judge_payload`/`ops::admit_payload`
/// admission gate, and the shared `ops::receipt_payload` chain — the *same*
/// functions for revenue and church. Only `P`, `objective`, and `state`
/// differ.
///
/// A broken seam — no lawful proposal, an unsolvable goal, a plan action that
/// fails to admit, or an evidence-gate disagreement — is a hard `Err`: the
/// pipe is only "real" if it stays green end to end.
pub fn run_pipeline<P: Pack>(
    state: &P::State,
    objective: &ObjectiveFunction,
    mission_name: &str,
    ts_ns: u64,
) -> Result<Value, String> {
    // 1. Observe → propose (the generic engine, specialized to P).
    let proposer = Proposer::<P>::new(objective.clone());
    let proposals = proposer.propose(state);
    if proposals.is_empty() {
        return Err(format!(
            "no lawful proposals for the {} state",
            P::pack_name()
        ));
    }

    // Seam invariant: proposer pre-filter and admission gate agree everywhere.
    for entity in P::entities(state) {
        for &target in P::all_stages() {
            if !evidence_gate_agrees::<P>(entity, target) {
                return Err(format!(
                    "evidence gate disagreement in pack {}: {} -> {}",
                    P::pack_name(),
                    P::entity_id(entity),
                    P::stage_pddl_name(target)
                ));
            }
        }
    }

    // 2. Propose → goal (top-ranked).
    let top = &proposals[0];
    let goal_atom = top.pddl_goal();

    // 3. Goal → plan (project state to a problem, solve over the shipped domain).
    let domain = domain_from_pddl(&P::pddl_domain_text())
        .map_err(|e| format!("domain parse ({}): {e}", P::pack_name()))?;
    let problem = problem_from_pddl(&P::build_problem(state, &goal_atom))
        .map_err(|e| format!("problem parse ({}): {e}", P::pack_name()))?;
    let ground = GroundProblem::build(&domain, &problem, None)
        .map_err(|e| format!("grounding failed: {e}"))?;
    let tape = ground
        .find_plan()
        .map_err(|e| format!("no plan reaches proposed goal {goal_atom}: {e}"))?;
    if tape.is_empty() {
        return Err(format!("empty plan for proposed goal {goal_atom}"));
    }

    // 4. Plan → admission: every action passes the SAME judge + admit gate.
    let mut plan_labels = Vec::with_capacity(tape.len());
    let mut plan_admissions = Vec::with_capacity(tape.len());
    for op in &tape.ops {
        let (id, target) = action_target::<P>(&op.action).ok_or_else(|| {
            format!(
                "plan action {} has no (stage ?e ?to) add effect",
                op.action.label
            )
        })?;
        let entity = P::entity_by_id(state, &id)
            .ok_or_else(|| format!("plan action moves unknown entity {id}"))?;
        let summary = admit_action::<P>(entity, target, &op.action.label)?;
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
        "mission": mission_name,
        "pack": P::pack_name(),
        "proposal_hash": top.proposal_hash,
        "goal": goal_atom,
        "target_id": top.target_id,
        "target_stage": P::stage_pddl_name(top.target_stage),
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
        "mission": mission_name,
        "pack": P::pack_name(),
        "ts_ns": ts_ns,
        "objective": { "name": objective.name, "version": objective.version },
        "step_1_proposals": {
            "count": proposals.len(),
            "proposals": proposals.iter().map(proposal_json::<P>).collect::<Vec<_>>(),
        },
        "step_2_top_goal": {
            "goal": goal_atom,
            "proposal_hash": top.proposal_hash,
            "target_id": top.target_id,
            "target_stage": P::stage_pddl_name(top.target_stage),
            "score": top.score,
        },
        "step_3_plan": {
            "domain": P::pack_name(),
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

// ─────────────────────────────────────────────────────────────────────────
// Maximum Reachable objective — MRR generalized to any pack.
// ─────────────────────────────────────────────────────────────────────────

/// The realized mission value of `entity` *at* `stage`: the unweighted sum of
/// the pack's [`Pack::ceiling_fluents`] evaluated for the move into `stage`.
/// Objective-independent physics — the authored weights change ranking, never
/// the ceiling (exactly the property `praxis_proposer::mrr` documents for
/// revenue).
fn entity_value<P: Pack>(entity: &P::Entity, stage: P::Stage) -> f64 {
    let names = P::fluent_names();
    let fluents = P::compute_fluents(entity, stage);
    P::ceiling_fluents()
        .iter()
        .map(|cf| {
            names
                .iter()
                .position(|n| n == cf)
                .map(|i| fluents[i])
                .unwrap_or(0.0)
        })
        .sum()
}

/// Compute the pack's **Maximum Reachable objective** for an observed state:
/// for each entity, the best realized mission value reachable over its current
/// stage plus its lawful forward targets (which already respect the evidence
/// gates), summed across entities.
///
/// Generalizes Maximum Reachable Revenue (`praxis_proposer::mrr`): with
/// `ceiling_fluents = ["realized_revenue"]` this reproduces MRR's headline
/// numbers exactly (proven in `tests/two_domains.rs`); with the church value
/// fluents it is the ceiling of people connected + cared for the welcome team
/// can lawfully reach. Objective-independent, and it respects evidence gates
/// in every pack because it only maximizes over [`Domain::lawful_targets`].
pub fn ceiling<P: Pack>(state: &P::State) -> Value {
    let entities = P::entities(state);
    let mut total_max = 0.0f64;
    let mut total_realized = 0.0f64;
    let mut rows = Vec::with_capacity(entities.len());

    for e in entities {
        let current = P::entity_stage(e);
        let realized = entity_value::<P>(e, current);
        let mut best = realized;
        for &t in &P::lawful_targets(e) {
            let v = entity_value::<P>(e, t);
            if v > best {
                best = v;
            }
        }
        total_max += best;
        total_realized += realized;
        // An entity that can realize nothing is attributed to its missing
        // evidence (why it is gated out), mirroring MRR's `blocked_on`.
        let blocked_on = if best <= 0.0 {
            Some(P::entity_missing_evidence(e))
        } else {
            None
        };
        rows.push(json!({
            "id": P::entity_id(e),
            "max_reachable_value": best,
            "already_realized_value": realized,
            "opportunity_value": best - realized,
            "blocked_on": blocked_on,
        }));
    }

    let utilization = if total_max == 0.0 {
        0.0
    } else {
        total_realized / total_max
    };

    json!({
        "status": "computed",
        "pack": P::pack_name(),
        "ceiling_fluents": P::ceiling_fluents(),
        "max_reachable_value": total_max,
        "already_realized_value": total_realized,
        "opportunity_value": total_max - total_realized,
        "utilization": utilization,
        "entities_considered": entities.len(),
        "entities": rows,
    })
}

// ─────────────────────────────────────────────────────────────────────────
// Pack impl: revenue. Reuses every helper already authored in `revenue.rs`.
// ─────────────────────────────────────────────────────────────────────────

impl Pack for RevenueDomain {
    fn evidence_vocabulary() -> &'static [&'static str] {
        &[revenue::EV_LEGAL, revenue::EV_SECURITY, revenue::EV_EXEC]
    }

    fn ceiling_fluents() -> &'static [&'static str] {
        &["realized_revenue"]
    }

    fn all_stages() -> &'static [Self::Stage] {
        &praxis_proposer::Stage::ALL
    }

    fn pddl_domain_text() -> String {
        revenue::revenue_domain_text()
    }

    fn build_problem(state: &RevenueState, goal_atom: &str) -> String {
        revenue::build_problem(state, goal_atom)
    }

    fn stage_required_evidence(target: Self::Stage) -> Vec<&'static str> {
        revenue::stage_required_evidence(target)
    }

    fn entity_evidence(entity: &Account) -> Vec<&'static str> {
        revenue::account_evidence(entity)
    }

    fn evidence_permits(entity: &Account, target: Self::Stage) -> bool {
        praxis_proposer::evidence_permits(entity, target)
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Pack impl: church. The parallel glue — different ontology, same substrate.
// ─────────────────────────────────────────────────────────────────────────

/// Evidence type names for the church pack, matching `Person` field names and
/// the church PDDL predicates (kebab in PDDL, snake here).
const CH_WELCOMED: &str = "welcomed";
const CH_FOLLOWED_UP: &str = "followed_up";
const CH_IN_GROUP: &str = "in_small_group";
const CH_CARE: &str = "care_assigned";

/// The domain half of [`CHURCH_PDDL`] (everything before the illustrative
/// problem block), so the pipe pairs the shipped, tested domain with a problem
/// projected from its own observed state. Mirrors [`revenue::revenue_domain_text`].
pub fn church_domain_text() -> String {
    match CHURCH_PDDL.find("(define (problem") {
        Some(idx) => CHURCH_PDDL[..idx].to_string(),
        None => CHURCH_PDDL.to_string(),
    }
}

/// The evidence a church stage advance into `target` requires. The
/// admission-side statement of the same gate `church::evidence_permits`
/// enforces — kept in lockstep so [`evidence_gate_agrees`] holds.
pub fn church_stage_required_evidence(target: church::Stage) -> Vec<&'static str> {
    use church::Stage;
    match target {
        Stage::FirstTime | Stage::Returning => vec![],
        Stage::Connected => vec![CH_WELCOMED, CH_FOLLOWED_UP],
        Stage::Serving => vec![CH_WELCOMED, CH_FOLLOWED_UP, CH_IN_GROUP],
        Stage::Leading => vec![CH_WELCOMED, CH_FOLLOWED_UP, CH_IN_GROUP, CH_CARE],
    }
}

/// The evidence a person actually carries, as `law` evidence strings.
pub fn church_person_evidence(p: &Person) -> Vec<&'static str> {
    let mut v = Vec::new();
    if p.welcomed {
        v.push(CH_WELCOMED);
    }
    if p.followed_up {
        v.push(CH_FOLLOWED_UP);
    }
    if p.in_small_group {
        v.push(CH_IN_GROUP);
    }
    if p.care_assigned {
        v.push(CH_CARE);
    }
    v
}

/// Project an observed [`ChurchState`] and a proposed goal atom into a PDDL8
/// problem over the `church-operations` domain. The exact parallel of
/// [`revenue::build_problem`]: static stage order + per-tier gate
/// classification, then each person's current `(stage …)` and one predicate
/// per hospitality act they carry, using the domain's own kebab spellings.
pub fn build_church_problem(state: &ChurchState, goal_atom: &str) -> String {
    let mut s = String::new();
    s.push_str("(define (problem church-observed)\n");
    s.push_str("  (:domain church-operations)\n");
    s.push_str("  (:objects\n");
    for p in &state.people {
        s.push_str(&format!("    {} - person\n", p.id));
    }
    s.push_str("    first-time returning connected serving leading - cstage)\n");
    s.push_str("  (:init\n");
    // Static assimilation order + per-tier gate classification
    // (mirrors ontology/church.pddl).
    s.push_str("    (next first-time returning)\n");
    s.push_str("    (next returning connected)\n");
    s.push_str("    (next connected serving)\n");
    s.push_str("    (next serving leading)\n");
    s.push_str("    (gate-free returning)\n");
    s.push_str("    (needs-followup connected)\n");
    s.push_str("    (needs-group serving)\n");
    s.push_str("    (needs-care leading)\n");
    for p in &state.people {
        s.push_str(&format!("    (stage {} {})\n", p.id, p.stage.pddl_name()));
        if p.welcomed {
            s.push_str(&format!("    (welcomed {})\n", p.id));
        }
        if p.followed_up {
            s.push_str(&format!("    (followed-up {})\n", p.id));
        }
        if p.in_small_group {
            s.push_str(&format!("    (in-small-group {})\n", p.id));
        }
        if p.care_assigned {
            s.push_str(&format!("    (care-assigned {})\n", p.id));
        }
    }
    s.push_str("  )\n");
    s.push_str(&format!("  (:goal {goal_atom}))\n"));
    s
}

impl Pack for ChurchDomain {
    fn evidence_vocabulary() -> &'static [&'static str] {
        &[CH_WELCOMED, CH_FOLLOWED_UP, CH_IN_GROUP, CH_CARE]
    }

    fn ceiling_fluents() -> &'static [&'static str] {
        // Realized mission value: how deeply a person is connected, and whether
        // their care need is met. Not volunteer capacity (a cost) or follow-up
        // timeliness (a process metric) — the ceiling bounds value, not score.
        &["people_connected", "care_completion_rate"]
    }

    fn all_stages() -> &'static [Self::Stage] {
        &church::Stage::ALL
    }

    fn pddl_domain_text() -> String {
        church_domain_text()
    }

    fn build_problem(state: &ChurchState, goal_atom: &str) -> String {
        build_church_problem(state, goal_atom)
    }

    fn stage_required_evidence(target: Self::Stage) -> Vec<&'static str> {
        church_stage_required_evidence(target)
    }

    fn entity_evidence(entity: &Person) -> Vec<&'static str> {
        church_person_evidence(entity)
    }

    fn evidence_permits(entity: &Person, target: Self::Stage) -> bool {
        church::evidence_permits(entity, target)
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Shared fixtures (used by the mission verb demo and tests/two_domains.rs).
// ─────────────────────────────────────────────────────────────────────────

/// The revenue demo snapshot (the 4-account fixture with mixed evidence).
pub fn revenue_fixture_state() -> Value {
    revenue::fixture_state()
}

/// The church demo snapshot: the parallel of the revenue fixture — a
/// fully-evidenced person one step from Leading, a welcomed-but-never-followed
/// -up first-timer capped at Returning, a no-touch first-timer, and someone
/// already Leading (terminal, the church analog of an already-closed account).
pub fn church_fixture_state() -> Value {
    json!({
        "people": [
            {
                "id": "visitor-apex", "stage": "connected", "welcomed": true,
                "followed_up": true, "in_small_group": true, "care_assigned": true,
                "days_in_stage": 12
            },
            {
                "id": "visitor-followup-gap", "stage": "first_time", "welcomed": true,
                "followed_up": false, "in_small_group": true, "care_assigned": true,
                "days_in_stage": 40
            },
            {
                "id": "visitor-fresh", "stage": "first_time", "welcomed": false,
                "followed_up": false, "in_small_group": false, "care_assigned": false,
                "days_in_stage": 90
            },
            {
                "id": "leader-emeritus", "stage": "leading", "welcomed": true,
                "followed_up": true, "in_small_group": true, "care_assigned": true,
                "days_in_stage": 5
            }
        ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn revenue_ceiling_reproduces_mrr_exactly() {
        // The generic ceiling, with ceiling_fluents = ["realized_revenue"],
        // must reproduce the bespoke MRR headline numbers — one substrate.
        let state: RevenueState = serde_json::from_value(revenue_fixture_state()).expect("fixture");
        let mrr = praxis_proposer::maximum_reachable_revenue(&state);
        let c = ceiling::<RevenueDomain>(&state);
        assert_eq!(
            c["max_reachable_value"].as_f64().unwrap() as i64,
            mrr.max_reachable_revenue_cents
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

    #[test]
    fn church_ceiling_respects_evidence_gates() {
        let state: ChurchState =
            serde_json::from_value(church_fixture_state()).expect("church fixture");
        let full = ceiling::<ChurchDomain>(&state);
        let full_max = full["max_reachable_value"].as_f64().unwrap();

        // Strip the follow-up from the deep person: it can no longer be walked
        // to Leading, so the reachable connection ceiling must drop.
        let mut stripped = state.clone();
        stripped.people[0].followed_up = false;
        stripped.people[0].in_small_group = false;
        stripped.people[0].care_assigned = false;
        stripped.people[0].stage = church::Stage::FirstTime;
        let after = ceiling::<ChurchDomain>(&stripped);
        assert!(
            after["max_reachable_value"].as_f64().unwrap() < full_max,
            "removing evidence must lower the church ceiling"
        );
    }

    #[test]
    fn both_packs_run_the_same_pipeline_green() {
        ensure_signing_key();
        let rev_state: RevenueState =
            serde_json::from_value(revenue_fixture_state()).expect("rev fixture");
        let rev_obj = RevenueDomain::load_objective(revenue::REVENUE_OBJECTIVE).expect("rev obj");
        let rev = run_pipeline::<RevenueDomain>(&rev_state, &rev_obj, "m", 1_000)
            .expect("revenue pipe green");
        assert_eq!(rev["pack"], json!("revenue"));
        assert_eq!(rev["chain_hash"].as_str().unwrap().len(), 64);

        let ch_state: ChurchState =
            serde_json::from_value(church_fixture_state()).expect("church fixture");
        let ch_obj = ChurchDomain::load_objective(CHURCH_OBJECTIVE).expect("church obj");
        let ch = run_pipeline::<ChurchDomain>(&ch_state, &ch_obj, "m", 1_000)
            .expect("church pipe green");
        assert_eq!(ch["pack"], json!("church"));
        assert_eq!(ch["chain_hash"].as_str().unwrap().len(), 64);
    }
}
