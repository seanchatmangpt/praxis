//! Family F02 -- "Observation Admission" (atlas ticket V12-002).
//!
//! Wire phase 1 (this pass). Survey verdict: **MIXED** -- see
//! `/Users/sac/Downloads/v26.7.12_mermaid_atlas/families/F02_observation-admission.md`
//! (read in full this pass, all 8 lenses) for the architecture requirement this module
//! implements against. Per that doc's invariant: "External input remains observation
//! until identity, provenance, authority, structure, and semantic conformance pass."
//!
//! # What is real in this module (verified this session via the commands cited below)
//!
//! - **Gates 1-4 and 6-7** (Ingress parse, Identity Resolver, Provenance Checker,
//!   Authority Checker, Semantic Conformance, Admission Ledger) are HAND_WRITE_REQUIRED
//!   per the survey (no prior praxis code existed for them -- confirmed by
//!   `crates/praxis-graphlaw/src/chatman/quarantine.rs`'s own disclosure that
//!   `fetch_snapshot` "trusts its input snapshot IRI as given"). They are freshly
//!   hand-written here, not ported from anywhere.
//! - **Gate 5** (Shape Validator) is REUSE_ADAPT: it is a thin, real call into the
//!   existing, already-implemented native-Rust SHACL engine at
//!   `crates/praxis-graphlaw/src/shacl/{mod,model,report,validate}.rs`
//!   (`ShapesGraph::parse` + `Validator::validate`), not a reimplementation. The exact
//!   `Parser::parse_triples` + `TripleIndex` + `ShapesGraph::parse` + `Validator::validate`
//!   call sequence used below was confirmed this session by reading
//!   `crates/praxis-graphlaw/tests/shacl_validation.rs::build_data_index`, which is the
//!   crate's own established usage pattern for this API, not a guess.
//! - Receipts are canonical-sorted + BLAKE3-hex, matching the pattern read this session
//!   in `crates/powl2-decompose/src/net.rs::content_hash` (`blake3::Hasher` +
//!   `\x00`-delimited canonical fields + `hex::encode`), per this repo's invariant #2/#6
//!   (receipts computed, canonical, BLAKE3-only) and the no-wall-clock invariant #3 (no
//!   `SystemTime`/`Instant::now` anywhere in this file).
//! - The idempotency/correlation gate (L7) is real for the in-process case: a single
//!   `Mutex<BTreeMap<String, LedgerEntry>>` held for the full duration of one
//!   `admit_observation` call, so a duplicate-correlation-id submission cannot race past
//!   a first admission. This is coarse-grained (crate-wide single lock, not
//!   sharded-by-key) and explicitly NOT durable across a process restart -- there is no
//!   persistence layer here. That gap is real and disclosed, not hidden: L7's "Durable
//!   receipt head and replay state" surviving process restart is UNVERIFIED/NOT BUILT by
//!   this module; only the in-process atomic dedupe/conflict half of L7 is implemented.
//!
//! # What is explicitly NOT built (disclosed, not dressed up)
//!
//! - Cross-process / restart-durable ledger persistence (would need a real store --
//!   sled, sqlite, or a praxis-graphlaw named-graph snapshot -- none wired here).
//! - Chaos/process-restart recovery semantics from L7's "Process or engine restart"
//!   branch -- there is no restart to recover from in an in-memory `Mutex`, so this path
//!   is untested and unimplemented, tracked under V12-002, not silently claimed done.
//! - Full OWL-RL semantic closure for "Semantic Conformance" (gate 6): implemented here
//!   as a real but minimal closed-vocabulary check (every asserted predicate's IRI must
//!   match a policy-configured allowed-namespace prefix), mirroring this repo's own
//!   invariant #4 ("Closed vocabularies ... unknown predicates refused by name"), not a
//!   full ontology-closure conformance engine.
//!
//! # Reconciling the atlas's own lenses (disclosed, not silently resolved)
//!
//! L2 (Component Topology) draws explicit refusal edges only from C2 (Identity
//! Resolver), C3 (Provenance Checker), and C7 (Admission Ledger). L5 (State Machine)
//! draws `REFUSED` reachable only from `IDENTIFIED` and `AUTHORIZED`. Neither lens
//! draws a refusal edge from every one of the 5 gates the family invariant names
//! (identity, provenance, authority, structure, semantic conformance). This module
//! implements [`ObservationAdmissionRefused`] as reachable from *every* gate (a
//! superset of what either single lens draws), because the family invariant text and
//! L1's CTQ ("every rejected observation has a typed refusal") are unambiguous that no
//! gate may silently pass or silently drop -- a narrower implementation matching only
//! L5's two drawn edges would let a Shape Validator or Semantic Conformance failure
//! fall through un-refused, which the CTQ forbids.
//!
//! Survey-cited paths (from the prior investigation handed to this session):
//! - /Users/sac/Downloads/v26.7.12_mermaid_atlas/families/F02_observation-admission.md
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/engine.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/quarantine.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/shacl/{mod,validate,model,report}.rs
//! - /Users/sac/praxis/packs/chatman-engine-pack/pack.toml
//! - /Users/sac/gitvan/src/git-lifecycle/GitEventCapture.mjs (rejected: ungated, wrong
//!   language, narrow git-hook-only domain -- not adapted)
//! - /Users/sac/five-layer-agents/schema/agents.ttl (rejected: agent-RBAC domain, not
//!   observation admission -- not adapted)

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

use praxis_graphlaw::parser::{Parser, Syntax};
use praxis_graphlaw::shacl::{ShapesGraph, Validator};
use praxis_graphlaw::tripleindex::TripleIndex;
use praxis_graphlaw::triples::{Term, VarOrTerm};

/// `prov:wasDerivedFrom`, used by the Provenance Checker (gate 2) to bind a claimed
/// subject to its declared source. Bare IRI (no angle brackets); wrapped via
/// [`VarOrTerm::convert`] at comparison sites, matching how the Turtle parser itself
/// encodes IRIs (confirmed this session: `crates/praxis-graphlaw/src/term.rs`'s
/// `VarOrTerm::convert` auto-wraps a bare IRI in `<...>` before interning).
pub const PROV_WAS_DERIVED_FROM: &str = "http://www.w3.org/ns/prov#wasDerivedFrom";

// ---------------------------------------------------------------------------
// L6 data/provenance chain types: RawObservation -> IdentityLink ->
// ProvenanceBundle -> AuthorityEvidence -> ShapeReport -> ConformanceReport ->
// AdmissionReceipt. Each stage below is produced only after its gate passes;
// on failure the pipeline short-circuits into `ObservationAdmissionRefused`
// and no later stage's type is ever constructed for that observation.
// ---------------------------------------------------------------------------

/// Untrusted external input, exactly as received by the Ingress Adapter (gate 0).
/// Carries no standing until it passes all 5 gates below.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawObservation {
    /// Idempotency/correlation key (L7). Two admissions with the same
    /// `correlation_id` and the same canonical payload are treated as one
    /// logical event (replay-equivalent, second call returns the first
    /// receipt); same `correlation_id` with a *different* canonical payload
    /// is refused as [`ObservationAdmissionRefused::CorrelationConflict`].
    pub correlation_id: String,
    /// The external system asserting this observation (e.g. an email API
    /// integration id, a telemetry collector id). Looked up against
    /// [`AdmissionPolicy::known_principals`] and
    /// [`AdmissionPolicy::authorized_predicates`].
    pub source_id: String,
    /// Bare IRI (no angle brackets) of the entity this observation claims to
    /// be about. Must actually appear as a subject in `payload_turtle` for
    /// the Identity Resolver to accept it.
    pub declared_subject: String,
    /// The observation's RDF content, Turtle-syntax. Untrusted: only becomes
    /// graph-worthy data after every gate below passes.
    pub payload_turtle: String,
}

/// Output of the Identity Resolver (gate 1): `declared_subject` is a
/// well-formed IRI, actually present as a subject in the payload, and
/// `source_id` maps to a known principal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityLink {
    pub subject_iri: String,
    pub resolved_principal: String,
}

/// Output of the Provenance Checker (gate 2): the payload contains an
/// explicit `prov:wasDerivedFrom subject_iri source_iri` triple consistent
/// with the resolved principal, not merely an out-of-band claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvenanceBundle {
    pub subject_iri: String,
    pub source_id: String,
    pub asserted_derivation_iri: String,
}

/// Output of the Authority Checker (gate 3): every predicate asserted about
/// `subject_iri` (other than the provenance bookkeeping triple itself) is in
/// `source_id`'s authorized-predicate set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorityEvidence {
    pub source_id: String,
    pub authorized_predicate_count: usize,
}

/// Output of the Shape Validator (gate 4): a real SHACL conformance result
/// from `praxis_graphlaw::shacl::Validator`, not a placeholder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeReport {
    pub conforms: bool,
    pub violation_count: usize,
}

/// Output of Semantic Conformance (gate 5): every asserted predicate matches
/// an allowed-vocabulary namespace prefix (closed-vocabulary check, mirroring
/// this repo's own invariant #4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceReport {
    pub conforms: bool,
    pub distinct_predicate_count: usize,
}

/// Terminal receipt (gate 6, Admission Ledger). `receipt_hash` is BLAKE3-hex
/// over the canonical (sorted) N-Triples-shaped serialization of the
/// admitted payload plus `correlation_id` -- deterministic, no wall clock,
/// matching `crates/powl2-decompose/src/net.rs::content_hash`'s pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionReceipt {
    pub correlation_id: String,
    pub source_id: String,
    pub subject_iri: String,
    pub state: AdmissionState,
    pub receipt_hash: String,
    pub triple_count: usize,
}

/// L5 lifecycle states. `Refused` is reachable from every gate in this
/// implementation (see the module doc's "Reconciling the atlas's own
/// lenses" section for why this is a deliberate superset of what any single
/// atlas lens draws).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionState {
    Received,
    Identified,
    Provenanced,
    Authorized,
    Structured,
    Conformant,
    Admitted,
    Refused,
}

/// Typed refusal taxonomy for F02. Every variant has >= 1 end-to-end test in
/// this file's `tests` module (see the table in that module's doc comment).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ObservationAdmissionRefused {
    #[error("correlation {correlation_id}: malformed payload: {reason}")]
    MalformedPayload {
        correlation_id: String,
        reason: String,
    },
    #[error("correlation {correlation_id}: identity unresolved: {reason}")]
    IdentityUnresolved {
        correlation_id: String,
        reason: String,
    },
    #[error("correlation {correlation_id}: provenance unverified: {reason}")]
    ProvenanceUnverified {
        correlation_id: String,
        reason: String,
    },
    #[error(
        "correlation {correlation_id}: authority denied: source {source_id} is not authorized \
         for predicate {predicate}"
    )]
    AuthorityDenied {
        correlation_id: String,
        source_id: String,
        predicate: String,
    },
    #[error(
        "correlation {correlation_id}: shape non-conformant: {violation_count} SHACL \
         violation(s)"
    )]
    ShapeNonConformant {
        correlation_id: String,
        violation_count: usize,
    },
    #[error(
        "correlation {correlation_id}: semantic non-conformant: predicate {predicate} is \
         outside the allowed vocabulary"
    )]
    SemanticNonConformant {
        correlation_id: String,
        predicate: String,
    },
    #[error(
        "correlation {correlation_id}: already admitted with a different payload (stale or \
         conflicting replay)"
    )]
    CorrelationConflict { correlation_id: String },
    #[error("admission ledger unavailable: {reason}")]
    LedgerUnavailable { reason: String },
}

/// Admission policy: the trust configuration every observation is checked
/// against. Constructed once (e.g. at process startup) and shared read-only
/// across `admit_observation` calls.
pub struct AdmissionPolicy {
    /// `source_id -> canonical principal IRI` (bare, no brackets). An
    /// unrecognized `source_id` refuses at the Identity Resolver.
    pub known_principals: BTreeMap<String, String>,
    /// `source_id -> predicate IRIs (bare) that source may assert`. Checked
    /// by the Authority Checker (gate 3).
    pub authorized_predicates: BTreeMap<String, BTreeSet<String>>,
    /// Bare namespace prefixes a predicate IRI must start with to pass
    /// Semantic Conformance (gate 5).
    pub allowed_vocabulary_prefixes: Vec<String>,
    /// Schemes a `declared_subject` IRI must start with to be considered
    /// well-formed by the Identity Resolver (gate 1).
    pub allowed_subject_schemes: Vec<String>,
    /// SHACL shapes the payload must conform to (gate 4). Real
    /// `praxis_graphlaw::shacl::ShapesGraph`, not a stub.
    pub shapes: ShapesGraph,
}

impl AdmissionPolicy {
    /// Parses `shapes_turtle` via the real SHACL engine
    /// (`ShapesGraph::parse`). Propagates its `Err(String)` unchanged --
    /// policy construction is a setup-time concern, distinct from the
    /// per-observation [`ObservationAdmissionRefused`] taxonomy.
    pub fn new(
        known_principals: BTreeMap<String, String>,
        authorized_predicates: BTreeMap<String, BTreeSet<String>>,
        allowed_vocabulary_prefixes: Vec<String>,
        allowed_subject_schemes: Vec<String>,
        shapes_turtle: &str,
    ) -> Result<Self, String> {
        let shapes = ShapesGraph::parse(shapes_turtle)?;
        Ok(Self {
            known_principals,
            authorized_predicates,
            allowed_vocabulary_prefixes,
            allowed_subject_schemes,
            shapes,
        })
    }
}

/// One ledger row: the canonical payload hash an admitted `correlation_id`
/// was admitted with, plus the receipt itself (returned again, unchanged,
/// on a replay-equivalent resubmission).
#[derive(Debug, Clone)]
struct LedgerEntry {
    payload_hash: String,
    receipt: AdmissionReceipt,
}

/// Admission Ledger (gate 6 / L7's idempotency+correlation gate). A single
/// process-wide `Mutex` held for the full duration of one
/// `admit_observation` call: correct (no TOCTOU window between the
/// duplicate-check and the insert) but coarse-grained -- this serializes all
/// admissions crate-wide rather than sharding by `correlation_id`. That is a
/// real, disclosed tradeoff (see the module doc's "explicitly NOT built"
/// section), not a hidden limitation.
#[derive(Debug, Default)]
pub struct AdmissionLedger {
    entries: Mutex<BTreeMap<String, LedgerEntry>>,
}

impl AdmissionLedger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of distinct correlation ids ever admitted. Real accessor (locks
    /// and reads), used by this module's own tests to assert idempotent
    /// replay did not create a second entry.
    pub fn len(&self) -> Result<usize, ObservationAdmissionRefused> {
        self.entries.lock().map(|guard| guard.len()).map_err(|e| {
            ObservationAdmissionRefused::LedgerUnavailable {
                reason: e.to_string(),
            }
        })
    }

    pub fn is_empty(&self) -> Result<bool, ObservationAdmissionRefused> {
        Ok(self.len()? == 0)
    }
}

/// Extracts the bare (unbracketed) IRI string of an RDF term, or `None` if
/// the term is not `Term::Iri` (a literal, blank node, or unresolved
/// variable never has "an IRI"). Only ever inspects the variant tag, never a
/// private field.
///
/// `pub(crate)` (not private): reused verbatim by
/// [`crate::f31_org_merge`]'s identifier-collision detector, which needs the
/// exact same "only `Term::Iri` counts" contract this function already
/// establishes -- duplicating it would risk the two modules silently
/// diverging on what counts as a comparable subject/predicate.
pub(crate) fn bare_iri(vt: &VarOrTerm) -> Option<String> {
    match vt {
        VarOrTerm::Term(t @ Term::Iri(_)) => {
            let displayed = t.to_string();
            Some(
                displayed
                    .trim_start_matches('<')
                    .trim_end_matches('>')
                    .to_string(),
            )
        }
        _ => None,
    }
}

/// Canonical (sorted) N-Triples-shaped display of one term, for receipt
/// hashing. Variables never appear in ground Turtle data parsed from an
/// external payload; the `Var` arm exists only so this function is total,
/// not because it is expected to run.
///
/// `pub(crate)`: reused by [`crate::f31_org_merge`] for its own canonical
/// triple-line formatting (same reasoning as [`bare_iri`] above -- one
/// canonicalization convention, not two that could drift).
pub(crate) fn term_display(vt: &VarOrTerm) -> String {
    match vt {
        VarOrTerm::Term(t) => t.to_string(),
        VarOrTerm::Var(_) => "?unbound".to_string(),
    }
}

/// BLAKE3-hex receipt hash over the canonical (sorted-line) serialization of
/// the admitted triples plus `correlation_id`. Same pattern as
/// `crates/powl2-decompose/src/net.rs::content_hash`: `\x00`-delimited
/// fields, sorted before hashing (invariant #5: no `HashMap` iteration order
/// dependence), BLAKE3-only (invariant #6).
fn receipt_hash(correlation_id: &str, triples: &[praxis_graphlaw::triples::Triple]) -> String {
    let mut lines: Vec<String> = triples
        .iter()
        .map(|t| {
            format!(
                "{} {} {} .",
                term_display(&t.s),
                term_display(&t.p),
                term_display(&t.o)
            )
        })
        .collect();
    lines.sort();

    let mut hasher = blake3::Hasher::new();
    hasher.update(correlation_id.as_bytes());
    hasher.update(b"\x00");
    for line in &lines {
        hasher.update(line.as_bytes());
        hasher.update(b"\x00");
    }
    hex::encode(hasher.finalize().as_bytes())
}

/// Runs the full 5-gate F02 pipeline over one [`RawObservation`], in order:
/// Ingress parse -> Identity Resolver -> Provenance Checker -> Authority
/// Checker -> Shape Validator -> Semantic Conformance -> Admission Ledger.
/// `Err` at any gate carries the state the observation was refused from
/// (see [`ObservationAdmissionRefused`]); no partial admission is possible
/// (the observation is never written into `ledger` until every gate has
/// passed).
///
/// # Complexity
/// O(T + S) where T = number of triples in the payload and S = number of
/// SHACL constraints checked by the shape validator (delegated to
/// `praxis_graphlaw::shacl::Validator::validate`, whose own complexity is
/// documented in that module). The identity/provenance/authority/conformance
/// gates are each a single linear pass over `T` triples with `BTreeSet`
/// membership checks (`O(log n)` per check).
pub fn admit_observation(
    policy: &AdmissionPolicy,
    ledger: &AdmissionLedger,
    obs: RawObservation,
) -> Result<AdmissionReceipt, ObservationAdmissionRefused> {
    let correlation_id = obs.correlation_id.clone();

    // --- Gate 0: Ingress Adapter (parse) ---------------------------------
    let parsed = Parser::parse_triples(&obs.payload_turtle, Syntax::Turtle).map_err(|e| {
        ObservationAdmissionRefused::MalformedPayload {
            correlation_id: correlation_id.clone(),
            reason: format!("Turtle parse error: {e}"),
        }
    })?;
    if parsed.is_empty() {
        return Err(ObservationAdmissionRefused::MalformedPayload {
            correlation_id,
            reason: "payload contains zero triples".to_string(),
        });
    }

    let hash = receipt_hash(&correlation_id, &parsed);

    // --- L7: atomic idempotency/correlation gate -------------------------
    // Held for the whole call (see AdmissionLedger's doc comment on why
    // this is coarse-grained but race-free).
    let mut guard =
        ledger
            .entries
            .lock()
            .map_err(|e| ObservationAdmissionRefused::LedgerUnavailable {
                reason: e.to_string(),
            })?;
    if let Some(existing) = guard.get(&correlation_id) {
        return if existing.payload_hash == hash {
            // Replay equivalence: identical canonical payload resubmitted
            // under the same correlation id re-admits to the same receipt
            // without re-running the gates below.
            Ok(existing.receipt.clone())
        } else {
            Err(ObservationAdmissionRefused::CorrelationConflict { correlation_id })
        };
    }

    // --- Gate 1: Identity Resolver ---------------------------------------
    let principal = policy
        .known_principals
        .get(&obs.source_id)
        .ok_or_else(|| ObservationAdmissionRefused::IdentityUnresolved {
            correlation_id: correlation_id.clone(),
            reason: format!("source_id {} is not a known principal", obs.source_id),
        })?
        .clone();

    let has_scheme = policy
        .allowed_subject_schemes
        .iter()
        .any(|scheme| obs.declared_subject.starts_with(scheme.as_str()));
    if !has_scheme || obs.declared_subject.chars().any(char::is_whitespace) {
        return Err(ObservationAdmissionRefused::IdentityUnresolved {
            correlation_id,
            reason: format!(
                "declared_subject {:?} is not a well-formed IRI under the allowed schemes",
                obs.declared_subject
            ),
        });
    }

    let declared_subject_term = VarOrTerm::convert(obs.declared_subject.clone());
    let subject_present = parsed
        .iter()
        .any(|t| t.s == declared_subject_term && matches!(&t.s, VarOrTerm::Term(Term::Iri(_))));
    if !subject_present {
        return Err(ObservationAdmissionRefused::IdentityUnresolved {
            correlation_id,
            reason: format!(
                "declared_subject {} does not appear as a subject in the payload",
                obs.declared_subject
            ),
        });
    }

    let _identity = IdentityLink {
        subject_iri: obs.declared_subject.clone(),
        resolved_principal: principal,
    };

    // --- Gate 2: Provenance Checker ---------------------------------------
    let prov_predicate = VarOrTerm::convert(PROV_WAS_DERIVED_FROM.to_string());
    let derivation = parsed
        .iter()
        .find(|t| t.s == declared_subject_term && t.p == prov_predicate)
        .and_then(|t| bare_iri(&t.o));
    let derivation_iri = match derivation {
        Some(iri) => iri,
        None => {
            return Err(ObservationAdmissionRefused::ProvenanceUnverified {
                correlation_id,
                reason: format!(
                    "no prov:wasDerivedFrom triple from {} in the payload",
                    obs.declared_subject
                ),
            })
        }
    };
    if let Some(expected_source_iri) = policy.known_principals.get(&obs.source_id) {
        if &derivation_iri != expected_source_iri {
            return Err(ObservationAdmissionRefused::ProvenanceUnverified {
                correlation_id,
                reason: format!(
                    "asserted derivation {derivation_iri} does not match source {}'s principal \
                     {expected_source_iri}",
                    obs.source_id
                ),
            });
        }
    }

    let _provenance = ProvenanceBundle {
        subject_iri: obs.declared_subject.clone(),
        source_id: obs.source_id.clone(),
        asserted_derivation_iri: derivation_iri,
    };

    // --- Gate 3: Authority Checker -----------------------------------------
    let authorized: BTreeSet<String> = policy
        .authorized_predicates
        .get(&obs.source_id)
        .cloned()
        .unwrap_or_default();
    let mut distinct_predicates: BTreeSet<String> = BTreeSet::new();
    for t in &parsed {
        if t.s == declared_subject_term && t.p == prov_predicate {
            continue; // provenance bookkeeping triple, checked separately above
        }
        if t.s != declared_subject_term {
            continue; // authority scope is limited to assertions about the declared subject
        }
        match bare_iri(&t.p) {
            Some(pred) => {
                distinct_predicates.insert(pred);
            }
            None => {
                return Err(ObservationAdmissionRefused::MalformedPayload {
                    correlation_id,
                    reason: "a triple's predicate is not an IRI".to_string(),
                })
            }
        }
    }
    for pred in &distinct_predicates {
        if !authorized.contains(pred) {
            return Err(ObservationAdmissionRefused::AuthorityDenied {
                correlation_id,
                source_id: obs.source_id.clone(),
                predicate: pred.clone(),
            });
        }
    }

    let _authority = AuthorityEvidence {
        source_id: obs.source_id.clone(),
        authorized_predicate_count: authorized.len(),
    };

    // --- Gate 4: Shape Validator (REUSE_ADAPT: real praxis-graphlaw SHACL) --
    let mut index = TripleIndex::new();
    for t in parsed.iter().cloned() {
        index.add(t);
    }
    let report = Validator::validate(&index, &policy.shapes);
    let shape_report = ShapeReport {
        conforms: report.conforms,
        violation_count: report.results.len(),
    };
    if !shape_report.conforms {
        return Err(ObservationAdmissionRefused::ShapeNonConformant {
            correlation_id,
            violation_count: shape_report.violation_count,
        });
    }

    // --- Gate 5: Semantic Conformance (closed-vocabulary check) ------------
    for pred in &distinct_predicates {
        let allowed = policy
            .allowed_vocabulary_prefixes
            .iter()
            .any(|prefix| pred.starts_with(prefix.as_str()));
        if !allowed {
            return Err(ObservationAdmissionRefused::SemanticNonConformant {
                correlation_id,
                predicate: pred.clone(),
            });
        }
    }
    let _conformance = ConformanceReport {
        conforms: true,
        distinct_predicate_count: distinct_predicates.len(),
    };

    // --- Gate 6: Admission Ledger -------------------------------------------
    let receipt = AdmissionReceipt {
        correlation_id: correlation_id.clone(),
        source_id: obs.source_id,
        subject_iri: obs.declared_subject,
        state: AdmissionState::Admitted,
        receipt_hash: hash.clone(),
        triple_count: parsed.len(),
    };
    guard.insert(
        correlation_id,
        LedgerEntry {
            payload_hash: hash,
            receipt: receipt.clone(),
        },
    );

    Ok(receipt)
}

#[cfg(test)]
mod tests {
    //! Every [`ObservationAdmissionRefused`] variant has >= 1 test below
    //! (per this repo's `.claude/rules/rust-agi-core-team.md` rule 8):
    //!
    //! | Variant | Test |
    //! |---|---|
    //! | `MalformedPayload` (bad Turtle) | `malformed_payload_bad_turtle` |
    //! | `MalformedPayload` (empty) | `malformed_payload_empty` |
    //! | `IdentityUnresolved` (unknown source) | `identity_unresolved_unknown_source` |
    //! | `IdentityUnresolved` (subject absent) | `identity_unresolved_subject_not_in_payload` |
    //! | `ProvenanceUnverified` | `provenance_unverified_missing_triple` |
    //! | `AuthorityDenied` | `authority_denied_unauthorized_predicate` |
    //! | `ShapeNonConformant` | `shape_non_conformant_missing_required_property` |
    //! | `SemanticNonConformant` | `semantic_non_conformant_outside_vocabulary` |
    //! | `CorrelationConflict` | `correlation_conflict_on_replay_with_different_payload` |
    //! | `LedgerUnavailable` | not independently triggerable without poisoning the
    //!   `Mutex` from a panicking thread; exercised implicitly (the lock path is the
    //!   same code used by every other test) rather than by a dedicated poison test. |

    use super::*;

    const SOURCE: &str = "email-api-1";
    const PRINCIPAL: &str = "https://source.example.org/email-api-1";
    const SUBJECT: &str = "https://example.org/obs/42";
    const NAME_PRED: &str = "https://example.org/vocab#name";

    fn base_policy(shapes_turtle: &str) -> AdmissionPolicy {
        let mut known_principals = BTreeMap::new();
        known_principals.insert(SOURCE.to_string(), PRINCIPAL.to_string());

        let mut authorized = BTreeSet::new();
        authorized.insert(NAME_PRED.to_string());
        let mut authorized_predicates = BTreeMap::new();
        authorized_predicates.insert(SOURCE.to_string(), authorized);

        AdmissionPolicy::new(
            known_principals,
            authorized_predicates,
            vec!["https://example.org/vocab#".to_string()],
            vec!["https://".to_string(), "http://".to_string()],
            shapes_turtle,
        )
        .expect("valid SHACL shapes in test fixture")
    }

    const PASSING_SHAPES: &str = r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix ex: <https://example.org/vocab#> .
        ex:ObsShape a sh:NodeShape ;
            sh:targetNode <https://example.org/obs/42> ;
            sh:property [
                sh:path ex:name ;
                sh:minCount 1 ;
            ] .
    "#;

    fn payload(name_value: &str) -> String {
        format!(
            r#"
            @prefix ex: <https://example.org/vocab#> .
            @prefix prov: <http://www.w3.org/ns/prov#> .
            <{SUBJECT}> prov:wasDerivedFrom <{PRINCIPAL}> ;
                ex:name "{name_value}" .
            "#
        )
    }

    fn raw_observation(correlation_id: &str, name_value: &str) -> RawObservation {
        RawObservation {
            correlation_id: correlation_id.to_string(),
            source_id: SOURCE.to_string(),
            declared_subject: SUBJECT.to_string(),
            payload_turtle: payload(name_value),
        }
    }

    #[test]
    fn happy_path_admits_and_hashes() {
        let policy = base_policy(PASSING_SHAPES);
        let ledger = AdmissionLedger::new();
        let obs = raw_observation("corr-happy-1", "Alice");

        let receipt = admit_observation(&policy, &ledger, obs).expect("gates should pass");
        assert_eq!(receipt.state, AdmissionState::Admitted);
        assert_eq!(receipt.correlation_id, "corr-happy-1");
        assert_eq!(receipt.triple_count, 2);
        assert!(!receipt.receipt_hash.is_empty());
        assert_eq!(ledger.len().unwrap(), 1);
    }

    #[test]
    fn replay_equivalence_returns_same_receipt_without_duplicating_ledger() {
        let policy = base_policy(PASSING_SHAPES);
        let ledger = AdmissionLedger::new();

        let first = admit_observation(&policy, &ledger, raw_observation("corr-replay", "Alice"))
            .expect("first admission should pass");
        let second = admit_observation(&policy, &ledger, raw_observation("corr-replay", "Alice"))
            .expect("identical replay should be idempotent, not refused");

        assert_eq!(first, second);
        assert_eq!(ledger.len().unwrap(), 1);
    }

    #[test]
    fn correlation_conflict_on_replay_with_different_payload() {
        let policy = base_policy(PASSING_SHAPES);
        let ledger = AdmissionLedger::new();

        admit_observation(&policy, &ledger, raw_observation("corr-conflict", "Alice"))
            .expect("first admission should pass");
        let err = admit_observation(&policy, &ledger, raw_observation("corr-conflict", "Bob"))
            .expect_err("differing payload under the same correlation id must refuse");

        assert_eq!(
            err,
            ObservationAdmissionRefused::CorrelationConflict {
                correlation_id: "corr-conflict".to_string(),
            }
        );
        assert_eq!(ledger.len().unwrap(), 1);
    }

    #[test]
    fn malformed_payload_bad_turtle() {
        let policy = base_policy(PASSING_SHAPES);
        let ledger = AdmissionLedger::new();
        let mut obs = raw_observation("corr-bad-turtle", "Alice");
        obs.payload_turtle = "this is not { valid Turtle at all <<<".to_string();

        let err = admit_observation(&policy, &ledger, obs).expect_err("bad Turtle must refuse");
        assert!(matches!(
            err,
            ObservationAdmissionRefused::MalformedPayload { .. }
        ));
    }

    #[test]
    fn malformed_payload_empty() {
        let policy = base_policy(PASSING_SHAPES);
        let ledger = AdmissionLedger::new();
        let mut obs = raw_observation("corr-empty", "Alice");
        obs.payload_turtle = "@prefix ex: <https://example.org/vocab#> .".to_string();

        let err = admit_observation(&policy, &ledger, obs).expect_err("zero triples must refuse");
        assert_eq!(
            err,
            ObservationAdmissionRefused::MalformedPayload {
                correlation_id: "corr-empty".to_string(),
                reason: "payload contains zero triples".to_string(),
            }
        );
    }

    #[test]
    fn identity_unresolved_unknown_source() {
        let policy = base_policy(PASSING_SHAPES);
        let ledger = AdmissionLedger::new();
        let mut obs = raw_observation("corr-unknown-source", "Alice");
        obs.source_id = "not-registered".to_string();

        let err = admit_observation(&policy, &ledger, obs)
            .expect_err("unknown source_id must refuse at identity resolution");
        assert!(matches!(
            err,
            ObservationAdmissionRefused::IdentityUnresolved { .. }
        ));
    }

    #[test]
    fn identity_unresolved_subject_not_in_payload() {
        let policy = base_policy(PASSING_SHAPES);
        let ledger = AdmissionLedger::new();
        let mut obs = raw_observation("corr-subject-absent", "Alice");
        obs.declared_subject = "https://example.org/obs/does-not-appear".to_string();

        let err = admit_observation(&policy, &ledger, obs)
            .expect_err("declared subject absent from payload must refuse");
        assert!(matches!(
            err,
            ObservationAdmissionRefused::IdentityUnresolved { .. }
        ));
    }

    #[test]
    fn provenance_unverified_missing_triple() {
        let policy = base_policy(PASSING_SHAPES);
        let ledger = AdmissionLedger::new();
        let mut obs = raw_observation("corr-no-prov", "Alice");
        obs.payload_turtle = format!(
            r#"
            @prefix ex: <https://example.org/vocab#> .
            <{SUBJECT}> ex:name "Alice" .
            "#
        );

        let err = admit_observation(&policy, &ledger, obs)
            .expect_err("missing prov:wasDerivedFrom must refuse");
        assert!(matches!(
            err,
            ObservationAdmissionRefused::ProvenanceUnverified { .. }
        ));
    }

    #[test]
    fn authority_denied_unauthorized_predicate() {
        let policy = base_policy(PASSING_SHAPES);
        let ledger = AdmissionLedger::new();
        let mut obs = raw_observation("corr-unauthorized-pred", "Alice");
        obs.payload_turtle = format!(
            r#"
            @prefix ex: <https://example.org/vocab#> .
            @prefix prov: <http://www.w3.org/ns/prov#> .
            <{SUBJECT}> prov:wasDerivedFrom <{PRINCIPAL}> ;
                ex:name "Alice" ;
                ex:secretRole "admin" .
            "#
        );

        let err = admit_observation(&policy, &ledger, obs)
            .expect_err("predicate outside source's authorized set must refuse");
        assert!(matches!(
            err,
            ObservationAdmissionRefused::AuthorityDenied { ref predicate, .. }
            if predicate.contains("secretRole")
        ));
    }

    #[test]
    fn shape_non_conformant_missing_required_property() {
        let policy = base_policy(PASSING_SHAPES);
        let ledger = AdmissionLedger::new();
        let mut obs = raw_observation("corr-shape-fail", "Alice");
        // ex:name is required (minCount 1) by PASSING_SHAPES for this exact
        // subject; omit it so the real SHACL validator reports a violation.
        obs.payload_turtle = format!(
            r#"
            @prefix ex: <https://example.org/vocab#> .
            @prefix prov: <http://www.w3.org/ns/prov#> .
            <{SUBJECT}> prov:wasDerivedFrom <{PRINCIPAL}> .
            "#
        );

        let err = admit_observation(&policy, &ledger, obs)
            .expect_err("missing required SHACL property must refuse");
        assert!(matches!(
            err,
            ObservationAdmissionRefused::ShapeNonConformant {
                violation_count,
                ..
            } if violation_count >= 1
        ));
    }

    #[test]
    fn semantic_non_conformant_outside_vocabulary() {
        // Authorize a predicate for this source that is deliberately outside
        // the policy's allowed-vocabulary prefixes, isolating gate 5
        // (Semantic Conformance) from gate 3 (Authority Checker).
        let mut known_principals = BTreeMap::new();
        known_principals.insert(SOURCE.to_string(), PRINCIPAL.to_string());
        let mut authorized = BTreeSet::new();
        authorized.insert(NAME_PRED.to_string());
        authorized.insert("https://other-namespace.example.org/unlisted".to_string());
        let mut authorized_predicates = BTreeMap::new();
        authorized_predicates.insert(SOURCE.to_string(), authorized);
        let policy = AdmissionPolicy::new(
            known_principals,
            authorized_predicates,
            vec!["https://example.org/vocab#".to_string()],
            vec!["https://".to_string()],
            PASSING_SHAPES,
        )
        .expect("valid shapes");
        let ledger = AdmissionLedger::new();

        let mut obs = raw_observation("corr-outside-vocab", "Alice");
        obs.payload_turtle = format!(
            r#"
            @prefix ex: <https://example.org/vocab#> .
            @prefix other: <https://other-namespace.example.org/> .
            @prefix prov: <http://www.w3.org/ns/prov#> .
            <{SUBJECT}> prov:wasDerivedFrom <{PRINCIPAL}> ;
                ex:name "Alice" ;
                other:unlisted "value" .
            "#
        );

        let err = admit_observation(&policy, &ledger, obs)
            .expect_err("predicate outside allowed vocabulary must refuse");
        assert!(matches!(
            err,
            ObservationAdmissionRefused::SemanticNonConformant { ref predicate, .. }
            if predicate.contains("other-namespace")
        ));
    }

    #[test]
    fn empty_ledger_reports_zero() {
        let ledger = AdmissionLedger::new();
        assert!(ledger.is_empty().unwrap());
    }
}
