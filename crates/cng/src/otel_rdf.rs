//! PROJ-763: maps Weaver-admitted OTLP spans into the named RDF graph
//! `urn:graph:otel` (closes gap G11; see `docs/otel-rdf-handoff.md`).
//!
//! # What "Weaver-admitted" means here
//!
//! The existing `just otel-weaver-live` campaign admits telemetry by running
//! the external `weaver registry live-check` process (weaver 0.22.1) against
//! `registry/otel/praxis-events.yaml`, over a live gRPC OTLP endpoint. That
//! external process is the actual admission gate in this codebase today —
//! this module does not invoke it (no network process inside a unit test)
//! and does not change it.
//!
//! What this module adds is [`admit`]: an in-process re-validation of the
//! *identical* five-attribute `event.praxis.activity_executed` contract
//! (`crate::telemetry_gen`, generated from
//! `crates/praxis-graphlaw/ontologies/core/otel-bridge.ttl`) that Weaver
//! enforces live — required attributes present, and `process.outcome`
//! restricted to the closed vocabulary Weaver's registry declares
//! (`completed` | `refused`). A span that fails `admit` is refused with
//! `CngRefusal::OtelSpanRefused` (`CNG_R27`) before any triple is produced.
//! This is the boundary the handoff doc calls "admitted OTEL signals" —
//! spans that already crossed the live gate, or (for callers/tests that
//! construct spans directly, as this module's own tests do) spans that pass
//! the same structural contract in-process.
//!
//! # RDF projection
//!
//! Each admitted span becomes a set of triples in the named graph
//! `urn:graph:otel`, following the public-ontology-first mapping already
//! declared in `otel-bridge.ttl`'s "Span-occurrence instance vocabulary"
//! section: the occurrence is typed `prov:Activity, ocel:Event`; the three
//! object-side attributes (`process.object.id`, `process.object.type`) use
//! real OCEL 2.0 instance properties (`ocel:objectId`, `ocel:objectTypeName`,
//! `ocel:hasObjectType`, `ocel:relatesTo`) with no minting; `process.outcome`
//! and `process.workflow.id` use bridge-minted properties
//! (`ob:hasOutcome`, `ob:workflowId`) for the reasons documented in the
//! ontology; the five raw OTLP protocol fields (trace id, span id, parent
//! span id, name, start/end time, status) are minted bridge properties with
//! no OCEL/PROV-O equivalent. Any span attribute outside the five required
//! ones is projected generically as an `ocel:Attribute` key-value node via
//! `ocel:hasAttribute`.
//!
//! `ob:startTimeUnixNano`/`ob:endTimeUnixNano` carry the OTLP producer's own
//! timestamps as asserted payload data. This is data, not a clock read by
//! this module: nothing in this file calls `SystemTime::now()` or
//! `Instant::now()`; the nanosecond values only ever come from the
//! `OtlpSpan` the caller constructed.
//!
//! # Determinism
//!
//! [`project_admitted_spans`] sorts the emitted `Vec<Quad>` by each quad's
//! canonical N-Quads text (`Quad`'s `Display` impl) before returning, and
//! extra (non-required) attributes are sorted by key before projection — so
//! output does not depend on caller-supplied attribute order. Same input
//! spans, in any attribute order, produce byte-identical output every call;
//! `otel_rdf_test.rs` proves this by running the same input twice and
//! diffing the serialized TriG bytes.

use oxigraph::io::RdfFormat;
use oxigraph::model::{GraphName, Literal, NamedNode, Quad, Term};
use oxigraph::store::Store;

use crate::powl::CngRefusal;
use crate::telemetry_gen;

/// The named graph admitted OTEL signals land in (`docs/otel-rdf-handoff.md`).
pub const OTEL_GRAPH_IRI: &str = "urn:graph:otel";

const OB_NS: &str = "https://ggen.io/ontology/otel-bridge#";
const OCEL_NS: &str = "https://www.ocel-standard.org/2.0/";
const PROV_NS: &str = "http://www.w3.org/ns/prov#";
const XSD_NS: &str = "http://www.w3.org/2001/XMLSchema#";
const RDF_TYPE_IRI: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

/// The five required `event.praxis.activity_executed` attribute ids
/// (`registry/otel/praxis-events.yaml`), reused verbatim from the
/// ggen-generated bindings so this module and the Weaver registry can never
/// silently drift apart.
const REQUIRED_ATTRS: [&str; 5] = [
    telemetry_gen::ATTR_WORKFLOW_ID,
    telemetry_gen::ATTR_OBJECT_ID,
    telemetry_gen::ATTR_OBJECT_TYPE,
    telemetry_gen::ATTR_ACTIVITY_IRI,
    telemetry_gen::ATTR_OUTCOME,
];

/// Closed `process.outcome` vocabulary (`ob:OutcomeScheme` in
/// `otel-bridge.ttl`): `ob:OutcomeCompleted` | `ob:OutcomeRefused`.
const OUTCOME_COMPLETED: &str = "completed";
const OUTCOME_REFUSED: &str = "refused";

/// OTLP span status code (a minimal, dependency-free mirror of the OTLP
/// `Status.StatusCode` enum — this module does not depend on the
/// `opentelemetry` SDK crates, which are gated behind the optional
/// `otel-live` feature; see the module doc for why).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanStatusCode {
    Unset,
    Ok,
    Error,
}

impl SpanStatusCode {
    fn as_str(self) -> &'static str {
        match self {
            SpanStatusCode::Unset => "unset",
            SpanStatusCode::Ok => "ok",
            SpanStatusCode::Error => "error",
        }
    }
}

/// OTLP span status (code + optional message).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpanStatus {
    pub code: SpanStatusCode,
    pub message: Option<String>,
}

impl Default for SpanStatus {
    fn default() -> Self {
        Self {
            code: SpanStatusCode::Unset,
            message: None,
        }
    }
}

/// A minimal, explicit mirror of the OTLP span data model: trace id, span
/// id, parent span id, name, start/end time (nanoseconds since the Unix
/// epoch, as supplied by the producer), attributes, and status. Deliberately
/// not `opentelemetry_sdk::trace::SpanData`: that type lives behind the
/// optional `otel-live` feature (path dependency on chicago-tdd-tools), and
/// this admission+projection path needs to be exercisable in the default
/// build/test surface. Callers on the `otel-live` path can construct one of
/// these from `SpanData` field-by-field; this module does not need to know
/// how.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtlpSpan {
    /// Lowercase-hex OTLP trace id. Must be non-empty to be admitted.
    pub trace_id: String,
    /// Lowercase-hex OTLP span id. Must be non-empty to be admitted.
    pub span_id: String,
    /// Lowercase-hex OTLP parent span id. `None` (or empty) for a root span.
    pub parent_span_id: Option<String>,
    /// OTLP span name.
    pub name: String,
    /// OTLP span start time, nanoseconds since the Unix epoch — producer
    /// data, not a clock read taken here.
    pub start_time_unix_nano: u64,
    /// OTLP span end time, nanoseconds since the Unix epoch — producer
    /// data, not a clock read taken here.
    pub end_time_unix_nano: u64,
    /// OTLP span attributes as string key/value pairs (this registry's
    /// semantic convention declares all five required attributes as
    /// `type: string`; see `registry/otel/praxis-events.yaml`).
    pub attributes: Vec<(String, String)>,
    /// OTLP span status.
    pub status: SpanStatus,
}

impl OtlpSpan {
    /// Looks up an attribute value by key.
    ///
    /// # Complexity
    /// O(a) in attribute count (a is small and bounded by the emitting
    /// producer; a `HashMap` would add nondeterministic iteration risk for
    /// no measured benefit at this scale).
    fn attr(&self, key: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
}

/// Re-validates a span against the same `event.praxis.activity_executed`
/// contract the external Weaver live-check enforces: all five required
/// attributes present and non-empty, `process.outcome` in the closed
/// vocabulary, `process.activity.iri` a legal IRI, and non-empty trace/span
/// ids.
///
/// # Errors
/// `CngRefusal::OtelSpanRefused` (`CNG_R27`) naming the first failing
/// check — trace/span id emptiness is checked before attribute presence,
/// attribute presence before the closed-vocabulary and IRI checks, so the
/// reported reason is always the earliest structural problem.
///
/// # Complexity
/// O(a) in attribute count (bounded array scan per required attribute).
pub fn admit(span: &OtlpSpan) -> Result<(), CngRefusal> {
    let span_key = format!("{}:{}", span.trace_id, span.span_id);
    if span.trace_id.is_empty() {
        return Err(CngRefusal::OtelSpanRefused {
            span: span_key,
            reason: "trace_id is empty".to_string(),
        });
    }
    if span.span_id.is_empty() {
        return Err(CngRefusal::OtelSpanRefused {
            span: span_key,
            reason: "span_id is empty".to_string(),
        });
    }
    for required in REQUIRED_ATTRS {
        match span.attr(required) {
            Some(v) if !v.is_empty() => {}
            _ => {
                return Err(CngRefusal::OtelSpanRefused {
                    span: span_key,
                    reason: format!("missing required attribute {required}"),
                });
            }
        }
    }
    // Safe: the loop above proved every REQUIRED_ATTRS lookup is `Some` and
    // non-empty for this exact span (assertion after explicit check).
    let outcome = span
        .attr(telemetry_gen::ATTR_OUTCOME)
        .expect("checked present and non-empty in the REQUIRED_ATTRS loop above");
    if outcome != OUTCOME_COMPLETED && outcome != OUTCOME_REFUSED {
        return Err(CngRefusal::OtelSpanRefused {
            span: span_key,
            reason: format!(
                "process.outcome {outcome:?} is outside the closed vocabulary \
                 ({OUTCOME_COMPLETED} | {OUTCOME_REFUSED})"
            ),
        });
    }
    let activity_iri = span
        .attr(telemetry_gen::ATTR_ACTIVITY_IRI)
        .expect("checked present and non-empty in the REQUIRED_ATTRS loop above");
    if NamedNode::new(activity_iri).is_err() {
        return Err(CngRefusal::OtelSpanRefused {
            span: span_key,
            reason: format!("process.activity.iri {activity_iri:?} is not a legal IRI"),
        });
    }
    Ok(())
}

/// Percent-encodes every byte outside the RFC 3986 unreserved set, so any
/// caller-supplied identifier (trace id, object id, attribute key, ...)
/// yields a legal IRI path segment without risking a `NamedNode::new`
/// parse failure downstream.
///
/// # Complexity
/// O(n) in input byte length.
fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        let unreserved = byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~');
        if unreserved {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn ns_node(ns: &str, local: &str) -> NamedNode {
    NamedNode::new(format!("{ns}{local}"))
        .expect("vocabulary IRI is a compile-time-controlled constant, never external input")
}

fn rdf_type() -> NamedNode {
    NamedNode::new(RDF_TYPE_IRI).expect("RDF_TYPE_IRI is a compile-time-controlled constant")
}

fn otel_graph_name() -> GraphName {
    GraphName::NamedNode(
        NamedNode::new(OTEL_GRAPH_IRI)
            .expect("OTEL_GRAPH_IRI is a compile-time-controlled constant"),
    )
}

fn xsd_long(value: u64) -> Literal {
    Literal::new_typed_literal(value.to_string(), ns_node(XSD_NS, "long"))
}

/// Projects a batch of already-admitted OTLP spans into `Vec<Quad>` in the
/// named graph `urn:graph:otel`. Runs [`admit`] on every span first — the
/// whole batch is refused (no partial projection) at the first inadmissible
/// span.
///
/// # Errors
/// `CngRefusal::OtelSpanRefused` (`CNG_R27`) if any span fails [`admit`], or
/// if `process.activity.iri` / an identifier cannot be encoded into a legal
/// IRI despite passing `admit` (defensive; `admit` already checks the IRI
/// case, so this path is exercised only if a future caller bypasses
/// `admit` directly — kept as a typed refusal rather than a panic either
/// way, per the no-panics-on-external-input invariant).
///
/// # Complexity
/// O(m log m) where m is the total emitted triple count (dominated by the
/// final canonical sort); triple construction itself is O(n·a) in span
/// count n and attributes per span a.
pub fn project_admitted_spans(spans: &[OtlpSpan]) -> Result<Vec<Quad>, CngRefusal> {
    let graph = otel_graph_name();
    let mut quads = Vec::new();

    for span in spans {
        admit(span)?;
        let span_key = format!("{}:{}", span.trace_id, span.span_id);

        let span_node = NamedNode::new(format!(
            "urn:otel:span:{}:{}",
            percent_encode(&span.trace_id),
            percent_encode(&span.span_id)
        ))
        .map_err(|e| CngRefusal::OtelSpanRefused {
            span: span_key.clone(),
            reason: format!("span IRI construction failed: {e}"),
        })?;

        quads.push(Quad::new(
            span_node.clone(),
            rdf_type(),
            Term::NamedNode(ns_node(PROV_NS, "Activity")),
            graph.clone(),
        ));
        quads.push(Quad::new(
            span_node.clone(),
            rdf_type(),
            Term::NamedNode(ns_node(OCEL_NS, "Event")),
            graph.clone(),
        ));

        quads.push(Quad::new(
            span_node.clone(),
            ns_node(OB_NS, "traceId"),
            Literal::new_simple_literal(&span.trace_id),
            graph.clone(),
        ));
        quads.push(Quad::new(
            span_node.clone(),
            ns_node(OB_NS, "spanId"),
            Literal::new_simple_literal(&span.span_id),
            graph.clone(),
        ));
        if let Some(parent) = span.parent_span_id.as_deref() {
            if !parent.is_empty() {
                quads.push(Quad::new(
                    span_node.clone(),
                    ns_node(OB_NS, "parentSpanId"),
                    Literal::new_simple_literal(parent),
                    graph.clone(),
                ));
            }
        }
        quads.push(Quad::new(
            span_node.clone(),
            ns_node(OB_NS, "spanName"),
            Literal::new_simple_literal(&span.name),
            graph.clone(),
        ));
        quads.push(Quad::new(
            span_node.clone(),
            ns_node(OB_NS, "startTimeUnixNano"),
            xsd_long(span.start_time_unix_nano),
            graph.clone(),
        ));
        quads.push(Quad::new(
            span_node.clone(),
            ns_node(OB_NS, "endTimeUnixNano"),
            xsd_long(span.end_time_unix_nano),
            graph.clone(),
        ));
        quads.push(Quad::new(
            span_node.clone(),
            ns_node(OB_NS, "spanStatusCode"),
            Literal::new_simple_literal(span.status.code.as_str()),
            graph.clone(),
        ));
        if let Some(message) = span.status.message.as_deref() {
            quads.push(Quad::new(
                span_node.clone(),
                ns_node(OB_NS, "spanStatusMessage"),
                Literal::new_simple_literal(message),
                graph.clone(),
            ));
        }

        // Safe: admit() proved these are Some and non-empty above.
        let workflow_id = span
            .attr(telemetry_gen::ATTR_WORKFLOW_ID)
            .expect("admit() proved presence");
        quads.push(Quad::new(
            span_node.clone(),
            ns_node(OB_NS, "workflowId"),
            Literal::new_simple_literal(workflow_id),
            graph.clone(),
        ));

        let activity_iri_str = span
            .attr(telemetry_gen::ATTR_ACTIVITY_IRI)
            .expect("admit() proved presence");
        let activity_iri =
            NamedNode::new(activity_iri_str).map_err(|e| CngRefusal::OtelSpanRefused {
                span: span_key.clone(),
                reason: format!(
                    "process.activity.iri {activity_iri_str:?} IRI construction failed: {e}"
                ),
            })?;
        quads.push(Quad::new(
            span_node.clone(),
            ns_node(OB_NS, "activityIri"),
            Term::NamedNode(activity_iri),
            graph.clone(),
        ));

        let outcome = span
            .attr(telemetry_gen::ATTR_OUTCOME)
            .expect("admit() proved presence and closed-vocabulary membership");
        let outcome_node = match outcome {
            OUTCOME_COMPLETED => ns_node(OB_NS, "OutcomeCompleted"),
            _ => ns_node(OB_NS, "OutcomeRefused"),
        };
        quads.push(Quad::new(
            span_node.clone(),
            ns_node(OB_NS, "hasOutcome"),
            Term::NamedNode(outcome_node),
            graph.clone(),
        ));

        let object_id = span
            .attr(telemetry_gen::ATTR_OBJECT_ID)
            .expect("admit() proved presence");
        let object_type = span
            .attr(telemetry_gen::ATTR_OBJECT_TYPE)
            .expect("admit() proved presence");
        let object_node = NamedNode::new(format!("urn:otel:object:{}", percent_encode(object_id)))
            .map_err(|e| CngRefusal::OtelSpanRefused {
                span: span_key.clone(),
                reason: format!("object IRI construction failed: {e}"),
            })?;
        let objecttype_node = NamedNode::new(format!(
            "urn:otel:objecttype:{}",
            percent_encode(object_type)
        ))
        .map_err(|e| CngRefusal::OtelSpanRefused {
            span: span_key.clone(),
            reason: format!("object type IRI construction failed: {e}"),
        })?;
        quads.push(Quad::new(
            object_node.clone(),
            rdf_type(),
            Term::NamedNode(ns_node(OCEL_NS, "Object")),
            graph.clone(),
        ));
        quads.push(Quad::new(
            object_node.clone(),
            ns_node(OCEL_NS, "objectId"),
            Literal::new_simple_literal(object_id),
            graph.clone(),
        ));
        quads.push(Quad::new(
            objecttype_node.clone(),
            rdf_type(),
            Term::NamedNode(ns_node(OCEL_NS, "ObjectType")),
            graph.clone(),
        ));
        quads.push(Quad::new(
            objecttype_node.clone(),
            ns_node(OCEL_NS, "objectTypeName"),
            Literal::new_simple_literal(object_type),
            graph.clone(),
        ));
        quads.push(Quad::new(
            object_node.clone(),
            ns_node(OCEL_NS, "hasObjectType"),
            Term::NamedNode(objecttype_node),
            graph.clone(),
        ));
        quads.push(Quad::new(
            span_node.clone(),
            ns_node(OCEL_NS, "relatesTo"),
            Term::NamedNode(object_node),
            graph.clone(),
        ));

        // Generic projection for any attribute outside the five required
        // ones. Sorted by key first so output does not depend on the
        // caller's attribute order (determinism).
        let mut extras: Vec<&(String, String)> = span
            .attributes
            .iter()
            .filter(|(k, _)| !REQUIRED_ATTRS.contains(&k.as_str()))
            .collect();
        extras.sort_by(|a, b| a.0.cmp(&b.0));
        for (key, value) in extras {
            let attr_node = NamedNode::new(format!(
                "urn:otel:attr:{}:{}:{}",
                percent_encode(&span.trace_id),
                percent_encode(&span.span_id),
                percent_encode(key)
            ))
            .map_err(|e| CngRefusal::OtelSpanRefused {
                span: span_key.clone(),
                reason: format!("attribute IRI construction failed for {key:?}: {e}"),
            })?;
            quads.push(Quad::new(
                attr_node.clone(),
                rdf_type(),
                Term::NamedNode(ns_node(OCEL_NS, "Attribute")),
                graph.clone(),
            ));
            quads.push(Quad::new(
                attr_node.clone(),
                ns_node(OCEL_NS, "attributeKey"),
                Literal::new_simple_literal(key),
                graph.clone(),
            ));
            quads.push(Quad::new(
                attr_node.clone(),
                ns_node(OCEL_NS, "attributeValue"),
                Literal::new_simple_literal(value),
                graph.clone(),
            ));
            quads.push(Quad::new(
                span_node.clone(),
                ns_node(OCEL_NS, "hasAttribute"),
                Term::NamedNode(attr_node),
                graph.clone(),
            ));
        }
    }

    // Canonical order: sorted by each quad's N-Quads text, independent of
    // insertion order. Never hash/serialize this Vec unsorted.
    quads.sort_by_key(|a| a.to_string());
    Ok(quads)
}

/// Projects admitted spans and serializes the result as TriG (the RDF
/// dataset syntax that preserves named-graph structure textually — plain
/// Turtle has no graph-name syntax). All triples appear inside one
/// `urn:graph:otel { ... }` block.
///
/// # Errors
/// `CngRefusal::OtelSpanRefused` (`CNG_R27`) via [`project_admitted_spans`];
/// `CngRefusal::IoRefused` (`CNG_R10`) if the in-memory oxigraph store or
/// TriG serializer itself fails (store construction, insert, or
/// serialization).
///
/// # Complexity
/// O(m log m) dominated by [`project_admitted_spans`]'s canonical sort,
/// plus O(m) store insert and O(m) serialization.
pub fn admitted_spans_to_trig(spans: &[OtlpSpan]) -> Result<String, CngRefusal> {
    let quads = project_admitted_spans(spans)?;
    let store = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("oxigraph store construction: {e}")))?;
    for quad in &quads {
        store
            .insert(quad.as_ref())
            .map_err(|e| CngRefusal::IoRefused(format!("quad insert failed for {quad}: {e}")))?;
    }
    let bytes = store
        .dump_to_writer(RdfFormat::TriG, Vec::new())
        .map_err(|e| CngRefusal::IoRefused(format!("TriG serialization failed: {e}")))?;
    String::from_utf8(bytes)
        .map_err(|e| CngRefusal::IoRefused(format!("TriG output was not valid UTF-8: {e}")))
}

/// `pub(crate)`: PROJ-764's `otel_ocel_test.rs` reuses `admissible_span()`
/// from this module rather than duplicating the fixture.
#[cfg(test)]
#[path = "otel_rdf_test.rs"]
pub(crate) mod otel_rdf_test;
