//! Arazzo Projection Receipt binding, plus (PROJ-752) the Rail A Tera
//! renderer T that manufactures an Arazzo 1.1.x document from PROJ-751's
//! real Q-stage SPARQL projection rows: `A_z = T(Q(W))` (PRD.md sec.7.4).

use crate::error::CoreError;
use praxis_graphlaw::chatman::powl_projection::ProjectionRow;
use serde::{Deserialize, Serialize};

/// Binding for the Arazzo Projection Receipt as required by PRD Iteration 8.
///
/// Binds:
/// - source POWL digest
/// - external-cut identity
/// - SPARQL projection digest
/// - Tera template digest
/// - Arazzo digest
/// - compiler version
/// - AIR digest
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArazzoProjectionReceipt {
    /// source POWL digest (hex)
    pub source_powl_digest_hex: String,
    /// external-cut identity
    pub external_cut_identity: String,
    /// SPARQL projection digest (hex)
    pub sparql_projection_digest_hex: String,
    /// Tera template digest (hex)
    pub tera_template_digest_hex: String,
    /// Arazzo digest (hex)
    pub arazzo_digest_hex: String,
    /// compiler version
    pub compiler_version: String,
    /// AIR digest (hex)
    pub air_digest_hex: String,
}

impl ArazzoProjectionReceipt {
    /// Compute the receipt's canonical BLAKE3 digest from its facts
    /// serialized to canonical N-Quads order.
    pub fn compute_digest(&self) -> Result<[u8; 32], CoreError> {
        // Construct canonical N-Quads representing these facts.
        let subject = format!(
            "<urn:praxis:arazzo:projection:{}>",
            self.external_cut_identity
        );
        let mut quads = vec![
            format!(
                "{subject} <urn:praxis:predicate:source_powl_digest> \"{}\" .",
                self.source_powl_digest_hex
            ),
            format!(
                "{subject} <urn:praxis:predicate:external_cut_identity> \"{}\" .",
                self.external_cut_identity
            ),
            format!(
                "{subject} <urn:praxis:predicate:sparql_projection_digest> \"{}\" .",
                self.sparql_projection_digest_hex
            ),
            format!(
                "{subject} <urn:praxis:predicate:tera_template_digest> \"{}\" .",
                self.tera_template_digest_hex
            ),
            format!(
                "{subject} <urn:praxis:predicate:arazzo_digest> \"{}\" .",
                self.arazzo_digest_hex
            ),
            format!(
                "{subject} <urn:praxis:predicate:compiler_version> \"{}\" .",
                self.compiler_version
            ),
            format!(
                "{subject} <urn:praxis:predicate:air_digest> \"{}\" .",
                self.air_digest_hex
            ),
        ];

        // "All facts in canonical N-Quads order" means lexicographically sorted.
        quads.sort();

        // Join with newlines and add a trailing newline (standard N-Quads).
        let nquads_str = format!("{}\n", quads.join("\n"));

        let digest = *blake3::hash(nquads_str.as_bytes()).as_bytes();
        Ok(digest)
    }

    /// Builds a receipt from the real materials of one Rail A projection
    /// run: the admitted POWL region's Turtle (`W`), the SPARQL projection
    /// text actually executed against it (`Q`), the Tera template text
    /// actually rendered (`T`), and the resulting Arazzo JSON document
    /// (`A_z = T(Q(W))`, PRD.md sec.7.4).
    ///
    /// `source_powl_digest_hex`, `sparql_projection_digest_hex`,
    /// `tera_template_digest_hex`, and `arazzo_digest_hex` are computed here
    /// via BLAKE3 over the real material bytes the caller supplies -- never
    /// hand-typed placeholder strings (the prior state this ticket closes:
    /// `arazzo_projection_receipt_digest_is_deterministic` below is the only
    /// other populated instance in the crate, and it is deliberately built
    /// from literal placeholder strings for a pure digest-stability test,
    /// not presented as a real projection run).
    ///
    /// `air_digest_hex` remains caller-supplied: Arazzo -> AIR lowering
    /// (PROJ-753) does not exist in this codebase yet, so there is no real
    /// AIR artifact to hash here. Callers ahead of PROJ-753 landing should
    /// pass an explicit not-yet-available sentinel (e.g.
    /// `"unavailable:PROJ-753"`) rather than fabricate a digest of nothing.
    ///
    /// # Complexity
    /// O(b) where b is the total byte length of the four hashed materials
    /// (one BLAKE3 pass per material).
    pub fn from_materials(
        source_powl_turtle: &str,
        external_cut_identity: &str,
        sparql_query_text: &str,
        tera_template_text: &str,
        arazzo_document_json: &str,
        compiler_version: &str,
        air_digest_hex: &str,
    ) -> Self {
        Self {
            source_powl_digest_hex: hex::encode(
                blake3::hash(source_powl_turtle.as_bytes()).as_bytes(),
            ),
            external_cut_identity: external_cut_identity.to_string(),
            sparql_projection_digest_hex: hex::encode(
                blake3::hash(sparql_query_text.as_bytes()).as_bytes(),
            ),
            tera_template_digest_hex: hex::encode(
                blake3::hash(tera_template_text.as_bytes()).as_bytes(),
            ),
            arazzo_digest_hex: hex::encode(
                blake3::hash(arazzo_document_json.as_bytes()).as_bytes(),
            ),
            compiler_version: compiler_version.to_string(),
            air_digest_hex: air_digest_hex.to_string(),
        }
    }
}

// ── T stage: Tera renderer (PROJ-752) ───────────────────────────────────

/// The Rail A Arazzo-manufacture Tera template. Single source of truth is
/// the checked-in template file itself
/// (`crates/praxis-core/templates/arazzo_projection.tera`), embedded at
/// compile time so there is exactly one copy of the template text.
const ARAZZO_PROJECTION_TEMPLATE: &str = include_str!("../templates/arazzo_projection.tera");

/// One ordered step resolved from a [`ProjectionRow`] slice: either a leaf
/// activity (from a `powl2:ActivityLeaf`/`powl2:SilentLeaf`) or an external
/// cut boundary (from a `powl2:ExternalCut`), never both -- the source
/// `Powl` enum makes the two mutually exclusive per node.
#[derive(Debug, Clone, PartialEq, Eq)]
struct StepProjection {
    /// The projected element's subject IRI (becomes the manufactured
    /// step's `operationId`: the identifier of the POWL element this step
    /// was projected from, not a resolved external API operation).
    element_id: String,
    /// Human-readable step description: the activity label, or a
    /// synthetic label for a silent leaf / external cut.
    label: String,
    /// True when this step is an external-cut boundary rather than a
    /// leaf activity.
    is_external_cut: bool,
    /// `powl2:hasRegion` target, present only when `is_external_cut`.
    region_id: Option<String>,
    /// `powl2:sparqlProjection` literal (Q), present only when
    /// `is_external_cut`.
    sparql_projection: Option<String>,
    /// `powl2:teraRenderer` literal (T), present only when
    /// `is_external_cut`.
    tera_renderer: Option<String>,
}

/// Tera context row for one manufactured Arazzo step. `Serialize`d directly
/// into the [`ARAZZO_PROJECTION_TEMPLATE`] context under `steps`.
#[derive(Debug, Clone, Serialize)]
struct StepView {
    step_id: String,
    description: String,
    operation_id: String,
    is_external_cut: bool,
    region_id: Option<String>,
    sparql_projection: Option<String>,
    tera_renderer: Option<String>,
}

/// Resolves the ordered sequence of [`StepProjection`]s reachable from
/// `element_id` by walking `rows`' `childIndex`/`childModel` bindings
/// depth-first in ascending index order (a container's children), bottoming
/// out at a leaf, silent leaf, or external-cut row (a node with no bound
/// `childIndex`/`childModel` of its own).
///
/// An external cut's own region is *not* recursed into: the region is, by
/// definition of Rail A's external-cut boundary (PRD.md sec.7.4), opaque to
/// the projecting workflow -- it is dispatched to, not enumerated as this
/// workflow's own steps. That the region's rows are present in the wider
/// `rows` slice (any admitted subtree round-trips through
/// `powl_to_turtle`/`run_render_model_projection`) does not make them steps
/// of *this* manufactured document.
///
/// # Complexity
/// O(n * d) where n = `rows.len()` and d = the tree depth rooted at
/// `element_id`: each call does one O(n) linear scan of `rows` (no
/// `HashMap`, no incidental ordering reliance -- children are sorted
/// explicitly by their parsed numeric index) and recurses once per child.
/// Acceptable for the admitted-region sizes this pipeline handles (bounded
/// PDDL plan tapes, same bound documented on
/// `project_pddl_tape_to_powl`/`Pddl8Tape`); not a hidden quadratic on
/// unbounded input.
///
/// # Errors
/// [`CoreError::UnresolvedProjectionElement`] if `element_id` (or a
/// `childModel` it references) has no row at all in `rows`, or if a bound
/// `childIndex` literal is not a valid non-negative integer -- both
/// indicate `rows` did not actually come from a well-formed
/// `run_render_model_projection` run over `element_id`'s own admitted
/// region.
fn flatten_ordered_steps(
    rows: &[ProjectionRow],
    element_id: &str,
) -> Result<Vec<StepProjection>, CoreError> {
    let own_rows: Vec<&ProjectionRow> =
        rows.iter().filter(|r| r.element_id == element_id).collect();
    if own_rows.is_empty() {
        return Err(CoreError::UnresolvedProjectionElement(
            element_id.to_string(),
        ));
    }

    // Ordered children (containers only): rows binding childIndex+childModel
    // for this element_id, keyed by the *parsed* numeric index in a
    // `BTreeMap` -- sorted deterministically by key (never a `HashMap`), and
    // deduplicating the exact-duplicate (childIndex, childModel) rows a
    // multi-`rdf:type` subject produces. `emit_powl_node` gives the model
    // root two `rdf:type`s (`powl2:Model` and its own container type, e.g.
    // `powl2:PartialOrder`), and `?elementId a ?elementType` yields one row
    // per type per SPARQL BGP semantics -- so a 2-child root's `hasChild`
    // OPTIONAL is cross-joined into 4 raw rows (2 types x 2 children), not
    // 2. A genuine inconsistency (the same index bound to two *different*
    // models) is refused rather than silently resolved by picking one.
    let mut children: std::collections::BTreeMap<u64, String> = std::collections::BTreeMap::new();
    for row in &own_rows {
        if let (Some(idx_lexical), Some(model)) = (&row.child_index, &row.child_model) {
            let idx: u64 = idx_lexical.parse().map_err(|_| {
                CoreError::UnresolvedProjectionElement(format!(
                    "non-numeric childIndex {idx_lexical:?} on element {element_id}"
                ))
            })?;
            if let Some(existing) = children.get(&idx) {
                if existing != model {
                    return Err(CoreError::UnresolvedProjectionElement(format!(
                        "childIndex {idx} on element {element_id} bound to two \
                         different childModel values: {existing:?} and {model:?}"
                    )));
                }
            } else {
                children.insert(idx, model.clone());
            }
        }
    }

    if !children.is_empty() {
        let mut out = Vec::new();
        for (_, child_id) in children {
            out.extend(flatten_ordered_steps(rows, &child_id)?);
        }
        return Ok(out);
    }

    if let Some(cut_row) = own_rows
        .iter()
        .find(|r| r.element_type.ends_with("ExternalCut"))
    {
        return Ok(vec![StepProjection {
            element_id: element_id.to_string(),
            label: format!("external_cut:{element_id}"),
            is_external_cut: true,
            region_id: cut_row.region_id.clone(),
            sparql_projection: cut_row.sparql_projection.clone(),
            tera_renderer: cut_row.tera_renderer.clone(),
        }]);
    }

    if let Some(leaf_row) = own_rows.iter().find(|r| r.activity_label.is_some()) {
        return Ok(vec![StepProjection {
            element_id: element_id.to_string(),
            label: leaf_row
                .activity_label
                .clone()
                .expect("filtered on activity_label.is_some() immediately above"),
            is_external_cut: false,
            region_id: None,
            sparql_projection: None,
            tera_renderer: None,
        }]);
    }

    // No children, no ExternalCut row, no activityLabel: a silent leaf
    // (powl2:Leaf, powl2:SilentLeaf -- Powl::Leaf(None)).
    Ok(vec![StepProjection {
        element_id: element_id.to_string(),
        label: "silent".to_string(),
        is_external_cut: false,
        region_id: None,
        sparql_projection: None,
        tera_renderer: None,
    }])
}

/// The T stage of Rail A's `A_z = T(Q(W))` (PRD.md sec.7.4): renders
/// PROJ-751's real Q-stage SPARQL projection rows into a manufactured
/// Arazzo 1.1.x JSON document via [`ARAZZO_PROJECTION_TEMPLATE`].
///
/// `rows` is the output of
/// `praxis_graphlaw::chatman::powl_projection::run_render_model_projection`
/// over an admitted POWL region's Turtle; `root_element_id` is that same
/// region's root subject IRI (the `<base_iri>/<path>` the caller passed to
/// `powl_to_turtle`). The returned `String` is the rendered JSON text --
/// callers that need an `ArazzoDescription` parse it with
/// `wasm4pm_arazzo::parse::DocumentIndex::add_document` (see this module's
/// own round-trip test).
///
/// # Errors
/// [`CoreError::UnresolvedProjectionElement`] (see [`flatten_ordered_steps`])
/// or [`CoreError::TemplateRenderFailed`] if the projection resolves to zero
/// steps, or if Tera itself refuses the render (a crate defect in
/// [`ARAZZO_PROJECTION_TEMPLATE`], since every context value passed to it is
/// a plain string/bool routed through the `json_encode` filter).
///
/// # Complexity
/// O(n * d) for [`flatten_ordered_steps`] plus O(s) for the Tera render,
/// where s is the rendered document's byte length.
pub fn render_arazzo_document(
    rows: &[ProjectionRow],
    root_element_id: &str,
    workflow_id: &str,
    title: &str,
) -> Result<String, CoreError> {
    let steps = flatten_ordered_steps(rows, root_element_id)?;
    if steps.is_empty() {
        return Err(CoreError::TemplateRenderFailed(format!(
            "projection rooted at {root_element_id} yielded zero Arazzo steps"
        )));
    }

    let step_views: Vec<StepView> = steps
        .into_iter()
        .enumerate()
        .map(|(i, s)| StepView {
            step_id: format!("step_{i:03}"),
            description: s.label,
            operation_id: s.element_id,
            is_external_cut: s.is_external_cut,
            region_id: s.region_id,
            sparql_projection: s.sparql_projection,
            tera_renderer: s.tera_renderer,
        })
        .collect();

    let mut context = tera::Context::new();
    context.insert("title", title);
    context.insert("workflow_id", workflow_id);
    context.insert("source_url", &format!("{root_element_id}#source"));
    context.insert("steps", &step_views);

    tera::Tera::one_off(ARAZZO_PROJECTION_TEMPLATE, &context, false)
        .map_err(|e| CoreError::TemplateRenderFailed(format!("{e:?}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arazzo_projection_receipt_digest_is_deterministic() {
        let receipt = ArazzoProjectionReceipt {
            source_powl_digest_hex: "00000000000000000000000000000001".to_string(),
            external_cut_identity: "cut-123".to_string(),
            sparql_projection_digest_hex: "00000000000000000000000000000002".to_string(),
            tera_template_digest_hex: "00000000000000000000000000000003".to_string(),
            arazzo_digest_hex: "00000000000000000000000000000004".to_string(),
            compiler_version: "v26.7.11".to_string(),
            air_digest_hex: "00000000000000000000000000000005".to_string(),
        };

        let digest1 = receipt.compute_digest().unwrap();
        let digest2 = receipt.compute_digest().unwrap();
        assert_eq!(digest1, digest2);
    }

    // ── PROJ-752: real T-stage render + full round-trip ─────────────────

    use powl2_decompose::Powl;
    use praxis_graphlaw::chatman::powl_projection::{
        powl_to_turtle, run_render_model_projection, RENDER_MODEL_PROJECTION_QUERY,
    };
    use wasm4pm_arazzo::parse::DocumentIndex;

    const TEST_PROJECTION: &str = "SELECT * WHERE { ?s ?p ?o }";
    const TEST_RENDERER: &str = "arazzo_projection.tera";

    /// Mirrors `powl_projection::tests::model_with_external_cut` (PROJ-751):
    /// a two-step `PartialOrder` whose second child is an `ExternalCut`, so
    /// this test exercises both step kinds [`render_arazzo_document`] must
    /// handle, not just the trivial all-leaves case.
    fn model_with_external_cut() -> Powl {
        Powl::PartialOrder {
            children: vec![
                Powl::Leaf(Some("intake".to_string())),
                Powl::ExternalCut {
                    region: Box::new(Powl::Leaf(Some("remote_settle".to_string()))),
                    projection: TEST_PROJECTION.to_string(),
                    renderer: TEST_RENDERER.to_string(),
                },
            ],
            order: std::collections::BTreeSet::from([(0usize, 1usize)]),
        }
    }

    /// The full Rail A round trip this ticket requires: an admitted POWL
    /// region (W) -> real SPARQL execution (Q, PROJ-751) -> real Tera
    /// render (T, this ticket) -> a manufactured Arazzo JSON document that
    /// actually parses via wasm4pm-arazzo's own `DocumentIndex`, not a
    /// hand-typed fixture standing in for one.
    #[test]
    fn manufactured_arazzo_round_trips_through_wasm4pm_arazzo_parser(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let model = model_with_external_cut();
        let base_iri = "urn:test:proj752";
        let turtle = powl_to_turtle(&model, base_iri, None)?;

        let rows = run_render_model_projection(&turtle)?;
        assert!(
            !rows.is_empty(),
            "PROJ-751's real SPARQL execution must yield rows over this fixture"
        );

        let root_element_id = format!("{base_iri}/n0");
        let arazzo_json = render_arazzo_document(
            &rows,
            &root_element_id,
            "manufactured-rail-a-workflow",
            "Rail A manufactured workflow (PROJ-752)",
        )?;

        // The rendered text must be valid JSON on its own terms (not merely
        // "parses as an ArazzoDescription", which would hide a JSON-shape
        // bug behind serde's leniency).
        let _: serde_json::Value = serde_json::from_str(&arazzo_json)?;

        // Real parse via wasm4pm-arazzo's own admission path -- proves this
        // is a real Arazzo 1.1.x document, not merely JSON shaped like one.
        let mut index = DocumentIndex::new();
        index.add_document(&arazzo_json, "urn:test:proj752/manufactured")?;
        assert_eq!(index.documents.len(), 1);
        let parsed = index
            .documents
            .get("urn:test:proj752/manufactured")
            .expect("just inserted under this exact base_uri");
        assert_eq!(parsed.arazzo, "1.1.0");
        assert_eq!(parsed.workflows.len(), 1);
        // Both the plain leaf ("intake") and the external-cut boundary
        // step round-tripped -- not just whichever one happens to serialize
        // without a template bug.
        assert_eq!(parsed.workflows[0].steps.len(), 2);
        assert!(parsed.workflows[0]
            .steps
            .iter()
            .any(|s| s.description.as_deref() == Some("intake")));
        assert!(parsed.workflows[0]
            .steps
            .iter()
            .any(|s| s.extensions.get("x-powl-external-cut").is_some()));

        // PROJ-753: resolve URIs, lower the parsed document into AIR, normalize
        // (resolve any cross-step Variable references), and compile to WASM --
        // completing the Rail A/B seam that this test previously stopped short
        // of (`air_digest_hex` used to be a hand-typed "unavailable:PROJ-753"
        // sentinel because no lowering function existed anywhere in the repo).
        wasm4pm_arazzo::resolve::normalize_uris(&mut index)?;
        let parsed = index
            .documents
            .get("urn:test:proj752/manufactured")
            .expect("still inserted under this exact base_uri after URI normalization");

        let bump = bumpalo::Bump::new();
        let mut air_program = wasm4pm_arazzo::lower::lower_description(parsed, &bump)?;
        wasm4pm_arazzo::normalizer::ArazzoNormalizer::normalize(&mut air_program, &bump)?;
        let air_wasm = wasm4pm_arazzo::compile::AirCompiler::compile_to_wasm(&air_program)?;
        let air_digest = wasm4pm_arazzo::compile::AirCompiler::digest_program(&air_program)?;

        // The compiled AIR module must carry the real POWL element IDs this
        // projection produced (the manufactured document's own operationId
        // values -- "intake"'s leaf and the external cut's boundary node), not
        // a hand-typed fixture string.
        let intake_operation_id = format!("{root_element_id}/c0");
        let external_cut_operation_id = format!("{root_element_id}/c1");
        assert!(
            contains_subslice(&air_wasm, intake_operation_id.as_bytes()),
            "compiled AIR module must contain the real intake step's operationId"
        );
        assert!(
            contains_subslice(&air_wasm, external_cut_operation_id.as_bytes()),
            "compiled AIR module must contain the real external-cut step's operationId"
        );

        // Receipt: every digest field is now a real BLAKE3 digest of real
        // materials this test just produced -- air_digest_hex included, no
        // placeholder strings anywhere.
        let receipt = ArazzoProjectionReceipt::from_materials(
            &turtle,
            &root_element_id,
            RENDER_MODEL_PROJECTION_QUERY,
            ARAZZO_PROJECTION_TEMPLATE,
            &arazzo_json,
            "26.7.11",
            &hex::encode(air_digest.0),
        );
        assert_eq!(
            receipt.source_powl_digest_hex,
            hex::encode(blake3::hash(turtle.as_bytes()).as_bytes())
        );
        assert_eq!(
            receipt.sparql_projection_digest_hex,
            hex::encode(blake3::hash(RENDER_MODEL_PROJECTION_QUERY.as_bytes()).as_bytes())
        );
        assert_eq!(
            receipt.tera_template_digest_hex,
            hex::encode(blake3::hash(ARAZZO_PROJECTION_TEMPLATE.as_bytes()).as_bytes())
        );
        assert_eq!(
            receipt.arazzo_digest_hex,
            hex::encode(blake3::hash(arazzo_json.as_bytes()).as_bytes())
        );
        assert_eq!(receipt.air_digest_hex, hex::encode(air_digest.0));
        let digest1 = receipt.compute_digest()?;
        let digest2 = receipt.compute_digest()?;
        assert_eq!(digest1, digest2, "receipt digest must be deterministic");

        Ok(())
    }

    fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
        !needle.is_empty() && haystack.windows(needle.len()).any(|w| w == needle)
    }

    #[test]
    fn render_arazzo_document_refuses_unresolved_root() {
        let result = render_arazzo_document(&[], "urn:test:proj752-negative/n0", "wf", "title");
        assert!(matches!(
            result,
            Err(CoreError::UnresolvedProjectionElement(_))
        ));
    }
}
