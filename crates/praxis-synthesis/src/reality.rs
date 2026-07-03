//! Reality addressing — binding admitted referents to PUBLIC ontology
//! coordinates, not praxis-private vocabulary.
//!
//! Three address spaces exist in this crate:
//! - content address: exact bytes (`chatman_common::provenance::content_address`,
//!   `graph_hash`, `event_hash` — already implemented everywhere).
//! - reality address: **this module** — an admitted referent's binding to a
//!   public-ontology time/space/provenance coordinate. What thing, when,
//!   where, attested by what.
//! - receipt address: action standing (`firing.rs`'s outer chain — already
//!   implemented).
//!
//! Doctrine: praxis does not invent private time/space/provenance vocabulary.
//! It binds three predicates already used elsewhere in the admitted graph
//! (`OWL-Time`'s `inXSDDateTimeStamp`, `GeoSPARQL`'s `asWKT`, `PROV-O`'s
//! `wasAttributedTo`) to one content-addressed subject. A referent with none
//! of the three anchors is not a reality address — it is refused by name,
//! not silently accepted as "addressed."

use serde::{Deserialize, Serialize};

use chatman_common::provenance::content_address;

use crate::graph::{Object, Triple};
use crate::Refusal;

/// `OWL-Time` instant predicate: <http://www.w3.org/2006/time#inXSDDateTimeStamp>.
pub const TIME_PREDICATE: &str = "http://www.w3.org/2006/time#inXSDDateTimeStamp";
/// `GeoSPARQL` well-known-text geometry predicate: <http://www.opengis.net/ont/geosparql#asWKT>.
pub const SPACE_PREDICATE: &str = "http://www.opengis.net/ont/geosparql#asWKT";
/// `PROV-O` attribution predicate: <http://www.w3.org/ns/prov#wasAttributedTo>.
pub const PROVENANCE_PREDICATE: &str = "http://www.w3.org/ns/prov#wasAttributedTo";

/// A referent's binding to public-ontology coordinates, derived from the
/// admitted graph. At least one anchor must be present — a subject with zero
/// anchors is not addressed and `bind` refuses it by name rather than
/// returning an all-`None` record that would look like a valid (if sparse)
/// address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealityAddressRecord {
    /// The admitted subject IRI this record addresses.
    subject: String,
    /// `OWL-Time` timestamp literal, if the graph declares one.
    time_anchor: Option<String>,
    /// `GeoSPARQL` WKT literal, if the graph declares one.
    space_anchor: Option<String>,
    /// `PROV-O` attribution target IRI, if the graph declares one.
    provenance_anchor: Option<String>,
}

impl RealityAddressRecord {
    /// The admitted subject IRI this record addresses.
    #[must_use]
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// `OWL-Time` timestamp literal, if present.
    #[must_use]
    pub fn time_anchor(&self) -> Option<&str> {
        self.time_anchor.as_deref()
    }

    /// `GeoSPARQL` WKT literal, if present.
    #[must_use]
    pub fn space_anchor(&self) -> Option<&str> {
        self.space_anchor.as_deref()
    }

    /// `PROV-O` attribution target IRI, if present.
    #[must_use]
    pub fn provenance_anchor(&self) -> Option<&str> {
        self.provenance_anchor.as_deref()
    }

    /// Content address of the record's canonical (serde) rendering — the
    /// reality address's own standing hash. Computed, never asserted.
    pub fn reality_hash(&self) -> Result<String, Refusal> {
        let json = serde_json::to_string(self).map_err(|e| Refusal::InvalidInput {
            detail: format!("reality address record failed to serialize: {e}"),
        })?;
        Ok(content_address(json.as_bytes()))
    }

    /// Bind one subject's public-ontology anchors from the admitted post-state
    /// triples. Refuses (rather than returning an empty record) when NONE of
    /// the three anchors are present — an unanchored subject is not a reality
    /// address, it is bare graph content.
    pub fn bind(triples: &[Triple], subject: &str) -> Result<Self, Refusal> {
        let literal = |pred: &str| {
            triples.iter().find_map(|t| {
                (t.s == subject && t.p == pred).then(|| match &t.o {
                    Object::Str(s) => s.clone(),
                    Object::Iri(i) => i.clone(),
                    Object::Int(v) => v.to_string(),
                })
            })
        };
        let time_anchor = literal(TIME_PREDICATE);
        let space_anchor = literal(SPACE_PREDICATE);
        let provenance_anchor = literal(PROVENANCE_PREDICATE);
        if time_anchor.is_none() && space_anchor.is_none() && provenance_anchor.is_none() {
            return Err(Refusal::RealityAddressIllFormed {
                subject: subject.to_string(),
                detail: "no OWL-Time, GeoSPARQL, or PROV-O anchor found; not a reality address"
                    .to_string(),
            });
        }
        Ok(Self { subject: subject.to_string(), time_anchor, space_anchor, provenance_anchor })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::parse_ttl;

    const TTL: &str = r#"
@prefix time: <http://www.w3.org/2006/time#> .
@prefix geo: <http://www.opengis.net/ont/geosparql#> .
@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix ex: <http://e/> .
<http://e/event1> time:inXSDDateTimeStamp "2026-07-03T00:00:00Z" .
<http://e/event1> geo:asWKT "POINT(0 0)" .
<http://e/event1> prov:wasAttributedTo <http://e/agentA> .
<http://e/bare> ex:unrelated "no anchors" .
"#;

    #[test]
    fn bind_succeeds_with_all_three_anchors_and_hash_is_stable() {
        let triples = parse_ttl(TTL).unwrap();
        let r = RealityAddressRecord::bind(&triples, "http://e/event1").unwrap();
        assert_eq!(r.time_anchor(), Some("2026-07-03T00:00:00Z"));
        assert_eq!(r.space_anchor(), Some("POINT(0 0)"));
        assert_eq!(r.provenance_anchor(), Some("http://e/agentA"));
        let h1 = r.reality_hash().unwrap();
        let h2 = RealityAddressRecord::bind(&triples, "http://e/event1").unwrap().reality_hash().unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn bind_refuses_a_subject_with_zero_anchors() {
        let triples = parse_ttl(TTL).unwrap();
        match RealityAddressRecord::bind(&triples, "http://e/bare") {
            Err(Refusal::RealityAddressIllFormed { detail, .. }) => {
                assert!(detail.contains("not a reality address"));
            }
            other => panic!("expected RealityAddressIllFormed, got {other:?}"),
        }
    }

    #[test]
    fn bind_succeeds_with_a_single_anchor() {
        let ttl = r#"@prefix time: <http://www.w3.org/2006/time#> .
<http://e/x> time:inXSDDateTimeStamp "2026-01-01T00:00:00Z" ."#;
        let triples = parse_ttl(ttl).unwrap();
        let r = RealityAddressRecord::bind(&triples, "http://e/x").unwrap();
        assert!(r.space_anchor().is_none());
        assert!(r.provenance_anchor().is_none());
        assert_eq!(r.time_anchor(), Some("2026-01-01T00:00:00Z"));
    }

    #[test]
    fn distinct_anchor_sets_produce_distinct_hashes() {
        let triples = parse_ttl(TTL).unwrap();
        let full = RealityAddressRecord::bind(&triples, "http://e/event1").unwrap();
        let ttl2 = r#"@prefix time: <http://www.w3.org/2006/time#> .
<http://e/event1> time:inXSDDateTimeStamp "2026-07-03T00:00:00Z" ."#;
        let sparse_triples = parse_ttl(ttl2).unwrap();
        let sparse = RealityAddressRecord::bind(&sparse_triples, "http://e/event1").unwrap();
        assert_ne!(full.reality_hash().unwrap(), sparse.reality_hash().unwrap());
    }
}
