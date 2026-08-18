//! Core domain types for {{project-name}}.
//!
//! **Rule:** This module holds data shapes only — no business logic, no I/O.
//! Logic belongs in dedicated modules (`chain.rs`, `verifier.rs`, etc.).

use serde::{Deserialize, Serialize};
use std::fmt;
use std::marker::PhantomData;

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
/// use {{project_name}}::Blake3Hash;
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
/// use {{project_name}}::ObjectRef;
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

// ── ObjectRef parsing ─────────────────────────────────────────────────────

/// Error returned when an `id:type[:qualifier]` string cannot be parsed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid object ref '{input}': {reason}")]
pub struct ObjectRefParseError {
    /// The original input string.
    pub input: String,
    /// Human-readable reason.
    pub reason: &'static str,
}

impl ObjectRef {
    /// Parse an `id:type` or `id:type:qualifier` string.
    ///
    /// The first colon-separated segment is the `id`, the second is `type_`,
    /// and an optional third segment becomes `qualifier`. Additional colons
    /// within the `id` segment (e.g. `"repo:main"`) are not allowed — encode
    /// slashes or other separators instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use {{project_name}}::types::ObjectRef;
    ///
    /// let obj = ObjectRef::parse("artifact-1:artifact:input").unwrap();
    /// assert_eq!(obj.id, "artifact-1");
    /// assert_eq!(obj.type_, "artifact");
    /// assert_eq!(obj.qualifier.as_deref(), Some("input"));
    ///
    /// let bare = ObjectRef::parse("suite:test-suite").unwrap();
    /// assert_eq!(bare.qualifier, None);
    /// ```
    pub fn parse(s: &str) -> Result<Self, ObjectRefParseError> {
        let parts: Vec<&str> = s.splitn(3, ':').collect();
        match parts.as_slice() {
            [id, type_] if !id.is_empty() && !type_.is_empty() => Ok(ObjectRef {
                id: (*id).to_string(),
                type_: (*type_).to_string(),
                qualifier: None,
            }),
            [id, type_, qualifier]
                if !id.is_empty() && !type_.is_empty() && !qualifier.is_empty() =>
            {
                Ok(ObjectRef {
                    id: (*id).to_string(),
                    type_: (*type_).to_string(),
                    qualifier: Some((*qualifier).to_string()),
                })
            }
            _ => Err(ObjectRefParseError {
                input: s.to_string(),
                reason: "expected 'id:type' or 'id:type:qualifier' with non-empty segments",
            }),
        }
    }
}

impl std::str::FromStr for ObjectRef {
    type Err = ObjectRefParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ObjectRef::parse(s)
    }
}

#[cfg(test)]
mod object_ref_parse_tests {
    use super::*;

    #[test]
    fn parse_id_type() {
        let obj: ObjectRef = "suite:test-suite".parse().unwrap();
        assert_eq!(obj.id, "suite");
        assert_eq!(obj.type_, "test-suite");
        assert!(obj.qualifier.is_none());
    }

    #[test]
    fn parse_id_type_qualifier() {
        let obj = ObjectRef::parse("artifact-1:artifact:input").unwrap();
        assert_eq!(obj.qualifier.as_deref(), Some("input"));
    }

    #[test]
    fn display_parse_round_trip() {
        let original = ObjectRef {
            id: "repo".to_string(),
            type_: "git".to_string(),
            qualifier: Some("output".to_string()),
        };
        let s = original.to_string();
        let parsed = ObjectRef::parse(&s).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn parse_rejects_missing_type() {
        assert!(ObjectRef::parse("onlyone").is_err());
        assert!(ObjectRef::parse("").is_err());
    }

    #[test]
    fn parse_rejects_empty_segments() {
        assert!(ObjectRef::parse(":type").is_err());
        assert!(ObjectRef::parse("id:").is_err());
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
/// use {{project_name}}::{canonical_bytes, Blake3Hash};
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

// ─── Typestate: Private Seal Module ────────────────────────────────────────
//
// INN-1 pattern from wasm4pm-compat: a `sealed` module whose only exported
// item is the `Sealed` trait.  Because the module is private and the trait
// has an unnameable inner bound, no code outside `types` can implement
// `Sealed`.  This makes `State<S>` and every function gated on `S: Sealed`
// non-forgeable at compile time.

/// Private internals for the typestate seal.
///
/// This module is not public.  Its sole purpose is to prevent external code
/// from implementing `Sealed` and therefore from constructing `State<S>` with
/// a type parameter not defined here.
mod sealed {
    mod private {
        pub trait Sealed {}
    }

    /// Marker for states that may legally appear in [`super::State<S>`].
    ///
    /// This trait is **not** re-exported from the crate root.  Downstream code
    /// cannot implement it; it may only be used as a bound in `where S: Sealed`.
    pub trait Sealed: private::Sealed {}

    /// The chain has been constructed but not yet verified.
    #[derive(Debug, Clone, Copy)]
    pub struct Pending;
    impl Sealed for Pending {}
    impl private::Sealed for Pending {}

    /// The chain has passed all admission checks.
    #[derive(Debug, Clone, Copy)]
    pub struct Verified;
    impl Sealed for Verified {}
    impl private::Sealed for Verified {}

    /// The chain was rejected by an admission check.
    #[derive(Debug, Clone, Copy)]
    pub struct Rejected;
    impl Sealed for Rejected {}
    impl private::Sealed for Rejected {}
}

// Re-export the state markers and bound so callers can write `State<Pending>`
// without naming the private `sealed` module.
pub use sealed::{Pending, Rejected, Sealed, Verified};

/// A zero-cost typestate wrapper.
///
/// `S` must implement [`Sealed`], which is only implemented for the three
/// built-in state markers: [`Pending`], [`Verified`], and [`Rejected`].
/// Attempting to construct `State<MySpy>` fails with:
///
/// ```text
/// E0277: the trait bound `MySpy: Sealed` is not satisfied
/// ```
///
/// # State Machine
///
/// ```text
/// Pending ──transition()──► Verified
/// Pending ──reject()──────► Rejected
/// ```
///
/// # Examples
///
/// ```rust
/// use {{project-name}}::{State, Pending, Verified};
///
/// fn verify(s: State<Pending>) -> State<Verified> {
///     s.transition()
/// }
///
/// let pending: State<Pending> = State::new();
/// let verified: State<Verified> = verify(pending);
/// // compile error — State<Verified> where State<Pending> expected:
/// // verify(verified);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct State<S: Sealed> {
    _state: std::marker::PhantomData<S>,
}

impl State<Pending> {
    /// Create a new chain in the `Pending` state.
    pub fn new() -> Self {
        State { _state: std::marker::PhantomData }
    }

    /// Advance to `Verified` after all checks pass.
    pub fn transition(self) -> State<Verified> {
        State { _state: std::marker::PhantomData }
    }

    /// Reject the chain, moving to the `Rejected` terminal state.
    pub fn reject(self) -> State<Rejected> {
        State { _state: std::marker::PhantomData }
    }
}

impl Default for State<Pending> {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Named Refusal Enums ────────────────────────────────────────────────────
//
// INN-1 pattern: domain errors are named enums, never bare `String`s.
// This makes error matching exhaustive and lets callers distinguish error kinds
// without parsing message text.  `#[derive(PartialEq, Eq)]` lets unit tests
// use `assert_eq!` on error values directly.

/// Reasons a receipt chain admission can be refused.
///
/// Use this instead of `anyhow::Error::msg("...")` for domain-level rejection
/// paths — named variants let callers match exhaustively and let tests assert
/// with `assert_eq!` rather than string-matching an error message.
///
/// # Examples
///
/// ```rust
/// use {{project-name}}::ReceiptRefusal;
///
/// fn admit(seq: u32, expected: u32) -> Result<(), ReceiptRefusal> {
///     if seq != expected {
///         return Err(ReceiptRefusal::SeqOutOfOrder { got: seq, expected });
///     }
///     Ok(())
/// }
///
/// assert_eq!(
///     admit(2, 0),
///     Err(ReceiptRefusal::SeqOutOfOrder { got: 2, expected: 0 })
/// );
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ReceiptRefusal {
    /// Events must arrive in strict monotone-increasing order.
    #[error("seq out of order: got {got}, expected {expected}")]
    SeqOutOfOrder {
        /// The sequence number that arrived.
        got: u32,
        /// The next expected sequence number.
        expected: u32,
    },

    /// The commitment digest is not a valid 64-character lowercase hex string.
    #[error("malformed commitment digest: {commitment:?}")]
    MalformedCommitment {
        /// The invalid digest string.
        commitment: String,
    },

    /// The event-ID is empty or contains only whitespace.
    #[error("empty or blank event_id")]
    EmptyEventId,

    /// The event-type is empty or contains only whitespace.
    #[error("empty or blank event_type")]
    EmptyEventType,

    /// Two events share the same event-ID (IDs must be unique within a chain).
    #[error("duplicate event_id: {id:?}")]
    DuplicateEventId {
        /// The repeated event identifier.
        id: String,
    },
}

// ─── Compile-Time Ontology KEY Uniqueness Proof ─────────────────────────────
//
// INN-6 pattern from wasm4pm-compat: a `const fn` that panics *at compile
// time* if any two IDs in a `&[&str]` array are identical.
//
// Usage (evaluated during compilation, zero run-time cost):
//
//   const _: () = assert_unique_ids(ONTOLOGY_VERB_IDS);
//
// The `_: ()` binding forces evaluation.  A duplicate triggers a build error
// before any binary is emitted.

/// Assert at compile time that every ID string in `ids` is unique.
///
/// Call this in a `const _: () = ...` binding so that duplicate IDs cause a
/// **compile-time error**, not a runtime panic.  The compiler will print the
/// `panic!` message in the build transcript.
///
/// # Examples
///
/// ```rust
/// use {{project-name}}::assert_unique_ids;
///
/// const VERB_IDS: &[&str] = &["build", "test", "deploy"];
/// const _: () = assert_unique_ids(VERB_IDS);  // ok — all unique
/// ```
///
/// A slice with a duplicate would not compile:
///
/// ```rust,compile_fail
/// use {{project-name}}::assert_unique_ids;
///
/// const BAD: &[&str] = &["build", "test", "build"];
/// const _: () = assert_unique_ids(BAD);   // error: duplicate ontology ID
/// ```
pub const fn assert_unique_ids(ids: &[&str]) -> () {
    // O(n²) — acceptable because n < 200 verbs and this runs only at compile time.
    let mut i = 0;
    while i < ids.len() {
        let mut j = i + 1;
        while j < ids.len() {
            let a = ids[i].as_bytes();
            let b = ids[j].as_bytes();
            if a.len() == b.len() {
                // Byte-by-byte comparison: `PartialEq` is not available in `const fn`.
                let mut k = 0;
                let mut equal = true;
                while k < a.len() {
                    if a[k] != b[k] {
                        equal = false;
                        break;
                    }
                    k += 1;
                }
                if equal {
                    // `panic!` in `const fn` is stable since Rust 1.73.
                    panic!("duplicate ontology ID detected — every ID must be globally unique");
                }
            }
            j += 1;
        }
        i += 1;
    }
}

/// Example ontology verb IDs — each corresponds to one RDF class in `domain.ttl`.
///
/// Checked for uniqueness at compile time:
///
/// ```rust
/// use {{project-name}}::{ONTOLOGY_VERB_IDS, assert_unique_ids};
/// const _: () = assert_unique_ids(ONTOLOGY_VERB_IDS);
/// ```
pub const ONTOLOGY_VERB_IDS: &[&str] = &[
    "dom:build",
    "dom:test",
    "dom:deploy",
    "dom:verify",
];

// Enforce uniqueness at compile time — adding a duplicate to ONTOLOGY_VERB_IDS
// will cause a build failure here, before any code runs.
const _: () = assert_unique_ids(ONTOLOGY_VERB_IDS);

// ─── Seal Pattern (basic variant, doc example) ──────────────────────────────
//
// For simple sealed types where a typestate machine is not needed, a private
// `_seal: ()` field is sufficient.  Struct-literal construction from outside
// the module fails at compile time with E0451 ("field `_seal` of struct `Foo`
// is private").
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
// will not compile.  Use the full typestate `State<S>` pattern (above) when
// you need to track *which* construction phase the value is in.

// ─── Evidence<T, State, W> — Typed Lifecycle Carrier ───────────────────────

/// Sealed state markers — cannot be named or implemented outside this module.
mod sealed {
    pub trait LifecycleState {}
}

/// Raw (unadmitted) evidence state marker.
pub struct Raw;
impl sealed::LifecycleState for Raw {}

/// Validated evidence state marker.
pub struct Validated;
impl sealed::LifecycleState for Validated {}

/// Admitted evidence state marker — only reachable via an [`Admit`] impl.
pub struct Admitted;
impl sealed::LifecycleState for Admitted {}

/// Typed lifecycle carrier.
///
/// `T` is the inner value, `State` is the lifecycle state (`Raw`, `Validated`, or `Admitted`),
/// and `Witness` is a witness authority tag (a domain marker struct chosen by the
/// crate that owns the admission gate).
///
/// `Evidence<T, Admitted, Witness>` can only be constructed by an [`Admit`] impl,
/// enforcing the one-way admission door at compile time.
pub struct Evidence<T, State: sealed::LifecycleState, Witness> {
    inner: T,
    _state: PhantomData<State>,
    _witness: PhantomData<Witness>,
}

impl<T, Witness> Evidence<T, Raw, Witness> {
    /// Create raw (unadmitted) evidence. Available to all callers.
    pub fn raw(inner: T) -> Self {
        Self { inner, _state: PhantomData, _witness: PhantomData }
    }

    /// Create raw (unadmitted) evidence. Alternative name.
    pub fn new(inner: T) -> Self {
        Self { inner, _state: PhantomData, _witness: PhantomData }
    }

    /// Borrow the inner value of raw evidence.
    pub fn inner(&self) -> &T {
        &self.inner
    }
}

impl<T, Witness> Evidence<T, Validated, Witness> {
    /// Borrow the inner value of validated evidence.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Private constructor — only validators within this crate may call this.
    pub(crate) fn validate_unchecked(inner: T) -> Self {
        Self { inner, _state: PhantomData, _witness: PhantomData }
    }
}

impl<T, Witness> Evidence<T, Admitted, Witness> {
    /// Borrow the inner value of admitted evidence.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Private constructor — only [`Admit`] impls within this crate may call this.
    pub(crate) fn admit_unchecked(inner: T) -> Self {
        Self { inner, _state: PhantomData, _witness: PhantomData }
    }
}

/// One-way admission door.
///
/// Implementations convert `Evidence<Input, Raw, Witness>` into
/// `Evidence<Input, Admitted, Witness>` or return a typed rejection reason.
/// Only the holder of an `Admit` impl can produce `AdmittedEvidence`.
pub trait Admit {
    type Input;
    type Witness;
    type Error;

    fn admit(
        input: Evidence<Self::Input, Raw, Self::Witness>,
    ) -> Result<Evidence<Self::Input, Admitted, Self::Witness>, Self::Error>;
}

/// Convenience alias for raw (unadmitted) evidence.
pub type RawEvidence<T, W> = Evidence<T, Raw, W>;

/// Convenience alias for validated evidence.
pub type ValidatedEvidence<T, W> = Evidence<T, Validated, W>;

/// Convenience alias for admitted evidence.
pub type AdmittedEvidence<T, W> = Evidence<T, Admitted, W>;

/// A cryptographic receipt verifying admission.
///
/// Constructed only via authorized validators using the Seal Pattern.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmittedReceipt {
    /// The BLAKE3 hash of the admitted state.
    pub chain_hash: [u8; 32],
    /// Epoch timestamp of admission in seconds.
    pub timestamp: u64,
    #[serde(skip)]
    _seal: (), // Private field prevents direct construction outside this module
}

impl AdmittedReceipt {
    /// Create a new sealed receipt. Restricted to crate/module validator authority.
    pub(crate) fn new(chain_hash: [u8; 32], timestamp: u64) -> Self {
        Self {
            chain_hash,
            timestamp,
            _seal: (),
        }
    }
}

// ─── Forward Compatibility (non_exhaustive) ─────────────────────────────────

/// Example of a `#[non_exhaustive]` enum for forward-compatible protocol variants.
///
/// Downstream crates must use a `_ => { /* handle unknown */ }` arm when matching,
/// which means adding new variants here is a non-breaking change.
///
/// # Examples
///
/// ```rust
/// use {{project_name}}::ProfileId;
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
#[repr(u8)]
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

// ─── CI/CD Policy Surface ──────────────────────────────────────────────────

/// Three-way policy verdict — not a binary pass/fail.
///
/// - `Pass`    — all checks clear; proceed without hesitation.
/// - `Warn`    — notable condition; non-blocking but should be logged/tracked.
/// - `Suggest` — informational observation; purely advisory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyVerdict {
    Pass,
    Warn,
    Suggest,
}

/// Tunable hyperparameters for [`CicdPolicy`] evaluation.
///
/// Defaults reproduce historical hard-coded behaviour so existing policies
/// require no changes when this struct is first introduced.
///
/// | Field                     | Default | Meaning |
/// |---------------------------|---------|---------|
/// | `behind_threshold`        | 3       | Commits behind remote before a `Warn` is emitted |
/// | `dirty_threshold`         | 5       | Dirty-file count before a `Warn` is emitted |
/// | `evidence_staleness_secs` | 3600    | Age of evidence (seconds) before it is considered stale |
#[derive(Debug, Clone)]
pub struct PolicyConfig {
    /// Number of commits behind the remote tracking branch that triggers a warning.
    /// Default: `3`.
    pub behind_threshold: usize,
    /// Number of dirty (modified/untracked) files that triggers a warning.
    /// Default: `5`.
    pub dirty_threshold: usize,
    /// Maximum age of cached evidence in seconds before it is considered stale.
    /// Default: `3600` (one hour).
    pub evidence_staleness_secs: u64,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            behind_threshold: 3,
            dirty_threshold: 5,
            evidence_staleness_secs: 3_600,
        }
    }
}

/// Autonomic policy trait for the cargo-cicd pattern.
///
/// Every new project gets a ready-made policy surface rather than ad-hoc
/// pass/fail logic. Implement this trait to plug custom checks into the
/// unified evaluation pipeline.
pub trait CicdPolicy {
    /// Short, stable identifier for this policy (used in logs and diagnostics).
    fn name(&self) -> &'static str;

    /// Evaluate against `config` and return a three-way verdict.
    fn evaluate(&self, config: &PolicyConfig) -> PolicyVerdict;
}

// ── Compile-time layout assertions ──────────────────────────────────────────
//
// These fire at compile time (before any test runs) if a wire type changes
// size unexpectedly. Adjust the expected value only when you deliberately
// change the type layout and understand the downstream impact.

#[allow(dead_code)]
mod layout_assertions {
    use super::*;
    use std::mem::{align_of, size_of};

    // Blake3Hash wraps a String; on 64-bit platforms a String is 3 * usize = 24 bytes.
    const _BLAKE3_HASH_SIZE: () = {
        assert!(size_of::<Blake3Hash>() == 24, "Blake3Hash size changed — check wire compatibility");
    };

    // ProfileId is a fieldless enum — expected to be 1 byte (niche-optimised by the compiler).
    // If this fires after adding a variant, update the comment but leave the assert.
    const _PROFILE_ID_SIZE: () = {
        assert!(size_of::<ProfileId>() == 1, "ProfileId size changed — may break OCEL serialization");
    };

    // PolicyVerdict is a fieldless enum — 1 byte.
    const _POLICY_VERDICT_SIZE: () = {
        assert!(size_of::<PolicyVerdict>() == 1, "PolicyVerdict size changed");
    };

    // Raw and Admitted are ZSTs used as PhantomData witness tags.
    const _RAW_IS_ZST: () = {
        assert!(size_of::<Raw>() == 0, "Raw marker must stay ZST");
    };
    const _VALIDATED_IS_ZST: () = {
        assert!(size_of::<Validated>() == 0, "Validated marker must stay ZST");
    };
    const _ADMITTED_IS_ZST: () = {
        assert!(size_of::<Admitted>() == 0, "Admitted marker must stay ZST");
    };

    // Evidence<T, S, W> with ZST state and witness must not add overhead.
    // Evidence<u64, Raw, Raw> should be same size as u64.
    const _EVIDENCE_NO_OVERHEAD: () = {
        assert!(
            size_of::<Evidence<u64, Raw, Raw>>() == size_of::<u64>(),
            "Evidence<T,S,W> must be zero-overhead over T when S and W are ZSTs"
        );
    };

    // Alignment checks — ensure no surprise padding.
    const _BLAKE3_HASH_ALIGN: () = {
        assert!(align_of::<Blake3Hash>() == align_of::<String>(), "Blake3Hash alignment changed");
    };
}

// ── Verdict: 7-stage pipeline result ────────────────────────────────────────

/// Outcome for a single verification stage in the certify pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageOutcome {
    /// Stage name (e.g. `"chain_integrity"`, `"continuity"`).
    pub stage: String,
    /// Whether this stage passed.
    pub passed: bool,
    /// Human-readable reason if the stage failed; `None` on pass.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl StageOutcome {
    /// Construct a passing stage outcome.
    pub fn pass(stage: impl Into<String>) -> Self {
        StageOutcome { stage: stage.into(), passed: true, reason: None }
    }

    /// Construct a failing stage outcome with a reason.
    pub fn fail(stage: impl Into<String>, reason: impl Into<String>) -> Self {
        StageOutcome { stage: stage.into(), passed: false, reason: Some(reason.into()) }
    }
}

/// The result of running the full certify pipeline.
///
/// `accepted` is `true` iff all stages passed. Use [`Verdict::first_failure`] to
/// retrieve the first failing stage without exposing raw field access to callers.
///
/// # Doctrine
///
/// The verifier **certifies, it does not decide.** `Verdict` captures what the
/// pipeline observed; policy decisions ("should we block this release?") are the
/// caller's responsibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verdict {
    /// `true` iff every stage passed.
    pub accepted: bool,
    /// Per-stage outcomes in pipeline order.
    pub stage_outcomes: Vec<StageOutcome>,
}

impl Verdict {
    /// Construct an ACCEPT verdict from a list of all-passing stage outcomes.
    pub fn accept(stages: Vec<StageOutcome>) -> Self {
        Verdict { accepted: true, stage_outcomes: stages }
    }

    /// Construct a REJECT verdict. Sets `accepted = false` regardless of individual outcomes.
    pub fn reject(stages: Vec<StageOutcome>) -> Self {
        Verdict { accepted: false, stage_outcomes: stages }
    }

    /// Return the first failing [`StageOutcome`], if any.
    ///
    /// Returns `None` for ACCEPT verdicts or (degenerate) REJECT verdicts with no
    /// recorded failure reason.
    pub fn first_failure(&self) -> Option<&StageOutcome> {
        self.stage_outcomes.iter().find(|s| !s.passed)
    }

    /// A single-line human-readable summary suitable for CLI output.
    ///
    /// Examples:
    /// - `"ACCEPT (7/7 stages passed)"`
    /// - `"REJECT at stage chain_integrity: chain hash mismatch"`
    pub fn summary(&self) -> String {
        if self.accepted {
            let n = self.stage_outcomes.len();
            format!("ACCEPT ({n}/{n} stages passed)")
        } else {
            match self.first_failure() {
                Some(s) => {
                    let reason = s.reason.as_deref().unwrap_or("no reason recorded");
                    format!("REJECT at stage {}: {reason}", s.stage)
                }
                None => "REJECT (no stage failure recorded)".to_string(),
            }
        }
    }

    /// Returns `true` if the verdict is an ACCEPT.
    pub fn is_accepted(&self) -> bool {
        self.accepted
    }
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary())
    }
}

#[cfg(test)]
mod verdict_tests {
    use super::*;

    #[test]
    fn accept_verdict_summary() {
        let v = Verdict::accept(vec![
            StageOutcome::pass("decode"),
            StageOutcome::pass("chain_integrity"),
        ]);
        assert!(v.is_accepted());
        assert_eq!(v.summary(), "ACCEPT (2/2 stages passed)");
        assert!(v.first_failure().is_none());
    }

    #[test]
    fn reject_verdict_first_failure() {
        let v = Verdict::reject(vec![
            StageOutcome::pass("decode"),
            StageOutcome::fail("chain_integrity", "chain hash mismatch"),
            StageOutcome::fail("continuity", "seq gap at 3"),
        ]);
        assert!(!v.is_accepted());
        let f = v.first_failure().unwrap();
        assert_eq!(f.stage, "chain_integrity");
        assert_eq!(v.summary(), "REJECT at stage chain_integrity: chain hash mismatch");
    }

    #[test]
    fn verdict_display_matches_summary() {
        let v = Verdict::accept(vec![StageOutcome::pass("decode")]);
        assert_eq!(format!("{v}"), v.summary());
    }
}

// ── PolicyConfig builder ─────────────────────────────────────────────────

impl PolicyConfig {
    /// Set the `behind_threshold` and return `self` for chaining.
    #[must_use]
    pub fn with_behind_threshold(mut self, n: usize) -> Self {
        self.behind_threshold = n;
        self
    }

    /// Set the `dirty_threshold` and return `self` for chaining.
    #[must_use]
    pub fn with_dirty_threshold(mut self, n: usize) -> Self {
        self.dirty_threshold = n;
        self
    }

    /// Set the `evidence_staleness_secs` and return `self` for chaining.
    #[must_use]
    pub fn with_evidence_staleness_secs(mut self, secs: u64) -> Self {
        self.evidence_staleness_secs = secs;
        self
    }
}

// ── PolicyVerdict ordering ────────────────────────────────────────────────

impl PolicyVerdict {
    /// Returns the "worse" of two verdicts.
    ///
    /// Severity order: `Pass` < `Suggest` < `Warn`.
    pub fn worse(self, other: PolicyVerdict) -> PolicyVerdict {
        use PolicyVerdict::*;
        match (self, other) {
            (Warn, _) | (_, Warn) => Warn,
            (Suggest, _) | (_, Suggest) => Suggest,
            _ => Pass,
        }
    }

    /// Returns `true` if this verdict is at least as severe as `threshold`.
    pub fn at_least(&self, threshold: &PolicyVerdict) -> bool {
        use PolicyVerdict::*;
        matches!(
            (self, threshold),
            (Warn, Warn)
                | (Warn, Suggest)
                | (Warn, Pass)
                | (Suggest, Suggest)
                | (Suggest, Pass)
                | (Pass, Pass)
        )
    }
}

/// Evaluate a collection of [`CicdPolicy`] implementations and return the
/// aggregate (worst) verdict.
///
/// # Example
///
/// ```rust
/// use {{project_name}}::types::{CicdPolicy, CicdPolicyRunner, PolicyConfig, PolicyVerdict};
///
/// struct AlwaysPass;
/// impl CicdPolicy for AlwaysPass {
///     fn name(&self) -> &'static str { "always-pass" }
///     fn evaluate(&self, _: &PolicyConfig) -> PolicyVerdict { PolicyVerdict::Pass }
/// }
///
/// let runner = CicdPolicyRunner::new(vec![Box::new(AlwaysPass)]);
/// let (verdict, findings) = runner.run(&PolicyConfig::default());
/// assert_eq!(verdict, PolicyVerdict::Pass);
/// assert_eq!(findings.len(), 1);
/// ```
pub struct CicdPolicyRunner {
    policies: Vec<Box<dyn CicdPolicy>>,
}

/// A single policy finding from a [`CicdPolicyRunner`] run.
#[derive(Debug, Clone)]
pub struct PolicyFinding {
    /// The policy that produced this finding.
    pub policy_name: &'static str,
    /// The verdict this policy returned.
    pub verdict: PolicyVerdict,
}

impl CicdPolicyRunner {
    /// Construct a runner from a list of boxed policies.
    pub fn new(policies: Vec<Box<dyn CicdPolicy>>) -> Self {
        CicdPolicyRunner { policies }
    }

    /// Run all policies and return `(aggregate_verdict, findings)`.
    ///
    /// The aggregate verdict is the worst verdict across all policies.
    pub fn run(&self, config: &PolicyConfig) -> (PolicyVerdict, Vec<PolicyFinding>) {
        let mut aggregate = PolicyVerdict::Pass;
        let mut findings = Vec::with_capacity(self.policies.len());
        for policy in &self.policies {
            let v = policy.evaluate(config);
            aggregate = aggregate.worse(v.clone());
            findings.push(PolicyFinding { policy_name: policy.name(), verdict: v });
        }
        (aggregate, findings)
    }
}

#[cfg(test)]
mod policy_runner_tests {
    use super::*;

    struct Fixed(PolicyVerdict);
    impl CicdPolicy for Fixed {
        fn name(&self) -> &'static str { "fixed" }
        fn evaluate(&self, _: &PolicyConfig) -> PolicyVerdict { self.0.clone() }
    }

    #[test]
    fn runner_returns_worst_verdict() {
        let runner = CicdPolicyRunner::new(vec![
            Box::new(Fixed(PolicyVerdict::Pass)),
            Box::new(Fixed(PolicyVerdict::Suggest)),
            Box::new(Fixed(PolicyVerdict::Pass)),
        ]);
        let (v, findings) = runner.run(&PolicyConfig::default());
        assert_eq!(v, PolicyVerdict::Suggest);
        assert_eq!(findings.len(), 3);
    }

    #[test]
    fn policy_config_builder_chain() {
        let cfg = PolicyConfig::default()
            .with_behind_threshold(10)
            .with_dirty_threshold(2)
            .with_evidence_staleness_secs(7200);
        assert_eq!(cfg.behind_threshold, 10);
        assert_eq!(cfg.dirty_threshold, 2);
        assert_eq!(cfg.evidence_staleness_secs, 7200);
    }

    #[test]
    fn verdict_at_least() {
        assert!(PolicyVerdict::Warn.at_least(&PolicyVerdict::Pass));
        assert!(PolicyVerdict::Warn.at_least(&PolicyVerdict::Warn));
        assert!(!PolicyVerdict::Pass.at_least(&PolicyVerdict::Warn));
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

    #[test]
    fn policy_config_defaults() {
        let cfg = PolicyConfig::default();
        assert_eq!(cfg.behind_threshold, 3);
        assert_eq!(cfg.dirty_threshold, 5);
        assert_eq!(cfg.evidence_staleness_secs, 3_600);
    }

    #[test]
    fn policy_verdict_equality() {
        assert_eq!(PolicyVerdict::Pass, PolicyVerdict::Pass);
        assert_ne!(PolicyVerdict::Pass, PolicyVerdict::Warn);
        assert_ne!(PolicyVerdict::Warn, PolicyVerdict::Suggest);
    }

    #[test]
    fn cicd_policy_trait_dispatch() {
        struct AlwaysWarn;
        impl CicdPolicy for AlwaysWarn {
            fn name(&self) -> &'static str { "always-warn" }
            fn evaluate(&self, _config: &PolicyConfig) -> PolicyVerdict { PolicyVerdict::Warn }
        }

        let policy = AlwaysWarn;
        assert_eq!(policy.name(), "always-warn");
        assert_eq!(policy.evaluate(&PolicyConfig::default()), PolicyVerdict::Warn);
    }

    // ── Typestate tests ─────────────────────────────────────────────────────

    #[test]
    fn state_pending_transitions_to_verified() {
        let s: State<Pending> = State::new();
        let _v: State<Verified> = s.transition();
        // Compilation proves the typestate: State<Pending> → State<Verified>.
    }

    #[test]
    fn state_pending_can_be_rejected() {
        let s: State<Pending> = State::new();
        let _r: State<Rejected> = s.reject();
    }

    #[test]
    fn state_default_is_pending() {
        let _: State<Pending> = State::default();
    }

    // ── ReceiptRefusal tests ────────────────────────────────────────────────

    #[test]
    fn receipt_refusal_seq_out_of_order_is_eq() {
        let e1 = ReceiptRefusal::SeqOutOfOrder { got: 2, expected: 0 };
        let e2 = ReceiptRefusal::SeqOutOfOrder { got: 2, expected: 0 };
        assert_eq!(e1, e2);
    }

    #[test]
    fn receipt_refusal_empty_event_id_display() {
        let e = ReceiptRefusal::EmptyEventId;
        assert_eq!(e.to_string(), "empty or blank event_id");
    }

    #[test]
    fn receipt_refusal_malformed_commitment_contains_value() {
        let e = ReceiptRefusal::MalformedCommitment { commitment: "bad".into() };
        assert!(e.to_string().contains("bad"));
    }

    #[test]
    fn receipt_refusal_duplicate_event_id_contains_id() {
        let e = ReceiptRefusal::DuplicateEventId { id: "evt-42".into() };
        assert!(e.to_string().contains("evt-42"));
    }

    // ── assert_unique_ids tests ─────────────────────────────────────────────

    #[test]
    fn unique_ids_passes_for_distinct_values() {
        // This is a runtime call; the compile-time check is the `const _` above.
        assert_unique_ids(&["a", "b", "c"]);
    }

    #[test]
    #[should_panic(expected = "duplicate ontology ID")]
    fn unique_ids_panics_for_duplicate() {
        assert_unique_ids(&["a", "b", "a"]);
    }

    #[test]
    fn ontology_verb_ids_are_unique() {
        // Belt-and-suspenders: also check at runtime so the test output names the set.
        assert_unique_ids(ONTOLOGY_VERB_IDS);
    }
}

#[cfg(test)]
mod canonical_determinism_tests {
    use super::*;

    /// Prove that canonical_bytes produces identical output across 100 calls.
    #[test]
    fn canonical_bytes_is_deterministic_across_100_calls() {
        use serde::Serialize;

        #[derive(Serialize)]
        struct Payload {
            z: &'static str,
            a: &'static str,
            m: u64,
        }

        let p = Payload { z: "last", a: "first", m: 42 };
        let first = canonical_bytes(&p).unwrap();
        for _ in 0..99 {
            assert_eq!(canonical_bytes(&p).unwrap(), first,
                "canonical_bytes must be stable across repeated calls");
        }
    }

    /// Prove that BLAKE3 of canonical bytes is stable (same input → same hash).
    #[test]
    fn blake3_of_canonical_bytes_is_stable() {
        use serde::Serialize;

        #[derive(Serialize)]
        struct Event {
            seq: u64,
            event_type: &'static str,
        }

        let e = Event { seq: 0, event_type: "build" };
        let bytes = canonical_bytes(&e).unwrap();
        let h1 = Blake3Hash::content_address(&bytes);
        let h2 = Blake3Hash::content_address(&bytes);
        assert_eq!(h1, h2);
        assert_eq!(h1.as_hex().len(), 64);
    }

    /// Prove that field declaration order does not affect the canonical hash.
    /// This is the key guarantee that makes receipts tamper-evident.
    #[test]
    fn field_order_does_not_affect_canonical_hash() {
        use serde::Serialize;

        #[derive(Serialize)]
        struct OrderA { a: i32, b: i32, c: i32 }

        #[derive(Serialize)]
        struct OrderB { c: i32, a: i32, b: i32 }

        let a = canonical_bytes(&OrderA { a: 1, b: 2, c: 3 }).unwrap();
        let b = canonical_bytes(&OrderB { c: 3, a: 1, b: 2 }).unwrap();
        assert_eq!(a, b, "canonical_bytes must normalize field order");
        assert_eq!(
            Blake3Hash::content_address(&a),
            Blake3Hash::content_address(&b),
            "canonical hashes must match regardless of struct field order"
        );
    }

    /// Prove that modifying any field changes the hash (tampering detection).
    #[test]
    fn tampered_field_changes_hash() {
        use serde::Serialize;

        #[derive(Serialize)]
        struct Event { seq: u64, payload: &'static str }

        let honest = canonical_bytes(&Event { seq: 1, payload: "build" }).unwrap();
        let tampered = canonical_bytes(&Event { seq: 1, payload: "build-TAMPERED" }).unwrap();
        assert_ne!(
            Blake3Hash::content_address(&honest),
            Blake3Hash::content_address(&tampered),
            "tampered payload must produce a different hash"
        );
    }
}

// ── Blake3Hash admission gate ─────────────────────────────────────────────

/// Witness authority tag for BLAKE3 hex-digest admission.
pub struct HashWitness;

/// Reason a BLAKE3 hex string was rejected at the admission gate.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidHash {
    /// The string is not exactly 64 characters.
    #[error("hash must be exactly 64 hex characters, got {0}")]
    WrongLength(usize),
    /// The string contains non-hexadecimal characters.
    #[error("hash contains non-hex character at position {0}: '{1}'")]
    NonHexChar(usize, char),
}

/// Admission gate for BLAKE3 hex digests.
///
/// Accepts a `RawEvidence<Blake3Hash, HashWitness>` and validates it is a
/// 64-character lowercase hex string. Returns `AdmittedEvidence` or an
/// `InvalidHash` rejection reason.
///
/// # Example
///
/// ```rust
/// use {{project_name}}::types::{
///     Blake3Hash, Evidence, HashAdmit, InvalidHash, RawEvidence,
/// };
/// use {{project_name}}::types::Admit;
///
/// let raw = Evidence::raw(Blake3Hash::content_address(b"data"));
/// let admitted = HashAdmit::admit(raw).unwrap();
/// let _ = admitted.inner(); // ← only reachable after validation
/// ```
pub struct HashAdmit;

impl Admit for HashAdmit {
    type Input = Blake3Hash;
    type Witness = HashWitness;
    type Error = InvalidHash;

    fn admit(
        input: Evidence<Blake3Hash, Raw, HashWitness>,
    ) -> Result<Evidence<Blake3Hash, Admitted, HashWitness>, InvalidHash> {
        let hex = input.inner().as_hex();
        if hex.len() != 64 {
            return Err(InvalidHash::WrongLength(hex.len()));
        }
        for (i, ch) in hex.chars().enumerate() {
            if !ch.is_ascii_hexdigit() {
                return Err(InvalidHash::NonHexChar(i, ch));
            }
        }
        Ok(Evidence::admit_unchecked(input.inner().clone()))
    }
}

#[cfg(test)]
mod hash_admit_tests {
    use super::*;

    #[test]
    fn admit_valid_hash() {
        let hash = Blake3Hash::content_address(b"hello");
        let raw: RawEvidence<Blake3Hash, HashWitness> = Evidence::raw(hash);
        assert!(HashAdmit::admit(raw).is_ok());
    }

    #[test]
    fn reject_wrong_length() {
        let short = Blake3Hash::from_hex("abc");
        let raw: RawEvidence<Blake3Hash, HashWitness> = Evidence::raw(short);
        assert!(matches!(HashAdmit::admit(raw), Err(InvalidHash::WrongLength(3))));
    }

    #[test]
    fn reject_non_hex_char() {
        let bad = Blake3Hash::from_hex(
            "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
        );
        let raw: RawEvidence<Blake3Hash, HashWitness> = Evidence::raw(bad);
        assert!(matches!(HashAdmit::admit(raw), Err(InvalidHash::NonHexChar(0, 'z'))));
    }
}
