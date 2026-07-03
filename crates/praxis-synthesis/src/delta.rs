//! Graph deltas — the only shape in which change reaches the admitted graph.
//!
//! A [`GraphDelta`] is two sorted, deduplicated triple sets (additions and
//! removals) parsed through the same bounded Turtle subset as the base graph.
//! Its `event_hash` is computed from the delta's canonical form — never
//! asserted — mirroring the `graph_hash`/`ttl_hash` doctrine: the exact
//! surface bytes remain nameable via [`delta_ttl_hash`], which is a receipt
//! field only and is never folded into any chain.

use serde::Serialize;

use chatman_common::provenance::content_address;

use crate::graph::{canonical_form, parse_ttl, render_object, Triple, MAX_TRIPLES};
use crate::Refusal;

/// Hard cap on triples per delta side (additions or removals).
pub const MAX_DELTA_TRIPLES: usize = 64;

/// A canonical graph delta: sorted, deduplicated additions and removals.
///
/// Fields are PRIVATE by adversarial-review law: an instance is obtainable
/// only through [`GraphDelta::from_triples`] / [`GraphDelta::parse`], so the
/// type itself witnesses that the per-delta caps and the assert-and-retract
/// exclusion held. (`Deserialize` is deliberately absent — a wire-forged
/// delta would bypass the constructors.)
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GraphDelta {
    /// Triples asserted by this event (sorted, deduplicated).
    additions: Vec<Triple>,
    /// Triples retracted by this event (sorted, deduplicated).
    removals: Vec<Triple>,
}

fn render_triple(t: &Triple) -> String {
    format!("<{}> <{}> {} .", t.s, t.p, render_object(&t.o))
}

impl GraphDelta {
    /// Triples asserted by this event (sorted, deduplicated).
    #[must_use]
    pub fn additions(&self) -> &[Triple] {
        &self.additions
    }

    /// Triples retracted by this event (sorted, deduplicated).
    #[must_use]
    pub fn removals(&self) -> &[Triple] {
        &self.removals
    }

    /// Build a delta from raw triple lists: sorts, dedups, enforces caps,
    /// and refuses a triple appearing on both sides (an event may not
    /// simultaneously assert and retract the same fact).
    pub fn from_triples(additions: Vec<Triple>, removals: Vec<Triple>) -> Result<Self, Refusal> {
        let mut additions = additions;
        let mut removals = removals;
        additions.sort_unstable();
        additions.dedup();
        removals.sort_unstable();
        removals.dedup();
        if additions.len() > MAX_DELTA_TRIPLES {
            return Err(Refusal::GraphCapExceeded {
                what: "delta_additions".to_string(),
                cap: MAX_DELTA_TRIPLES as u64,
                actual: additions.len() as u64,
            });
        }
        if removals.len() > MAX_DELTA_TRIPLES {
            return Err(Refusal::GraphCapExceeded {
                what: "delta_removals".to_string(),
                cap: MAX_DELTA_TRIPLES as u64,
                actual: removals.len() as u64,
            });
        }
        if let Some(both) = additions.iter().find(|t| removals.binary_search(t).is_ok()) {
            return Err(Refusal::InvalidInput {
                detail: format!(
                    "delta asserts and retracts the same triple: {}",
                    render_triple(both)
                ),
            });
        }
        Ok(Self { additions, removals })
    }

    /// Parse a delta from two Turtle-subset documents (additions, removals).
    /// Every parse failure is the same typed [`Refusal`] family as the base
    /// graph parser — decidable checks only.
    pub fn parse(adds_ttl: &str, removes_ttl: &str) -> Result<Self, Refusal> {
        Self::from_triples(parse_ttl(adds_ttl)?, parse_ttl(removes_ttl)?)
    }

    /// The delta's canonical form: labeled canonical N-Triples sections.
    #[must_use]
    pub fn canonical_form(&self) -> String {
        format!(
            "additions\n{}removals\n{}",
            canonical_form(&self.additions),
            canonical_form(&self.removals)
        )
    }

    /// Content address of the canonical form — the event's law-hash.
    /// Computed, never asserted.
    #[must_use]
    pub fn event_hash(&self) -> String {
        content_address(self.canonical_form().as_bytes())
    }

    /// Apply the delta to a base graph. Removing a triple absent from the
    /// base is a typed [`Refusal::AdmissionRefused`] naming the triple —
    /// retracting what was never admitted would silently rewrite history.
    /// The post-state is sorted, deduplicated, and re-capped.
    pub fn apply(&self, base: &[Triple]) -> Result<Vec<Triple>, Refusal> {
        let mut post: Vec<Triple> = base.to_vec();
        post.sort_unstable();
        post.dedup();
        for r in &self.removals {
            match post.binary_search(r) {
                Ok(i) => {
                    post.remove(i);
                }
                Err(_) => {
                    return Err(Refusal::AdmissionRefused {
                        subject: render_triple(r),
                        detail: "removal of a triple not present in the base graph".to_string(),
                    })
                }
            }
        }
        for a in &self.additions {
            if let Err(i) = post.binary_search(a) {
                post.insert(i, a.clone());
            }
        }
        if post.len() > MAX_TRIPLES {
            return Err(Refusal::GraphCapExceeded {
                what: "triples".to_string(),
                cap: MAX_TRIPLES as u64,
                actual: post.len() as u64,
            });
        }
        Ok(post)
    }
}

/// Content address of the exact raw surface bytes of both delta documents.
/// A receipt field only — never folded into any chain.
#[must_use]
pub fn delta_ttl_hash(adds_ttl: &str, removes_ttl: &str) -> String {
    content_address(format!("adds\n{adds_ttl}\nremoves\n{removes_ttl}").as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "@prefix ex: <http://e/> .\nex:a ex:p ex:b .\nex:a ex:q 1 .\n";

    #[test]
    fn event_hash_is_surface_invariant_and_ttl_hash_is_not() {
        let d1 = GraphDelta::parse("@prefix ex: <http://e/> .\nex:x ex:p ex:y .", "").unwrap();
        let d2 = GraphDelta::parse("<http://e/x>   <http://e/p>   <http://e/y> .", "").unwrap();
        assert_eq!(d1.event_hash(), d2.event_hash());
        assert_ne!(
            delta_ttl_hash("@prefix ex: <http://e/> .\nex:x ex:p ex:y .", ""),
            delta_ttl_hash("<http://e/x>   <http://e/p>   <http://e/y> .", "")
        );
    }

    #[test]
    fn apply_adds_removes_and_refuses_phantom_removal() {
        let base = parse_ttl(BASE).unwrap();
        let d = GraphDelta::parse(
            "<http://e/n> <http://e/p> 2 .",
            "@prefix ex: <http://e/> .\nex:a ex:q 1 .",
        )
        .unwrap();
        let post = d.apply(&base).unwrap();
        assert_eq!(post.len(), 2);

        let phantom =
            GraphDelta::parse("", "@prefix ex: <http://e/> .\nex:ghost ex:p ex:z .").unwrap();
        match phantom.apply(&base) {
            Err(Refusal::AdmissionRefused { detail, .. }) => {
                assert!(detail.contains("not present"));
            }
            other => panic!("expected AdmissionRefused, got {other:?}"),
        }
    }

    #[test]
    fn add_and_remove_same_triple_is_refused() {
        let r = GraphDelta::parse(
            "<http://e/a> <http://e/p> <http://e/b> .",
            "<http://e/a> <http://e/p> <http://e/b> .",
        );
        assert!(matches!(r, Err(Refusal::InvalidInput { .. })));
    }

    #[test]
    fn delta_cap_fires() {
        let mut adds = String::new();
        for i in 0..=MAX_DELTA_TRIPLES {
            adds.push_str(&format!("<http://e/s{i}> <http://e/p> {i} .\n"));
        }
        match GraphDelta::parse(&adds, "") {
            Err(Refusal::GraphCapExceeded { what, .. }) => assert_eq!(what, "delta_additions"),
            other => panic!("expected cap refusal, got {other:?}"),
        }
    }
}
