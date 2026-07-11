//! PROJ-705 — Goal partitioning + bounded canonical candidate enumeration.
//!
//! Union-find over goal atoms: two goal atoms are connected iff their
//! achiever action sets intersect, or any achiever pair carries a derived
//! mutex or custody-conflict edge (`rules/decomp.dl`). Components are
//! canonically ordered by least atom label; candidates are enumerated as
//! 2-way splits (helper = a nonempty proper component subset, main = the
//! rest) in lexicographic candidate-id order, bounded by
//! [`DECOMP_MAX_COMPONENTS`] / [`DECOMP_MAX_CANDIDATES`]. The single-actor
//! plan is ALWAYS candidate #0 (`0-single`) — an explicit typed candidate,
//! never a silent fallback.

use std::collections::BTreeSet;

use bcinr_pddl::Pddl8GroundAtom;

use super::rules::DerivedEdges;

/// Maximum goal components considered for splitting; beyond this bound only
/// the single-actor candidate is enumerated (recorded honestly, not
/// silently truncated component-wise).
pub const DECOMP_MAX_COMPONENTS: usize = 8;

/// Maximum candidates examined per decomposition (including candidate #0).
pub const DECOMP_MAX_CANDIDATES: usize = 32;

/// Canonical id of the single-actor candidate. Sorts before every split id
/// (split ids start with an atom label, `[a-z]`).
pub const SINGLE_ACTOR_CANDIDATE_ID: &str = "0-single";

/// One enumeration candidate: the single-actor whole-goal plan when
/// `helper_goal` is empty, otherwise a 2-way split with the named helper
/// goal atoms (main goal = full original goal from the interface state).
#[derive(Debug, Clone)]
pub struct Candidate {
    /// Canonical id: `0-single`, or the sorted helper goal-atom labels
    /// joined with `+`.
    pub id: String,
    /// Helper goal atoms; empty for the single-actor candidate.
    pub helper_goal: Vec<Pddl8GroundAtom>,
}

/// Union-find with path compression (no ranks; n ≤ goal-atom count).
///
/// # Complexity
/// Near-O(1) amortized per find/union at these sizes.
struct UnionFind {
    parent: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        UnionFind {
            parent: (0..n).collect(),
        }
    }

    /// # Complexity
    /// O(α(n)) amortized.
    fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]];
            x = self.parent[x];
        }
        x
    }

    /// # Complexity
    /// O(α(n)) amortized.
    fn union(&mut self, a: usize, b: usize) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra != rb {
            self.parent[ra] = rb;
        }
    }
}

/// True iff any achiever of `a` and any achiever of `b` are coupled: same
/// action, mutex pair, or custody-conflict pair.
///
/// # Complexity
/// O(|ach(a)| · |ach(b)| · log |edges|).
fn coupled(a: &str, b: &str, edges: &DerivedEdges) -> bool {
    let empty = BTreeSet::new();
    let ach_a = edges.achievers.get(a).unwrap_or(&empty);
    let ach_b = edges.achievers.get(b).unwrap_or(&empty);
    for x in ach_a {
        if ach_b.contains(x) {
            return true;
        }
        for y in ach_b {
            if edges.mutex.contains(&(x.clone(), y.clone()))
                || edges.custody.contains(&(x.clone(), y.clone()))
            {
                return true;
            }
        }
    }
    false
}

/// Partitions the goal atoms into independence components (union-find over
/// the derived coupling relation), each component sorted by atom label,
/// components ordered by their least atom label.
///
/// # Complexity
/// O(g² · k) pairwise coupling checks over g goal atoms (k = achiever-set
/// cost, see `coupled`) — g is bounded by the STRIPS8 conjunct cap.
pub fn partition_goals(
    goal: &[Pddl8GroundAtom],
    edges: &DerivedEdges,
) -> Vec<Vec<Pddl8GroundAtom>> {
    let mut atoms: Vec<Pddl8GroundAtom> = goal.to_vec();
    atoms.sort();
    atoms.dedup();
    let n = atoms.len();
    let mut uf = UnionFind::new(n);
    // O(g²) pairs.
    for i in 0..n {
        for j in (i + 1)..n {
            if coupled(&atoms[i].label(), &atoms[j].label(), edges) {
                uf.union(i, j);
            }
        }
    }
    let mut by_root: std::collections::BTreeMap<usize, Vec<Pddl8GroundAtom>> =
        std::collections::BTreeMap::new();
    for i in 0..n {
        let root = uf.find(i);
        by_root.entry(root).or_default().push(atoms[i].clone());
    }
    let mut components: Vec<Vec<Pddl8GroundAtom>> = by_root.into_values().collect();
    for component in &mut components {
        component.sort();
    }
    // Order components by their (sorted) least atom; components are
    // nonempty by construction, and `Option<T: Ord>` ordering keeps this
    // total without a silent default. O(k log k).
    components.sort_by_key(|component| component.first().cloned());
    components
}

/// Canonical candidate id for a helper goal-atom set.
///
/// # Complexity
/// O(n log n).
pub fn candidate_id(helper_goal: &[Pddl8GroundAtom]) -> String {
    if helper_goal.is_empty() {
        return SINGLE_ACTOR_CANDIDATE_ID.to_string();
    }
    let mut labels: Vec<String> = helper_goal.iter().map(|a| a.label()).collect();
    labels.sort();
    labels.join("+")
}

/// Enumerates the bounded canonical candidate list: candidate #0 is always
/// the single-actor plan; when 2 ≤ components ≤ [`DECOMP_MAX_COMPONENTS`],
/// every nonempty proper component subset becomes a helper-goal candidate,
/// sorted by canonical id and truncated to [`DECOMP_MAX_CANDIDATES`] total.
///
/// # Complexity
/// O(2^k · g log g) subset enumeration over k ≤ 8 components (≤ 254
/// subsets) with g goal atoms each, then O(c log c) canonical sort.
pub fn enumerate_candidates(components: &[Vec<Pddl8GroundAtom>]) -> Vec<Candidate> {
    let mut candidates = vec![Candidate {
        id: SINGLE_ACTOR_CANDIDATE_ID.to_string(),
        helper_goal: Vec::new(),
    }];
    let k = components.len();
    if k < 2 || k > DECOMP_MAX_COMPONENTS {
        return candidates;
    }
    let mut splits: Vec<Candidate> = Vec::new();
    // All nonempty proper subsets of the component index set. O(2^k).
    for mask in 1u32..((1u32 << k) - 1) {
        let mut helper_goal: Vec<Pddl8GroundAtom> = Vec::new();
        for (i, component) in components.iter().enumerate() {
            if mask & (1 << i) != 0 {
                helper_goal.extend(component.iter().cloned());
            }
        }
        helper_goal.sort();
        splits.push(Candidate {
            id: candidate_id(&helper_goal),
            helper_goal,
        });
    }
    splits.sort_by(|a, b| a.id.cmp(&b.id));
    splits.dedup_by(|a, b| a.id == b.id);
    splits.truncate(DECOMP_MAX_CANDIDATES.saturating_sub(1));
    candidates.extend(splits);
    candidates
}
