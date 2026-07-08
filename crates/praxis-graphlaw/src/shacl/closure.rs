use super::index_utils::get_triples_by_predicate;
/// Subclass closure computation and bitset-based closure matrix
///
/// Implements efficient computation and storage of transitive closure
/// for subclass relationships in RDF ontologies.
use crate::tripleindex::TripleIndex;
use fixedbitset::FixedBitSet;
use std::collections::HashSet;

/// Compute and cache the transitive closure of rdfs:subClassOf relationships
pub struct SubclassClosure {
    ancestors: std::collections::HashMap<usize, HashSet<usize>>,
}

impl SubclassClosure {
    /// Create a new SubclassClosure from a data graph
    pub fn new(data: &TripleIndex, rdfs_subclass_of: usize) -> Self {
        let mut ancestors = std::collections::HashMap::new();
        let subclass_triples = get_triples_by_predicate(data, rdfs_subclass_of);

        let mut direct_parents: std::collections::HashMap<usize, Vec<usize>> =
            std::collections::HashMap::new();
        for (sub, parent) in subclass_triples {
            direct_parents.entry(sub).or_default().push(parent);
        }

        let mut keys_to_compute: Vec<usize> = direct_parents.keys().cloned().collect();
        for parents in direct_parents.values() {
            for &p in parents {
                keys_to_compute.push(p);
            }
        }
        keys_to_compute.sort_unstable();
        keys_to_compute.dedup();

        for &class in &keys_to_compute {
            Self::compute_ancestors(class, &direct_parents, &mut ancestors);
        }

        SubclassClosure { ancestors }
    }

    fn compute_ancestors(
        class: usize,
        direct_parents: &std::collections::HashMap<usize, Vec<usize>>,
        ancestors: &mut std::collections::HashMap<usize, HashSet<usize>>,
    ) -> HashSet<usize> {
        if let Some(cached) = ancestors.get(&class) {
            return cached.clone();
        }

        let mut visited = HashSet::new();
        let mut queue = vec![class];
        visited.insert(class);

        let mut i = 0;
        while i < queue.len() {
            let curr = queue[i];
            i += 1;
            if let Some(parents) = direct_parents.get(&curr) {
                for &p in parents {
                    if visited.insert(p) {
                        queue.push(p);
                    }
                }
            }
        }

        ancestors.insert(class, visited.clone());
        visited
    }

    /// Check if `sub` is a subclass of `parent` (including reflexive case)
    pub fn is_subclass(&self, sub: usize, parent: usize) -> bool {
        if sub == parent {
            return true;
        }
        if let Some(ancestors) = self.ancestors.get(&sub) {
            ancestors.contains(&parent)
        } else {
            false
        }
    }
}

/// Bitset-based transitive closure matrix for dense closure sites.
/// Replaces HashMap<usize, HashSet<usize>> for better performance and
/// memory density when closure cardinality is high (> 80% of ID space used).
///
/// PROJ-409 Canonical Rendering Rule: Raw bitset memory is NEVER hashed.
/// Only the sorted edge list (render_canonical() output) is used for BLAKE3
/// hashing. This ensures platform-independence and determinism.
#[derive(Debug, Clone)]
pub struct ClosureMatrix {
    /// One bitset per row: matrix[from_id] contains all reachable nodes
    matrix: Vec<FixedBitSet>,
    /// Highest ID in the closure
    pub max_id: u32,
}

impl ClosureMatrix {
    /// Create a new ClosureMatrix with capacity for up to max_id+1 nodes
    pub fn new(max_id: u32) -> Self {
        let capacity = (max_id + 1) as usize;
        ClosureMatrix {
            matrix: vec![FixedBitSet::with_capacity(capacity); capacity],
            max_id,
        }
    }

    /// Add a direct edge (from → to) and transitively close if needed
    pub fn add_edge(&mut self, from: usize, to: usize) {
        if from <= self.max_id as usize && to <= self.max_id as usize {
            self.matrix[from].insert(to);
        }
    }

    /// Get all reachable nodes from a source ID (as a FixedBitSet reference)
    pub fn reachable(&self, from: usize) -> Option<&FixedBitSet> {
        if from <= self.max_id as usize {
            Some(&self.matrix[from])
        } else {
            None
        }
    }

    /// Check if `to` is reachable from `from`
    pub fn is_reachable(&self, from: usize, to: usize) -> bool {
        if from <= self.max_id as usize && to <= self.max_id as usize {
            self.matrix[from].contains(to)
        } else {
            false
        }
    }

    /// Compute transitive closure using iterative fixpoint (Floyd-Warshall-like)
    /// until no new edges are discovered.
    pub fn compute_transitive_closure(&mut self) {
        let num_nodes = self.matrix.len();
        let mut changed = true;
        while changed {
            changed = false;
            for from in 0..num_nodes {
                // Collect all current reachable nodes from `from` into a vec
                // (can't borrow matrix while modifying it)
                let reachable: Vec<usize> = self.matrix[from].ones().collect();
                for via in reachable {
                    if via < num_nodes {
                        // Union self.matrix[via] into self.matrix[from]
                        let via_reachable = self.matrix[via].clone();
                        for target in via_reachable.ones() {
                            if !self.matrix[from].contains(target) {
                                self.matrix[from].insert(target);
                                changed = true;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Render the closure as a sorted list of edges for deterministic hashing.
    /// PROJ-409 Canonical Rendering Rule: this is the ONLY form used for hashing,
    /// never the raw bitset memory.
    pub fn render_canonical(&self) -> Vec<(u32, u32)> {
        let mut edges = Vec::new();
        for (from_id, bitset) in self.matrix.iter().enumerate() {
            for to_id in bitset.ones() {
                edges.push((from_id as u32, to_id as u32));
            }
        }
        edges.sort_unstable();
        edges
    }
}

/// Check if a value node has a given class (checking the subclass hierarchy)
pub(crate) fn has_class(
    data: &TripleIndex,
    x: usize,
    class: usize,
    rdf_type: usize,
    closure: &SubclassClosure,
) -> bool {
    let types = super::index_utils::get_objects(data, x, rdf_type);
    for t in types {
        if closure.is_subclass(t, class) {
            return true;
        }
    }
    false
}
