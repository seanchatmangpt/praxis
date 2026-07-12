//! PROJ-765: PROV-O transformation ancestry + digest-chain receipt for the
//! `G_OTEL -> G_OCEL` SPARQL CONSTRUCT projection
//! (`otel_ocel::project_otel_to_ocel`, PROJ-764). Populates `G_RECEIPT`
//! (`otel_ocel::RECEIPT_GRAPH_IRI`) — the fifth named graph PROJ-764
//! declared but deliberately left unpopulated (its own module doc:
//! "`G_RECEIPT`'s PROV-O ancestry and digest-chain receipt is populated by
//! the separate `crate::otel_receipt` module").
//!
//! # What this closes
//!
//! `docs/otel-rdf-handoff.md` ("Provenance (PROV-O)"):
//! > Each CONSTRUCT run records PROV-O provenance of the construction: the
//! > query digest and the input-graph digest derive the output-graph
//! > digest. A consumer can replay `digest(P) + digest(G_OTEL) ->
//! > digest(G_OCEL)` and refuse on mismatch — same discipline as the
//! > engine's computed (never asserted) BLAKE3 receipts.
//!
//! [`verify_receipt_otel_to_ocel`] is that replay-and-refuse consumer:
//! given a claimed receipt head, it independently recomputes the same three
//! digests plus the fold from `store`'s current content and refuses
//! (`CngRefusal::AuditMismatch`) on any drift.
//!
//! [`receipt_otel_to_ocel`] records exactly that: three constitutional
//! digests — the CONSTRUCT query text (`P`, `otel_ocel::
//! OTEL_TO_OCEL_CONSTRUCT`), the admitted `urn:graph:otel` content as of
//! this run, and the derived `urn:graph:ocel` content this same call
//! computes — as three `prov:Entity`/`prov:Plan` nodes content-addressed
//! by their own BLAKE3 digest (`urn:blake3:<hex>`, the same
//! content-addressing scheme `pipeline.rs::import_artifacts` already uses
//! for admitted PDDL artifacts), linked from one `prov:Activity` via
//! `prov:used` / `prov:hadPlan` / `prov:generated`. The three digests are
//! then folded, in one fixed tagged order, into a single receipt head
//! (`cngr:receiptHead`) — the "digest chain" this ticket is scoped to —
//! mirroring `praxis-graphlaw`'s `chatman::engine::receipt_root` tagged
//! ordered-digest-folding convention. That crate is not a dependency of
//! `cng`'s default build surface, so this module reimplements the same
//! discipline locally with `blake3::Hasher`, matching this crate's own
//! established chain-digest idiom (`bench/run.rs`'s `receipt_chain`,
//! `bench/dispatch.rs`'s ledger-link fold — ordered `Hasher::update` calls
//! over a fixed tag plus canonical inputs, never an unordered combine).
//!
//! # PROV-O vocabulary reuse
//!
//! `prov:Activity` / `prov:Entity` / `prov:Plan` / `prov:used` /
//! `prov:hadPlan` / `prov:generated` all come from
//! `crate::powl::PROV_PREFIX` (`http://www.w3.org/ns/prov#`), the exact
//! binding `powl.rs`'s decomposition-provenance serializer already uses
//! for `prov:wasDerivedFrom` — reused verbatim, not reinvented. That
//! existing usage is a distinct, unrelated concern (per-leaf decomposition
//! provenance in emitted POWL Turtle); this module's `G_RECEIPT` ancestry
//! for the OTEL->OCEL transformation step is new. The digest-chain fields
//! PROV-O has no native term for (content digests, the folded receipt
//! head, the transformation-kind tag) are minted under this module's own
//! `cngr:` namespace (`https://truex.io/ontology/cng-receipt#`),
//! following the same truex.io-namespace convention `powl.rs::
//! POWL2_PREFIX` and `queries/decomp/construct-provenance.rq`'s `decomp:`
//! namespace already establish in this crate.
//!
//! # Scope boundary against PROJ-764's forbidden-alternatives clause
//!
//! This module writes zero triples into `urn:graph:ocel` — it only reads
//! `urn:graph:otel` / calls [`otel_ocel::project_otel_to_ocel`] (already
//! CONSTRUCT-derived by PROJ-764) and writes exclusively into
//! `urn:graph:receipts`. Computing a receipt over already-admitted graph
//! content is provenance/receipt construction, not OCEL construction —
//! `docs/otel-rdf-handoff.md`'s forbidden-alternatives list names
//! imperative *OCEL* construction specifically ("Imperative Rust OCEL
//! construction (building OCEL objects in code instead of CONSTRUCT)");
//! it says nothing about provenance/receipt construction, which every
//! other receipt in this crate (`pipeline.rs`, `bench/run.rs`,
//! `bench/dispatch.rs`) already computes in Rust, never SPARQL.
//!
//! # Determinism
//!
//! No wall clock anywhere in this module. [`receipt_otel_to_ocel`] is a
//! pure function of `store`'s current `urn:graph:otel` content plus the
//! fixed CONSTRUCT query text baked in at compile time; the same graph
//! content always yields byte-identical receipt quads (proven by
//! `otel_receipt_test.rs`'s two-runs-byte-identical test). All digest
//! folding uses fixed-order `blake3::Hasher::update` calls — no
//! `HashMap`, no floating point.

use oxigraph::model::{GraphName, Literal, NamedNode, Quad, Term};
use oxigraph::store::Store;

use crate::otel_ocel;
use crate::powl::{CngRefusal, PROV_PREFIX};

/// This module's own minted vocabulary namespace for the digest-chain
/// fields PROV-O has no native term for.
const CNGR_NS: &str = "https://truex.io/ontology/cng-receipt#";

const RDF_TYPE_IRI: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

/// Tag folded into the receipt-head digest, versioned so a future change
/// to the folding order/inputs can never silently collide with today's
/// heads.
const RECEIPT_HEAD_TAG: &str = "cng/otel-ocel/receipt-head/v1";

/// The `transformationKind` literal recorded on every receipt this module
/// produces — one fixed transformation today (`G_OTEL -> G_OCEL`), named
/// so a future second transformation kind (if one is ever added) cannot be
/// confused with this one in a shared `G_RECEIPT` graph.
const TRANSFORMATION_KIND: &str = "otel-to-ocel";

fn ns_node(ns: &str, local: &str) -> NamedNode {
    NamedNode::new(format!("{ns}{local}"))
        .expect("vocabulary IRI is a compile-time-controlled constant, never external input")
}

fn rdf_type() -> NamedNode {
    NamedNode::new(RDF_TYPE_IRI).expect("RDF_TYPE_IRI is a compile-time-controlled constant")
}

fn receipt_graph_name() -> GraphName {
    GraphName::NamedNode(
        NamedNode::new(otel_ocel::RECEIPT_GRAPH_IRI)
            .expect("RECEIPT_GRAPH_IRI is a compile-time-controlled constant"),
    )
}

/// Strips this crate's `blake3:` digest tag, leaving bare hex — so the
/// digest can be embedded as an IRI's local part (`urn:blake3:<hex>`)
/// without a colon inside it.
fn digest_hex(tagged: &str) -> &str {
    tagged.strip_prefix("blake3:").unwrap_or(tagged)
}

/// Mints the content-addressed `urn:blake3:<hex>` node for a digest,
/// reusing `pipeline.rs::import_artifacts`'s existing content-addressing
/// scheme verbatim.
///
/// # Errors
/// `CngRefusal::OcelConstructRefused` (`CNG_R28`, stage `receipt_iri`) if
/// the digest hex fails to form a legal IRI (defensive; BLAKE3 hex output
/// is always `[0-9a-f]+`, so this path is unreachable in practice).
fn content_addressed_node(tagged_digest: &str) -> Result<NamedNode, CngRefusal> {
    NamedNode::new(format!("urn:blake3:{}", digest_hex(tagged_digest))).map_err(|e| {
        CngRefusal::OcelConstructRefused {
            stage: "receipt_iri".to_string(),
            reason: format!("content-addressed node IRI construction failed: {e}"),
        }
    })
}

/// Folds the three constitutional digests (query, input graph, output
/// graph) into one tagged receipt-head digest — the "digest chain" this
/// ticket is scoped to.
///
/// # Complexity
/// O(1): three fixed-size digest strings, one `Hasher` pass.
fn fold_receipt_head(query_digest: &str, input_digest: &str, output_digest: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(RECEIPT_HEAD_TAG.as_bytes());
    hasher.update(b"\0");
    hasher.update(query_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update(input_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update(output_digest.as_bytes());
    format!("blake3:{}", hasher.finalize().to_hex())
}

/// The three constitutional digests plus their folded head, for one
/// `store`'s current graph content.
struct ReceiptDigests {
    /// Digest of the CONSTRUCT query text (`P`).
    query_digest: String,
    /// Digest of the admitted `urn:graph:otel` content.
    input_digest: String,
    /// Digest of the derived `urn:graph:ocel` content.
    output_digest: String,
    /// [`fold_receipt_head`] over the three digests above, in that order.
    receipt_head: String,
}

/// Computes [`ReceiptDigests`] for `store`'s current graph content — the
/// single definition both [`receipt_otel_to_ocel`] (which builds the full
/// PROV-O ancestry quads around these digests) and
/// [`verify_receipt_otel_to_ocel`] (which only needs the folded head to
/// compare against a claimed one) call. There is exactly one formula for
/// "what should this store's receipt head be"; a verifier built by
/// independently re-deriving that formula (as `otel_receipt_test.rs`'s
/// `receipt_head_chains_the_three_constitutional_digests` test does, by
/// necessity, since a test cannot call a private production function it
/// duplicates) can silently drift from what sealing actually computes the
/// moment either copy changes — sharing this helper closes that gap for the
/// production verifier the same way [`fold_receipt_head`] itself already
/// avoids duplicating the fold arithmetic.
///
/// # Errors
/// `CngRefusal::OcelConstructRefused` if reading `urn:graph:otel`'s content
/// for the input digest fails, or if the underlying `G_OTEL -> G_OCEL`
/// CONSTRUCT projection fails (any of its own documented stages).
///
/// # Complexity
/// O(m log m) where m is `max(|G_OTEL|, |G_OCEL|)` (dominated by the two
/// canonical-content-digest sorts, one via
/// [`otel_ocel::graph_content_digest`] and one local to this function).
fn compute_receipt_digests(store: &Store) -> Result<ReceiptDigests, CngRefusal> {
    let query_digest = format!(
        "blake3:{}",
        blake3::hash(otel_ocel::OTEL_TO_OCEL_CONSTRUCT.as_bytes()).to_hex()
    );
    let input_digest = otel_ocel::graph_content_digest(store, otel_ocel::OTEL_GRAPH_IRI)?;

    let ocel_quads = otel_ocel::project_otel_to_ocel(store)?;
    let mut ocel_lines: Vec<String> = ocel_quads.iter().map(|q| q.to_string()).collect();
    ocel_lines.sort();
    ocel_lines.dedup();
    let output_digest = format!(
        "blake3:{}",
        blake3::hash(ocel_lines.join("\n").as_bytes()).to_hex()
    );

    let receipt_head = fold_receipt_head(&query_digest, &input_digest, &output_digest);
    Ok(ReceiptDigests {
        query_digest,
        input_digest,
        output_digest,
        receipt_head,
    })
}

/// Independently recomputes [`ReceiptDigests`] for `store`'s current graph
/// content — via the identical [`compute_receipt_digests`] helper
/// [`receipt_otel_to_ocel`] itself calls, never a second hand-maintained
/// formula — and compares the folded receipt head to `claimed_head`,
/// refusing on any mismatch. The independence this function provides is
/// against a stale or tampered `claimed_head`, not against
/// `compute_receipt_digests` itself: this is the same "replay through the
/// real code path and compare" discipline
/// `praxis_graphlaw::chatman::engine::ChatmanEngine::verify_replay` uses,
/// not a from-scratch reimplementation of the digest algorithm (see that
/// function's own doc comment for why the latter would only ever prove two
/// copies of the same bug agree).
///
/// # Errors
/// Everything [`compute_receipt_digests`] can refuse with, unchanged; plus
/// `CngRefusal::AuditMismatch` (`CNG_R11`) if the recomputed receipt head
/// does not equal `claimed_head`.
///
/// # Complexity
/// Same as [`compute_receipt_digests`]: O(m log m) where m is
/// `max(|G_OTEL|, |G_OCEL|)`.
pub fn verify_receipt_otel_to_ocel(store: &Store, claimed_head: &str) -> Result<(), CngRefusal> {
    let digests = compute_receipt_digests(store)?;
    if digests.receipt_head != claimed_head {
        return Err(CngRefusal::AuditMismatch(format!(
            "otel-to-ocel receipt head mismatch: claimed {claimed_head} recomputed {}",
            digests.receipt_head
        )));
    }
    Ok(())
}

/// Records real PROV-O ancestry + a digest-chain receipt, in
/// `urn:graph:receipts` (`G_RECEIPT`), for one `G_OTEL -> G_OCEL` CONSTRUCT
/// transformation over `store`'s current graph content. Reads (never
/// writes) `urn:graph:otel`; calls [`otel_ocel::project_otel_to_ocel`] to
/// obtain the exact `G_OCEL` quads this receipt describes (never a
/// caller-supplied value — receipts are computed, never asserted); writes
/// only into `urn:graph:receipts`.
///
/// This function does not insert the result into `store` — callers that
/// want the receipt materialized call `otel_ocel::insert_quads` with the
/// returned `Vec`, mirroring `otel_rdf::project_admitted_spans` and
/// `otel_ocel::project_otel_to_ocel`'s existing pure-function-then-insert
/// split.
///
/// # Errors
/// `CngRefusal::OcelConstructRefused` (`CNG_R28`) if the underlying
/// CONSTRUCT projection fails (any of its own documented stages), if
/// reading `urn:graph:otel`'s content for the input digest fails (stage
/// `digest_iteration`), or if a digest-derived receipt/entity IRI fails to
/// construct (stage `receipt_iri`, defensive/unreachable in practice).
///
/// # Complexity
/// O(m log m) where m is `max(|G_OTEL|, |G_OCEL|)` (dominated by the two
/// canonical-content-digest sorts, one via
/// [`otel_ocel::graph_content_digest`] and one local to this function);
/// O(1) beyond that for the fixed, small receipt-quad count itself.
pub fn receipt_otel_to_ocel(store: &Store) -> Result<Vec<Quad>, CngRefusal> {
    let ReceiptDigests {
        query_digest,
        input_digest,
        output_digest,
        receipt_head,
    } = compute_receipt_digests(store)?;

    let query_node = content_addressed_node(&query_digest)?;
    let otel_node = content_addressed_node(&input_digest)?;
    let ocel_node = content_addressed_node(&output_digest)?;
    let activity_node = NamedNode::new(format!(
        "urn:cng:receipt:otel-to-ocel:{}",
        digest_hex(&receipt_head)
    ))
    .map_err(|e| CngRefusal::OcelConstructRefused {
        stage: "receipt_iri".to_string(),
        reason: format!("receipt activity IRI construction failed: {e}"),
    })?;

    let graph = receipt_graph_name();
    let mut quads = Vec::new();

    quads.push(Quad::new(
        activity_node.clone(),
        rdf_type(),
        Term::NamedNode(ns_node(PROV_PREFIX, "Activity")),
        graph.clone(),
    ));
    quads.push(Quad::new(
        activity_node.clone(),
        rdf_type(),
        Term::NamedNode(ns_node(CNGR_NS, "ConstructTransformationReceipt")),
        graph.clone(),
    ));
    quads.push(Quad::new(
        activity_node.clone(),
        ns_node(CNGR_NS, "transformationKind"),
        Term::Literal(Literal::new_simple_literal(TRANSFORMATION_KIND)),
        graph.clone(),
    ));
    quads.push(Quad::new(
        activity_node.clone(),
        ns_node(PROV_PREFIX, "used"),
        Term::NamedNode(otel_node.clone()),
        graph.clone(),
    ));
    quads.push(Quad::new(
        activity_node.clone(),
        ns_node(PROV_PREFIX, "hadPlan"),
        Term::NamedNode(query_node.clone()),
        graph.clone(),
    ));
    quads.push(Quad::new(
        activity_node.clone(),
        ns_node(PROV_PREFIX, "generated"),
        Term::NamedNode(ocel_node.clone()),
        graph.clone(),
    ));
    quads.push(Quad::new(
        activity_node,
        ns_node(CNGR_NS, "receiptHead"),
        Term::Literal(Literal::new_simple_literal(&receipt_head)),
        graph.clone(),
    ));

    for (node, digest, source_graph_iri) in [
        (&otel_node, &input_digest, otel_ocel::OTEL_GRAPH_IRI),
        (&ocel_node, &output_digest, otel_ocel::OCEL_GRAPH_IRI),
    ] {
        quads.push(Quad::new(
            node.clone(),
            rdf_type(),
            Term::NamedNode(ns_node(PROV_PREFIX, "Entity")),
            graph.clone(),
        ));
        quads.push(Quad::new(
            node.clone(),
            ns_node(CNGR_NS, "contentDigest"),
            Term::Literal(Literal::new_simple_literal(digest)),
            graph.clone(),
        ));
        let source_node =
            NamedNode::new(source_graph_iri).map_err(|e| CngRefusal::OcelConstructRefused {
                stage: "receipt_iri".to_string(),
                reason: format!("source graph IRI construction failed: {e}"),
            })?;
        quads.push(Quad::new(
            node.clone(),
            ns_node(CNGR_NS, "sourceGraph"),
            Term::NamedNode(source_node),
            graph.clone(),
        ));
    }
    quads.push(Quad::new(
        query_node.clone(),
        rdf_type(),
        Term::NamedNode(ns_node(PROV_PREFIX, "Plan")),
        graph.clone(),
    ));
    quads.push(Quad::new(
        query_node,
        ns_node(CNGR_NS, "contentDigest"),
        Term::Literal(Literal::new_simple_literal(&query_digest)),
        graph,
    ));

    // Canonical order: sorted by each quad's N-Quads text, independent of
    // insertion order — matches `otel_rdf::project_admitted_spans` and
    // `otel_ocel::project_otel_to_ocel`'s own canonicalization convention.
    quads.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
    Ok(quads)
}

#[cfg(test)]
#[path = "otel_receipt_test.rs"]
mod otel_receipt_test;
