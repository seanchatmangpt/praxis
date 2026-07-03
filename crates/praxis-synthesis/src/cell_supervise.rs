//! The supervised cell — supervision composed to fleet scale.
//!
//! Group supervisors restart failing members within intensity; members that
//! recover carry the [`crate::fleet::lane::S_RECOVERED`] combo byte; members
//! that crash-loop are parked as first-class receipts; and the MAPE-K loop
//! closes at **epoch boundaries only** (mid-epoch mutation is refused —
//! determinism): group roll-ups are the monitor plane, crashloop templates
//! detected across ≥ `QUORUM` distinct groups are quarantined by a receipted
//! [`EpochPlan`], and the plan hashes chain into the cell receipt.
//!
//! Fault injection is seed-deterministic (splitmix64), so
//! [`crate::cell::challenge_member`]-style selective replay reproduces a
//! recovered member's byte and hash exactly given the same seed.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use chatman_common::provenance::{content_address, fold_event, genesis_seed};

use crate::cell::{run_member, CellReceipt, GroupReceipt, MemberRecord};
use crate::dag::MemoCache;
use crate::fleet::lane;
use crate::solver8::CoreCache;

/// Domain seed for the epoch-plan chain.
pub const SUPERVISE_CHAIN_DOMAIN: &str = "praxis-synthesis/cell/supervise/v1";

/// Distinct groups that must independently witness a template crash-looping
/// before the MAPE-K loop quarantines it — cross-group replication is the
/// evidence bar that the template (not the host) is at fault.
pub const QUORUM: usize = 3;

/// Group-supervisor policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupervisorPolicy {
    /// Restarts allowed per member (≤ 8).
    pub max_restarts: u8,
}

impl Default for SupervisorPolicy {
    fn default() -> Self {
        Self { max_restarts: 3 }
    }
}

/// Deterministic fault script for one cell run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaultScript {
    /// Seed for the transient-fault lottery.
    pub seed: u64,
    /// Probability (per mille) that a member suffers ONE transient fault.
    pub transient_per_mille: u16,
    /// A template whose members crash on EVERY attempt (None = no crashloop).
    pub crashloop_template: Option<usize>,
}

pub(crate) fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// One receipted MAPE-K decision, applied at the NEXT epoch boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanAction {
    /// Quarantine a crash-looping template (members refuse cheaply).
    QuarantineTemplate {
        /// The template index.
        template: usize,
        /// Distinct groups that witnessed the crashloop.
        witness_groups: usize,
    },
}

/// The epoch plan: analyze → plan output, hashed into the cell chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpochPlan {
    /// Epoch that produced this plan.
    pub epoch: u32,
    /// Actions to apply at the next boundary.
    pub actions: Vec<PlanAction>,
    /// Content address of the canonical action list.
    pub plan_hash: String,
}

/// Run one supervised member: the group supervisor retries transient faults
/// within intensity; crashloop members park with a first-class refusal;
/// quarantined templates refuse immediately (cheap, deterministic).
#[must_use]
pub fn run_member_supervised(
    agent: usize,
    templates: usize,
    quarantine: &BTreeSet<usize>,
    policy: SupervisorPolicy,
    script: FaultScript,
    memo: &mut MemoCache,
    cores: &mut CoreCache,
) -> MemberRecord {
    let t = agent % templates.max(1);
    // Quarantined template: immediate typed refusal — visible in the
    // existing refusal register via the existing ':'-head bucketing.
    if quarantine.contains(&t) {
        let rendered = format!("template quarantined: t{t} (crashloop quorum)");
        return MemberRecord {
            agent,
            byte: lane::H_HALTED | lane::E_ERROR,
            terminal_hash: content_address(rendered.as_bytes()),
            refusal: rendered,
            restarts: 0,
        };
    }
    // Crashloop template: every attempt fails → intensity exhausts → park.
    if script.crashloop_template == Some(t) {
        let rendered =
            format!("crash loop: template t{t} failed every restart (parked)");
        return MemberRecord {
            agent,
            byte: lane::S_PARKED, // H|B: halted on intensity, parked
            terminal_hash: content_address(rendered.as_bytes()),
            refusal: rendered,
            restarts: policy.max_restarts,
        };
    }
    // Transient lottery: one injected fault, then the real run succeeds —
    // the member recovers and carries the S_RECOVERED combo (H|…|A).
    let roll = splitmix64(script.seed ^ agent as u64) % 1000;
    let transient = roll < u64::from(script.transient_per_mille);
    let mut record = run_member(agent, templates, memo, cores);
    if transient && record.byte & lane::A_ADMITTED != 0 {
        record.byte |= lane::H_HALTED; // halted once en route → S_RECOVERED
        record.restarts = 1;
    }
    record
}

/// Roll up supervised members (delegates to the cell's roll-up via its
/// public pieces: recomputed here to keep `cell::roll_up` private).
fn roll_up(group: usize, members: Vec<MemberRecord>) -> GroupReceipt {
    let admitted =
        members.iter().filter(|m| m.byte & lane::A_ADMITTED != 0).count();
    let refused = members.len() - admitted;
    let recovered = members
        .iter()
        .filter(|m| m.byte & lane::S_RECOVERED == lane::S_RECOVERED)
        .count();
    let parked = members
        .iter()
        .filter(|m| {
            m.byte & lane::A_ADMITTED == 0 && m.byte & lane::S_PARKED == lane::S_PARKED
        })
        .count();
    let geometry_gaps = members
        .iter()
        .filter(|m| m.byte & lane::S_GEOMETRY_GAP == lane::S_GEOMETRY_GAP)
        .count();
    let restarts = members.iter().map(|m| usize::from(m.restarts)).sum();
    let mut reasons: BTreeMap<String, usize> = BTreeMap::new();
    for m in &members {
        if !m.refusal.is_empty() {
            let head = m.refusal.split(':').next().unwrap_or("refused").to_string();
            *reasons.entry(head).or_default() += 1;
        }
    }
    let mut top: Vec<(String, usize)> = reasons.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    top.truncate(3);
    GroupReceipt {
        group,
        admitted,
        refused,
        top_refusals: top,
        replay_root: crate::cell::replay_root(&members),
        members,
        recovered,
        parked,
        geometry_gaps,
        restarts,
    }
}

/// Analyze one epoch's roll-ups: a template crash-looping in ≥ [`QUORUM`]
/// distinct groups earns quarantine.
#[must_use]
pub fn analyze_epoch(epoch: u32, groups: &[GroupReceipt]) -> EpochPlan {
    let mut witnesses: BTreeMap<usize, BTreeSet<usize>> = BTreeMap::new();
    for gr in groups {
        for m in &gr.members {
            if let Some(rest) = m.refusal.strip_prefix("crash loop: template t") {
                if let Some(tstr) = rest.split(' ').next() {
                    if let Ok(t) = tstr.parse::<usize>() {
                        witnesses.entry(t).or_default().insert(gr.group);
                    }
                }
            }
        }
    }
    let actions: Vec<PlanAction> = witnesses
        .into_iter()
        .filter(|(_, gs)| gs.len() >= QUORUM)
        .map(|(template, gs)| PlanAction::QuarantineTemplate {
            template,
            witness_groups: gs.len(),
        })
        .collect();
    let plan_hash = content_address(
        serde_json::to_string(&actions).unwrap_or_default().as_bytes(),
    );
    EpochPlan { epoch, actions, plan_hash }
}

/// Run a supervised cell for `epochs` epochs. Parameters change ONLY at
/// epoch boundaries (mid-epoch mutation refused by construction: the
/// quarantine set is immutable within an epoch). Returns the final epoch's
/// receipts, with the full epoch-plan chain and cumulative quarantine set
/// folded into the cell receipt.
#[must_use]
pub fn run_cell_supervised(
    n: usize,
    g: usize,
    templates: usize,
    epochs: u32,
    policy: SupervisorPolicy,
    script: FaultScript,
) -> (CellReceipt, Vec<GroupReceipt>, Vec<EpochPlan>) {
    let per = n / g.max(1);
    let mut quarantine: BTreeSet<usize> = BTreeSet::new();
    let mut plans: Vec<EpochPlan> = Vec::new();
    let mut final_groups: Vec<GroupReceipt> = Vec::new();

    for epoch in 0..epochs.max(1) {
        let mut groups = Vec::with_capacity(g);
        for gi in 0..g {
            let mut memo = MemoCache::new();
            let mut cores = CoreCache::new();
            let members: Vec<MemberRecord> = (0..per)
                .map(|m| {
                    run_member_supervised(
                        gi * per + m,
                        templates,
                        &quarantine,
                        policy,
                        script,
                        &mut memo,
                        &mut cores,
                    )
                })
                .collect();
            groups.push(roll_up(gi, members));
        }
        // MAPE-K: monitor (roll-ups) → analyze → plan (receipted) →
        // execute at the NEXT boundary.
        let plan = analyze_epoch(epoch, &groups);
        for action in &plan.actions {
            let PlanAction::QuarantineTemplate { template, .. } = action;
            quarantine.insert(*template);
        }
        plans.push(plan);
        final_groups = groups;
    }

    // Cell receipt over the FINAL epoch's roots (same fold as cell.rs).
    let admitted = final_groups.iter().map(|gr| gr.admitted).sum();
    let refused = final_groups.iter().map(|gr| gr.refused).sum();
    let recovered = final_groups.iter().map(|gr| gr.recovered).sum();
    let parked = final_groups.iter().map(|gr| gr.parked).sum();
    let geometry_gaps = final_groups.iter().map(|gr| gr.geometry_gaps).sum();
    let mut register: BTreeMap<String, usize> = BTreeMap::new();
    for gr in &final_groups {
        for (reason, count) in &gr.top_refusals {
            *register.entry(reason.clone()).or_default() += count;
        }
    }
    let mut cell_hash = genesis_seed(crate::cell::CELL_CHAIN_DOMAIN);
    for gr in &final_groups {
        cell_hash = fold_event(&cell_hash, gr.replay_root.as_bytes());
    }
    // Fold the epoch-plan chain (supervision's own lineage).
    let mut plan_chain = genesis_seed(SUPERVISE_CHAIN_DOMAIN);
    let mut epoch_plan_hashes = Vec::with_capacity(plans.len());
    for p in &plans {
        plan_chain = fold_event(&plan_chain, p.plan_hash.as_bytes());
        epoch_plan_hashes.push(plan_chain.clone());
    }
    (
        CellReceipt {
            n,
            g,
            admitted,
            refused,
            refusal_register: register,
            cell_hash,
            recovered,
            parked,
            geometry_gaps,
            quarantined_templates: quarantine.into_iter().collect(),
            epoch_plan_hashes,
        },
        final_groups,
        plans,
    )
}
