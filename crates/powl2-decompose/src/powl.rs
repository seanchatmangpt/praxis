//! POWL 2.0 model (Kourani et al. Defs 3.6–3.9): a hierarchy of leaves,
//! partial orders (concurrency), and choice graphs (generalized decision +
//! cyclic logic), plus a *bounded* language interpreter.
//!
//! The full language of a model with a cyclic choice graph is infinite, so
//! [`Powl::language_upto`] enumerates only label-sequences of length `≤ k`.
//! That is exactly what the differential round-trip test compares against the
//! WF-net's bounded token-game language — three independent computations of
//! the same bounded language must agree.

use std::collections::BTreeSet;

/// A sequence of activity labels (silent `τ` steps are already elided).
pub type Trace = Vec<String>;
/// A (bounded) language: a set of traces.
pub type Language = BTreeSet<Trace>;

/// A directed choice graph over child indices `0..n` plus the artificial
/// start `▷` and end `□` nodes (Def 3.6). Edges are index pairs; [`START`]
/// and [`END`] are the sentinels for `▷`/`□`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoiceGraph {
    /// number of child submodels (graph nodes, excluding ▷/□).
    pub n: usize,
    /// directed edges over `{START, 0..n, END}`.
    pub edges: BTreeSet<(GNode, GNode)>,
}

/// A node of a [`ChoiceGraph`]: the start `▷`, the end `□`, or a child index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GNode {
    /// The artificial start node `▷`.
    Start,
    /// The artificial end node `□`.
    End,
    /// Child submodel index.
    Child(usize),
}

/// The `▷` start sentinel.
pub const START: GNode = GNode::Start;
/// The `□` end sentinel.
pub const END: GNode = GNode::End;

/// A POWL 2.0 model over transition labels (Def 3.7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Powl {
    /// A leaf transition: `Some(activity)` or silent `τ` (`None`).
    Leaf(Option<String>),
    /// A partial order `≺(ψ₁,…,ψₙ)` over its children. `order` holds the
    /// (transitively closed) strict partial order as index pairs `(i,j)`
    /// meaning `ψᵢ ≺ ψⱼ` (all of `ψᵢ` before all of `ψⱼ`).
    PartialOrder {
        /// child submodels.
        children: Vec<Powl>,
        /// `(i,j) ∈ order` iff `ψᵢ ≺ ψⱼ`.
        order: BTreeSet<(usize, usize)>,
    },
    /// A choice graph `γ(ψ₁,…,ψₙ)` (Def 3.6/3.9): exclusive paths and cycles.
    Choice {
        /// child submodels.
        children: Vec<Powl>,
        /// the routing graph over child indices.
        graph: ChoiceGraph,
    },
    /// An external execution cut identifying a POWL region whose execution boundary
    /// leaves the current process cell.
    ExternalCut {
        /// The admitted POWL region (W).
        region: Box<Powl>,
        /// The declared SPARQL projection (Q).
        projection: String,
        /// The declared Tera renderer (T).
        renderer: String,
    },
}

impl Powl {
    /// The bounded language of the model: all traces of length `≤ max_len`
    /// (Def 3.9, truncated). For acyclic models with `max_len` at least the
    /// longest trace this is the *exact* language.
    #[must_use]
    pub fn language_upto(&self, max_len: usize) -> Language {
        match self {
            Powl::Leaf(None) => {
                let mut l = Language::new();
                l.insert(vec![]);
                l
            }
            Powl::Leaf(Some(a)) => {
                let mut l = Language::new();
                if max_len >= 1 {
                    l.insert(vec![a.clone()]);
                }
                l
            }
            Powl::PartialOrder { children, order } => {
                let child_langs: Vec<Language> =
                    children.iter().map(|c| c.language_upto(max_len)).collect();
                shuffle_language(&child_langs, order, max_len)
            }
            Powl::Choice { children, graph } => {
                let child_langs: Vec<Language> =
                    children.iter().map(|c| c.language_upto(max_len)).collect();
                choice_language(&child_langs, graph, max_len)
            }
            Powl::ExternalCut { region, .. } => region.language_upto(max_len),
        }
    }
}

impl ChoiceGraph {
    /// Successors of `node` in the graph (deterministic order).
    #[must_use]
    pub fn successors(&self, node: GNode) -> Vec<GNode> {
        self.edges
            .iter()
            .filter(|(u, _)| *u == node)
            .map(|(_, v)| *v)
            .collect()
    }
}

/// `L(γ(ψ₁,…))` bounded: union over all `▷→…→□` paths of the concatenation of
/// the visited children's languages, keeping traces of length `≤ max_len`.
/// Cyclic graphs terminate because a path visiting more than `max_len`
/// *labelled* children can only produce over-length traces (silent children
/// are capped separately to keep enumeration finite).
fn choice_language(child_langs: &[Language], graph: &ChoiceGraph, max_len: usize) -> Language {
    let mut out = Language::new();
    // (current node, accumulated language, path-length budget)
    let mut stack: Vec<(GNode, Vec<Trace>, usize)> = vec![(START, vec![vec![]], 0)];
    // Cap total nodes visited on a path to bound cyclic enumeration.
    let node_budget = max_len + 2;
    while let Some((node, acc, steps)) = stack.pop() {
        if steps > node_budget {
            continue;
        }
        for next in graph.successors(node) {
            match next {
                GNode::End => {
                    for t in &acc {
                        if t.len() <= max_len {
                            out.insert(t.clone());
                        }
                    }
                }
                GNode::Child(i) => {
                    let mut extended = Vec::new();
                    for prefix in &acc {
                        for suffix in &child_langs[i] {
                            let mut cat = prefix.clone();
                            cat.extend(suffix.iter().cloned());
                            if cat.len() <= max_len {
                                extended.push(cat);
                            }
                        }
                    }
                    if !extended.is_empty() {
                        stack.push((next, extended, steps + 1));
                    }
                }
                GNode::Start => {}
            }
        }
    }
    out
}

/// `L(≺(ψ₁,…))` bounded: for every choice of one trace per child, all
/// order-preserving interleavings (Def 3.8), keeping length `≤ max_len`.
fn shuffle_language(
    child_langs: &[Language],
    order: &BTreeSet<(usize, usize)>,
    max_len: usize,
) -> Language {
    let mut out = Language::new();
    let choices: Vec<Vec<Trace>> = child_langs
        .iter()
        .map(|l| l.iter().cloned().collect())
        .collect();
    let mut selection: Vec<Trace> = vec![vec![]; choices.len()];
    cartesian(&choices, 0, &mut selection, &mut |sel| {
        interleave(sel, order, max_len, &mut out);
    });
    out
}

/// Enumerate the cartesian product of one trace per child, invoking `f` on
/// each full selection.
fn cartesian(
    choices: &[Vec<Trace>],
    idx: usize,
    selection: &mut Vec<Trace>,
    f: &mut impl FnMut(&[Trace]),
) {
    if idx == choices.len() {
        f(selection);
        return;
    }
    for candidate in &choices[idx] {
        selection[idx] = candidate.clone();
        cartesian(choices, idx + 1, selection, f);
    }
}

/// Order-preserving shuffle (Def 3.8): child `i`'s next element may be emitted
/// only when every predecessor `j ≺ i` has been fully emitted.
fn interleave(
    seqs: &[Trace],
    order: &BTreeSet<(usize, usize)>,
    max_len: usize,
    out: &mut Language,
) {
    let n = seqs.len();
    let mut pos = vec![0usize; n];
    let mut acc = Vec::new();
    interleave_rec(seqs, order, &mut pos, &mut acc, max_len, out);
}

fn interleave_rec(
    seqs: &[Trace],
    order: &BTreeSet<(usize, usize)>,
    pos: &mut [usize],
    acc: &mut Trace,
    max_len: usize,
    out: &mut Language,
) {
    if acc.len() > max_len {
        return;
    }
    let n = seqs.len();
    if (0..n).all(|i| pos[i] == seqs[i].len()) {
        out.insert(acc.clone());
        return;
    }
    for i in 0..n {
        if pos[i] >= seqs[i].len() {
            continue;
        }
        // predecessors j ≺ i must be fully emitted
        let ready = (0..n).all(|j| !order.contains(&(j, i)) || pos[j] == seqs[j].len());
        if !ready {
            continue;
        }
        acc.push(seqs[i][pos[i]].clone());
        pos[i] += 1;
        interleave_rec(seqs, order, pos, acc, max_len, out);
        pos[i] -= 1;
        acc.pop();
    }
}
