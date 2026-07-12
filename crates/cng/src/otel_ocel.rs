//! PROJ-764: derives `urn:graph:ocel` from the admitted `urn:graph:otel`
//! graph (PROJ-763, `crate::otel_rdf`) via a real SPARQL CONSTRUCT query,
//! and names the 5-layer named-graph separation the mission's evidence
//! pipeline requires (`docs/otel-rdf-handoff.md`, "Named-graph layering").
//!
//! # The `G_OTEL -> G_OCEL` projection
//!
//! [`project_otel_to_ocel`] runs `queries/otel-to-ocel.construct.rq`
//! (compiled in via `include_str!` — no inline SPARQL in Rust source, per
//! `tests/no_inline_ttl_guard.rs`) against the `urn:graph:otel` named graph
//! inside a caller-supplied [`Store`], deriving OCEL 2.0 events, event
//! types, objects, object types, and attributes. This is the mapping the
//! handoff doc names `G_OCEL = CONSTRUCT_P(G_OTEL)`.
//!
//! ## Verifying the "forbidden alternatives" boundary against the doc's own
//! text (not a paraphrase)
//!
//! `docs/otel-rdf-handoff.md` "Forbidden alternatives" reads, verbatim:
//! > 2. Imperative Rust OCEL construction (building OCEL objects in code
//! >    instead of CONSTRUCT).
//! >
//! > Either alternative would make OCEL an asserted artifact rather than a
//! > computed projection of the admitted OTEL graph, breaking the receipt
//! > discipline.
//!
//! The object of "instead of CONSTRUCT" is the *derived* OCEL graph — the
//! doc's own named-graph table lists `urn:graph:ocel` as "CONSTRUCT-derived
//! OCEL, never hand-built", as a layer distinct from `urn:graph:otel`
//! ("admitted OTEL signals, this boundary's output"). PROJ-763's
//! `otel_rdf::project_admitted_spans` is imperative Rust, but it writes only
//! into `urn:graph:otel` (mapping raw OTLP wire fields — trace/span ids,
//! timestamps, status — into RDF, reusing OCEL 2.0 vocabulary terms where
//! they already fit the span-occurrence shape with no minting needed, per
//! that module's own doc comment). It never writes `urn:graph:ocel`. This
//! module is therefore the *only* code path in the crate that populates
//! `urn:graph:ocel`, and it does so exclusively through
//! [`project_otel_to_ocel`]'s CONSTRUCT query — satisfying the doc's actual
//! constraint on its own terms, not by asserting the paraphrase is correct.
//!
//! # The 5-layer graph separation
//!
//! [`SOURCE_GRAPH_IRI`] / `otel_rdf::OTEL_GRAPH_IRI` / [`OCEL_GRAPH_IRI`] /
//! [`RESULT_GRAPH_IRI`] / [`RECEIPT_GRAPH_IRI`] name the five layers. This
//! ticket populates three with real content: `G_SOURCE` (authored ontology
//! Turtle, via [`load_source_graph`] — the caller supplies the Turtle text,
//! e.g. read from `otel-bridge.ttl`; this module never embeds Turtle
//! inline), `G_OTEL` (PROJ-763, `otel_rdf::project_admitted_spans`), and
//! `G_OCEL` (this module, [`project_otel_to_ocel`]). `G_RESULT` and
//! `G_RECEIPT` are declared here as distinct, reserved named-graph
//! constants — the separation mechanism itself is real and tested
//! (`otel_ocel_test.rs` proves all five graph names are queryable in
//! isolation and that the three layers this module populates hold
//! manifestly different content, not five labels on one shared triple
//! set) — but this module itself deliberately leaves their *content*
//! unpopulated here: `G_RECEIPT`'s PROV-O ancestry and digest-chain
//! receipt is populated by the separate `crate::otel_receipt` module
//! (PROJ-765, "PROV-O transformation ancestry + CONSTRUCT provenance
//! receipt"); `G_RESULT`'s Rail F evidence/measurement-profile wiring is
//! populated by the separate `crate::measurement` module (PROJ-766).
//! Populating either here, in this module, would preempt or duplicate
//! that separately scoped work rather than keeping the layering honest —
//! [`graph_content_digest`] is this module's one shared export both of
//! those modules call, so all three receipts/measurements canonicalize
//! named-graph content identically.

use std::collections::BTreeMap;

use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::model::{GraphName, NamedNode, Quad};
use oxigraph::sparql::{QueryResults, SparqlEvaluator};
use oxigraph::store::Store;

use crate::powl::CngRefusal;

/// `G_SOURCE` — authored TTL (plans, ontologies), `docs/otel-rdf-handoff.md`
/// "Named-graph layering". Populated by [`load_source_graph`] from
/// caller-supplied Turtle text (this module contains no inline Turtle).
pub const SOURCE_GRAPH_IRI: &str = "urn:graph:source";

/// `G_OTEL` — admitted OTEL signals (PROJ-763, `otel_rdf::OTEL_GRAPH_IRI`).
/// Re-exported here so callers assembling all five layers have a single
/// import path.
pub const OTEL_GRAPH_IRI: &str = crate::otel_rdf::OTEL_GRAPH_IRI;

/// `G_OCEL` — CONSTRUCT-derived OCEL, never hand-built. Populated
/// exclusively by [`project_otel_to_ocel`].
pub const OCEL_GRAPH_IRI: &str = "urn:graph:ocel";

/// `G_RESULT` — verdicts, gate outcomes. Declared here as part of the
/// 5-layer separation; content population is PROJ-766's scope (see the
/// module doc).
pub const RESULT_GRAPH_IRI: &str = "urn:graph:results";

/// `G_RECEIPT` — sealed receipt envelopes. Declared here as part of the
/// 5-layer separation; content population (PROV-O ancestry + digest-chain
/// receipt) is PROJ-765's scope (see the module doc).
pub const RECEIPT_GRAPH_IRI: &str = "urn:graph:receipts";

/// All five layer IRIs, in the order the handoff doc lists them.
///
/// # Complexity
/// O(1).
pub const GRAPH_LAYERS: [&str; 5] = [
    SOURCE_GRAPH_IRI,
    OTEL_GRAPH_IRI,
    OCEL_GRAPH_IRI,
    RESULT_GRAPH_IRI,
    RECEIPT_GRAPH_IRI,
];

/// The `G_OTEL -> G_OCEL` CONSTRUCT query. Reads `GRAPH <urn:graph:otel>`
/// (must stay byte-identical to [`OTEL_GRAPH_IRI`]; the query file's own
/// header comment cross-references this).
///
/// `pub(crate)`: PROJ-765's `otel_receipt` module reads this same query
/// text to compute `digest(P)` — the query digest half of the handoff
/// doc's `digest(P) + digest(G_OTEL) -> digest(G_OCEL)` receipt contract —
/// so the receipted digest can never drift from the query this module
/// actually executes.
pub(crate) const OTEL_TO_OCEL_CONSTRUCT: &str = include_str!("queries/otel-to-ocel.construct.rq");

fn named_graph(iri: &str) -> GraphName {
    // Safe: called only from `ocel_graph_name`/`source_graph_name` below,
    // both of which pass one of this module's own `&'static str` constants,
    // never external input (assertion on a compile-time-controlled literal,
    // matching the existing convention in `otel_rdf.rs::ns_node`). Callers
    // accepting a runtime-supplied graph IRI (`graph_content_digest`'s
    // public `graph_iri` parameter) use `named_graph_checked` instead, which
    // never panics.
    GraphName::NamedNode(
        NamedNode::new(iri)
            .expect("graph IRI is a compile-time-controlled constant, never external input"),
    )
}

/// Fallible counterpart to [`named_graph`] for callers that accept a
/// runtime-supplied graph IRI rather than one of this module's own
/// compile-time constants. Mirrors `otel_rdf.rs`'s existing convention of
/// `.map_err`-ing `NamedNode::new` into a typed refusal for external input
/// (see e.g. `otel_rdf.rs`'s `activity_iri_str` handling), as opposed to
/// `named_graph`'s `.expect()` for compile-time-controlled literals.
///
/// # Errors
/// `CngRefusal::OcelConstructRefused` (`CNG_R28`, stage `graph_iri_parse`)
/// if `graph_iri` does not parse as a legal absolute IRI.
///
/// # Complexity
/// O(n) in `graph_iri.len()` (IRI syntax validation).
fn named_graph_checked(graph_iri: &str) -> Result<GraphName, CngRefusal> {
    let node = NamedNode::new(graph_iri).map_err(|e| CngRefusal::OcelConstructRefused {
        stage: "graph_iri_parse".to_string(),
        reason: format!("graph_iri {graph_iri:?} is not a legal IRI: {e}"),
    })?;
    Ok(GraphName::NamedNode(node))
}

fn ocel_graph_name() -> GraphName {
    named_graph(OCEL_GRAPH_IRI)
}

fn source_graph_name() -> GraphName {
    named_graph(SOURCE_GRAPH_IRI)
}

/// Runs the `G_OTEL -> G_OCEL` SPARQL CONSTRUCT query against `store`'s
/// `urn:graph:otel` named graph and returns the derived triples as `Quad`s
/// in the `urn:graph:ocel` named graph, canonically ordered and
/// deduplicated.
///
/// This function does not insert the result into `store` — callers that
/// want the projection materialized call [`insert_quads`] with the
/// returned `Vec`, mirroring `otel_rdf::project_admitted_spans`'s
/// pure-function-then-insert split.
///
/// # Errors
/// `CngRefusal::OcelConstructRefused` (`CNG_R28`) if the query fails to
/// parse, fails to execute, does not yield a graph result, or a produced
/// triple fails to decode.
///
/// # Complexity
/// O(m log m) where m is the number of CONSTRUCT solution rows (each
/// admitted span contributes one row per extra attribute, since the query's
/// two independent `OPTIONAL` blocks are left-joined onto the per-event
/// binding) — dominated by the canonicalizing `BTreeMap` insert used to
/// deduplicate CONSTRUCT's per-row re-assertion of shared event-level
/// triples into a true set.
pub fn project_otel_to_ocel(store: &Store) -> Result<Vec<Quad>, CngRefusal> {
    let prepared = SparqlEvaluator::new()
        .parse_query(OTEL_TO_OCEL_CONSTRUCT)
        .map_err(|e| CngRefusal::OcelConstructRefused {
            stage: "query_parse".to_string(),
            reason: e.to_string(),
        })?;
    let results =
        prepared
            .on_store(store)
            .execute()
            .map_err(|e| CngRefusal::OcelConstructRefused {
                stage: "query_execution".to_string(),
                reason: e.to_string(),
            })?;
    let target_graph = ocel_graph_name();
    // BTreeMap keyed by canonical N-Quads text: dedups CONSTRUCT's
    // per-solution-row re-assertion of shared triples (repo law: BTreeMap in
    // every digest/canonicalization path, never HashMap iteration order).
    let mut canonical: BTreeMap<String, Quad> = BTreeMap::new();
    match results {
        QueryResults::Graph(triples) => {
            for triple in triples {
                let triple = triple.map_err(|e| CngRefusal::OcelConstructRefused {
                    stage: "query_execution".to_string(),
                    reason: format!("construct eval: {e}"),
                })?;
                let quad = triple.in_graph(target_graph.clone());
                canonical.insert(quad.to_string(), quad);
            }
        }
        _ => {
            return Err(CngRefusal::OcelConstructRefused {
                stage: "not_a_graph_result".to_string(),
                reason: "CONSTRUCT query did not yield a graph".to_string(),
            });
        }
    }
    Ok(canonical.into_values().collect())
}

/// Inserts `quads` into `store`. A thin, typed-refusal-wrapped
/// `Store::insert` loop shared by every layer-population path in this
/// module (`urn:graph:ocel` via [`project_otel_to_ocel`]; `urn:graph:otel`
/// via `otel_rdf::project_admitted_spans`).
///
/// # Errors
/// `CngRefusal::OcelConstructRefused` (`CNG_R28`, stage `graph_insert`) if
/// the store refuses an insert.
///
/// # Complexity
/// O(n) in `quads.len()`.
pub fn insert_quads(store: &Store, quads: &[Quad]) -> Result<(), CngRefusal> {
    for quad in quads {
        store
            .insert(quad.as_ref())
            .map_err(|e| CngRefusal::OcelConstructRefused {
                stage: "graph_insert".to_string(),
                reason: format!("insert failed for {quad}: {e}"),
            })?;
    }
    Ok(())
}

/// Canonical BLAKE3 content digest of every quad currently in the named
/// graph `graph_iri` within `store`: each quad's canonical N-Quads text
/// (`Quad`'s `Display` impl, which — unlike a triple's — includes the
/// graph name), sorted, deduplicated, newline-joined, then hashed.
/// Mirrors `bench/run.rs::evidence_digest`'s existing
/// sort-then-hash-canonical-N-Triples pattern (this crate's one
/// established canonicalize-then-BLAKE3 convention), scoped to a single
/// named graph rather than a whole store. Reused by PROJ-765's
/// `otel_receipt` module (the `G_OTEL`/`G_OCEL` content digests half of
/// the handoff doc's `digest(P) + digest(G_OTEL) -> digest(G_OCEL)`
/// receipt contract) and PROJ-766's `measurement` module (a
/// [`MeasurementProfile`](crate::measurement::MeasurementProfile)'s
/// `source_ocel_digest` field), so both tickets' receipts share one
/// canonicalization rule rather than two independently-invented ones.
///
/// # Errors
/// `CngRefusal::OcelConstructRefused` (`CNG_R28`, stage `graph_iri_parse`)
/// if `graph_iri` does not parse as a legal absolute IRI, or stage
/// `digest_iteration` if quad iteration over the named graph fails.
///
/// # Complexity
/// O(m log m) where m is the number of quads in the named graph (the sort
/// dominates; iteration and hashing are both O(m)).
pub fn graph_content_digest(store: &Store, graph_iri: &str) -> Result<String, CngRefusal> {
    let graph = named_graph_checked(graph_iri)?;
    let mut lines: Vec<String> = Vec::new();
    for quad in store.quads_for_pattern(None, None, None, Some(graph.as_ref())) {
        let quad = quad.map_err(|e| CngRefusal::OcelConstructRefused {
            stage: "digest_iteration".to_string(),
            reason: format!("quad iteration failed for graph {graph_iri}: {e}"),
        })?;
        lines.push(quad.to_string());
    }
    lines.sort();
    lines.dedup();
    let text = lines.join("\n");
    Ok(format!("blake3:{}", blake3::hash(text.as_bytes()).to_hex()))
}

/// Parses caller-supplied `turtle` text and inserts every resulting triple
/// into `store` under [`SOURCE_GRAPH_IRI`] (`G_SOURCE`). The Turtle text
/// itself is never embedded in this crate's Rust source — callers read it
/// from a `.ttl` file (e.g. `otel_ocel_test.rs` loads
/// `otel-bridge.ttl`, the same fixture `otel_rdf_test.rs` parses).
///
/// # Errors
/// `CngRefusal::OcelConstructRefused` (`CNG_R28`, stage `source_load`) if
/// `turtle` fails to parse as Turtle or a triple fails to insert.
///
/// # Complexity
/// O(t) in the parsed triple count `t`.
pub fn load_source_graph(store: &Store, turtle: &str) -> Result<usize, CngRefusal> {
    let parser = RdfParser::from_format(RdfFormat::Turtle).with_default_graph(source_graph_name());
    let mut count = 0usize;
    for quad in parser.for_slice(turtle.as_bytes()) {
        let quad = quad.map_err(|e| CngRefusal::OcelConstructRefused {
            stage: "source_load".to_string(),
            reason: format!("Turtle parse failed: {e}"),
        })?;
        store
            .insert(quad.as_ref())
            .map_err(|e| CngRefusal::OcelConstructRefused {
                stage: "source_load".to_string(),
                reason: format!("insert failed for {quad}: {e}"),
            })?;
        count += 1;
    }
    Ok(count)
}

#[cfg(test)]
#[path = "otel_ocel_test.rs"]
mod otel_ocel_test;
