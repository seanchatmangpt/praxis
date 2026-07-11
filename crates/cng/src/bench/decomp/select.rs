//! PROJ-710 — Deterministic selection law + per-candidate receipts.
//!
//! Lexicographic ordering over (Makespan, DispatchCost, Risk), canonical
//! candidate id as the stable tie-break:
//! - Makespan = longest path (node count) through the composed partial
//!   order (Kahn topological pass; runner.rs Kahn precedent).
//! - DispatchCost = total action count + subworkflows ×
//!   [`DISPATCH_OVERHEAD_STEPS`] — models coordination cost so the
//!   single-actor candidate can win.
//! - Risk = cross-workflow mustPrecede edge count (interface coupling).
//!
//! `NoAdmissibleDecomposition` / `NoBeneficialDecomposition` are typed
//! SUCCESS results (`DecompositionOutcome`), receipted, never refusals or
//! silent fallbacks. `CNG_R21 DecompositionInadmissible` fires only when an
//! inadmissible candidate is demanded (forced) or would be selected.

use std::collections::BTreeSet;

use crate::powl::CngRefusal;

use super::SINGLE_ACTOR_CANDIDATE_ID;

/// Modeled per-subworkflow dispatch/coordination overhead in plan steps
/// (documented constant of the selection law, not a measurement).
pub const DISPATCH_OVERHEAD_STEPS: u64 = 2;

/// Selection-law score triple; lexicographic order is the law.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Score {
    pub makespan: u64,
    pub dispatch_cost: u64,
    pub risk: u64,
}

/// Per-candidate verdict recorded in the receipt graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateStatus {
    /// Won the lexicographic argmin.
    Selected,
    /// Passed every proof obligation but lost the argmin.
    Admissible,
    /// Failed a proof obligation (reason names it).
    Inadmissible,
}

impl CandidateStatus {
    /// Stable receipt string.
    pub fn as_str(&self) -> &'static str {
        match self {
            CandidateStatus::Selected => "Selected",
            CandidateStatus::Admissible => "Admissible",
            CandidateStatus::Inadmissible => "Inadmissible",
        }
    }
}

/// One examined candidate's receipt — accepted AND rejected candidates are
/// evidence, never dropped.
#[derive(Debug, Clone)]
pub struct CandidateReceipt {
    pub candidate_id: String,
    pub status: CandidateStatus,
    /// `selected` / `admissible`, or the named failed proof obligation
    /// (refusal code + detail) for inadmissible candidates.
    pub reason: String,
    pub score: Score,
}

/// Longest-path length in NODES over a DAG given as index edges, via
/// Kahn's algorithm (precedent: runner.rs / hooks compile topological
/// machinery). A cycle refuses — the composed order must be a partial
/// order.
///
/// # Errors
/// `CNG_R21 DecompositionInadmissible` when the edge set is cyclic.
///
/// # Complexity
/// O(n + |edges|).
pub fn longest_path_nodes(
    n: usize,
    edges: &BTreeSet<(usize, usize)>,
    candidate_id: &str,
) -> Result<u64, CngRefusal> {
    let mut indegree = vec![0usize; n];
    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n];
    for &(from, to) in edges {
        if from >= n || to >= n {
            return Err(CngRefusal::DecompositionInadmissible {
                candidate: candidate_id.to_string(),
                reason: format!("order edge ({from}, {to}) out of range for {n} nodes"),
            });
        }
        adjacency[from].push(to);
        indegree[to] += 1;
    }
    let mut queue: Vec<usize> = (0..n).filter(|&i| indegree[i] == 0).collect();
    let mut dist = vec![1u64; n];
    let mut processed = 0usize;
    let mut head = 0usize;
    while head < queue.len() {
        let node = queue[head];
        head += 1;
        processed += 1;
        for &next in &adjacency[node] {
            if dist[node] + 1 > dist[next] {
                dist[next] = dist[node] + 1;
            }
            indegree[next] -= 1;
            if indegree[next] == 0 {
                queue.push(next);
            }
        }
    }
    if processed != n {
        return Err(CngRefusal::DecompositionInadmissible {
            candidate: candidate_id.to_string(),
            reason: "composed order relation is cyclic".to_string(),
        });
    }
    Ok(dist.into_iter().max().unwrap_or(0))
}

/// Score for a 2-subworkflow split: nodes are helper ops (0..h) then main
/// ops (h..h+m); edges are the two intra-subworkflow chains plus the
/// cross-workflow mustPrecede pairs (given as (helper index, main index)
/// with `main_index` already offset).
///
/// # Errors
/// See [`longest_path_nodes`].
///
/// # Complexity
/// O(h + m + |cross|).
pub fn score_split(
    candidate_id: &str,
    helper_len: usize,
    main_len: usize,
    cross_index_edges: &BTreeSet<(usize, usize)>,
) -> Result<Score, CngRefusal> {
    let n = helper_len + main_len;
    let mut edges: BTreeSet<(usize, usize)> = cross_index_edges.clone();
    // Intra-subworkflow total-order chains. O(h + m).
    for i in 1..helper_len {
        edges.insert((i - 1, i));
    }
    for i in 1..main_len {
        edges.insert((helper_len + i - 1, helper_len + i));
    }
    let makespan = longest_path_nodes(n, &edges, candidate_id)?;
    Ok(Score {
        makespan,
        dispatch_cost: n as u64 + 2 * DISPATCH_OVERHEAD_STEPS,
        risk: cross_index_edges.len() as u64,
    })
}

/// Score for the single-actor candidate: makespan = tape length, one
/// subworkflow's overhead, zero interface risk.
///
/// # Complexity
/// O(1).
pub fn score_single(tape_len: usize) -> Score {
    Score {
        makespan: tape_len as u64,
        dispatch_cost: tape_len as u64 + DISPATCH_OVERHEAD_STEPS,
        risk: 0,
    }
}

/// The selection verdict: winning candidate id plus split/single facts the
/// caller turns into a `DecompositionOutcome`.
#[derive(Debug, Clone)]
pub struct SelectionVerdict {
    pub selected_id: String,
    /// Best admissible SPLIT candidate id (None when no split survived).
    pub best_split_id: Option<String>,
    /// Number of inadmissible candidates.
    pub rejected: usize,
}

/// Applies the selection law over the receipts, marking the winner
/// `Selected`. `forced` demands a specific candidate: if that candidate is
/// inadmissible (or absent), `CNG_R21` refuses — an inadmissible candidate
/// is never selected.
///
/// # Errors
/// `CNG_R21 DecompositionInadmissible`.
///
/// # Complexity
/// O(c log c) over c candidates.
pub fn select(
    receipts: &mut [CandidateReceipt],
    forced: Option<&str>,
) -> Result<SelectionVerdict, CngRefusal> {
    if let Some(id) = forced {
        let receipt = receipts
            .iter()
            .find(|r| r.candidate_id == id)
            .ok_or_else(|| CngRefusal::DecompositionInadmissible {
                candidate: id.to_string(),
                reason: "demanded candidate was never enumerated".to_string(),
            })?;
        if receipt.status == CandidateStatus::Inadmissible {
            return Err(CngRefusal::DecompositionInadmissible {
                candidate: id.to_string(),
                reason: receipt.reason.clone(),
            });
        }
    }

    let mut admissible: Vec<(Score, String)> = receipts
        .iter()
        .filter(|r| r.status != CandidateStatus::Inadmissible)
        .filter(|r| forced.is_none_or(|id| r.candidate_id == id))
        .map(|r| (r.score, r.candidate_id.clone()))
        .collect();
    admissible.sort();
    let Some((_, selected_id)) = admissible.first().cloned() else {
        return Err(CngRefusal::DecompositionInadmissible {
            candidate: forced.unwrap_or(SINGLE_ACTOR_CANDIDATE_ID).to_string(),
            reason: "no admissible candidate exists (single-actor included)".to_string(),
        });
    };

    let mut splits: Vec<(Score, String)> = receipts
        .iter()
        .filter(|r| {
            r.status != CandidateStatus::Inadmissible && r.candidate_id != SINGLE_ACTOR_CANDIDATE_ID
        })
        .map(|r| (r.score, r.candidate_id.clone()))
        .collect();
    splits.sort();
    let rejected = receipts
        .iter()
        .filter(|r| r.status == CandidateStatus::Inadmissible)
        .count();

    for receipt in receipts.iter_mut() {
        if receipt.candidate_id == selected_id {
            receipt.status = CandidateStatus::Selected;
            receipt.reason = "selected".to_string();
        }
    }

    Ok(SelectionVerdict {
        selected_id,
        best_split_id: splits.first().map(|(_, id)| id.clone()),
        rejected,
    })
}
