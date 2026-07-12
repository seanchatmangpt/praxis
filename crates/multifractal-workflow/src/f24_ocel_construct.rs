//! Family F24 -- "OCEL CONSTRUCT Capitalization" (atlas ticket V12-024).
//!
//! Survey verdict: **MIXED**. Per `.claude/rules/no-overclaiming.md`, this doc
//! comment states plainly which parts are real (verified this session by reading
//! the wrapped dependency source and by running this file's own tests) and which
//! parts are honest not-yet-implemented stubs -- no part is dressed up to look
//! more complete than it is.
//!
//! ## What is REAL (ALREADY_BUILT: thin-wraps existing, tested `cng` code)
//!
//! - **SPARQL CONSTRUCT Engine** ([`run_construct`], wrapping
//!   [`cng::otel_ocel::project_otel_to_ocel`]) executes a real oxigraph
//!   `SparqlEvaluator` CONSTRUCT query (`crates/cng/src/queries/otel-to-ocel.construct.rq`,
//!   `include_str!`'d, not a stub) against an admitted `G_OTEL` graph to derive
//!   `G_OCEL` -- never imperative Rust object construction. This module does not
//!   reimplement that engine; it calls it.
//! - **G_OCEL / G_RECEIPT writers** ([`run_construct`]) insert the CONSTRUCT
//!   result into `urn:graph:ocel` and the PROV-O + digest-chain receipt into
//!   `urn:graph:receipts` via [`cng::otel_ocel::insert_quads`] -- distinct named
//!   graphs with real, non-aliased content (proven by `cng`'s own
//!   `otel_ocel_test.rs`, re-exercised end to end by this module's own test
//!   below).
//! - **Transformation Provenance** ([`run_construct`], via
//!   [`cng::otel_receipt::receipt_otel_to_ocel`]) records PROV-O ancestry plus a
//!   `digest(P) + digest(G_OTEL) -> digest(G_OCEL)` receipt chain.
//! - **Graph Equivalence Replay** ([`run_construct`], via
//!   [`cng::otel_receipt::verify_receipt_otel_to_ocel`]) independently
//!   recomputes the receipt head from the store's own content and refuses on
//!   drift before returning -- the returned head is proven computed, not merely
//!   asserted, the same discipline `crates/cng/src/bin/otel-rdf-demo.rs`'s real
//!   (non-test) entrypoint already exercises.
//! - **G_RESULT writer** ([`build_default_measurement_profile`], wrapping
//!   [`cng::measurement::compute_execution_measure`] /
//!   [`cng::measurement::build_measurement_profile`] /
//!   [`cng::measurement::project_measurement_profile`]) runs a real SPARQL
//!   SELECT mass-by-workflow query over `G_OCEL` and projects a computed
//!   [`cng::measurement::MeasurementProfile`] into `urn:graph:results`. The
//!   `q_range`/`fitting_method`/`min_evidence_threshold`/`confidence_criteria`
//!   values this function passes are **this module's own default choice**, not
//!   sourced from a PRD-mandated default (not verified against PRD.md sec.16/17
//!   this session) -- disclosed here rather than presented as spec-derived.
//!
//! ## What is genuinely HAND_WRITTEN here (new, but small and real)
//!
//! - **CONSTRUCT Profile Resolver** ([`ConstructProfile::resolve`]): `cng`
//!   exposes exactly one hardcoded CONSTRUCT query (confirmed at survey time:
//!   zero hits for `ProfileResolver`/"projection profile" anywhere in
//!   `crates/cng/src`), so there was no multi-profile registry to wrap. This
//!   module adds a real, closed, by-name resolver over the one profile that
//!   exists today (`"otel-to-ocel"`) that genuinely refuses unknown names
//!   ([`OCELConstructionRefused::UnknownProfile`]) rather than being a decorative
//!   pass-through -- structured so a second profile can be added as a new match
//!   arm, not a placeholder that always succeeds.
//! - **`OCELConstructionRefused`** (the atlas's own refusal name -- confirmed at
//!   survey time to have zero repo-wide hits; the existing `cng` variant is the
//!   similarly-named but distinct `CngRefusal::OcelConstructRefused`). This
//!   module's own typed refusal wraps `cng`'s refusals
//!   ([`OCELConstructionRefused::Upstream`], `#[from] CngRefusal` -- covers
//!   invalid CONSTRUCT input and stale/non-replaying receipt-head mismatches,
//!   since `cng::otel_receipt::verify_receipt_otel_to_ocel` already refuses
//!   those with `CngRefusal::AuditMismatch`) and adds this module's own
//!   [`OCELConstructionRefused::UnknownProfile`] and
//!   [`OCELConstructionRefused::NotYetImplemented`] variants.
//!
//! ## What is an HONEST STUB (HAND_WRITE_REQUIRED, tracked under V12-024)
//!
//! No existing praxis or `~/` code builds either of the following (confirmed at
//! survey time by a repo-wide grep for this family's own vocabulary -- zero
//! hits for idempotency/correlation/duplicate/restart/recovery in
//! `otel_ocel.rs`/`otel_receipt.rs`) and neither is implemented here; each fails
//! loud with [`OCELConstructionRefused::NotYetImplemented`] rather than faking
//! success:
//!
//! - [`idempotency_gate`] -- the L7 "atomic idempotency and correlation gate"
//!   (duplicate-event / restart / stale-result chaos tolerance, durable
//!   receipt-head recovery). This is nontrivial concurrent-systems engineering
//!   (durable dedup keys, receipt-head recovery across process restart) with no
//!   existing code in this repo to adapt -- confirmed absent, not merely
//!   unwired.
//! - [`mfw_feedback_adapter`] -- the MFW Feedback Adapter. Its stated job is to
//!   feed an [`OcelConstructOutcome`] into `multifractal-workflow` as a required
//!   consequence (e.g. into F09 "MFW Growth Operator"). Read this session:
//!   `crate::f09_mfw_growth` is itself still a Wire-phase module with no
//!   consequence-intake function to call into (its own `plan_growth` takes an
//!   already-resolved `ContinuationGoal`, not an OCEL outcome), so there is
//!   nothing real on the other end of this adapter to wire to yet. This function
//!   refuses loudly rather than silently dropping the consequence or pretending
//!   it was fed somewhere.
//!
//! The three v26.7.12 claim-gate booleans this family's survey names
//! (`OTEL_OCEL_PRODUCTION_REACHABLE`, `CONSTRUCT_SEMANTIC_CAPITALIZATION_PROVEN`,
//! `OCEL_PROCESS_EVIDENCE_PROVEN`) are **not** claimed by this module: they are
//! standing-index claims scoped to the whole milestone's evidence chain, not
//! something a single crate module emits, and this module does not touch
//! `target/praxis-standing/standing.json`.
//!
//! Survey-cited paths for F24 (informed research from the v26.7.12 family
//! survey handed to this wiring session inline):
//! - /Users/sac/Downloads/v26.7.12_mermaid_atlas/families/F24_ocel-construct.md
//! - /Users/sac/praxis/crates/cng/src/otel_ocel.rs
//! - /Users/sac/praxis/crates/cng/src/otel_ocel_test.rs
//! - /Users/sac/praxis/crates/cng/src/otel_receipt.rs
//! - /Users/sac/praxis/crates/cng/src/measurement.rs
//! - /Users/sac/praxis/crates/cng/src/otel_rdf.rs
//! - /Users/sac/praxis/crates/cng/src/bin/otel-rdf-demo.rs
//! - /Users/sac/praxis/crates/cng/src/queries/otel-to-ocel.construct.rq
//! - /Users/sac/praxis/crates/cng/src/powl.rs
//! - /Users/sac/praxis/crates/cng/src/lib.rs
//! - /Users/sac/praxis/docs/otel-rdf-handoff.md
//! - /Users/sac/praxis/crates/multifractal-workflow/src/f09_mfw_growth.rs

use cng::powl::CngRefusal;
use oxigraph::model::{NamedNode, Quad, Term};
use oxigraph::store::Store;

/// Thin re-exports of the ALREADY_BUILT `cng` primitives this module wraps --
/// callers that only need the underlying `G_OTEL -> G_OCEL` pipeline (without
/// F24's profile-resolver gate) can reach it directly through this module
/// rather than depending on `cng` themselves.
pub use cng::measurement::{
    build_measurement_profile, compute_execution_measure, project_measurement_profile,
    DeclaredProcessScale, ExecutionMeasure, MeasurementProfile,
};
pub use cng::otel_ocel::{
    graph_content_digest, insert_quads, load_source_graph, project_otel_to_ocel, OCEL_GRAPH_IRI,
    OTEL_GRAPH_IRI, RECEIPT_GRAPH_IRI, RESULT_GRAPH_IRI, SOURCE_GRAPH_IRI,
};
pub use cng::otel_receipt::{receipt_otel_to_ocel, verify_receipt_otel_to_ocel};

/// The `cngr:receiptHead` predicate IRI. `cng::otel_receipt`'s own `CNGR_NS`
/// binding is private to that module, so this constant independently names the
/// same IRI rather than reaching into the module's internals -- mirroring
/// `crates/cng/src/bin/otel-rdf-demo.rs::RECEIPT_HEAD_PRED_IRI`'s identical
/// choice and its own documented rationale (drift would be caught immediately,
/// as a broken [`extract_receipt_head`] lookup, rather than silently).
const RECEIPT_HEAD_PRED_IRI: &str = "https://truex.io/ontology/cng-receipt#receiptHead";

/// F24's typed refusal taxonomy (atlas ticket V12-024). The atlas names this
/// exact type `OCELConstructionRefused`; the pre-existing, similarly-named
/// `cng::powl::CngRefusal::OcelConstructRefused` variant is a different type
/// this enum wraps (see [`OCELConstructionRefused::Upstream`]), not renamed in
/// place, since `cng`'s own refusal algebra and stable `CNG_R*` codes are a
/// contract this module must not disturb.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OCELConstructionRefused {
    /// The requested [`ConstructProfile`] name is not in the closed set this
    /// resolver knows about. Real refusal, not a decorative always-succeeds
    /// pass-through: exactly one profile name resolves today, and every other
    /// input reaches this arm.
    #[error("unknown CONSTRUCT profile {requested:?}; available profiles: {available:?}")]
    UnknownProfile {
        /// The profile name that was requested.
        requested: String,
        /// The full set of profile names this resolver currently accepts.
        available: Vec<String>,
    },
    /// A `cng`-level refusal propagated unchanged: invalid CONSTRUCT input
    /// (`CngRefusal::OcelConstructRefused`, e.g. malformed query/graph
    /// input), or a stale/non-replaying receipt (`CngRefusal::AuditMismatch`,
    /// raised by [`cng::otel_receipt::verify_receipt_otel_to_ocel`] when a
    /// claimed receipt head does not match what replay recomputes), or a
    /// measurement-evidence gap (`CngRefusal::MeasurementEvidenceInsufficient`)
    /// from the G_RESULT writer.
    #[error("upstream cng refusal during OCEL construction: {0}")]
    Upstream(#[from] CngRefusal),
    /// The receipt quads a CONSTRUCT run produced carry no `cngr:receiptHead`
    /// literal -- would mean `cng::otel_receipt::receipt_otel_to_ocel`'s own
    /// documented output shape changed underneath this caller (defensive;
    /// unreachable in practice against the current `cng` version this crate
    /// depends on).
    #[error("receipt quads from a CONSTRUCT run carried no cngr:receiptHead literal")]
    MissingReceiptHead,
    /// A pipeline stage that is genuinely `HAND_WRITE_REQUIRED` and not yet
    /// built was reached. Fails loud rather than faking success -- see the
    /// module doc comment's "HONEST STUB" section.
    #[error(
        "F24 stage `{stage}` is HAND_WRITE_REQUIRED and not yet implemented (V12-024): {detail}"
    )]
    NotYetImplemented {
        /// The unbuilt stage's name.
        stage: &'static str,
        /// Why it is not built and what would be needed.
        detail: &'static str,
    },
}

/// The CONSTRUCT Profile Resolver's closed set of named projection profiles
/// over an admitted `G_OTEL` graph. Real but genuinely small: `cng` has exactly
/// one CONSTRUCT query today (`crates/cng/src/queries/otel-to-ocel.construct.rq`),
/// so this enum has exactly one variant -- adding a second real profile means
/// adding both a variant here and the underlying query in `cng`, not editing
/// this resolver's shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstructProfile {
    /// The `G_OTEL -> G_OCEL` projection ([`cng::otel_ocel::project_otel_to_ocel`]).
    OtelToOcel,
}

impl ConstructProfile {
    /// The profile's stable name, as accepted by [`Self::resolve`].
    ///
    /// # Complexity
    /// O(1).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::OtelToOcel => "otel-to-ocel",
        }
    }

    /// All profile names this resolver currently accepts, in a fixed order.
    ///
    /// # Complexity
    /// O(1): one fixed-size array today.
    #[must_use]
    pub const fn available() -> &'static [&'static str] {
        &["otel-to-ocel"]
    }

    /// Resolves `name` to a [`ConstructProfile`], refusing any name outside
    /// [`Self::available`].
    ///
    /// # Errors
    /// [`OCELConstructionRefused::UnknownProfile`] if `name` does not match a
    /// known profile.
    ///
    /// # Complexity
    /// O(1): one string comparison against the closed profile set.
    pub fn resolve(name: &str) -> Result<Self, OCELConstructionRefused> {
        match name {
            "otel-to-ocel" => Ok(Self::OtelToOcel),
            other => Err(OCELConstructionRefused::UnknownProfile {
                requested: other.to_string(),
                available: Self::available().iter().map(|s| (*s).to_string()).collect(),
            }),
        }
    }
}

/// The real, computed outcome of running one [`ConstructProfile`] against a
/// store's admitted `G_OTEL` content: the derived `G_OCEL` quads, the
/// `G_RECEIPT` PROV-O + digest-chain quads, and the receipt head those receipt
/// quads assert -- already independently replay-verified (Graph Equivalence
/// Replay) before [`run_construct`] returns it.
#[derive(Debug, Clone)]
pub struct OcelConstructOutcome {
    /// Which profile produced this outcome.
    pub profile: ConstructProfile,
    /// The derived `G_OCEL` quads (already inserted into the caller's store).
    pub ocel_quads: Vec<Quad>,
    /// The `G_RECEIPT` PROV-O + digest-chain quads (already inserted into the
    /// caller's store).
    pub receipt_quads: Vec<Quad>,
    /// The folded, independently-replay-verified receipt head.
    pub receipt_head: String,
}

/// Pulls the `cngr:receiptHead` literal out of the receipt quads
/// [`cng::otel_receipt::receipt_otel_to_ocel`] returned. Adapted from
/// `crates/cng/src/bin/otel-rdf-demo.rs::extract_receipt_head` (same shape,
/// re-derived here rather than imported since that binary's helper is private
/// to its own crate binary target).
///
/// # Errors
/// [`OCELConstructionRefused::MissingReceiptHead`] if no such quad exists.
///
/// # Complexity
/// O(r) in `receipt_quads.len()`.
fn extract_receipt_head(receipt_quads: &[Quad]) -> Result<String, OCELConstructionRefused> {
    let pred = NamedNode::new(RECEIPT_HEAD_PRED_IRI).map_err(|e| {
        OCELConstructionRefused::Upstream(CngRefusal::OcelConstructRefused {
            stage: "receipt_iri".to_string(),
            reason: format!("receipt head predicate IRI construction failed: {e}"),
        })
    })?;
    receipt_quads
        .iter()
        .find(|q| q.predicate == pred)
        .and_then(|q| match &q.object {
            Term::Literal(lit) => Some(lit.value().to_string()),
            _ => None,
        })
        .ok_or(OCELConstructionRefused::MissingReceiptHead)
}

/// The SPARQL CONSTRUCT Engine, end to end: resolves `profile_name` via the
/// CONSTRUCT Profile Resolver, runs the declared CONSTRUCT (never imperative
/// Rust OCEL construction) over `store`'s admitted `G_OTEL` content, writes
/// the derived `G_OCEL` and PROV-O/digest-chain `G_RECEIPT` quads into
/// `store`, and independently replay-verifies the receipt head before
/// returning (Graph Equivalence Replay) -- the returned
/// [`OcelConstructOutcome`] is proven computed, not merely asserted.
///
/// # Errors
/// [`OCELConstructionRefused::UnknownProfile`] if `profile_name` is not a
/// known profile; [`OCELConstructionRefused::Upstream`] if the CONSTRUCT
/// query fails (invalid input), the receipt computation fails, or replay
/// verification detects drift (stale/non-replaying result);
/// [`OCELConstructionRefused::MissingReceiptHead`] (defensive, unreachable in
/// practice) if the receipt quads carry no head literal.
///
/// # Complexity
/// O(m log m) where m is `max(|G_OTEL|, |G_OCEL|)`, dominated by the
/// canonical-content-digest sorts inside `cng::otel_ocel`/`cng::otel_receipt`.
pub fn run_construct(
    profile_name: &str,
    store: &Store,
) -> Result<OcelConstructOutcome, OCELConstructionRefused> {
    let profile = ConstructProfile::resolve(profile_name)?;

    // Only one profile exists today (`ConstructProfile::OtelToOcel`), so
    // dispatch is a single arm -- but the resolver's refuse-unknown-name
    // behavior above is real, not decorative, and this match is where a
    // second profile's dispatch would be added.
    let ocel_quads = match profile {
        ConstructProfile::OtelToOcel => cng::otel_ocel::project_otel_to_ocel(store)?,
    };
    cng::otel_ocel::insert_quads(store, &ocel_quads)?;

    let receipt_quads = cng::otel_receipt::receipt_otel_to_ocel(store)?;
    cng::otel_ocel::insert_quads(store, &receipt_quads)?;

    let receipt_head = extract_receipt_head(&receipt_quads)?;

    // Graph Equivalence Replay: independently recompute the receipt head from
    // the store's current content and refuse on drift. Proves the returned
    // head is computed, not merely asserted -- the same discipline
    // otel-rdf-demo.rs's real (non-test) entrypoint already exercises.
    cng::otel_receipt::verify_receipt_otel_to_ocel(store, &receipt_head)?;

    Ok(OcelConstructOutcome {
        profile,
        ocel_quads,
        receipt_quads,
        receipt_head,
    })
}

/// The G_RESULT writer, with this module's own default measurement-profile
/// parameters (see the module doc's disclosure that these are F24's own
/// defaults, not a verified PRD-mandated default): runs
/// [`cng::measurement::build_measurement_profile`] at
/// [`DeclaredProcessScale::Workflow`] (the scale `cng::measurement`'s own doc
/// comment confirms has a real `G_OCEL` data source today, grouped by
/// `process.workflow.id`), then [`cng::measurement::project_measurement_profile`]
/// to produce the `G_RESULT` quads. Does not insert the result into `store` --
/// mirrors the pure-function-then-insert split every other writer in this
/// module follows; callers materialize with [`insert_quads`].
///
/// # Errors
/// [`OCELConstructionRefused::Upstream`] if no admitted `G_OCEL` workflow
/// evidence exists yet, or the measured family count is below the (here,
/// deliberately permissive) minimum evidence threshold of 1.
///
/// # Complexity
/// O(e log e) where e is the admitted `G_OCEL` event count.
pub fn build_default_measurement_profile(
    store: &Store,
) -> Result<Vec<Quad>, OCELConstructionRefused> {
    let (profile, measures) = cng::measurement::build_measurement_profile(
        store,
        DeclaredProcessScale::Workflow,
        (-5..=5).collect(),
        "least-squares".to_string(),
        1,
        "single-run, no confidence interval computed (F24 default profile)".to_string(),
    )?;
    let quads = cng::measurement::project_measurement_profile(&profile, &measures)?;
    Ok(quads)
}

/// HAND_WRITE_REQUIRED (V12-024): the L7 "atomic idempotency and correlation
/// gate" -- duplicate-event / restart / stale-result chaos handling with
/// durable receipt-head recovery. Not built: confirmed at survey time by a
/// repo-wide grep for idempotency/correlation/duplicate/restart/recovery
/// vocabulary in `cng::otel_ocel`/`cng::otel_receipt` (zero hits), and this
/// module does not add that logic today -- an atomic idempotency/correlation
/// gate across process restarts is nontrivial concurrent-systems engineering
/// that cannot be honestly represented as a thin wrapper over existing code,
/// because no such existing code exists in this repo to wrap. Refuses loudly
/// rather than silently pretending `_correlation_key` was deduplicated.
///
/// # Errors
/// Always [`OCELConstructionRefused::NotYetImplemented`] currently.
pub fn idempotency_gate(_correlation_key: &str) -> Result<(), OCELConstructionRefused> {
    Err(OCELConstructionRefused::NotYetImplemented {
        stage: "l7_idempotency_correlation_gate",
        detail: "atomic idempotency/correlation gate over duplicate events, process/engine \
                 restarts, and stale/malformed results, plus durable receipt-head recovery, \
                 is HAND_WRITE_REQUIRED per the F24 survey and not yet built; zero existing \
                 idempotency/correlation/duplicate/restart/recovery code exists in cng's \
                 otel_ocel/otel_receipt modules for this to wrap",
    })
}

/// HAND_WRITE_REQUIRED (V12-024): the MFW Feedback Adapter -- feeds
/// `outcome`'s OCEL consequence into `multifractal-workflow` as a required
/// downstream consequence (e.g. F09 "MFW Growth Operator"). Not wired today:
/// `crate::f09_mfw_growth` (read this session) is itself still a Wire-phase
/// module with no consequence-intake function to call into -- its
/// `plan_growth` takes an already-resolved `ContinuationGoal`, not an OCEL
/// outcome, and no `resolve_continuation_goal`-from-OCEL-evidence path exists.
/// There is nothing real on the other end of this adapter to wire to yet.
/// Refuses loudly rather than silently dropping the consequence or pretending
/// it was fed somewhere.
///
/// # Errors
/// Always [`OCELConstructionRefused::NotYetImplemented`] currently.
pub fn mfw_feedback_adapter(
    _outcome: &OcelConstructOutcome,
) -> Result<(), OCELConstructionRefused> {
    Err(OCELConstructionRefused::NotYetImplemented {
        stage: "mfw_feedback_adapter",
        detail: "feeding an OCEL construction outcome into multifractal-workflow as a required \
                 consequence (e.g. F09's growth operator) is HAND_WRITE_REQUIRED per the F24 \
                 survey and not yet built; f09_mfw_growth has no consequence-intake API today \
                 for this to call into",
    })
}

#[cfg(test)]
mod tests {
    use cng::otel_rdf::{self, OtlpSpan, SpanStatus, SpanStatusCode};
    use cng::telemetry_gen;

    use super::*;

    /// One real, hand-constructed admissible span: all five required
    /// `event.praxis.activity_executed` attributes present, `process.outcome`
    /// in the closed vocabulary. Deliberately the same shape (not the
    /// identical literal fixture -- this module does not import test-only
    /// code cross-crate) as `crates/cng/src/bin/otel-rdf-demo.rs::fixture_span`,
    /// so this module's tests exercise the exact contract that binary's real
    /// entrypoint already exercises.
    fn fixture_span() -> OtlpSpan {
        OtlpSpan {
            trace_id: "4bf92f3577b34da6a3ce929d0e0e4736".to_string(),
            span_id: "00f067aa0ba902b7".to_string(),
            parent_span_id: None,
            name: telemetry_gen::REGISTRY_GROUP_ID.to_string(),
            start_time_unix_nano: 1_700_000_000_000_000_000,
            end_time_unix_nano: 1_700_000_000_500_000_000,
            attributes: vec![
                (
                    telemetry_gen::ATTR_WORKFLOW_ID.to_string(),
                    "wf-f24-test-1".to_string(),
                ),
                (
                    telemetry_gen::ATTR_OBJECT_ID.to_string(),
                    "order-f24-test-1".to_string(),
                ),
                (
                    telemetry_gen::ATTR_OBJECT_TYPE.to_string(),
                    "Order".to_string(),
                ),
                (
                    telemetry_gen::ATTR_ACTIVITY_IRI.to_string(),
                    "urn:praxis:activity:ship-order".to_string(),
                ),
                (
                    telemetry_gen::ATTR_OUTCOME.to_string(),
                    "completed".to_string(),
                ),
            ],
            status: SpanStatus {
                code: SpanStatusCode::Ok,
                message: None,
            },
        }
    }

    /// A store with one admitted span already projected into `G_OTEL`, ready
    /// to drive [`run_construct`] end to end.
    fn store_with_admitted_otel() -> Store {
        let span = fixture_span();
        otel_rdf::admit(&span).expect("fixture span is admissible");
        let store = Store::new().expect("in-memory store");
        let otel_quads = otel_rdf::project_admitted_spans(&[span]).expect("project admitted span");
        insert_quads(&store, &otel_quads).expect("insert G_OTEL quads");
        store
    }

    #[test]
    fn construct_profile_resolve_known_name() {
        assert_eq!(
            ConstructProfile::resolve("otel-to-ocel"),
            Ok(ConstructProfile::OtelToOcel)
        );
    }

    #[test]
    fn construct_profile_resolve_unknown_name_is_refused() {
        let result = ConstructProfile::resolve("does-not-exist");
        assert_eq!(
            result,
            Err(OCELConstructionRefused::UnknownProfile {
                requested: "does-not-exist".to_string(),
                available: vec!["otel-to-ocel".to_string()],
            })
        );
    }

    #[test]
    fn run_construct_unknown_profile_never_touches_the_store() {
        let store = store_with_admitted_otel();
        let result = run_construct("not-a-real-profile", &store);
        assert!(matches!(
            result,
            Err(OCELConstructionRefused::UnknownProfile { .. })
        ));
        // No G_OCEL quads were written -- the profile resolver refused before
        // the CONSTRUCT engine ever ran.
        let digest = graph_content_digest(&store, OCEL_GRAPH_IRI).expect("digest is computable");
        let empty_digest = format!("blake3:{}", blake3::hash(b"").to_hex());
        assert_eq!(
            digest, empty_digest,
            "G_OCEL must be empty when the profile resolver refused"
        );
    }

    #[test]
    fn run_construct_succeeds_end_to_end_over_admitted_otel() {
        let store = store_with_admitted_otel();
        let outcome = run_construct("otel-to-ocel", &store).expect("real chain succeeds");

        assert_eq!(outcome.profile, ConstructProfile::OtelToOcel);
        assert!(
            !outcome.ocel_quads.is_empty(),
            "CONSTRUCT over one admitted span must derive at least one G_OCEL quad"
        );
        assert!(
            !outcome.receipt_quads.is_empty(),
            "receipt_otel_to_ocel must produce PROV-O ancestry quads"
        );
        assert_eq!(
            outcome.receipt_head.len(),
            "blake3:".len() + 64,
            "receipt head is a tagged 64-hex-char BLAKE3 digest"
        );

        // Graph Equivalence Replay, exercised a second time independently of
        // run_construct's own internal call: the store's G_OCEL content must
        // replay to the exact same head this outcome already carries.
        assert_eq!(
            verify_receipt_otel_to_ocel(&store, &outcome.receipt_head),
            Ok(())
        );

        // A tampered/stale head must be refused, never silently accepted --
        // this is the "stale/non-replaying input" half of the family
        // invariant, exercised against the real replay verifier.
        let tampered_head = format!("blake3:{}", "0".repeat(64));
        assert_ne!(outcome.receipt_head, tampered_head);
        assert!(matches!(
            verify_receipt_otel_to_ocel(&store, &tampered_head),
            Err(CngRefusal::AuditMismatch(_))
        ));
    }

    #[test]
    fn run_construct_is_deterministic_across_two_independent_stores() {
        let store_a = store_with_admitted_otel();
        let store_b = store_with_admitted_otel();
        let outcome_a = run_construct("otel-to-ocel", &store_a).expect("chain succeeds (a)");
        let outcome_b = run_construct("otel-to-ocel", &store_b).expect("chain succeeds (b)");
        assert_eq!(
            outcome_a.receipt_head, outcome_b.receipt_head,
            "identical admitted G_OTEL content must yield a byte-identical receipt head"
        );
    }

    #[test]
    fn build_default_measurement_profile_writes_real_g_result_quads() {
        let store = store_with_admitted_otel();
        run_construct("otel-to-ocel", &store).expect("populate G_OCEL first");

        let quads = build_default_measurement_profile(&store)
            .expect("one admitted workflow family is enough evidence");
        assert!(
            !quads.is_empty(),
            "a real measurement profile must project at least one G_RESULT quad"
        );

        insert_quads(&store, &quads).expect("insert G_RESULT quads");
        let digest = graph_content_digest(&store, RESULT_GRAPH_IRI).expect("digest is computable");
        let empty_digest = format!("blake3:{}", blake3::hash(b"").to_hex());
        assert_ne!(
            digest, empty_digest,
            "G_RESULT must hold real content distinct from an empty graph"
        );
    }

    #[test]
    fn build_default_measurement_profile_refuses_over_empty_g_ocel() {
        let store = Store::new().expect("in-memory store");
        let result = build_default_measurement_profile(&store);
        assert!(
            matches!(
                result,
                Err(OCELConstructionRefused::Upstream(
                    CngRefusal::MeasurementEvidenceInsufficient { .. }
                ))
            ),
            "zero admitted G_OCEL evidence must refuse, not silently produce an empty/zero \
             measurement profile: got {result:?}"
        );
    }

    #[test]

    fn idempotency_gate_is_honestly_unimplemented() {
        assert_eq!(
            idempotency_gate("any-correlation-key"),
            Err(OCELConstructionRefused::NotYetImplemented {
                stage: "l7_idempotency_correlation_gate",
                detail: "atomic idempotency/correlation gate over duplicate events, \
                         process/engine restarts, and stale/malformed results, plus durable \
                         receipt-head recovery, is HAND_WRITE_REQUIRED per the F24 survey and \
                         not yet built; zero existing idempotency/correlation/duplicate/\
                         restart/recovery code exists in cng's otel_ocel/otel_receipt modules \
                         for this to wrap",
            })
        );
    }

    #[test]
    fn mfw_feedback_adapter_is_honestly_unimplemented() {
        let store = store_with_admitted_otel();
        let outcome = run_construct("otel-to-ocel", &store).expect("real chain succeeds");
        assert_eq!(
            mfw_feedback_adapter(&outcome),
            Err(OCELConstructionRefused::NotYetImplemented {
                stage: "mfw_feedback_adapter",
                detail: "feeding an OCEL construction outcome into multifractal-workflow as a \
                         required consequence (e.g. F09's growth operator) is \
                         HAND_WRITE_REQUIRED per the F24 survey and not yet built; \
                         f09_mfw_growth has no consequence-intake API today for this to call \
                         into",
            })
        );
    }
}
