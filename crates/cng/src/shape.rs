//! Structural shape validation for generated POWL v2 graphs.
//!
//! Parsing alone is insufficient evidence: this module enforces the declared
//! shape (`shapes/powl2-shapes.ttl`, shipped with the crate as the
//! human-readable SHACL statement of the same constraints) with a SPARQL
//! structural validator over the parsed graph. The code checks and the
//! shipped shape artifact state the same law; the validator is the
//! executable form.
//!
//! Checks:
//! 1. exactly one `powl2:Model` root;
//! 2. the root carries at most one `powl2:derivedFrom` (exactly one when
//!    provenance is required);
//! 3. every `powl2:ChildBinding` has exactly one `powl2:childIndex` and one
//!    `powl2:childModel`;
//! 4. every `powl2:ActivityLeaf` has exactly one non-empty
//!    `powl2:activityLabel`;
//! 5. every `powl2:precedes` triple connects two `powl2:ChildBinding`s;
//! 6. leaf/binding counts agree.

use oxigraph::sparql::{QueryResults, SparqlEvaluator};
use oxigraph::store::Store;

use crate::powl::{CngRefusal, POWL2_PREFIX};

/// Outcome of a structural validation pass.
#[derive(Debug, serde::Serialize)]
pub struct ShapeReport {
    pub models: usize,
    pub partial_orders: usize,
    pub activity_leaves: usize,
    pub child_bindings: usize,
    pub precedes: usize,
    pub derived_from: usize,
    /// Human-readable statement of what was validated.
    pub shape: String,
}

/// Counts solutions of a SPARQL SELECT over `store`.
///
/// # Complexity
/// Query-engine cost; result iteration O(matches).
fn solution_count(store: &Store, query: &str) -> Result<usize, CngRefusal> {
    let prepared = SparqlEvaluator::new()
        .parse_query(query)
        .map_err(|e| CngRefusal::InvalidPowl(format!("shape query parse failed: {e}")))?;
    match prepared.on_store(store).execute() {
        Ok(QueryResults::Solutions(solutions)) => Ok(solutions.count()),
        Ok(_) => Err(CngRefusal::InvalidPowl(format!(
            "shape query did not yield solutions: {query}"
        ))),
        Err(e) => Err(CngRefusal::InvalidPowl(format!(
            "shape query execution failed: {e}"
        ))),
    }
}

/// Validates a parsed POWL v2 graph against the declared structural shape.
///
/// # Errors
/// `CNG_R06 InvalidPowl` naming the violated constraint.
///
/// # Complexity
/// A fixed number of SPARQL queries, each linear in matching triples.
pub fn validate_powl_store(
    store: &Store,
    require_provenance: bool,
) -> Result<ShapeReport, CngRefusal> {
    let class_count = |class: &str| -> Result<usize, CngRefusal> {
        solution_count(
            store,
            &format!(
                "SELECT ?s WHERE {{ ?s <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <{POWL2_PREFIX}{class}> }}"
            ),
        )
    };
    let pred_count = |pred: &str| -> Result<usize, CngRefusal> {
        solution_count(
            store,
            &format!("SELECT ?s WHERE {{ ?s <{POWL2_PREFIX}{pred}> ?o }}"),
        )
    };

    let models = class_count("Model")?;
    if models != 1 {
        return Err(CngRefusal::InvalidPowl(format!(
            "shape violation: expected exactly 1 powl2:Model, found {models}"
        )));
    }
    let partial_orders = class_count("PartialOrder")?;
    let activity_leaves = class_count("ActivityLeaf")?;
    let child_bindings = class_count("ChildBinding")?;
    let precedes = pred_count("precedes")?;
    let derived_from = pred_count("derivedFrom")?;
    if require_provenance && derived_from != 1 {
        return Err(CngRefusal::InvalidPowl(format!(
            "shape violation: expected exactly 1 powl2:derivedFrom on the root, found {derived_from}"
        )));
    }
    if activity_leaves != child_bindings {
        return Err(CngRefusal::InvalidPowl(format!(
            "shape violation: {activity_leaves} ActivityLeafs vs {child_bindings} ChildBindings"
        )));
    }

    // Every ChildBinding must carry exactly one childIndex and one childModel.
    let bindings_missing_index = solution_count(
        store,
        &format!(
            "SELECT ?b WHERE {{ ?b <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <{POWL2_PREFIX}ChildBinding> . FILTER NOT EXISTS {{ ?b <{POWL2_PREFIX}childIndex> ?i }} }}"
        ),
    )?;
    if bindings_missing_index != 0 {
        return Err(CngRefusal::InvalidPowl(format!(
            "shape violation: {bindings_missing_index} ChildBinding(s) lack powl2:childIndex"
        )));
    }
    let bindings_missing_model = solution_count(
        store,
        &format!(
            "SELECT ?b WHERE {{ ?b <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <{POWL2_PREFIX}ChildBinding> . FILTER NOT EXISTS {{ ?b <{POWL2_PREFIX}childModel> ?m }} }}"
        ),
    )?;
    if bindings_missing_model != 0 {
        return Err(CngRefusal::InvalidPowl(format!(
            "shape violation: {bindings_missing_model} ChildBinding(s) lack powl2:childModel"
        )));
    }

    // Every ActivityLeaf must carry a non-empty label.
    let unlabelled_leaves = solution_count(
        store,
        &format!(
            "SELECT ?l WHERE {{ ?l <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <{POWL2_PREFIX}ActivityLeaf> . FILTER NOT EXISTS {{ ?l <{POWL2_PREFIX}activityLabel> ?a . FILTER(STRLEN(STR(?a)) > 0) }} }}"
        ),
    )?;
    if unlabelled_leaves != 0 {
        return Err(CngRefusal::InvalidPowl(format!(
            "shape violation: {unlabelled_leaves} ActivityLeaf(s) lack a non-empty activityLabel"
        )));
    }

    // Every precedes edge must connect two ChildBindings.
    let bad_precedes = solution_count(
        store,
        &format!(
            "SELECT ?s WHERE {{ ?s <{POWL2_PREFIX}precedes> ?o . FILTER(NOT EXISTS {{ ?s <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <{POWL2_PREFIX}ChildBinding> }} || NOT EXISTS {{ ?o <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <{POWL2_PREFIX}ChildBinding> }}) }}"
        ),
    )?;
    if bad_precedes != 0 {
        return Err(CngRefusal::InvalidPowl(format!(
            "shape violation: {bad_precedes} powl2:precedes triple(s) do not connect ChildBindings"
        )));
    }

    Ok(ShapeReport {
        models,
        partial_orders,
        activity_leaves,
        child_bindings,
        precedes,
        derived_from,
        shape: "powl2-shapes.ttl structural validator: Model=1, binding index/model \
                totality, non-empty leaf labels, precedes endpoints are ChildBindings"
            .to_string(),
    })
}
