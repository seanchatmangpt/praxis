//! Deterministic RDF graph over an in-memory oxigraph [`Store`].
//!
//! Provides [`DeterministicGraph`] — Turtle loading, SPARQL querying, and a
//! BLAKE3 `state_hash` computed over canonicalized, sorted N-Quads — plus a
//! [`Delta`] type describing additions/deletions between two graphs.
//!
//! Canonicalization: graphs without blank nodes are sorted lexicographically
//! by their N-Quads string form; graphs with blank nodes go through a bounded
//! color-refinement pass (5 iterations of BLAKE3 neighborhood signatures)
//! and blank nodes are relabelled `c14n{i}` in signature order so isomorphic
//! graphs hash identically regardless of blank-node labels.

use std::collections::{BTreeSet, HashMap, HashSet};

use oxigraph::{
    io::RdfFormat,
    model::{BlankNode, GraphName, NamedOrBlankNode, Quad, Term},
    sparql::{QueryResults, SparqlEvaluator},
    store::Store,
};

use crate::error::{AppError, Result};

/// Number of color-refinement iterations for blank-node canonicalization.
/// Five rounds are sufficient for the small ontology graphs ggen operates on.
const REFINEMENT_ITERATIONS: usize = 5;

/// An in-memory RDF store with deterministic state hashing.
pub struct DeterministicGraph {
    store: Store,
}

impl DeterministicGraph {
    /// Create an empty in-memory graph.
    ///
    /// # Errors
    /// Returns `[FM-GRAPH-001]` if the oxigraph store cannot be initialized.
    pub fn new() -> Result<Self> {
        let store = Store::new()
            .map_err(|e| AppError::fm_graph(1, format!("failed to create in-memory store: {e}")))?;
        Ok(Self { store })
    }

    /// Parse and insert Turtle content, returning the number of quads added.
    ///
    /// # Errors
    /// Returns `[FM-GRAPH-002]` on Turtle syntax or storage errors.
    pub fn insert_turtle(&self, ttl: &str) -> Result<usize> {
        let before = self
            .store
            .len()
            .map_err(|e| AppError::fm_graph(2, format!("store length unavailable: {e}")))?;
        self.store
            .load_from_slice(RdfFormat::Turtle, ttl)
            .map_err(|e| AppError::fm_graph(2, format!("turtle load failed: {e}")))?;
        let after = self
            .store
            .len()
            .map_err(|e| AppError::fm_graph(2, format!("store length unavailable: {e}")))?;
        Ok(after.saturating_sub(before))
    }

    /// Execute a SPARQL query against the graph.
    ///
    /// # Errors
    /// Returns `[FM-GRAPH-003]` on parse or evaluation errors.
    pub fn query(&self, sparql: &str) -> Result<QueryResults<'static>> {
        SparqlEvaluator::new()
            .parse_query(sparql)
            .map_err(|e| AppError::fm_graph(3, format!("SPARQL parse failed: {e}")))?
            .on_store(&self.store)
            .execute()
            .map_err(|e| AppError::fm_graph(3, format!("SPARQL evaluation failed: {e}")))
    }

    /// Return every quad currently in the store.
    ///
    /// # Errors
    /// Returns `[FM-GRAPH-004]` if iteration over the store fails.
    pub fn all_quads(&self) -> Result<Vec<Quad>> {
        self.store
            .iter()
            .collect::<std::result::Result<Vec<Quad>, _>>()
            .map_err(|e| AppError::fm_graph(4, format!("store iteration failed: {e}")))
    }

    /// BLAKE3 hash of the canonicalized graph state.
    ///
    /// The hash is computed over the sorted canonical N-Quads strings joined
    /// by `\n`. Isomorphic graphs (including blank-node relabelings) produce
    /// the same hash; insertion order never matters.
    ///
    /// # Errors
    /// Returns `[FM-GRAPH-004]`/`[FM-GRAPH-005]` on iteration or
    /// canonicalization failures.
    pub fn state_hash(&self) -> Result<[u8; 32]> {
        let quads = self.all_quads()?;
        let lines = canonical_nquad_lines(&quads)?;
        Ok(*blake3::hash(lines.join("\n").as_bytes()).as_bytes())
    }
}

/// A set difference between two graph states, expressed as canonical
/// N-Quads strings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Delta {
    /// Canonical N-Quads present in the target but not the baseline (sorted).
    pub additions: Vec<String>,
    /// Canonical N-Quads present in the baseline but not the target (sorted).
    pub deletions: Vec<String>,
}

impl Delta {
    /// Compute the delta that transforms `baseline` into `target`.
    ///
    /// # Errors
    /// Propagates canonicalization/iteration failures from either graph.
    pub fn compute(baseline: &DeterministicGraph, target: &DeterministicGraph) -> Result<Self> {
        let base: BTreeSet<String> =
            canonical_nquad_lines(&baseline.all_quads()?)?.into_iter().collect();
        let tgt: BTreeSet<String> =
            canonical_nquad_lines(&target.all_quads()?)?.into_iter().collect();
        Ok(Self {
            additions: tgt.difference(&base).cloned().collect(),
            deletions: base.difference(&tgt).cloned().collect(),
        })
    }

    /// Apply this delta to `graph`: remove the deletions, insert the additions.
    ///
    /// Deletions are matched by canonical N-Quads string against the graph's
    /// current canonicalization; additions are parsed as N-Quads and inserted.
    ///
    /// # Errors
    /// Returns `[FM-GRAPH-006]` if a deletion does not match any quad in the
    /// graph (fail closed), or on parse/storage errors while applying.
    pub fn apply(&self, graph: &DeterministicGraph) -> Result<()> {
        if !self.deletions.is_empty() {
            let quads = graph.all_quads()?;
            let pairs = canonical_pairs(&quads)?;
            let by_canonical: HashMap<&str, &Quad> =
                pairs.iter().map(|(s, q)| (s.as_str(), q)).collect();
            for del in &self.deletions {
                let quad = by_canonical.get(del.as_str()).ok_or_else(|| {
                    AppError::fm_graph(
                        6,
                        format!("deletion not present in graph (refusing partial apply): {del}"),
                    )
                })?;
                graph.store.remove(quad.as_ref()).map_err(|e| {
                    AppError::fm_graph(6, format!("failed to remove quad `{del}`: {e}"))
                })?;
            }
        }
        if !self.additions.is_empty() {
            // `Quad`'s Display form omits the terminating ` .` required by
            // the N-Quads grammar; add it per line before parsing.
            let doc: String = self.additions.iter().map(|a| format!("{a} .\n")).collect();
            graph.store.load_from_slice(RdfFormat::NQuads, doc.as_str()).map_err(|e| {
                AppError::fm_graph(6, format!("failed to insert delta additions: {e}"))
            })?;
        }
        Ok(())
    }

    /// The inverse delta: additions and deletions swapped, so that applying
    /// `self` then `self.inverse()` is a net no-op.
    #[must_use]
    pub fn inverse(&self) -> Self {
        Self { additions: self.deletions.clone(), deletions: self.additions.clone() }
    }

    /// Compose two deltas: the result of applying `self` first, then `other`.
    ///
    /// Cancellation semantics: an addition in `self` removed again by a
    /// deletion in `other` vanishes from the composite (and vice versa).
    /// Output vectors are sorted and deduplicated (deterministic).
    #[must_use]
    pub fn compose(&self, other: &Delta) -> Self {
        let a1: BTreeSet<&String> = self.additions.iter().collect();
        let d1: BTreeSet<&String> = self.deletions.iter().collect();
        let a2: BTreeSet<&String> = other.additions.iter().collect();
        let d2: BTreeSet<&String> = other.deletions.iter().collect();

        // Net additions: self's additions that survive other's deletions,
        // plus other's additions that are not merely undoing self's deletions.
        let additions: BTreeSet<String> = a1
            .iter()
            .filter(|s| !d2.contains(**s))
            .chain(a2.iter().filter(|s| !d1.contains(**s)))
            .map(|s| (**s).clone())
            .collect();
        // Net deletions, symmetrically.
        let deletions: BTreeSet<String> = d1
            .iter()
            .filter(|s| !a2.contains(**s))
            .chain(d2.iter().filter(|s| !a1.contains(**s)))
            .map(|s| (**s).clone())
            .collect();

        Self {
            additions: additions.into_iter().collect(),
            deletions: deletions.into_iter().collect(),
        }
    }

    /// True when the delta contains no additions and no deletions.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.additions.is_empty() && self.deletions.is_empty()
    }

    /// BLAKE3 hash of the delta: sorted additions prefixed `+`, then sorted
    /// deletions prefixed `-`.
    #[must_use]
    pub fn hash(&self) -> [u8; 32] {
        let mut additions = self.additions.clone();
        additions.sort();
        let mut deletions = self.deletions.clone();
        deletions.sort();
        let mut hasher = blake3::Hasher::new();
        for a in &additions {
            hasher.update(b"+");
            hasher.update(a.as_bytes());
        }
        for d in &deletions {
            hasher.update(b"-");
            hasher.update(d.as_bytes());
        }
        *hasher.finalize().as_bytes()
    }
}

/// Sorted canonical N-Quads lines for a quad slice.
fn canonical_nquad_lines(quads: &[Quad]) -> Result<Vec<String>> {
    Ok(canonical_pairs(quads)?.into_iter().map(|(s, _)| s).collect())
}

/// Canonicalize quads, returning `(canonical N-Quads string, original quad)`
/// pairs sorted by the canonical string.
///
/// Blank nodes are relabelled `c14n{i}` after bounded color refinement so
/// the canonical strings are stable across blank-node renamings.
fn canonical_pairs(quads: &[Quad]) -> Result<Vec<(String, Quad)>> {
    let blank_nodes = collect_blank_nodes(quads);

    let relabel: HashMap<BlankNode, BlankNode> = if blank_nodes.is_empty() {
        HashMap::new()
    } else {
        canonical_blank_node_map(quads, &blank_nodes)?
    };

    let mut pairs: Vec<(String, Quad)> =
        quads.iter().map(|q| (relabel_quad(q, &relabel).to_string(), q.clone())).collect();
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(pairs)
}

/// Collect every blank node appearing in subject, object, or graph position.
fn collect_blank_nodes(quads: &[Quad]) -> HashSet<BlankNode> {
    let mut blanks = HashSet::new();
    for q in quads {
        if let NamedOrBlankNode::BlankNode(b) = &q.subject {
            blanks.insert(b.clone());
        }
        if let Term::BlankNode(b) = &q.object {
            blanks.insert(b.clone());
        }
        if let GraphName::BlankNode(b) = &q.graph_name {
            blanks.insert(b.clone());
        }
    }
    blanks
}

/// Bounded color refinement: compute stable BLAKE3 neighborhood signatures
/// for every blank node, then map each to a `c14n{i}` label in signature
/// order (ties broken by original label for determinism within this graph).
fn canonical_blank_node_map(
    quads: &[Quad],
    blank_nodes: &HashSet<BlankNode>,
) -> Result<HashMap<BlankNode, BlankNode>> {
    let mut labels: HashMap<BlankNode, String> =
        blank_nodes.iter().map(|b| (b.clone(), "bnode".to_string())).collect();

    for _ in 0..REFINEMENT_ITERATIONS {
        let mut next: HashMap<BlankNode, String> = HashMap::new();
        for bnode in blank_nodes {
            let mut neighborhood: Vec<String> = quads
                .iter()
                .filter(|q| quad_touches(q, bnode))
                .map(|q| neighborhood_line(q, bnode, &labels))
                .collect();
            neighborhood.sort();
            let signature = blake3::hash(neighborhood.join("\n").as_bytes()).to_hex().to_string();
            next.insert(bnode.clone(), signature);
        }
        labels = next;
    }

    let mut ordered: Vec<&BlankNode> = blank_nodes.iter().collect();
    ordered.sort_by(|a, b| {
        let la = labels.get(*a).map_or("", String::as_str);
        let lb = labels.get(*b).map_or("", String::as_str);
        la.cmp(lb).then_with(|| a.to_string().cmp(&b.to_string()))
    });

    let mut map = HashMap::new();
    for (idx, bnode) in ordered.into_iter().enumerate() {
        let canonical = BlankNode::new(format!("c14n{idx}")).map_err(|e| {
            AppError::fm_graph(5, format!("canonical blank node label rejected: {e}"))
        })?;
        map.insert(bnode.clone(), canonical);
    }
    Ok(map)
}

/// Does `quad` mention `bnode` in any position?
fn quad_touches(quad: &Quad, bnode: &BlankNode) -> bool {
    matches!(&quad.subject, NamedOrBlankNode::BlankNode(b) if b == bnode)
        || matches!(&quad.object, Term::BlankNode(b) if b == bnode)
        || matches!(&quad.graph_name, GraphName::BlankNode(b) if b == bnode)
}

/// Render one neighborhood line for the signature of `bnode`, substituting
/// `_:self` for the node itself and current labels for other blank nodes.
fn neighborhood_line(
    quad: &Quad,
    bnode: &BlankNode,
    labels: &HashMap<BlankNode, String>,
) -> String {
    let blank_repr = |b: &BlankNode| -> String {
        if b == bnode {
            "_:self".to_string()
        } else {
            format!("_:{}", labels.get(b).map_or("", String::as_str))
        }
    };
    let s = match &quad.subject {
        NamedOrBlankNode::BlankNode(b) => blank_repr(b),
        NamedOrBlankNode::NamedNode(n) => n.to_string(),
    };
    let o = match &quad.object {
        Term::BlankNode(b) => blank_repr(b),
        other => other.to_string(),
    };
    let g = match &quad.graph_name {
        GraphName::DefaultGraph => String::new(),
        GraphName::NamedNode(n) => n.to_string(),
        GraphName::BlankNode(b) => blank_repr(b),
    };
    format!("{s} {} {o} {g}", quad.predicate)
}

/// Rewrite a quad's blank nodes through the canonical relabeling map.
/// Nodes absent from the map (i.e. non-blank positions) pass through.
fn relabel_quad(quad: &Quad, map: &HashMap<BlankNode, BlankNode>) -> Quad {
    let subject = match &quad.subject {
        NamedOrBlankNode::BlankNode(b) => {
            NamedOrBlankNode::BlankNode(map.get(b).cloned().unwrap_or_else(|| b.clone()))
        }
        other => other.clone(),
    };
    let object = match &quad.object {
        Term::BlankNode(b) => Term::BlankNode(map.get(b).cloned().unwrap_or_else(|| b.clone())),
        other => other.clone(),
    };
    let graph_name = match &quad.graph_name {
        GraphName::BlankNode(b) => {
            GraphName::BlankNode(map.get(b).cloned().unwrap_or_else(|| b.clone()))
        }
        other => other.clone(),
    };
    Quad::new(subject, quad.predicate.clone(), object, graph_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn delta(adds: &[&str], dels: &[&str]) -> Delta {
        Delta {
            additions: adds.iter().map(ToString::to_string).collect(),
            deletions: dels.iter().map(ToString::to_string).collect(),
        }
    }

    #[test]
    fn compose_with_inverse_is_empty_and_hashes_as_empty() {
        let d = delta(&["<http://e/a> <http://e/p> \"1\""], &["<http://e/b> <http://e/p> \"2\""]);
        let net = d.compose(&d.inverse());
        assert!(net.is_empty(), "delta ∘ delta⁻¹ must cancel: {net:?}");
        assert_eq!(
            net.hash(),
            Delta::default().hash(),
            "empty-composite hash must equal empty delta hash"
        );
    }

    #[test]
    fn inverse_is_an_involution() {
        let d = delta(&["a", "b"], &["c"]);
        assert_eq!(d.inverse().inverse(), d);
    }

    #[test]
    fn compose_cancels_crosswise_and_keeps_survivors_sorted() {
        // self adds x, deletes y; other deletes x (cancel), adds y (cancel),
        // and adds z (survives).
        let d1 = delta(&["x"], &["y"]);
        let d2 = delta(&["z", "y"], &["x"]);
        let net = d1.compose(&d2);
        assert_eq!(net.additions, vec!["z".to_string()]);
        assert!(net.deletions.is_empty());
    }

    #[test]
    fn empty_delta_is_empty() {
        assert!(Delta::default().is_empty());
        assert!(!delta(&["a"], &[]).is_empty());
    }
}
