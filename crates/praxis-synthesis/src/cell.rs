//! Step 4 of the cell roadmap — the minimum phase-change cell.
//!
//! Do not build the trillion-agent city; build the smallest lawful cell
//! whose behavior composes. A cell is N agents in G groups. Each agent
//! emits a status byte and a receipt-or-refusal; each group emits ONE
//! roll-up receipt (admitted/refused counts, top refusal reasons, a replay
//! root over its members); the cell emits one receipt folding the group
//! roll-ups. The composition law under test:
//!
//! > A verifier can check the cell-level projection, and selectively replay
//! > any single agent or group, **without reading every interior event**.
//!
//! Verification here is hierarchical Merkle recomputation: the cell hash is
//! a function of the group hashes alone; a group hash is a function of its
//! members' receipt chains alone. Reading interiors is only ever needed for
//! the one member being challenged — that is the trillion-agent law in
//! miniature.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use chatman_common::provenance::{content_address, fold_event, genesis_seed};

use crate::dag::{HashRunner, MemoCache};
use crate::fleet::{lane, template};
use crate::sequence::SequenceProblem;
use crate::solver8::{CoreCache, Solver8};
use crate::verify::admit;
use crate::{Dag, Refusal};

/// Domain seed for cell/group chains.
pub const CELL_CHAIN_DOMAIN: &str = "praxis-synthesis/cell/v1";

/// One agent's projection: everything the fleet needs to know about it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberRecord {
    /// Agent index within the cell.
    pub agent: usize,
    /// agent8-lane status byte.
    pub byte: u8,
    /// The member's terminal hash: the pipeline chain if admitted, or the
    /// content address of the rendered refusal if refused. Either way, one
    /// hash — the member's lawful projection.
    pub terminal_hash: String,
    /// Refusal reason (empty when admitted).
    pub refusal: String,
    /// Restart attempts this member consumed (supervision; additive).
    #[serde(default)]
    pub restarts: u8,
}

/// One group's roll-up receipt — the only thing the cell reads per group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupReceipt {
    /// Group index.
    pub group: usize,
    /// Members admitted end-to-end.
    pub admitted: usize,
    /// Members refused.
    pub refused: usize,
    /// Top refusal reasons (reason → count), most frequent first, ≤ 3.
    pub top_refusals: Vec<(String, usize)>,
    /// Replay root: BLAKE3 over the sorted member terminal hashes — commits
    /// every interior without carrying any.
    pub replay_root: String,
    /// Member records (retained for selective replay; a remote verifier
    /// receives only the fields above plus this on demand).
    pub members: Vec<MemberRecord>,
    /// Members admitted after >= 1 restart (counted inside `admitted`).
    #[serde(default)]
    pub recovered: usize,
    /// Members parked (counted inside `refused`).
    #[serde(default)]
    pub parked: usize,
    /// Crashes that landed outside the derived geometry (informational).
    #[serde(default)]
    pub geometry_gaps: usize,
    /// Total restart attempts across the group.
    #[serde(default)]
    pub restarts: usize,
}

/// The cell receipt: what a verifier reads INSTEAD of 10,000 interiors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellReceipt {
    /// Agents in the cell.
    pub n: usize,
    /// Groups.
    pub g: usize,
    /// Total admitted / refused.
    pub admitted: usize,
    /// Total refused.
    pub refused: usize,
    /// Cell-wide refusal register (reason → count).
    pub refusal_register: BTreeMap<String, usize>,
    /// The cell hash: rolling fold over the group replay roots, in group
    /// order, from the domain genesis. A function of G hashes — not of
    /// N interiors.
    pub cell_hash: String,
    /// Members admitted after recovery (inside `admitted`; additive).
    #[serde(default)]
    pub recovered: usize,
    /// Members parked (inside `refused`; additive).
    #[serde(default)]
    pub parked: usize,
    /// Geometry gaps observed cell-wide.
    #[serde(default)]
    pub geometry_gaps: usize,
    /// Templates quarantined by the MAPE-K loop.
    #[serde(default)]
    pub quarantined_templates: Vec<usize>,
    /// Hash chain of the receipted epoch plans (MAPE-K decisions).
    #[serde(default)]
    pub epoch_plan_hashes: Vec<String>,
}

/// Run one agent (identified by cell-wide index) and produce its record.
/// Deterministic in the index — this IS the selective-replay entry point.
#[must_use]
pub fn run_member(
    agent: usize,
    templates: usize,
    memo: &mut MemoCache,
    cores: &mut CoreCache,
) -> MemberRecord {
    let t = agent % templates.max(1);
    let (mut program, caps, goal, constraints) = template(t);
    let mut byte = 0u8;
    let outcome: Result<String, Refusal> = (|| {
        program.saturate()?;
        byte |= lane::P_SATURATED;
        let problem = SequenceProblem::with_constraints(&program, caps, goal, 8, constraints)?;
        let plan = Solver8.solve_cached(&problem, cores)?;
        byte |= lane::R_PLANNED;
        let dag = Dag::from_plan(&plan, &problem);
        let receipt = dag.execute(&mut HashRunner, memo)?;
        byte |= lane::C_EXECUTED;
        let verdict = admit(&mut program, &problem, &plan, &dag, &receipt);
        if !verdict.ok {
            return Err(Refusal::VerificationFailed {
                failed: verdict.failed(),
            });
        }
        byte |= lane::A_ADMITTED;
        // Terminal hash: fold plan + dag root (the member's whole run).
        Ok(fold_event(
            &receipt.root_hash,
            plan.receipt.plan_hash.as_bytes(),
        ))
    })();
    match outcome {
        Ok(terminal_hash) => MemberRecord {
            agent,
            byte,
            terminal_hash,
            refusal: String::new(),
            restarts: 0,
        },
        Err(refusal) => {
            byte |= lane::H_HALTED;
            byte |= match refusal {
                Refusal::UnsatProof { .. } => lane::U_UNSAT_CERTIFIED,
                Refusal::BudgetExceeded { .. } | Refusal::TupleCapExceeded { .. } => lane::B_BUDGET,
                _ => lane::E_ERROR,
            };
            let rendered = refusal.to_string();
            MemberRecord {
                agent,
                byte,
                terminal_hash: content_address(rendered.as_bytes()),
                refusal: rendered,
                restarts: 0,
            }
        }
    }
}

/// Compute a group's replay root from member terminal hashes (sorted — the
/// root is member-order independent).
#[must_use]
pub fn replay_root(members: &[MemberRecord]) -> String {
    let mut hashes: Vec<&str> = members.iter().map(|m| m.terminal_hash.as_str()).collect();
    hashes.sort_unstable();
    content_address(hashes.join("\n").as_bytes())
}

fn roll_up(group: usize, members: Vec<MemberRecord>) -> GroupReceipt {
    let admitted = members
        .iter()
        .filter(|m| m.byte & lane::A_ADMITTED != 0)
        .count();
    let refused = members.len() - admitted;
    let recovered = members
        .iter()
        .filter(|m| m.byte & lane::S_RECOVERED == lane::S_RECOVERED)
        .count();
    let parked = members
        .iter()
        .filter(|m| m.byte & lane::A_ADMITTED == 0 && m.byte & lane::S_PARKED == lane::S_PARKED)
        .count();
    let geometry_gaps = members
        .iter()
        .filter(|m| m.byte & lane::S_GEOMETRY_GAP == lane::S_GEOMETRY_GAP)
        .count();
    let restarts = members.iter().map(|m| usize::from(m.restarts)).sum();
    let mut reasons: BTreeMap<String, usize> = BTreeMap::new();
    for m in &members {
        if !m.refusal.is_empty() {
            // Bucket by the refusal's head (before the first ':').
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
        replay_root: replay_root(&members),
        members,
        recovered,
        parked,
        geometry_gaps,
        restarts,
    }
}

/// Run the cell: `n` agents in `g` groups (n/g members each), shared caches
/// within each group (locality: no global coordination — each group's memo
/// and core caches are its own shard).
#[must_use]
pub fn run_cell(n: usize, g: usize, templates: usize) -> (CellReceipt, Vec<GroupReceipt>) {
    let per = n / g.max(1);
    let mut groups = Vec::with_capacity(g);
    for gi in 0..g {
        // Local shard: local caches, local receipts, local refusals.
        let mut memo = MemoCache::new();
        let mut cores = CoreCache::new();
        let members: Vec<MemberRecord> = (0..per)
            .map(|m| run_member(gi * per + m, templates, &mut memo, &mut cores))
            .collect();
        groups.push(roll_up(gi, members));
    }
    let admitted = groups.iter().map(|gr| gr.admitted).sum();
    let refused = groups.iter().map(|gr| gr.refused).sum();
    let mut register: BTreeMap<String, usize> = BTreeMap::new();
    for gr in &groups {
        for (reason, count) in &gr.top_refusals {
            *register.entry(reason.clone()).or_default() += count;
        }
    }
    // The cell hash reads G roots — never N interiors.
    let mut cell_hash = genesis_seed(CELL_CHAIN_DOMAIN);
    for gr in &groups {
        cell_hash = fold_event(&cell_hash, gr.replay_root.as_bytes());
    }
    let recovered = groups.iter().map(|gr| gr.recovered).sum();
    let parked = groups.iter().map(|gr| gr.parked).sum();
    let geometry_gaps = groups.iter().map(|gr| gr.geometry_gaps).sum();
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
            quarantined_templates: Vec::new(),
            epoch_plan_hashes: Vec::new(),
        },
        groups,
    )
}

/// Verify the cell receipt from the group roll-ups ALONE — no member data
/// is touched. Returns false on any mismatch.
#[must_use]
pub fn verify_cell(cell: &CellReceipt, groups: &[GroupReceipt]) -> bool {
    if groups.len() != cell.g {
        return false;
    }
    let mut h = genesis_seed(CELL_CHAIN_DOMAIN);
    for gr in groups {
        h = fold_event(&h, gr.replay_root.as_bytes());
    }
    h == cell.cell_hash
        && groups.iter().map(|g| g.admitted).sum::<usize>() == cell.admitted
        && groups.iter().map(|g| g.refused).sum::<usize>() == cell.refused
}

/// Verify one group's roll-up from its member records alone.
#[must_use]
pub fn verify_group(group: &GroupReceipt) -> bool {
    replay_root(&group.members) == group.replay_root
        && group
            .members
            .iter()
            .filter(|m| m.byte & lane::A_ADMITTED != 0)
            .count()
            == group.admitted
}

/// Domain seed for the supra-cell chain (cells → supra, mirroring
/// groups → cell).
pub const SUPRA_CHAIN_DOMAIN: &str = "praxis-synthesis/supracell/v1";

/// Per-cell projection retained by the supra roll-up: aggregates + the cell
/// hash + its group replay roots. NO member records — O(groups) per cell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellSummary {
    /// Index of the cell within the supra run.
    pub cell_index: usize,
    /// Members in the cell.
    pub n: usize,
    /// Members admitted end-to-end.
    pub admitted: usize,
    /// Members refused.
    pub refused: usize,
    /// The cell's own hash (fold of its group replay roots).
    pub cell_hash: String,
    /// Group replay roots (enables verify_cell-equivalent recomputation
    /// without any member data).
    pub group_roots: Vec<String>,
    /// Wall time the cell took to run, in nanoseconds.
    pub elapsed_ns: u128,
}

/// Summarize one executed cell, dropping all member interiors.
#[must_use]
pub fn summarize_cell(
    cell_index: usize,
    elapsed_ns: u128,
    cell: &CellReceipt,
    groups: &[GroupReceipt],
) -> CellSummary {
    CellSummary {
        cell_index,
        n: cell.n,
        admitted: cell.admitted,
        refused: cell.refused,
        cell_hash: cell.cell_hash.clone(),
        group_roots: groups.iter().map(|gr| gr.replay_root.clone()).collect(),
        elapsed_ns,
    }
}

/// The count-bound line the supra folds per cell: index, counts, and the
/// cell hash in one canonical rendering (elapsed_ns deliberately excluded —
/// timing is reported, never hashed). Binding the counts here means a
/// tampered `admitted`/`refused` breaks the supra, not just an internal
/// consistency check.
#[must_use]
pub fn summary_line(s: &CellSummary) -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}",
        s.cell_index, s.n, s.admitted, s.refused, s.cell_hash
    )
}

/// The supra hash: rolling fold over count-bound summary lines in cell
/// order from the supra genesis — a function of C summaries (counts + cell
/// hashes), never of N interiors.
#[must_use]
pub fn supra_hash(summaries: &[CellSummary]) -> String {
    let mut h = genesis_seed(SUPRA_CHAIN_DOMAIN);
    for s in summaries {
        h = fold_event(&h, summary_line(s).as_bytes());
    }
    h
}

/// Verify the supra receipt from cell summaries ALONE: refold each cell hash
/// from its G group roots (the existing cell-from-group law, one level up),
/// check the count invariant, then refold the supra from the count-bound
/// summary lines rebuilt around the RECOMPUTED cell hashes. Reads
/// O(C + C·G); no member data is touched. Returns false on any mismatch —
/// a tampered group root, cell hash, or count anywhere breaks the supra.
#[must_use]
pub fn verify_supra(supra: &str, summaries: &[CellSummary]) -> bool {
    let mut rebuilt = Vec::with_capacity(summaries.len());
    for s in summaries {
        let mut h = genesis_seed(CELL_CHAIN_DOMAIN);
        for root in &s.group_roots {
            h = fold_event(&h, root.as_bytes());
        }
        if h != s.cell_hash || s.admitted + s.refused != s.n {
            return false;
        }
        let mut recomputed = s.clone();
        recomputed.cell_hash = h;
        rebuilt.push(recomputed);
    }
    supra_hash(&rebuilt) == supra
}

/// Selectively replay ONE agent (fresh caches — full recomputation) and
/// check its terminal hash against the recorded member projection. The
/// challenge path: interiors are read only for the member challenged.
#[must_use]
pub fn challenge_member(record: &MemberRecord, templates: usize) -> bool {
    let mut memo = MemoCache::new();
    let mut cores = CoreCache::new();
    let fresh = run_member(record.agent, templates, &mut memo, &mut cores);
    fresh.terminal_hash == record.terminal_hash && fresh.byte == record.byte
}
