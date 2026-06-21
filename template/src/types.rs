//! Core domain types for {{project-name}}.
//!
//! **Rule:** This module holds data shapes only — no business logic, no I/O.
//! Logic belongs in dedicated modules (`chain.rs`, `verifier.rs`, etc.).

use serde::{Deserialize, Serialize};
use std::fmt;

// ─── Content Addressing ────────────────────────────────────────────────────

/// A BLAKE3 digest rendered as a lowercase hex string.
///
/// Stored as hex so receipts serialize to canonical, human-diffable JSON.
/// Use [`Blake3Hash::content_address`] to hash arbitrary bytes, and
/// [`Blake3Hash::from_hex`] when round-tripping through JSON.
///
/// # Examples
///
/// ```rust
/// use {{project-name}}::Blake3Hash;
///
/// let h = Blake3Hash::content_address(b"hello world");
/// assert_eq!(h.as_hex().len(), 64);
///
/// let round_tripped = Blake3Hash::from_hex(h.as_hex());
/// assert_eq!(h, round_tripped);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Blake3Hash(pub String);

impl Blake3Hash {
    /// Compute the BLAKE3 digest of `bytes` and return it as a hex-encoded hash.
    ///
    /// This is the primary constructor for new digests. Use [`from_hex`] only
    /// when deserializing a digest that was previously computed.
    pub fn content_address(bytes: &[u8]) -> Self {
        Blake3Hash(blake3::hash(bytes).to_hex().to_string())
    }

    /// Construct from an already-computed lowercase hex string.
    ///
    /// Used during deserialization round-trips where the digest is already known.
    /// No validation is performed on the input; malformed hex will surface later
    /// when the digest is compared or re-verified.
    pub fn from_hex(hex: impl Into<String>) -> Self {
        Blake3Hash(hex.into())
    }

    /// Borrow the lowercase hex representation of this hash.
    pub fn as_hex(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Blake3Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for Blake3Hash {
    fn from(s: String) -> Self {
        Blake3Hash(s)
    }
}

impl From<Blake3Hash> for String {
    fn from(h: Blake3Hash) -> Self {
        h.0
    }
}

// ─── Object References ─────────────────────────────────────────────────────

/// A qualified reference to an object within an operation-event.
///
/// Object IDs follow the pattern `id:type[:qualifier]` but are stored as
/// separate fields so consumers can filter on `type_` without string parsing.
///
/// # Examples
///
/// ```rust
/// use {{project-name}}::ObjectRef;
///
/// let obj = ObjectRef {
///     id: "artifact-1".to_string(),
///     type_: "artifact".to_string(),
///     qualifier: Some("input".to_string()),
/// };
/// assert_eq!(obj.to_string(), "artifact-1:artifact:input");
///
/// let bare = ObjectRef { id: "repo:main".to_string(), type_: "git".to_string(), qualifier: None };
/// assert_eq!(bare.to_string(), "repo:main:git");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectRef {
    /// Stable identifier of the referenced object.
    pub id: String,
    /// OCEL object type (the class of the object). Renamed from `type` in JSON
    /// to avoid the Rust keyword.
    #[serde(rename = "type")]
    pub type_: String,
    /// Optional qualifier describing the role of this object in the event
    /// (e.g., `"input"`, `"output"`, `"subject"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qualifier: Option<String>,
}

impl fmt::Display for ObjectRef {
    /// Renders as `id:type` or `id:type:qualifier`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.qualifier {
            Some(q) => write!(f, "{}:{}:{}", self.id, self.type_, q),
            None => write!(f, "{}:{}", self.id, self.type_),
        }
    }
}

// ─── Canonical Serialization ────────────────────────────────────────────────

/// Produce deterministic, sorted-key JSON bytes for any serializable value.
///
/// This is the canonical byte form used for content addressing and chain hashing.
/// Object keys are recursively sorted so the same logical value always produces
/// identical bytes regardless of in-memory field order or Rust struct declaration
/// order.
///
/// Pair with [`Blake3Hash::content_address`] to get a stable digest:
///
/// ```rust
/// use {{project-name}}::{canonical_bytes, Blake3Hash};
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Payload { b: i32, a: i32 }
///
/// let bytes = canonical_bytes(&Payload { b: 2, a: 1 }).unwrap();
/// // JSON is {"a":1,"b":2} — keys sorted, no whitespace
/// let hash = Blake3Hash::content_address(&bytes);
/// assert_eq!(hash.as_hex().len(), 64);
/// ```
///
/// # Errors
///
/// Returns `serde_json::Error` if the value cannot be serialized to JSON.
pub fn canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
    let v = serde_json::to_value(value)?;
    let sorted = sort_value(v);
    serde_json::to_vec(&sorted)
}

/// Recursively sort the keys of all JSON objects within a [`serde_json::Value`].
fn sort_value(value: serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    match value {
        Value::Object(map) => {
            // Collect into BTreeMap to impose deterministic lexicographic key order,
            // then rebuild as a serde_json::Map (preserves insertion order in output).
            let sorted: std::collections::BTreeMap<String, Value> =
                map.into_iter().map(|(k, v)| (k, sort_value(v))).collect();
            let mut out = serde_json::Map::new();
            for (k, v) in sorted {
                out.insert(k, v);
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(sort_value).collect()),
        other => other,
    }
}

// ─── Seal Pattern (doc example) ────────────────────────────────────────────
//
// Use a private `_seal: ()` field on any type whose construction must pass
// through a canonical builder (e.g., a chain assembler or admission gate).
// Struct-literal construction from outside the module fails at compile time
// with E0451 ("field `_seal` of struct `Foo` is private").
//
// ```rust
// pub struct Receipt {
//     pub events: Vec<OperationEvent>,
//     pub chain_hash: Blake3Hash,
//     #[serde(skip)]
//     _seal: (),   // private — only constructible via ChainAssembler::finalize
// }
//
// impl Receipt {
//     pub(crate) fn sealed(events: Vec<OperationEvent>, chain_hash: Blake3Hash) -> Self {
//         Receipt { events, chain_hash, _seal: () }
//     }
// }
// ```
//
// External code that tries `Receipt { events: vec![], chain_hash: h, _seal: () }`
// will not compile. This is a value-level immutability guarantee, not just a
// runtime check.

// ─── Forward Compatibility (non_exhaustive) ─────────────────────────────────

/// Example of a `#[non_exhaustive]` enum for forward-compatible protocol variants.
///
/// Downstream crates must use a `_ => { /* handle unknown */ }` arm when matching,
/// which means adding new variants here is a non-breaking change.
///
/// # Examples
///
/// ```rust
/// use {{project-name}}::ProfileId;
///
/// let profile = ProfileId::CoreV1;
/// assert_eq!(profile.as_str(), "core/v1");
///
/// // Matching is exhaustive within this crate but not outside it:
/// match profile {
///     ProfileId::CoreV1 => {},
///     _ => { /* future variant */ },
/// }
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileId {
    /// Core v1: every event has a non-empty `event_type` and a valid commitment.
    CoreV1,
}

impl ProfileId {
    /// Stable string identifier for serialization and CLI output.
    pub fn as_str(&self) -> &'static str {
        match self {
            ProfileId::CoreV1 => "core/v1",
        }
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blake3_hash_round_trips_through_hex() {
        let h = Blake3Hash::content_address(b"affidavit");
        assert_eq!(h.as_hex().len(), 64);
        let rt = Blake3Hash::from_hex(h.as_hex());
        assert_eq!(h, rt);
    }

    #[test]
    fn canonical_bytes_sorts_keys() {
        use serde::Serialize;

        #[derive(Serialize)]
        struct Payload {
            z: i32,
            a: i32,
            m: i32,
        }

        let bytes = canonical_bytes(&Payload { z: 3, a: 1, m: 2 }).unwrap();
        let s = std::str::from_utf8(&bytes).unwrap();
        // Keys must appear in sorted order.
        let a_pos = s.find("\"a\"").unwrap();
        let m_pos = s.find("\"m\"").unwrap();
        let z_pos = s.find("\"z\"").unwrap();
        assert!(a_pos < m_pos && m_pos < z_pos, "keys not sorted: {s}");
    }

    #[test]
    fn canonical_bytes_same_value_same_bytes() {
        use serde::Serialize;

        #[derive(Serialize)]
        struct Inner {
            b: &'static str,
            a: &'static str,
        }

        let b1 = canonical_bytes(&Inner { b: "two", a: "one" }).unwrap();
        let b2 = canonical_bytes(&Inner { b: "two", a: "one" }).unwrap();
        assert_eq!(b1, b2);
    }

    #[test]
    fn object_ref_display_with_qualifier() {
        let obj = ObjectRef {
            id: "artifact-1".to_string(),
            type_: "artifact".to_string(),
            qualifier: Some("input".to_string()),
        };
        assert_eq!(obj.to_string(), "artifact-1:artifact:input");
    }

    #[test]
    fn object_ref_display_without_qualifier() {
        let obj = ObjectRef {
            id: "repo:main".to_string(),
            type_: "git".to_string(),
            qualifier: None,
        };
        assert_eq!(obj.to_string(), "repo:main:git");
    }

    #[test]
    fn object_ref_qualifier_skipped_in_json() {
        let obj = ObjectRef {
            id: "x".to_string(),
            type_: "t".to_string(),
            qualifier: None,
        };
        let json = serde_json::to_string(&obj).unwrap();
        assert!(!json.contains("qualifier"), "None qualifier should be omitted: {json}");
    }

    #[test]
    fn profile_id_as_str() {
        assert_eq!(ProfileId::CoreV1.as_str(), "core/v1");
    }
}
