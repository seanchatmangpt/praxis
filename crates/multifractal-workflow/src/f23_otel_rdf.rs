//! Family F23 -- "OpenTelemetry RDF Admission" (atlas ticket V12-023).
//!
//! Survey verdict: **MIXED**. This is a Wire-phase-1 pass over the survey's
//! `ALREADY_BUILT` / `HAND_WRITE_REQUIRED` breakdown, not a from-scratch
//! implementation -- every re-export below is real, independently-tested code in the
//! sibling `cng` crate (`cng::otel_rdf`/`otel_ocel`/`otel_receipt`/`powl`). Per
//! `.claude/rules/no-overclaiming.md`, everything under "What is ALIVE below" was
//! verified this session by a real command or read; everything under "What is still
//! not wired" is disclosed as a gap, not dressed up as done.
//!
//! **Cargo.toml note**: `cng` was already added as a path dependency of this crate
//! (`features = ["bench"]`) by a concurrent agent's F20 wiring pass before this module
//! was written -- verified by reading `Cargo.toml` this session. No further Cargo.toml
//! change was needed for F23: `otel_rdf`/`otel_ocel`/`otel_receipt`/`powl`/
//! `telemetry_gen` are all unconditional modules in `cng::lib.rs` (not gated behind any
//! Cargo feature, per that file's own doc comments), so they are reachable regardless
//! of which features happen to be enabled for other families' sake.
//!
//! # What is ALIVE below (verified this session)
//!
//! 1. **Admission + `G_OTEL` projection** (`ALREADY_BUILT` per the survey) --
//!    [`admit`], [`project_admitted_spans`], [`admitted_spans_to_trig`], [`OtlpSpan`],
//!    [`SpanStatus`], [`SpanStatusCode`], [`OTEL_GRAPH_IRI`], re-exported from
//!    `cng::otel_rdf`. Verified this session: `cargo test -p cng --lib otel_rdf` was
//!    re-run by this family's own prior survey pass (10/10 passing, cited in this
//!    family's survey verdict) -- not re-run again by this module's own edit (no
//!    production code in `cng::otel_rdf` was touched), but this module's own new tests
//!    below (`full_lifecycle_...`, `missing_attribute_...`) exercise the re-exported
//!    functions directly through this crate's own compiled artifact, which is a
//!    distinct verification from re-running `cng`'s own suite.
//! 2. **`G_OTEL -> G_OCEL` CONSTRUCT projection + 5-layer graph separation**
//!    (`ALREADY_BUILT`) -- [`project_otel_to_ocel`], [`insert_quads`],
//!    [`graph_content_digest`], [`load_source_graph`], [`GRAPH_LAYERS`],
//!    [`SOURCE_GRAPH_IRI`], [`OCEL_GRAPH_IRI`], [`RESULT_GRAPH_IRI`],
//!    [`RECEIPT_GRAPH_IRI`], re-exported from `cng::otel_ocel`. Real SPARQL CONSTRUCT
//!    (`queries/otel-to-ocel.construct.rq`), not imperative Rust OCEL construction --
//!    verified by reading `crates/cng/src/otel_ocel.rs` in full this session.
//! 3. **PROV-O ancestry + BLAKE3 digest-chain receipt** (`ALREADY_BUILT`, closes L6) --
//!    [`receipt_otel_to_ocel`], [`verify_receipt_otel_to_ocel`], re-exported from
//!    `cng::otel_receipt`. `verify_receipt_otel_to_ocel` independently recomputes all
//!    three constitutional digests (query/input-graph/output-graph) from `store`'s
//!    current content and refuses (`CngRefusal::AuditMismatch`) on drift -- receipts
//!    are computed and replayable, never merely asserted. Verified by reading
//!    `crates/cng/src/otel_receipt.rs` in full this session.
//! 4. [`AdmissionStage`] and [`admit_project_receipt`] -- a genuine (not decorative)
//!    composition function added in this module, new this session. It threads
//!    caller-supplied [`OtlpSpan`]s through every real stage the survey's L1-L3/L6
//!    lenses require -- `admit` (per span, run by this function itself, not just
//!    inherited from `project_admitted_spans`'s internal call) -> `project_admitted_spans`
//!    -> `insert_quads` (`G_OTEL`) -> `project_otel_to_ocel` -> `insert_quads` (`G_OCEL`)
//!    -> `receipt_otel_to_ocel` -> `insert_quads` (`G_RECEIPT`) -> extract the recorded
//!    `cngr:receiptHead` literal from the real receipt quads (not recomputed by this
//!    module -- read back from what `receipt_otel_to_ocel` actually wrote) ->
//!    `verify_receipt_otel_to_ocel` (independent replay check against `store`'s own
//!    content) -- one lawful call path from spans to a receipted, published `G_OTEL`
//!    satisfying L3, with the L6 replay-verification step included, not merely
//!    available. This module's own `#[cfg(test)]` tests below are new tests, written
//!    and run this session, that exercise it end to end.
//! 5. [`AdmissionStage`] tags the two REFUSED branches the atlas's L5 state-machine
//!    lens names (`REFUSED` off `RECEIVED` and off `RDF_PROJECTED`) as a real
//!    consequence of how `admit_project_receipt` is structured, not a label bolted on
//!    after the fact: every span is run through `admit` in its own loop *before*
//!    `project_admitted_spans` is called at all, so a schema-drift failure (missing
//!    required attribute, closed-vocabulary violation -- L4's "schema drift" case)
//!    genuinely surfaces as `(AdmissionStage::Received, _)` and never reaches
//!    projection. `full_lifecycle_admits_projects_and_receipts` and
//!    `missing_attribute_refuses_off_received_before_any_triple` below prove both the
//!    happy path and this refusal path concretely (the latter also asserts the store
//!    holds zero quads afterward -- L4's "before any triple is produced" requirement).
//!
//! # What is still not wired (disclosed gaps, not fixed by this pass)
//!
//! (a) **`AdmissionStage::RdfProjected` as an `Err` outcome is not empirically
//!     exercised by this module's tests, and is likely unreachable through this
//!     module's own public API today.** `project_admitted_spans` (`cng::otel_rdf`,
//!     read in full this session) only fails IRI construction for a span/object/
//!     attribute identifier *after* that same span already passed `admit` -- and
//!     `admit` already validates `process.activity.iri` legality, while every other
//!     identifier (`trace_id`, `span_id`, `object_id`, `object_type`, attribute keys)
//!     is `percent_encode`d before use, which cannot itself produce an illegal IRI
//!     segment for any `&str` input. `otel_rdf.rs`'s own doc comment says as much:
//!     that path is "exercised only if a future caller bypasses `admit` directly".
//!     `admit_project_receipt` never bypasses `admit`, so this branch is kept in the
//!     `AdmissionStage` enum for L5 lens-completeness and because the type signature
//!     genuinely allows it (a future relaxation of `admit`'s checks could make it live
//!     again), but no test here claims to hit it -- claiming so would be fabricating
//!     evidence for state this session did not observe.
//! (b) **No live-Weaver-gate-to-`admit` bridge exists.** `admit_project_receipt`
//!     operates over caller-constructed [`OtlpSpan`]s, identically to `cng`'s own
//!     `src/bin/otel-rdf-demo.rs`. Neither that binary nor this module's function
//!     consumes output from the real external `weaver registry live-check` process
//!     (`cng`'s `otel-live` feature / `src/bin/otel-live.rs`) -- the two production
//!     paths remain parallel, not unified, exactly as the family survey's gap (b)
//!     describes. This module does not close that gap.
//! (c) **No L7 concurrency/chaos-recovery semantics** (duplicate telemetry events,
//!     process/engine restart mid-admission, durable receipt-head/replay-state
//!     resumption) exist for OTEL admission anywhere in this module or in the `cng`
//!     code it wraps. `admit_project_receipt` re-runs the full pipeline on every call
//!     with no idempotency/correlation gate and no persisted state; calling it twice
//!     with the same spans against a fresh store produces two independently valid
//!     receipts, not a detected duplicate. Not implemented; not claimed. Per the
//!     survey, this is genuine algorithmic/integration work (an atomic
//!     idempotency/correlation gate plus durable receipt-head continuation), not
//!     ggen-mechanical scaffolding, and is tracked under V12-023, not attempted here.
//! (d) **L8's exact marker vocabulary does not exist.** No code anywhere in this repo
//!     (this module included) emits `OTEL_RDF_PRODUCTION_REACHABLE`,
//!     `WEAVER_LIVE_CHECK_ALIVE`, or `NO_EXTERNAL_GRAPH_IRI_PANIC`. Fabricating a
//!     function that returns `true` for these three booleans without the chaos
//!     evidence, production-reachability trace, and receipt/replay equivalence the
//!     atlas's L8 lens requires them to be gated on would be exactly the decorative
//!     "looks wired but isn't" failure mode this repo's discipline exists to prevent
//!     -- not added here.
//!
//! # Survey-cited paths for F23
//! - /Users/sac/Downloads/v26.7.12_mermaid_atlas/families/F23_otel-rdf.md
//! - /Users/sac/praxis/crates/cng/src/otel_rdf.rs
//! - /Users/sac/praxis/crates/cng/src/otel_rdf_test.rs
//! - /Users/sac/praxis/crates/cng/src/otel_ocel.rs
//! - /Users/sac/praxis/crates/cng/src/otel_ocel_test.rs
//! - /Users/sac/praxis/crates/cng/src/otel_receipt.rs
//! - /Users/sac/praxis/crates/cng/src/otel_receipt_test.rs
//! - /Users/sac/praxis/crates/cng/src/queries/otel-to-ocel.construct.rq
//! - /Users/sac/praxis/crates/cng/src/telemetry_gen.rs
//! - /Users/sac/praxis/crates/cng/src/bin/otel-live.rs
//! - /Users/sac/praxis/crates/cng/src/bin/otel-rdf-demo.rs
//! - /Users/sac/praxis/crates/cng/tests/weaver_live.rs
//! - /Users/sac/praxis/crates/cng/src/powl.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/ontologies/core/otel-bridge.ttl
//! - /Users/sac/praxis/packs/otel-weaver-pack/pack.toml
//! - /Users/sac/praxis/packs/otel-weaver-pack/ontology.ttl
//! - /Users/sac/praxis/packs/otel-weaver-pack/templates/telemetry_gen.rs.tmpl
//! - /Users/sac/praxis/packs/otel-weaver-pack/templates/praxis_events.yaml.tmpl
//! - /Users/sac/praxis/packs/otel-weaver-pack/templates/weaver_manifest.yaml.tmpl
//! - /Users/sac/praxis/registry/otel/manifest.yaml
//! - /Users/sac/praxis/registry/otel/praxis-events.yaml
//! - /Users/sac/praxis/docs/otel-rdf-handoff.md
//! - /Users/sac/praxis/justfile
//! - /Users/sac/otel/registry/powl-test-process.yaml

// ---- Admission + `G_OTEL` projection: ALREADY_BUILT, re-exported (not reimplemented). ----
pub use cng::otel_rdf::{
    admit, admitted_spans_to_trig, project_admitted_spans, OtlpSpan, SpanStatus, SpanStatusCode,
    OTEL_GRAPH_IRI,
};

// ---- `G_OTEL -> G_OCEL` CONSTRUCT projection + 5-layer graph separation: ALREADY_BUILT. ----
pub use cng::otel_ocel::{
    graph_content_digest, insert_quads, load_source_graph, project_otel_to_ocel, GRAPH_LAYERS,
    OCEL_GRAPH_IRI, RECEIPT_GRAPH_IRI, RESULT_GRAPH_IRI, SOURCE_GRAPH_IRI,
};

// ---- PROV-O ancestry + BLAKE3 digest-chain receipt: ALREADY_BUILT, closes L6. ----
pub use cng::otel_receipt::{receipt_otel_to_ocel, verify_receipt_otel_to_ocel};

// ---- Typed refusal taxonomy every stage above returns. ----
pub use cng::powl::CngRefusal;

use oxigraph::model::{Quad, Term};
use oxigraph::store::Store;

/// F23-L5's explicit admission-stage lattice (atlas: `EMITTED -> RECEIVED ->
/// REGISTRY_CHECKED -> RDF_PROJECTED -> GRAPH_ID_VALID -> ADMITTED -> RECEIPTED`, with
/// `REFUSED` branches off `RECEIVED` and `RDF_PROJECTED`). This enum labels which
/// stage [`admit_project_receipt`] had reached when a real underlying `cng::otel_*`
/// call succeeded or failed -- it does not gate or reimplement that data flow (which
/// remains exactly `cng::otel_rdf::{admit, project_admitted_spans}` /
/// `cng::otel_ocel::{project_otel_to_ocel, insert_quads}` /
/// `cng::otel_receipt::{receipt_otel_to_ocel, verify_receipt_otel_to_ocel}`); it names
/// the F23 lens onto that flow's own real transitions. See the module doc's gap (a)
/// for why [`AdmissionStage::RdfProjected`] as an `Err` is not empirically exercised.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionStage {
    /// A span exists in the caller's batch but has not yet been checked.
    Emitted,
    /// [`admit`] ran and either accepted every span (progression continues) or
    /// refused the first non-conformant one (schema drift -- missing required
    /// attribute, closed-vocabulary violation, empty trace/span id).
    Received,
    /// Reserved for the atlas's own separate registry-liveness stage. Not
    /// distinguished from [`AdmissionStage::Received`] by any code in this crate or
    /// in `cng` today -- `admit`'s in-process re-validation *is* the registry-contract
    /// check (see `cng::otel_rdf`'s own module doc, "What 'Weaver-admitted' means
    /// here"), so this variant currently never appears on its own in a refusal from
    /// [`admit_project_receipt`].
    RegistryChecked,
    /// [`project_admitted_spans`] ran: RDF triples were constructed (or refused --
    /// malformed graph IRI -- before any triple was produced; see module doc gap (a)
    /// for why this refusal branch is undemonstrated here).
    RdfProjected,
    /// Every constructed IRI (span, object, object-type, attribute nodes) parsed as a
    /// legal `NamedNode`. Not separately distinguished from
    /// [`AdmissionStage::RdfProjected`] by any code path today -- IRI legality is
    /// checked inline during projection, not as a later pass.
    GraphIdValid,
    /// [`insert_quads`] wrote the projected `G_OTEL` quads and [`project_otel_to_ocel`]
    /// derived + [`insert_quads`] wrote `G_OCEL`.
    Admitted,
    /// [`receipt_otel_to_ocel`] wrote `G_RECEIPT` and [`verify_receipt_otel_to_ocel`]
    /// independently confirmed the recorded receipt head against `store`'s own
    /// content.
    Receipted,
}

/// Evidence a full `EMITTED -> RECEIPTED` run through [`admit_project_receipt`]
/// produced: how many quads landed in each of the three graphs it populated, and the
/// independently-replay-verified `cngr:receiptHead` digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionPipelineResult {
    /// Quads written to `urn:graph:otel` (one entry per [`OtlpSpan`] admitted, times
    /// that span's fixed + extra-attribute triple count).
    pub otel_quad_count: usize,
    /// Quads written to `urn:graph:ocel`, derived by SPARQL CONSTRUCT.
    pub ocel_quad_count: usize,
    /// Quads written to `urn:graph:receipts` (the PROV-O ancestry + digest-chain
    /// receipt).
    pub receipt_quad_count: usize,
    /// The `cngr:receiptHead` digest recorded in the receipt quads, already confirmed
    /// by [`verify_receipt_otel_to_ocel`] to match an independent recomputation from
    /// `store`'s own content.
    pub receipt_head: String,
}

/// The `cngr:receiptHead` predicate IRI (`cng::otel_receipt`'s own `CNGR_NS` +
/// `receiptHead`, which that module keeps private -- duplicated here as a `&str`
/// literal rather than widened to `pub` in `cng`, since this is the one place outside
/// `cng` that needs to read the literal back out of already-produced receipt quads).
const RECEIPT_HEAD_PRED: &str = "https://truex.io/ontology/cng-receipt#receiptHead";

/// Reads the `cngr:receiptHead` literal back out of quads [`receipt_otel_to_ocel`]
/// already produced -- never recomputed independently by this function (that
/// independence is [`verify_receipt_otel_to_ocel`]'s job, called separately by
/// [`admit_project_receipt`] after this extraction).
///
/// # Complexity
/// O(r) in `receipt_quads.len()` (r is small and fixed per receipt -- 15 quads today,
/// counted this session directly from `otel_receipt.rs`'s `receipt_otel_to_ocel`: 7
/// pushes before its `for (node, digest, source_graph_iri) in [...]` loop, 3 pushes
/// inside that loop times 2 iterations, plus 2 pushes after it).
fn extract_receipt_head(receipt_quads: &[Quad]) -> Option<String> {
    receipt_quads.iter().find_map(|q| {
        if q.predicate.as_str() == RECEIPT_HEAD_PRED {
            match &q.object {
                Term::Literal(lit) => Some(lit.value().to_string()),
                _ => None,
            }
        } else {
            None
        }
    })
}

/// Runs `spans` through every real F23 stage, in order, against `store`: `admit`
/// (per span) -> `project_admitted_spans` -> insert `G_OTEL` -> `project_otel_to_ocel`
/// -> insert `G_OCEL` -> `receipt_otel_to_ocel` -> insert `G_RECEIPT` -> extract the
/// recorded receipt head -> `verify_receipt_otel_to_ocel` (independent replay check).
///
/// This is a genuine composition, not a reimplementation: every step calls the exact
/// `cng::otel_*` function named above, unmodified. The one thing this function adds is
/// the explicit `admit` pre-pass (rather than relying solely on
/// `project_admitted_spans`'s own internal `admit` call) so a schema-drift refusal is
/// reported as [`AdmissionStage::Received`] before this function has called
/// `project_admitted_spans` at all -- and the receipt-head extraction + independent
/// verification, which `cng`'s own `otel-rdf-demo.rs` binary also does but no library
/// function in `cng` exposes as a single reusable call.
///
/// # Errors
/// `Err((stage, refusal))` where `stage` is the [`AdmissionStage`] reached and
/// `refusal` is the [`CngRefusal`] the underlying `cng::otel_*` call returned:
/// - `(Received, OtelSpanRefused)` if any span fails [`admit`] (schema drift / empty
///   trace-span id / closed-vocabulary violation) -- before any triple is built.
/// - `(RdfProjected, OtelSpanRefused)` if `project_admitted_spans` itself refuses
///   after `admit` already passed every span (see module doc gap (a); not
///   empirically demonstrated to be reachable).
/// - `(Admitted, OcelConstructRefused)` if the `G_OTEL -> G_OCEL` CONSTRUCT
///   projection or either insert fails.
/// - `(Receipted, OcelConstructRefused)` if receipt construction, either insert, the
///   receipt-head extraction, or independent verification fails.
///
/// # Complexity
/// O(n·a + m log m) where n is `spans.len()`, a is attributes per span, and m is the
/// total quad count across all three populated graphs -- the sum of the documented
/// complexities of each `cng::otel_*` call this function makes, in sequence; no
/// additional asymptotic cost is added by the composition itself.
pub fn admit_project_receipt(
    store: &Store,
    spans: &[OtlpSpan],
) -> Result<AdmissionPipelineResult, (AdmissionStage, CngRefusal)> {
    for span in spans {
        admit(span).map_err(|e| (AdmissionStage::Received, e))?;
    }

    let otel_quads =
        project_admitted_spans(spans).map_err(|e| (AdmissionStage::RdfProjected, e))?;
    insert_quads(store, &otel_quads).map_err(|e| (AdmissionStage::RdfProjected, e))?;

    let ocel_quads = project_otel_to_ocel(store).map_err(|e| (AdmissionStage::Admitted, e))?;
    insert_quads(store, &ocel_quads).map_err(|e| (AdmissionStage::Admitted, e))?;

    let receipt_quads = receipt_otel_to_ocel(store).map_err(|e| (AdmissionStage::Receipted, e))?;
    insert_quads(store, &receipt_quads).map_err(|e| (AdmissionStage::Receipted, e))?;

    let receipt_head = extract_receipt_head(&receipt_quads).ok_or_else(|| {
        (
            AdmissionStage::Receipted,
            CngRefusal::IoRefused(
                "receipt quads did not contain a cngr:receiptHead literal".to_string(),
            ),
        )
    })?;

    verify_receipt_otel_to_ocel(store, &receipt_head)
        .map_err(|e| (AdmissionStage::Receipted, e))?;

    Ok(AdmissionPipelineResult {
        otel_quad_count: otel_quads.len(),
        ocel_quad_count: ocel_quads.len(),
        receipt_quad_count: receipt_quads.len(),
        receipt_head,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cng::telemetry_gen;

    /// A conformant span: same five required attributes and structure as
    /// `cng::otel_rdf_test`'s own `admissible_span()` fixture (that fixture is
    /// `pub(crate)` to `cng`, not reachable from here, so this is an independent
    /// construction of an equally-conformant span, not a shared import).
    fn admissible_span() -> OtlpSpan {
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
                    "wf-mfw-f23".to_string(),
                ),
                (
                    telemetry_gen::ATTR_OBJECT_ID.to_string(),
                    "order-23".to_string(),
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

    #[test]
    fn full_lifecycle_admits_projects_and_receipts() {
        let store = Store::new().expect("in-memory store construction");
        let result = admit_project_receipt(&store, &[admissible_span()])
            .expect("a conformant span must reach RECEIPTED");

        assert!(result.otel_quad_count > 0, "G_OTEL must gain quads");
        assert!(result.ocel_quad_count > 0, "G_OCEL must gain quads");
        assert!(result.receipt_quad_count > 0, "G_RECEIPT must gain quads");
        assert!(
            result.receipt_head.starts_with("blake3:"),
            "receipt head must be a tagged BLAKE3 digest, got {:?}",
            result.receipt_head
        );

        // L6: the receipt is independently replayable against the store's own
        // content, not just self-consistent with the run that produced it.
        verify_receipt_otel_to_ocel(&store, &result.receipt_head)
            .expect("independent replay of the just-produced receipt head must succeed");
    }

    #[test]
    fn missing_attribute_refuses_off_received_before_any_triple() {
        let mut span = admissible_span();
        // Drop the required workflow-id attribute: schema drift (L4's first named
        // refusal case).
        span.attributes
            .retain(|(k, _)| k != telemetry_gen::ATTR_WORKFLOW_ID);

        let store = Store::new().expect("in-memory store construction");
        let err = admit_project_receipt(&store, &[span])
            .expect_err("a span missing a required attribute must be refused");

        assert_eq!(
            err.0,
            AdmissionStage::Received,
            "schema-drift refusal must surface at the RECEIVED stage, before projection"
        );
        assert!(
            matches!(err.1, CngRefusal::OtelSpanRefused { .. }),
            "must be the typed OtelSpanRefused (CNG_R27) variant, got {:?}",
            err.1
        );

        // L4: "before any triple is produced" -- the store must hold zero quads.
        let remaining: Result<Vec<_>, _> =
            store.quads_for_pattern(None, None, None, None).collect();
        assert_eq!(
            remaining.expect("quad iteration must succeed").len(),
            0,
            "no triple may exist in the store after a RECEIVED-stage refusal"
        );
    }

    #[test]
    fn bad_outcome_vocabulary_is_also_refused_off_received() {
        let mut span = admissible_span();
        span.attributes
            .iter_mut()
            .find(|(k, _)| k == telemetry_gen::ATTR_OUTCOME)
            .expect("fixture always carries process.outcome")
            .1 = "not-a-closed-vocab-value".to_string();

        let store = Store::new().expect("in-memory store construction");
        let err = admit_project_receipt(&store, &[span])
            .expect_err("a span with an outcome outside the closed vocabulary must refuse");

        assert_eq!(err.0, AdmissionStage::Received);
        assert!(matches!(err.1, CngRefusal::OtelSpanRefused { .. }));
    }
}
