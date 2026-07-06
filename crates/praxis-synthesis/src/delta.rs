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

use crate::graph::{
    canonical_form, parse_ttl, render_object, Object, Triple, MAX_IRI_LEN, MAX_LIT_LEN, MAX_TRIPLES,
};
use crate::Refusal;

/// Hard cap on triples per delta side (additions or removals).
pub const MAX_DELTA_TRIPLES: usize = 64;

/// Bytes an `IRIREF` may never contain — mirrors the lexer's own exclusion
/// set (`lex_iriref` in `graph.rs`) plus `<`/`>` themselves, which the
/// lexer never has to reject explicitly because a bare `<` cannot occur
/// inside a real `IRIREF` token (it would already have ended the previous
/// one) and `>` terminates it. A hand-built [`Triple`] has no such
/// guarantee, so both are checked here explicitly.
const IRI_FORBIDDEN_BYTES: [u8; 10] = [
    b'<', b'>', b' ', b'\t', b'\n', b'\r', b'"', b'{', b'}', b'|',
];

/// Re-run the decidable caps and delimiter-safety checks the lexer would
/// have enforced on a parsed IRI/literal, against a hand-built term. This
/// is what makes [`GraphDelta::from_triples`] as strict as [`GraphDelta::parse`]:
/// without it, a caller could construct a [`Triple`] directly (both fields
/// are public, by design, so callers can build receipts/specs from parsed
/// data) and hand it to `from_triples`, skipping every bound the Turtle
/// front end exists to enforce.
fn validate_term_shape(t: &Triple) -> Result<(), Refusal> {
    validate_iri_term(&t.s)?;
    validate_iri_term(&t.p)?;
    match &t.o {
        Object::Iri(iri) => validate_iri_term(iri)?,
        Object::Str(s) => {
            if s.len() > MAX_LIT_LEN {
                return Err(Refusal::GraphCapExceeded {
                    what: "lit_len".to_string(),
                    cap: MAX_LIT_LEN as u64,
                    actual: s.len() as u64,
                });
            }
        }
        Object::Int(_) => {}
    }
    Ok(())
}

fn validate_iri_term(iri: &str) -> Result<(), Refusal> {
    if iri.len() > MAX_IRI_LEN {
        return Err(Refusal::GraphCapExceeded {
            what: "iri_len".to_string(),
            cap: MAX_IRI_LEN as u64,
            actual: iri.len() as u64,
        });
    }
    if iri
        .bytes()
        .any(|b| IRI_FORBIDDEN_BYTES.contains(&b) || b < 0x20)
    {
        return Err(Refusal::InvalidInput {
            detail: format!(
                "IRI term contains a delimiter-unsafe or control byte, which a parsed \
                 IRIREF can never contain: {iri:?}"
            ),
        });
    }
    Ok(())
}

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
        for t in additions.iter().chain(removals.iter()) {
            validate_term_shape(t)?;
        }
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
        Ok(Self {
            additions,
            removals,
        })
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

    /// Adversarial finding: `from_triples` is a public constructor that
    /// bypasses the lexer entirely, so a hand-built `Triple` used to skip
    /// `MAX_LIT_LEN`/`MAX_IRI_LEN` — a caller could hand-construct an
    /// oversized literal that a real parse could never produce and have it
    /// admitted downstream as if it had passed through `RiceQuarantine`.
    #[test]
    fn hand_built_triple_cannot_bypass_length_caps() {
        let oversized = Triple {
            s: "http://e/s".to_string(),
            p: "http://e/p".to_string(),
            o: Object::Str("x".repeat(MAX_LIT_LEN + 1)),
        };
        match GraphDelta::from_triples(vec![oversized], Vec::new()) {
            Err(Refusal::GraphCapExceeded { what, .. }) => assert_eq!(what, "lit_len"),
            other => panic!("expected lit_len cap refusal, got {other:?}"),
        }

        let oversized_iri = Triple {
            s: "http://e/".to_string() + &"x".repeat(MAX_IRI_LEN + 1),
            p: "http://e/p".to_string(),
            o: Object::Int(1),
        };
        match GraphDelta::from_triples(vec![oversized_iri], Vec::new()) {
            Err(Refusal::GraphCapExceeded { what, .. }) => assert_eq!(what, "iri_len"),
            other => panic!("expected iri_len cap refusal, got {other:?}"),
        }
    }

    /// Adversarial finding: canonicalization injection — a hand-built
    /// `Triple` whose subject smuggles a `<`/`>` delimiter can render to
    /// bytes indistinguishable from two separate, legitimately-parsed
    /// triples, breaking `canonical_form`'s soundness claim ("every term is
    /// ground/delimiter-safe"). A real parse can never produce such a term
    /// (the lexer's IRIREF grammar refuses it), so `from_triples` must
    /// refuse it too.
    #[test]
    fn hand_built_triple_cannot_smuggle_delimiter_characters() {
        let smuggled = Triple {
            s: "http://e/a> <http://e/b".to_string(),
            p: "http://e/c".to_string(),
            o: Object::Iri("http://e/d".to_string()),
        };
        match GraphDelta::from_triples(vec![smuggled], Vec::new()) {
            Err(Refusal::InvalidInput { detail }) => {
                assert!(detail.contains("delimiter"), "detail: {detail}");
            }
            other => panic!("expected InvalidInput refusal, got {other:?}"),
        }
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
