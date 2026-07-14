//! PROJ-766: Rail G measurement-profile schema + per-workflow-family
//! execution measure `mu_x`, populating `G_RESULT`
//! (`otel_ocel::RESULT_GRAPH_IRI`) — the fourth named graph PROJ-764
//! declared but deliberately left unpopulated (its own module doc:
//! "`G_RESULT`'s Rail F evidence/measurement-profile wiring is populated
//! by the separate `crate::measurement` module").
//!
//! # Scope boundary against PROJ-767
//!
//! PROJ-767 (`bench::multifractal`, `pub(super)`-scoped to the `bench`
//! feature) already builds the `Z(q,epsilon)` / `tau(q)` / `D(q)` /
//! `f(alpha)` partition-function estimator — the *statistics* fitted over
//! a measure. This module is the other, independent half the ticket table
//! (`docs/jira/v26.7.11/tickets/index.md`: PROJ-766 depends only on 764,
//! not 767) scopes separately: the *measure itself*, `mu_x`, defined per
//! PRD.md sec.16 ("For workflow family x, define execution measure
//! mu_x"), computed here as a real SPARQL SELECT over admitted `G_OCEL`
//! evidence (PRD.md sec.14: "SPARQL SELECT SHALL measure"), plus the
//! declared measurement-profile schema PRD.md sec.16/17 item 21 requires
//! (scale, q-range, fitting method, minimum evidence threshold,
//! confidence criteria, source OCEL digest — PRD.md:783, verbatim field
//! list). This module does not compute `Z`/`tau`/`D`/`f(alpha)` — wiring
//! PROJ-767's estimator onto this module's raw `mu_x` output is a
//! distinct, not-yet-scoped follow-up, named here rather than silently
//! implied done.
//!
//! # Which declared process scales are real today
//!
//! PRD.md sec.16 declares 11 process scales. Three have a real data source
//! in the `G_OCEL` graph PROJ-764 constructs today:
//! [`DeclaredProcessScale::Workflow`] (grouped by the `process.workflow.id`
//! OCEL attribute value `otel-to-ocel.construct.rq` already asserts),
//! [`DeclaredProcessScale::Activity`] (grouped by `ocel:activityName`
//! directly), and [`DeclaredProcessScale::ObjectCentricAggregationLevel`]
//! (grouped by `ocel:objectTypeName`, reached via `ocel:relatesTo ->
//! ocel:hasObjectType`, from the required `process.object.type` OTLP
//! attribute — `measurement-mass-by-object-type.rq`).
//!
//! The other 8 were investigated individually against the actual OTLP
//! producer surface (`registry/otel/praxis-events.yaml`'s five *required*
//! attributes are the entire attribute contract; no producer in this
//! codebase ever emits an attribute outside that set) and each refuses
//! `CngRefusal::MeasurementEvidenceInsufficient` (`CNG_R29`) with a
//! scale-specific reason (see [`DeclaredProcessScale::mass_query_or_reason`])
//! rather than fabricating a zero, a placeholder measure, or a mislabeled
//! proxy — matching this rail's own anti-relabeling mandate
//! (`docs/jira/v26.7.11/RAIL_G_MEASUREMENT_DESIGN.md`, the same discipline
//! PROJ-767's honest monofractal finding already established):
//!
//! - **enterprise goal**, **program**, **process**, **subprocess**: no
//!   attribute anywhere upstream of this pipeline encodes any of these four
//!   grouping levels; `process.workflow.id` is the only process-hierarchy
//!   identifier the registry declares, and it names a workflow *instance*,
//!   not a process definition or anything coarser.
//! - **child workflow**: `G_OCEL` carries OTLP span `parent_span_id`
//!   (activity-level span nesting inside one workflow instance), not a
//!   workflow-level socket-attachment relation. The actual child-workflow
//!   evidence (`bench-obs.ttl`'s `hasChildWorkflow`/`hasParentActivity`
//!   properties, `crates/cng/src/bench/roles.rs`) lives in the `bench`
//!   feature's separate observation-store pipeline, which this module
//!   cannot depend on (its own scope boundary: "depends only on `otel_ocel`
//!   and the hard `oxigraph` dependency, so it is unconditional like
//!   `otel_ocel`/`otel_rdf` themselves").
//! - **broker actuation**: no OTLP producer in this codebase emits a
//!   broker-related attribute; PRD sec.13's broker contract has no
//!   telemetry projection today.
//! - **recursive POWL depth**: `G_OCEL` has no correlation between OTLP
//!   spans and POWL AST node depth — POWL execution is not wired to any
//!   span-emitting engine yet (`RAIL_A_B_STATUS.md`). Substituting OTLP
//!   span `parent_span_id` nesting would measure a different, unrelated
//!   quantity (activity call nesting within one workflow) under this
//!   scale's name, which is exactly the relabeling this module refuses to
//!   do.
//! - **bounded execution cost band**: PRD sec.6.6's bounded-descent cost
//!   vector `C(W) = <d,a,u,r>` (remaining decomposition depth, unresolved
//!   activities/uncertainty/resource dependencies) is a POWL
//!   decomposition-time budget, never emitted as OTLP telemetry anywhere in
//!   this codebase.
//!
//! # Determinism
//!
//! No wall clock, no floating point in any digest or receipted-graph
//! path: [`ExecutionMeasure`] carries the raw integer `mass` (event count)
//! only — normalization (`mass / total_mass`) is left to a consumer,
//! deliberately not precomputed as an `f64` literal baked into `G_RESULT`
//! (the rust-agi-core-team no-float-in-canonical-paths rule). `?family`/
//! `?mass` rows come back `ORDER BY ?family` from the `.rq` file itself,
//! so no `HashMap`/nondeterministic iteration is needed to make the
//! result order canonical. `source_ocel_digest` reuses
//! `otel_ocel::graph_content_digest` — the same canonicalize-then-BLAKE3
//! rule PROJ-765's receipt uses — so it is computed, never asserted
//! (invariant #2).

use oxigraph::model::{GraphName, Literal, NamedNode, Quad, Term};
use oxigraph::sparql::{QueryResults, SparqlEvaluator};
use oxigraph::store::Store;

use crate::otel_ocel::{self, OCEL_GRAPH_IRI, RESULT_GRAPH_IRI};
use crate::powl::CngRefusal;

/// This module's own minted vocabulary namespace (no PRD/OCEL/PROV-O term
/// covers "declared measurement profile" or "execution measure" directly),
/// following the same truex.io-namespace convention `powl.rs::
/// POWL2_PREFIX` and `otel_receipt.rs`'s `cngr:` namespace already
/// establish in this crate.
const MN_NS: &str = "https://truex.io/ontology/cng-measurement#";

const RDF_TYPE_IRI: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

const MASS_BY_WORKFLOW: &str = include_str!("queries/measurement-mass-by-workflow.rq");
const MASS_BY_ACTIVITY: &str = include_str!("queries/measurement-mass-by-activity.rq");
const MASS_BY_OBJECT_TYPE: &str = include_str!("queries/measurement-mass-by-object-type.rq");

/// One of the 11 declared process scales PRD.md sec.16 names, in the
/// order the PRD lists them. See the module doc for which three have a
/// real `G_OCEL` data source today.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeclaredProcessScale {
    EnterpriseGoal,
    Program,
    Process,
    Subprocess,
    Workflow,
    Activity,
    ChildWorkflow,
    BrokerActuation,
    RecursivePowlDepth,
    ObjectCentricAggregationLevel,
    BoundedExecutionCostBand,
}

impl DeclaredProcessScale {
    /// Canonical scale name, verbatim from PRD.md sec.16's bullet list.
    ///
    /// # Complexity
    /// O(1).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EnterpriseGoal => "enterprise goal",
            Self::Program => "program",
            Self::Process => "process",
            Self::Subprocess => "subprocess",
            Self::Workflow => "workflow",
            Self::Activity => "activity",
            Self::ChildWorkflow => "child workflow",
            Self::BrokerActuation => "broker actuation",
            Self::RecursivePowlDepth => "recursive POWL depth",
            Self::ObjectCentricAggregationLevel => "object-centric aggregation level",
            Self::BoundedExecutionCostBand => "bounded execution cost band",
        }
    }

    /// The `.rq` SELECT that computes this scale's raw mass-per-family
    /// (`Ok`), or the specific, investigated reason no `G_OCEL`-evidence
    /// data source exists for it yet (`Err`) — see the module doc's "which
    /// declared process scales are real today" section for the per-scale
    /// investigation each `Err` reason summarizes.
    ///
    /// # Complexity
    /// O(1).
    fn mass_query_or_reason(self) -> Result<&'static str, &'static str> {
        match self {
            Self::Workflow => Ok(MASS_BY_WORKFLOW),
            Self::Activity => Ok(MASS_BY_ACTIVITY),
            Self::ObjectCentricAggregationLevel => Ok(MASS_BY_OBJECT_TYPE),
            Self::EnterpriseGoal => Err(
                "no attribute in the event.praxis.activity_executed OTEL registry (or anywhere \
                 in the admitted G_OCEL evidence) encodes an enterprise-goal grouping above \
                 workflow id; enterprise goal is not instrumented anywhere upstream of this \
                 pipeline",
            ),
            Self::Program => Err(
                "no attribute encodes a program-level grouping between enterprise goal and \
                 process; program is not instrumented anywhere upstream of this pipeline",
            ),
            Self::Process => Err(
                "process.workflow.id captures a workflow instance, not a process definition; no \
                 separate process-level identifier is instrumented anywhere upstream of this \
                 pipeline",
            ),
            Self::Subprocess => Err(
                "no attribute encodes a subprocess grouping beneath process and above workflow; \
                 subprocess is not instrumented anywhere upstream of this pipeline",
            ),
            Self::ChildWorkflow => Err(
                "G_OCEL carries OTLP span parent_span_id (activity-level span nesting within one \
                 workflow instance) but no workflow-level socket-attachment relation; the real \
                 child-workflow evidence (bench-obs.ttl's hasChildWorkflow/hasParentActivity \
                 properties) exists only under the feature-gated bench observation pipeline, \
                 which this unconditional module cannot depend on, so no genuine child-workflow \
                 identifier exists in this module's G_OCEL",
            ),
            Self::BrokerActuation => Err(
                "no attribute records broker actuation routing or step; no OTLP producer in \
                 this codebase emits a broker-related attribute today",
            ),
            Self::RecursivePowlDepth => Err(
                "G_OCEL has no correlation between OTLP spans and POWL AST node depth: POWL \
                 execution is not wired to any span-emitting engine yet, and substituting OTLP \
                 span parent_span_id nesting (activity call nesting within one workflow, \
                 unrelated to POWL recursion) would mislabel a different quantity as this scale",
            ),
            Self::BoundedExecutionCostBand => Err(
                "no attribute records the bounded-descent cost vector C(W) = <d,a,u,r> (PRD \
                 sec.6.6); that budget is a POWL decomposition-time concept, never emitted as \
                 OTLP telemetry anywhere in this codebase",
            ),
        }
    }
}

/// One workflow family's raw execution measure `mu_x` at a declared
/// scale: `mass` is the count of admitted `G_OCEL` events attributed to
/// `family` (PRD.md sec.16's `mu_x(B_i(epsilon))`, before normalization).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionMeasure {
    pub family: String,
    pub mass: u64,
}

/// A declared, computed measurement profile (PRD.md:783's verbatim field
/// list): the scale, the q-range and fitting method a downstream
/// tau(q)/D(q)/f(alpha) estimator (PROJ-767) would use, the minimum
/// evidence threshold this module enforces before admitting a
/// measurement, a confidence-criteria note, and the source OCEL digest
/// this profile's measures were computed against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasurementProfile {
    pub scale: DeclaredProcessScale,
    pub q_range: Vec<i64>,
    pub fitting_method: String,
    pub min_evidence_threshold: u64,
    pub confidence_criteria: String,
    /// Computed by [`build_measurement_profile`] from the store's actual
    /// `G_OCEL` content — never caller-asserted (invariant #2, "receipts
    /// are computed, never asserted").
    pub source_ocel_digest: String,
}

fn ns_node(ns: &str, local: &str) -> NamedNode {
    NamedNode::new(format!("{ns}{local}"))
        .expect("vocabulary IRI is a compile-time-controlled constant, never external input")
}

fn rdf_type() -> NamedNode {
    NamedNode::new(RDF_TYPE_IRI).expect("RDF_TYPE_IRI is a compile-time-controlled constant")
}

fn result_graph_name() -> GraphName {
    GraphName::NamedNode(
        NamedNode::new(RESULT_GRAPH_IRI)
            .expect("RESULT_GRAPH_IRI is a compile-time-controlled constant"),
    )
}

fn term_value(term: &Term) -> String {
    match term {
        Term::Literal(l) => l.value().to_string(),
        Term::NamedNode(n) => n.as_str().to_string(),
        other => other.to_string(),
    }
}

/// Percent-encodes every byte outside the RFC 3986 unreserved set, so any
/// measured family value (workflow id, activity name) or scale name
/// yields a legal IRI path segment. Mirrors `otel_rdf::percent_encode`
/// (duplicated locally, matching this crate's established
/// self-contained-module convention rather than sharing tiny helpers
/// across modules).
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

/// Strips this crate's `blake3:` digest tag, leaving bare hex.
fn digest_hex(tagged: &str) -> &str {
    tagged.strip_prefix("blake3:").unwrap_or(tagged)
}

const XSD_NS: &str = "http://www.w3.org/2001/XMLSchema#";

fn xsd_non_negative_integer() -> NamedNode {
    ns_node(XSD_NS, "nonNegativeInteger")
}

/// Executes `scale`'s mass-by-family SELECT over `store`'s `G_OCEL` graph.
///
/// # Errors
/// `CngRefusal::MeasurementEvidenceInsufficient` (`CNG_R29`) if `scale`
/// has no real data source in this codebase yet, if the query fails to
/// parse/execute, or if it yields zero rows (no admitted evidence at that
/// scale).
///
/// # Complexity
/// O(e log e) where e is the admitted `G_OCEL` event count (the query's
/// own `GROUP BY`/`ORDER BY` cost; oxigraph, not this function, performs
/// the sort).
pub fn compute_execution_measure(
    store: &Store,
    scale: DeclaredProcessScale,
) -> Result<Vec<ExecutionMeasure>, CngRefusal> {
    let query = scale.mass_query_or_reason().map_err(|reason| {
        CngRefusal::MeasurementEvidenceInsufficient {
            scale: scale.as_str().to_string(),
            reason: reason.to_string(),
        }
    })?;
    let prepared = SparqlEvaluator::new().parse_query(query).map_err(|e| {
        CngRefusal::MeasurementEvidenceInsufficient {
            scale: scale.as_str().to_string(),
            reason: format!("mass query parse failed: {e}"),
        }
    })?;
    let results = prepared.on_store(store).execute().map_err(|e| {
        CngRefusal::MeasurementEvidenceInsufficient {
            scale: scale.as_str().to_string(),
            reason: format!("mass query execution failed: {e}"),
        }
    })?;
    let mut measures = Vec::new();
    match results {
        QueryResults::Solutions(solutions) => {
            for solution in solutions {
                let solution =
                    solution.map_err(|e| CngRefusal::MeasurementEvidenceInsufficient {
                        scale: scale.as_str().to_string(),
                        reason: format!("mass query row failed: {e}"),
                    })?;
                let family = solution.get("family").map(term_value).ok_or_else(|| {
                    CngRefusal::MeasurementEvidenceInsufficient {
                        scale: scale.as_str().to_string(),
                        reason: "mass query row missing ?family binding".to_string(),
                    }
                })?;
                let mass_text = solution.get("mass").map(term_value).ok_or_else(|| {
                    CngRefusal::MeasurementEvidenceInsufficient {
                        scale: scale.as_str().to_string(),
                        reason: "mass query row missing ?mass binding".to_string(),
                    }
                })?;
                let mass = mass_text.parse::<u64>().map_err(|e| {
                    CngRefusal::MeasurementEvidenceInsufficient {
                        scale: scale.as_str().to_string(),
                        reason: format!("mass value {mass_text:?} did not parse as u64: {e}"),
                    }
                })?;
                measures.push(ExecutionMeasure { family, mass });
            }
        }
        _ => {
            return Err(CngRefusal::MeasurementEvidenceInsufficient {
                scale: scale.as_str().to_string(),
                reason: "mass query did not yield SELECT solutions".to_string(),
            });
        }
    }
    if measures.is_empty() {
        return Err(CngRefusal::MeasurementEvidenceInsufficient {
            scale: scale.as_str().to_string(),
            reason: "zero admitted G_OCEL events at this declared process scale".to_string(),
        });
    }
    Ok(measures)
}

/// Builds a real, computed [`MeasurementProfile`] for `scale`: runs
/// [`compute_execution_measure`] first (propagating its refusal if
/// evidence is insufficient), then refuses if the measured family count
/// is below `min_evidence_threshold`, then computes `source_ocel_digest`
/// from `store`'s actual `G_OCEL` content
/// (`otel_ocel::graph_content_digest`) — never caller-asserted.
///
/// # Errors
/// `CngRefusal::MeasurementEvidenceInsufficient` (`CNG_R29`) if
/// [`compute_execution_measure`] refuses, or if the measured family count
/// is below `min_evidence_threshold`.
/// `CngRefusal::OcelConstructRefused` (`CNG_R28`) if the source-digest
/// read of `G_OCEL` fails.
///
/// # Complexity
/// O(e log e), dominated by [`compute_execution_measure`] and
/// [`otel_ocel::graph_content_digest`]'s canonical sort.
pub fn build_measurement_profile(
    store: &Store,
    scale: DeclaredProcessScale,
    q_range: Vec<i64>,
    fitting_method: String,
    min_evidence_threshold: u64,
    confidence_criteria: String,
) -> Result<(MeasurementProfile, Vec<ExecutionMeasure>), CngRefusal> {
    let measures = compute_execution_measure(store, scale)?;
    if (measures.len() as u64) < min_evidence_threshold {
        return Err(CngRefusal::MeasurementEvidenceInsufficient {
            scale: scale.as_str().to_string(),
            reason: format!(
                "{} distinct families measured, below the declared minimum evidence \
                 threshold of {min_evidence_threshold}",
                measures.len()
            ),
        });
    }
    let source_ocel_digest = otel_ocel::graph_content_digest(store, OCEL_GRAPH_IRI)?;
    let profile = MeasurementProfile {
        scale,
        q_range,
        fitting_method,
        min_evidence_threshold,
        confidence_criteria,
        source_ocel_digest,
    };
    Ok((profile, measures))
}

/// Projects a computed [`MeasurementProfile`] + its [`ExecutionMeasure`]s
/// into RDF quads under `G_RESULT` (`otel_ocel::RESULT_GRAPH_IRI`).
/// Content-addressed: the profile IRI is derived from `scale` +
/// `source_ocel_digest`, so re-running this over identical `G_OCEL`
/// content yields byte-identical quads (proven by
/// `measurement_test.rs`'s determinism test) — never a wall-clock-derived
/// or randomly-minted identity.
///
/// This function does not insert the result into a store — callers that
/// want the profile materialized call `otel_ocel::insert_quads` with the
/// returned `Vec`, mirroring `otel_rdf`/`otel_ocel`/`otel_receipt`'s
/// existing pure-function-then-insert split.
///
/// # Errors
/// `CngRefusal::OcelConstructRefused` (`CNG_R28`, stage `receipt_iri`) if
/// a minted IRI fails to construct (defensive; the inputs — a fixed scale
/// name and hex digest — are always legal IRI path segments today).
///
/// # Complexity
/// O(f log f) where f is `measures.len()` (the final canonical sort).
pub fn project_measurement_profile(
    profile: &MeasurementProfile,
    measures: &[ExecutionMeasure],
) -> Result<Vec<Quad>, CngRefusal> {
    let graph = result_graph_name();
    let scale_slug = percent_encode(profile.scale.as_str());
    let digest_slug = digest_hex(&profile.source_ocel_digest);

    let profile_node = NamedNode::new(format!(
        "urn:cng:measurement-profile:{scale_slug}:{digest_slug}"
    ))
    .map_err(|e| CngRefusal::OcelConstructRefused {
        stage: "receipt_iri".to_string(),
        reason: format!("measurement profile IRI construction failed: {e}"),
    })?;

    let mut quads = Vec::new();
    quads.push(Quad::new(
        profile_node.clone(),
        rdf_type(),
        Term::NamedNode(ns_node(MN_NS, "MeasurementProfile")),
        graph.clone(),
    ));
    quads.push(Quad::new(
        profile_node.clone(),
        ns_node(MN_NS, "scale"),
        Term::Literal(Literal::new_simple_literal(profile.scale.as_str())),
        graph.clone(),
    ));
    let q_range_text = profile
        .q_range
        .iter()
        .map(i64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    quads.push(Quad::new(
        profile_node.clone(),
        ns_node(MN_NS, "qRange"),
        Term::Literal(Literal::new_simple_literal(&q_range_text)),
        graph.clone(),
    ));
    quads.push(Quad::new(
        profile_node.clone(),
        ns_node(MN_NS, "fittingMethod"),
        Term::Literal(Literal::new_simple_literal(&profile.fitting_method)),
        graph.clone(),
    ));
    quads.push(Quad::new(
        profile_node.clone(),
        ns_node(MN_NS, "minEvidenceThreshold"),
        Term::Literal(Literal::new_typed_literal(
            profile.min_evidence_threshold.to_string(),
            xsd_non_negative_integer(),
        )),
        graph.clone(),
    ));
    quads.push(Quad::new(
        profile_node.clone(),
        ns_node(MN_NS, "confidenceCriteria"),
        Term::Literal(Literal::new_simple_literal(&profile.confidence_criteria)),
        graph.clone(),
    ));
    quads.push(Quad::new(
        profile_node.clone(),
        ns_node(MN_NS, "sourceOcelDigest"),
        Term::Literal(Literal::new_simple_literal(&profile.source_ocel_digest)),
        graph.clone(),
    ));

    for measure in measures {
        let family_slug = percent_encode(&measure.family);
        let measure_node = NamedNode::new(format!(
            "urn:cng:execution-measure:{scale_slug}:{family_slug}:{digest_slug}"
        ))
        .map_err(|e| CngRefusal::OcelConstructRefused {
            stage: "receipt_iri".to_string(),
            reason: format!("execution measure IRI construction failed: {e}"),
        })?;
        quads.push(Quad::new(
            measure_node.clone(),
            rdf_type(),
            Term::NamedNode(ns_node(MN_NS, "ExecutionMeasure")),
            graph.clone(),
        ));
        quads.push(Quad::new(
            measure_node.clone(),
            ns_node(MN_NS, "hasProfile"),
            Term::NamedNode(profile_node.clone()),
            graph.clone(),
        ));
        quads.push(Quad::new(
            measure_node.clone(),
            ns_node(MN_NS, "family"),
            Term::Literal(Literal::new_simple_literal(&measure.family)),
            graph.clone(),
        ));
        quads.push(Quad::new(
            measure_node.clone(),
            ns_node(MN_NS, "mass"),
            Term::Literal(Literal::new_typed_literal(
                measure.mass.to_string(),
                xsd_non_negative_integer(),
            )),
            graph.clone(),
        ));
        quads.push(Quad::new(
            profile_node.clone(),
            ns_node(MN_NS, "hasExecutionMeasure"),
            Term::NamedNode(measure_node),
            graph.clone(),
        ));
    }

    // Canonical order: sorted by each quad's N-Quads text, independent of
    // insertion order — matches `otel_rdf`/`otel_ocel`/`otel_receipt`'s own
    // canonicalization convention.
    quads.sort_by_key(|a| a.to_string());
    Ok(quads)
}

#[cfg(test)]
#[path = "measurement_test.rs"]
mod measurement_test;
