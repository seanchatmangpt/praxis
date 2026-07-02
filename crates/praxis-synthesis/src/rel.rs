//! Columnar sorted relations with structural atom identity (Lane 1 of the
//! phase-change program).
//!
//! An atom's identity is its full `(pred, [u32; 8])` tuple — 8-capped arity
//! (the byte-governor doctrine) makes identity *structural*, eliminating the
//! 64-bit packed-key birthday problem entirely: there is nothing to collide.
//!
//! Storage per predicate is a lexicographically sorted `Vec<[u32; 8]>` plus a
//! small unsorted tail; probes with a bound *prefix* of positions binary-search
//! the sorted range (the qlever move `pddl-index` made for grounding,
//! generalized), and the tail is merged in amortized batches. Semi-naive
//! deltas stay flat vectors — cache-linear.

use serde::{Deserialize, Serialize};

use crate::Refusal;

/// Hard arity cap — the doctrine's namesake. Structural identity fits in
/// eight u32 lanes; unused lanes are zero and excluded by `arity`.
pub const ARITY_CAP: usize = 8;

/// A ground tuple: eight u32 lanes, `arity` of them significant.
pub type Tuple = [u32; ARITY_CAP];

/// Pack a slice (len ≤ 8) into a [`Tuple`]. Returns `None` past the cap.
#[must_use]
pub fn pack(args: &[u32]) -> Option<Tuple> {
    if args.len() > ARITY_CAP {
        return None;
    }
    let mut t = [0u32; ARITY_CAP];
    t[..args.len()].copy_from_slice(args);
    Some(t)
}

/// One predicate's relation: sorted body + unsorted tail, merged in batches.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Rel {
    arity: u8,
    /// Lexicographically sorted, deduplicated.
    sorted: Vec<Tuple>,
    /// Recent inserts not yet merged (kept small relative to `sorted`).
    tail: Vec<Tuple>,
}

impl Rel {
    /// Empty relation of the given arity.
    #[must_use]
    pub fn new(arity: u8) -> Self {
        Self { arity, sorted: Vec::new(), tail: Vec::new() }
    }

    /// Declared arity.
    #[must_use]
    pub fn arity(&self) -> u8 {
        self.arity
    }

    /// Total tuples (sorted + tail).
    #[must_use]
    pub fn len(&self) -> usize {
        self.sorted.len() + self.tail.len()
    }

    /// Whether the relation is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sorted.is_empty() && self.tail.is_empty()
    }

    /// Exact membership.
    #[must_use]
    pub fn contains(&self, t: &Tuple) -> bool {
        self.sorted.binary_search(t).is_ok() || self.tail.contains(t)
    }

    /// Insert; returns `true` if newly added. Amortizes tail merges so the
    /// tail stays ≤ √-ish of the body (merge when tail² > body).
    pub fn insert(&mut self, t: Tuple) -> bool {
        if self.contains(&t) {
            return false;
        }
        self.tail.push(t);
        if self.tail.len() * self.tail.len() > self.sorted.len().max(64) {
            self.merge_tail();
        }
        true
    }

    /// Force-merge the tail into the sorted body.
    pub fn merge_tail(&mut self) {
        if self.tail.is_empty() {
            return;
        }
        self.sorted.append(&mut self.tail);
        self.sorted.sort_unstable();
        self.sorted.dedup();
    }

    /// Iterate all tuples in deterministic (sorted-then-tail) order.
    /// Call [`merge_tail`](Self::merge_tail) first for fully sorted order.
    pub fn iter(&self) -> impl Iterator<Item = &Tuple> {
        self.sorted.iter().chain(self.tail.iter())
    }

    /// Tuples whose first `k` lanes equal `prefix[..k]`: binary-searched range
    /// over the sorted body plus a filtered scan of the (small) tail.
    pub fn prefix_range<'a>(
        &'a self,
        prefix: &'a Tuple,
        k: usize,
    ) -> impl Iterator<Item = &'a Tuple> + 'a {
        let (lo, hi) = if k == 0 {
            (0, self.sorted.len())
        } else {
            let lo = self.sorted.partition_point(|t| t[..k] < prefix[..k]);
            let hi = self.sorted.partition_point(|t| t[..k] <= prefix[..k]);
            (lo, hi)
        };
        self.sorted[lo..hi]
            .iter()
            .chain(self.tail.iter().filter(move |t| t[..k] == prefix[..k]))
    }
}

/// A keyed family of relations with a global tuple budget.
#[derive(Debug, Clone, Default)]
pub struct RelStore {
    rels: std::collections::BTreeMap<u32, Rel>,
    len: usize,
    cap: u64,
}

impl RelStore {
    /// Empty store with the given global tuple cap.
    #[must_use]
    pub fn with_cap(cap: u64) -> Self {
        Self { rels: std::collections::BTreeMap::new(), len: 0, cap }
    }

    /// Total tuples across all relations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the store holds no tuples.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The relation for `pred`, if any.
    #[must_use]
    pub fn rel(&self, pred: u32) -> Option<&Rel> {
        self.rels.get(&pred)
    }

    /// Sorted predicate ids present.
    pub fn preds(&self) -> impl Iterator<Item = u32> + '_ {
        self.rels.keys().copied()
    }

    /// Exact membership.
    #[must_use]
    pub fn contains(&self, pred: u32, t: &Tuple) -> bool {
        self.rels.get(&pred).is_some_and(|r| r.contains(t))
    }

    /// Insert under the global cap. `Ok(true)` if newly added; refuses (with
    /// counts) when the cap would be exceeded. Arity is fixed by first insert;
    /// mismatches are refused.
    pub fn insert(&mut self, pred: u32, arity: u8, t: Tuple) -> Result<bool, Refusal> {
        if usize::from(arity) > ARITY_CAP {
            return Err(Refusal::InvalidInput {
                detail: format!("arity {arity} exceeds ARITY_CAP ({ARITY_CAP})"),
            });
        }
        if self.len as u64 >= self.cap {
            return Err(Refusal::TupleCapExceeded {
                derived: self.len as u64,
                cap: self.cap,
                iteration: 0,
            });
        }
        let rel = self.rels.entry(pred).or_insert_with(|| Rel::new(arity));
        if rel.arity() != arity {
            return Err(Refusal::InvalidInput {
                detail: format!(
                    "predicate {pred} arity mismatch: declared {}, got {arity}",
                    rel.arity()
                ),
            });
        }
        let added = rel.insert(t);
        if added {
            self.len += 1;
        }
        Ok(added)
    }

    /// Merge every relation's tail (fully sorted state; do before hashing).
    pub fn merge_all(&mut self) {
        for rel in self.rels.values_mut() {
            rel.merge_tail();
        }
    }
}
