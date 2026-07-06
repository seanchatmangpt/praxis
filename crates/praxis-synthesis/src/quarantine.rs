//! Rice quarantine + admission — the only door into the admitted graph.
//!
//! Raw meaning is bytes until it survives the quarantine: [`RiceQuarantine`]
//! runs *decidable checks only* (the bounded Turtle parser, hard caps, the
//! closed `wf:` vocabulary) — no semantic evaluation of raw content, ever.
//! An LLM proposer is just one [`Origin`]; its output is never executable.
//!
//! [`Admission`] then judges the parsed delta against a [`Reference`] — the
//! current admitted graph — computing the post-state and its hash. The
//! logical epoch increments on admission; there is no wall clock anywhere.
//!
//! Lineage: praxis park/re-admission quarantine discipline; knhk
//! `refusal.rs` StructuralAdmissionGate; unrdf `admission/receipts.mjs`
//! (`afterHash = H(before + delta)` — computed, never asserted).

use serde::{Deserialize, Serialize};

use chatman_common::provenance::content_address;

use crate::delta::GraphDelta;
use crate::graph::{graph_hash, parse_ttl, vocab_check, Triple};
use crate::Refusal;

/// Where a meaning source came from. Provenance only — every origin passes
/// through the identical decidable checks; no origin is trusted more.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Origin {
    /// Typed directly by the operator.
    Operator,
    /// Emitted by a quarantined proposer (e.g. an LLM). Advisory until admitted.
    Proposer,
    /// Produced by an external event bridge (commit, conformance deviation).
    Bridge,
}

/// Raw candidate bytes plus declared origin. NEVER executable as-is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeaningSource {
    /// Declared provenance tag.
    pub origin: Origin,
    /// Candidate additions document (Turtle subset).
    pub adds_ttl: String,
    /// Candidate removals document (Turtle subset).
    pub removes_ttl: String,
}

/// The Rice boundary: only decidable properties are checked. Anything the
/// parser or caps cannot decide about the bytes is not decided — it is
/// refused with a typed [`Refusal`] naming the culprit.
pub struct RiceQuarantine;

impl RiceQuarantine {
    /// Inspect raw bytes: parse both documents through the bounded subset.
    /// Success yields a canonical [`GraphDelta`]; failure is the parser's
    /// own typed refusal. No semantic judgment happens here.
    pub fn inspect(source: &MeaningSource) -> Result<GraphDelta, Refusal> {
        GraphDelta::parse(&source.adds_ttl, &source.removes_ttl)
    }
}

/// The admitted base state a delta is judged against.
///
/// Fields are PRIVATE by adversarial-review law: a `Reference` exists only
/// via [`Reference::genesis`], so the hash is always COMPUTED from the
/// canonical triples, never asserted. (`Deserialize` is deliberately absent
/// — a wire-forged reference would bypass the constructor.)
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Reference {
    /// The admitted triples (sorted canonical order).
    triples: Vec<Triple>,
    /// Computed content address of the canonical form of `triples`.
    graph_hash: String,
    /// Logical epoch of this state (increments per admission; no wall clock).
    epoch: u64,
}

impl Reference {
    /// The admitted triples (sorted canonical order).
    #[must_use]
    pub fn triples(&self) -> &[Triple] {
        &self.triples
    }

    /// Computed content address of the canonical form of the triples.
    #[must_use]
    pub fn graph_hash(&self) -> &str {
        &self.graph_hash
    }

    /// Logical epoch of this state.
    #[must_use]
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Build the genesis reference (epoch 0) from a TTL document.
    pub fn genesis(ttl: &str) -> Result<Self, Refusal> {
        let mut triples = parse_ttl(ttl)?;
        vocab_check(&triples)?;
        triples.sort_unstable();
        triples.dedup();
        let graph_hash = graph_hash(&triples);
        Ok(Self {
            triples,
            graph_hash,
            epoch: 0,
        })
    }
}

/// The admission verdict rendered into the record. String-typed on the wire
/// so the record's serde rendering is a stable content-address input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdmissionVerdict {
    /// The delta was applied; the post-state hash is computed.
    Admitted,
    /// The delta was refused; the refusal is carried alongside.
    Refused,
}

/// The hashed admission record: every field computed, nothing asserted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmissionRecord {
    /// Epoch AFTER this admission (base epoch + 1).
    pub epoch: u64,
    /// Computed hash of the base graph the delta was judged against.
    pub base_graph_hash: String,
    /// Computed hash of the post-state graph (recomputed by applying the
    /// delta and re-canonicalizing — never taken from the event).
    pub post_graph_hash: String,
    /// The delta's computed event hash.
    pub event_hash: String,
    /// Verdict.
    pub verdict: AdmissionVerdict,
}

impl AdmissionRecord {
    /// Content address of the record's serde rendering.
    pub fn admission_hash(&self) -> Result<String, Refusal> {
        let json = serde_json::to_string(self).map_err(|e| Refusal::InvalidInput {
            detail: format!("admission record failed to serialize: {e}"),
        })?;
        Ok(content_address(json.as_bytes()))
    }
}

/// An admitted event: the record plus the new state it produced.
///
/// Fields are PRIVATE by adversarial-review law: an `AdmittedEvent` exists
/// only via [`Admission::admit`], so the type itself witnesses that the
/// post-state was computed by applying an inspected delta to a reference
/// under the closed-world vocabulary — a hand-built post-state cannot reach
/// [`crate::ground::ground_fired_action`]. (`Deserialize` is deliberately
/// absent for the same reason.)
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AdmittedEvent {
    /// The hashed admission record.
    record: AdmissionRecord,
    /// The post-state triples (sorted canonical order).
    post: Vec<Triple>,
    /// The delta that was admitted.
    delta: GraphDelta,
}

impl AdmittedEvent {
    /// The hashed admission record.
    #[must_use]
    pub fn record(&self) -> &AdmissionRecord {
        &self.record
    }

    /// The post-state triples (sorted canonical order).
    #[must_use]
    pub fn post(&self) -> &[Triple] {
        &self.post
    }

    /// The delta that was admitted.
    #[must_use]
    pub fn delta(&self) -> &GraphDelta {
        &self.delta
    }
}

/// The admission gate.
pub struct Admission;

impl Admission {
    /// Judge a quarantine-passed delta against the reference: apply it,
    /// enforce the closed-world vocabulary on the POST-state, recompute the
    /// post hash, and increment the logical epoch. Every failure is a typed
    /// [`Refusal`]; the caller receipts refusals (they are never silent).
    pub fn admit(reference: &Reference, delta: &GraphDelta) -> Result<AdmittedEvent, Refusal> {
        let post = delta.apply(&reference.triples)?;
        vocab_check(&post)?;
        let record = AdmissionRecord {
            epoch: reference.epoch + 1,
            base_graph_hash: reference.graph_hash.clone(),
            post_graph_hash: graph_hash(&post),
            event_hash: delta.event_hash(),
            verdict: AdmissionVerdict::Admitted,
        };
        Ok(AdmittedEvent {
            record,
            post,
            delta: delta.clone(),
        })
    }

    /// Render the refusal-path record for a delta that failed admission:
    /// the refusal is receipted with the base state and event hash bound in
    /// (post hash = base hash: nothing changed).
    #[must_use]
    pub fn refusal_record(reference: &Reference, delta: &GraphDelta) -> AdmissionRecord {
        AdmissionRecord {
            epoch: reference.epoch,
            base_graph_hash: reference.graph_hash.clone(),
            post_graph_hash: reference.graph_hash.clone(),
            event_hash: delta.event_hash(),
            verdict: AdmissionVerdict::Refused,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "@prefix ex: <http://e/> .\nex:a ex:p ex:b .\n";

    fn source(adds: &str, removes: &str) -> MeaningSource {
        MeaningSource {
            origin: Origin::Proposer,
            adds_ttl: adds.to_string(),
            removes_ttl: removes.to_string(),
        }
    }

    #[test]
    fn quarantine_admission_computes_post_hash_and_epoch() {
        let reference = Reference::genesis(BASE).unwrap();
        let delta = RiceQuarantine::inspect(&source("<http://e/x> <http://e/q> 1 .", "")).unwrap();
        let admitted = Admission::admit(&reference, &delta).unwrap();
        assert_eq!(admitted.record.epoch, 1);
        assert_eq!(admitted.record.base_graph_hash, reference.graph_hash);
        assert_eq!(admitted.record.post_graph_hash, graph_hash(&admitted.post));
        assert_ne!(admitted.record.post_graph_hash, reference.graph_hash);
        assert!(admitted.record.admission_hash().unwrap().len() > 16);
    }

    #[test]
    fn quarantine_refuses_malformed_bytes_decidably() {
        let r = RiceQuarantine::inspect(&source("<u:s> <u:p> [] .", ""));
        assert!(matches!(r, Err(Refusal::GraphMalformed { .. })));
    }

    #[test]
    fn post_state_vocab_violation_is_refused_at_admission() {
        let reference = Reference::genesis(BASE).unwrap();
        let delta = RiceQuarantine::inspect(&source(
            "@prefix wf: <http://seanchatmangpt.github.io/praxis/workflow#> .\n\
             <http://e/w> wf:specHash \"b3:deadbeef\" .",
            "",
        ))
        .unwrap();
        match Admission::admit(&reference, &delta) {
            Err(Refusal::UnknownPredicate { predicate, .. }) => {
                assert!(predicate.ends_with("specHash"));
            }
            other => panic!("expected UnknownPredicate at admission, got {other:?}"),
        }
    }

    #[test]
    fn refusal_record_binds_base_and_event_without_state_change() {
        let reference = Reference::genesis(BASE).unwrap();
        let delta = GraphDelta::parse("", "<http://e/ghost> <http://e/p> 1 .").unwrap();
        assert!(Admission::admit(&reference, &delta).is_err());
        let record = Admission::refusal_record(&reference, &delta);
        assert_eq!(record.verdict, AdmissionVerdict::Refused);
        assert_eq!(record.post_graph_hash, record.base_graph_hash);
        assert_eq!(record.epoch, reference.epoch);
    }
}
