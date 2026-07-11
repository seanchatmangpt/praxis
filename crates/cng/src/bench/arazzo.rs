//! Arazzo dialect projection (PROJ-621): admit an Arazzo-as-RDF workflow
//! description (80/20 profile, `ontologies/arazzo.ttl`), validate it against
//! `shapes/arazzo-shapes.ttl` (shape-driven SPARQL, same generic queries as
//! the dispatch/registry gates), and project each `arz:Step` — in
//! `arz:stepIndex` order with `arz:dependsOn` enforced — into a
//! `DispatchContract` executed `EXTERNAL_MACHINE_DISPATCH` through the
//! broker loopback adapter. Arazzo is an ORCHESTRATION SURFACE; POWL stays
//! the canonical workflow model.
//!
//! Profile refusals (`CNG_R18 ArazzoProfileRefused`, naming the feature):
//! - closed-shape violations (undeclared predicates on targeted nodes);
//! - `arz:criterionType` outside {simple, regex, jsonpath} (e.g. `xpath`);
//! - `arz:actionType` outside {end, goto} on success actions or
//!   {end, goto, retry} on failure actions (e.g. `function`);
//! - steps whose operation target is `arz:operationPath`/`arz:workflowRef`
//!   instead of `arz:operationId` (only operationId projects to a dispatch
//!   target in this profile increment).
//!
//! Projection mapping (declarative-only where noted):
//! `operationId` → contract `activityIdentity`/target actor surface;
//! `hasParameter` names → input artifact set label; `successCriterion`
//! condition → semantic-conformance expectation (recorded in
//! `refusalConditions`); `onFailure` retry → `retryLaw` with `retryLimit`
//! (`retryAfter` is DECLARATIVE ONLY — logical ticks, never seconds).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::model::{NamedNodeRef, Term, TermRef};
use oxigraph::store::Store;

use crate::powl::CngRefusal;

use super::dispatch::{
    shape_violations, DispatchAdapter, DispatchContract, DispatchOutcome, SynthesisMode,
};
use super::manufacture::term_value;
use super::roles::ObsWriter;
use super::templates::QuerySet;

/// The arz: vocabulary prefix (ontologies/arazzo.ttl).
const ARZ_PREFIX: &str = "https://truex.io/ontology/arazzo#";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

/// One admitted Arazzo step, projected from the RDF description.
#[derive(Debug)]
pub(super) struct ArazzoStep {
    pub(super) step_id: String,
    pub(super) step_index: u64,
    pub(super) operation_id: String,
    /// stepIds this step depends on (must be admitted first).
    pub(super) depends_on: Vec<String>,
    /// Declared retry law text from the onFailure retry action, if any
    /// (declarative only; ticks, never seconds).
    pub(super) retry_law: String,
    /// The success criterion condition (semantic-conformance expectation).
    pub(super) success_condition: String,
    /// Parameter names (input artifact set label material).
    pub(super) parameters: Vec<String>,
}

/// `NamedNodeRef` for `ARZ_PREFIX + local` (or an absolute IRI).
fn iri(full: &str) -> Result<NamedNodeRef<'_>, CngRefusal> {
    NamedNodeRef::new(full).map_err(|e| CngRefusal::MalformedTtl(format!("{full}: {e}")))
}

/// All object values of `(subject, <pred>, ?o)`, in store order.
///
/// # Complexity
/// O(matches) pattern scan.
fn objects_of(store: &Store, subject: &Term, pred_iri: &str) -> Result<Vec<Term>, CngRefusal> {
    let pred = iri(pred_iri)?;
    let subject_ref = match subject {
        Term::NamedNode(n) => n.as_ref(),
        other => {
            return Err(CngRefusal::MalformedTtl(format!(
                "arazzo node {other} is not an IRI"
            )))
        }
    };
    let mut out = Vec::new();
    for quad in store.quads_for_pattern(Some(subject_ref.into()), Some(pred), None, None) {
        let quad = quad.map_err(|e| CngRefusal::MalformedTtl(format!("arazzo scan: {e}")))?;
        out.push(quad.object);
    }
    Ok(out)
}

/// First object value of `(subject, <pred>, ?o)` as a plain string.
fn object_value(
    store: &Store,
    subject: &Term,
    pred_iri: &str,
) -> Result<Option<String>, CngRefusal> {
    Ok(objects_of(store, subject, pred_iri)?
        .first()
        .map(term_value))
}

/// Every subject typed `<class_iri>`, sorted by IRI (deterministic order).
///
/// # Complexity
/// O(instances) scan + O(n log n) sort.
fn subjects_of_type(store: &Store, class_iri: &str) -> Result<Vec<Term>, CngRefusal> {
    let type_pred = iri(RDF_TYPE)?;
    let class = iri(class_iri)?;
    let mut out: Vec<Term> = Vec::new();
    for quad in store.quads_for_pattern(None, Some(type_pred), Some(TermRef::from(class)), None) {
        let quad = quad.map_err(|e| CngRefusal::MalformedTtl(format!("arazzo type scan: {e}")))?;
        out.push(quad.subject.into());
    }
    out.sort_by_key(|t| t.to_string());
    Ok(out)
}

/// Admits and validates an Arazzo description: parse, closed-shape law
/// (shape-driven SPARQL over `shapes/arazzo-shapes.ttl`), then the named
/// profile checks (criterion/action types). Returns the admitted store.
///
/// # Errors
/// `CNG_R18 ArazzoProfileRefused` naming the refused feature; `CNG_R01/R10`
/// for unreadable/unparseable inputs.
///
/// # Complexity
/// O(description triples) load + two shape SELECTs + O(criteria + actions)
/// pattern scans.
pub(super) fn admit_arazzo(
    description_path: &Path,
    queries: &QuerySet,
) -> Result<Store, CngRefusal> {
    let ttl = fs::read_to_string(description_path)
        .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", description_path.display())))?;
    let shapes_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("shapes")
        .join("arazzo-shapes.ttl");
    if let Some((entry, field)) = shape_violations(&ttl, &shapes_path, queries)?.first() {
        return Err(CngRefusal::ArazzoProfileRefused {
            feature: format!("undeclared predicate {field} on {entry}"),
        });
    }
    let store = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("arazzo store construction: {e}")))?;
    store
        .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), ttl.as_bytes())
        .map_err(|e| CngRefusal::MalformedTtl(format!("arazzo load: {e}")))?;

    // Named profile checks (the sh:in laws of arazzo-shapes.ttl, enforced
    // here so the refusal names the feature).
    //
    // # Complexity
    // O(criteria + actions) pattern scans.
    for criterion in subjects_of_type(&store, &format!("{ARZ_PREFIX}Criterion"))? {
        if let Some(kind) = object_value(&store, &criterion, &format!("{ARZ_PREFIX}criterionType"))?
        {
            if !matches!(kind.as_str(), "simple" | "regex" | "jsonpath") {
                return Err(CngRefusal::ArazzoProfileRefused {
                    feature: format!("criterionType={kind}"),
                });
            }
        }
    }
    for (class, allowed) in [
        ("SuccessAction", &["end", "goto"][..]),
        ("FailureAction", &["end", "goto", "retry"][..]),
    ] {
        for action in subjects_of_type(&store, &format!("{ARZ_PREFIX}{class}"))? {
            if let Some(kind) = object_value(&store, &action, &format!("{ARZ_PREFIX}actionType"))? {
                if !allowed.contains(&kind.as_str()) {
                    return Err(CngRefusal::ArazzoProfileRefused {
                        feature: format!("{class} actionType={kind}"),
                    });
                }
            }
        }
    }
    Ok(store)
}

/// Projects the admitted steps, ordered by `(stepIndex, stepId)`.
///
/// # Errors
/// `CNG_R18` for a step without `arz:operationId` (operationPath/
/// workflowRef targets are outside this projection profile).
///
/// # Complexity
/// O(steps × per-step properties) scans + O(s log s) sort.
pub(super) fn project_steps(store: &Store) -> Result<Vec<ArazzoStep>, CngRefusal> {
    // stepIRI → stepId map for dependsOn resolution.
    let step_nodes = subjects_of_type(store, &format!("{ARZ_PREFIX}Step"))?;
    let mut id_of: BTreeMap<String, String> = BTreeMap::new();
    for node in &step_nodes {
        let step_id = object_value(store, node, &format!("{ARZ_PREFIX}stepId"))?
            .ok_or_else(|| CngRefusal::MalformedTtl(format!("step {node} has no stepId")))?;
        id_of.insert(node.to_string(), step_id);
    }
    let mut steps = Vec::with_capacity(step_nodes.len());
    for node in &step_nodes {
        let step_id = id_of.get(&node.to_string()).cloned().unwrap_or_default();
        let operation_id = object_value(store, node, &format!("{ARZ_PREFIX}operationId"))?
            .ok_or_else(|| CngRefusal::ArazzoProfileRefused {
                feature: format!(
                    "step {step_id} has no operationId (operationPath/workflowRef \
                     targets are outside the projection profile)"
                ),
            })?;
        let step_index = object_value(store, node, &format!("{ARZ_PREFIX}stepIndex"))?
            .ok_or_else(|| CngRefusal::MalformedTtl(format!("step {step_id} has no stepIndex")))?
            .parse::<u64>()
            .map_err(|e| CngRefusal::MalformedTtl(format!("stepIndex of {step_id}: {e}")))?;
        let mut depends_on = Vec::new();
        for dep in objects_of(store, node, &format!("{ARZ_PREFIX}dependsOn"))? {
            depends_on.push(id_of.get(&dep.to_string()).cloned().ok_or_else(|| {
                CngRefusal::MalformedTtl(format!("step {step_id} dependsOn unknown step {dep}"))
            })?);
        }
        depends_on.sort();
        // onFailure retry → declared retry law (declarative only; retryAfter
        // is logical ticks, never seconds).
        let mut retry_law = "none".to_string();
        for action in objects_of(store, node, &format!("{ARZ_PREFIX}onFailure"))? {
            if object_value(store, &action, &format!("{ARZ_PREFIX}actionType"))?.as_deref()
                == Some("retry")
            {
                let limit = object_value(store, &action, &format!("{ARZ_PREFIX}retryLimit"))?
                    .unwrap_or_else(|| "0".to_string());
                retry_law = format!("retry:limit={limit};afterTicks=1;declarative-only");
            }
        }
        let success_condition =
            match objects_of(store, node, &format!("{ARZ_PREFIX}successCriterion"))?.first() {
                Some(criterion) => {
                    object_value(store, criterion, &format!("{ARZ_PREFIX}condition"))?
                        .unwrap_or_else(|| "none".to_string())
                }
                None => "none".to_string(),
            };
        let mut parameters = Vec::new();
        for param in objects_of(store, node, &format!("{ARZ_PREFIX}hasParameter"))? {
            if let Some(name) = object_value(store, &param, &format!("{ARZ_PREFIX}name"))? {
                parameters.push(name);
            }
        }
        parameters.sort();
        steps.push(ArazzoStep {
            step_id,
            step_index,
            operation_id,
            depends_on,
            retry_law,
            success_condition,
            parameters,
        });
    }
    steps.sort_by(|a, b| (a.step_index, &a.step_id).cmp(&(b.step_index, &b.step_id)));
    Ok(steps)
}

/// Projects one admitted step into a dispatch contract (see module docs for
/// the field mapping). O(parameters).
fn step_contract(step: &ArazzoStep, set_id: &str, category: &str, tick: usize) -> DispatchContract {
    let base = super::dispatch::workday_contract(
        set_id,
        category,
        tick,
        super::dispatch::ExecutionClass::ExternalMachineDispatch,
    );
    let dispatch_id = format!("arz-{set_id}-{}", step.step_id);
    DispatchContract {
        dispatch_id: dispatch_id.clone(),
        workflow_instance: format!("{set_id}-{}", step.step_id),
        parent_workflow: set_id.to_string(),
        // Arazzo steps are leaf dispatches: no child manufacture.
        recursive_depth: 0,
        closure_law: None,
        activity_identity: step.operation_id.clone(),
        // hasParameter names → the input artifact set label (sorted; empty
        // parameter lists collapse to the bare step label).
        input_artifact_set: if step.parameters.is_empty() {
            format!("params-{}", step.step_id)
        } else {
            format!("params-{}-{}", step.step_id, step.parameters.join("-"))
        },
        expected_output_artifact_set: format!("outputs-{dispatch_id}"),
        retry_law: step.retry_law.clone(),
        // successCriteria → the semantic-conformance expectation the
        // re-entry pipeline enforces (recorded as a refusal condition).
        refusal_conditions: format!("semantic:{}", step.success_condition),
        idempotency_key: format!(
            "idem-{}",
            blake3::hash(format!("idem|{dispatch_id}").as_bytes()).to_hex()[..12].to_string()
        ),
        correlation_id: format!(
            "corr-{}",
            blake3::hash(format!("corr|{dispatch_id}").as_bytes()).to_hex()[..12].to_string()
        ),
        ..base
    }
}

/// Runs the full Arazzo projection for one workday tick: admit + validate
/// the description, project the steps in `(stepIndex, stepId)` order with
/// `dependsOn` enforced (a step dispatches only after every dependency's
/// consequence was admitted), and dispatch each step
/// `EXTERNAL_MACHINE_DISPATCH` through the loopback adapter. Returns the
/// number of admitted step dispatches.
///
/// # Errors
/// `CNG_R18` profile refusals; `CNG_R05` on a dependsOn ordering violation;
/// `CNG_R09` when a loopback step dispatch fails to admit (the deterministic
/// mechanism is broken).
///
/// # Complexity
/// O(steps) sequential dispatch lifecycles (each O(deadline_ticks) polls).
pub(super) fn run_arazzo_projection(
    adapter: &mut DispatchAdapter<'_>,
    writer: &mut ObsWriter<'_>,
    obs_store: &Store,
    description_path: &Path,
    set_id: &str,
    category: &str,
    tick: usize,
) -> Result<usize, CngRefusal> {
    let store = admit_arazzo(description_path, adapter_queries(adapter))?;
    let steps = project_steps(&store)?;
    // PROJ-745: before any step of this projection can reach
    // DispatchState::ArazzoRendered (inside adapter.dispatch(), below),
    // verify that the arazzo-pack's ggen-rendered YAML backing this
    // projection is receipted — recomputed BLAKE3 over the on-disk render
    // matches the ggen sync receipt's recorded digest for that output. The
    // RDF admitted above (`description_path`) is the source of truth for
    // step semantics; this check only confirms the projection artifact a
    // ggen sync run produced from that graph was not silently altered or
    // never rendered. A missing/mismatched render refuses CNG_R11
    // AuditMismatch before any step dispatches.
    verify_arazzo_render_digest(adapter.project_root())?;
    let mut admitted: BTreeSet<String> = BTreeSet::new();
    for step in &steps {
        for dep in &step.depends_on {
            if !admitted.contains(dep) {
                return Err(CngRefusal::UnsupportedConstruct(format!(
                    "arazzo step {} depends on {dep}, which has not been admitted \
                     yet; dependsOn ordering violated",
                    step.step_id
                )));
            }
        }
        let contract = step_contract(step, set_id, category, tick);
        let outcome = adapter.dispatch(
            writer,
            obs_store,
            contract,
            tick,
            true,
            SynthesisMode::LoopbackDeterministic,
            1,
        )?;
        if outcome != DispatchOutcome::Admitted {
            return Err(CngRefusal::HardcodingSuspicion(format!(
                "loopback arazzo step {} did not admit ({outcome:?}); the \
                 deterministic loopback mechanism is broken",
                step.step_id
            )));
        }
        admitted.insert(step.step_id.clone());
    }
    Ok(steps.len())
}

/// The adapter's query set (borrow helper). O(1).
fn adapter_queries<'a>(adapter: &DispatchAdapter<'a>) -> &'a QuerySet {
    adapter.queries()
}

/// Default admitted Arazzo description path:
/// `<CARGO_MANIFEST_DIR>/examples/arazzo-api-orchestration.ttl`. O(1).
pub(super) fn default_description_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("arazzo-api-orchestration.ttl")
}

/// Relative path (from a ggen project root) of the arazzo-pack's rendered
/// Arazzo YAML — the `to:` target of
/// `packs/arazzo-pack/templates/arazzo.yaml.tmpl`.
const ARAZZO_RENDERED_YAML_REL_PATH: &str = "generated/arazzo.yaml";

/// One verified ggen-pack render: the output path checked and the BLAKE3
/// digest that was recomputed and confirmed to match the ggen sync receipt.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(super) struct ArazzoRenderVerification {
    /// Rendered output path, relative to the ggen project root.
    pub(super) output_path: String,
    /// Recomputed BLAKE3 hex digest of the rendered file's bytes.
    pub(super) digest: String,
}

/// Minimal shape of a ggen `.ggen-v2/receipt.json` document — only the
/// `payload.outputs` map this seam reads. `cng` does not depend on the
/// `ggen` crate; re-deriving its full `ReceiptPayload` schema here for one
/// field lookup would be scope this seam does not need.
#[derive(Debug, serde::Deserialize)]
struct GgenReceiptDocument {
    payload: GgenReceiptPayload,
}

/// See [`GgenReceiptDocument`].
#[derive(Debug, serde::Deserialize)]
struct GgenReceiptPayload {
    outputs: BTreeMap<String, String>,
}

/// Verifies the arazzo-pack's rendered Arazzo YAML (`generated/arazzo.yaml`)
/// against the ggen sync receipt's recorded digest for that output
/// (`.ggen-v2/receipt.json`, `payload.outputs["generated/arazzo.yaml"]` —
/// see `packs/arazzo-pack/README.md`'s "Downstream verification seam").
/// Recomputes BLAKE3 over the on-disk file's bytes; never re-admits or
/// re-parses the YAML as truth. This is a byte-digest integrity check only
/// — the caller performs the actual `DispatchState::ArazzoRendered`
/// (`bench::dispatch`) transition once this returns `Ok`.
///
/// # Errors
/// `CNG_R11 AuditMismatch` when: the rendered file is missing/unreadable,
/// the receipt is missing/unreadable/unparseable, the receipt has no entry
/// for `generated/arazzo.yaml`, or the recomputed digest disagrees with the
/// recorded one.
///
/// # Complexity
/// O(rendered file bytes) BLAKE3 hash + O(receipt bytes) JSON parse.
///
/// Wired call site: `run_arazzo_projection` (PROJ-745), before any step of
/// the projection reaches `DispatchState::ArazzoRendered`.
pub(super) fn verify_arazzo_render_digest(
    project_root: &Path,
) -> Result<ArazzoRenderVerification, CngRefusal> {
    let output_path = project_root.join(ARAZZO_RENDERED_YAML_REL_PATH);
    let bytes = fs::read(&output_path).map_err(|e| {
        CngRefusal::AuditMismatch(format!(
            "arazzo render not auditable — cannot read {}: {e}",
            output_path.display()
        ))
    })?;
    let recomputed = blake3::hash(&bytes).to_hex().to_string();

    let receipt_path = project_root.join(".ggen-v2").join("receipt.json");
    let receipt_text = fs::read_to_string(&receipt_path).map_err(|e| {
        CngRefusal::AuditMismatch(format!(
            "arazzo render not auditable — cannot read ggen receipt {}: {e}",
            receipt_path.display()
        ))
    })?;
    let receipt: GgenReceiptDocument = serde_json::from_str(&receipt_text).map_err(|e| {
        CngRefusal::AuditMismatch(format!(
            "arazzo render not auditable — cannot parse ggen receipt {}: {e}",
            receipt_path.display()
        ))
    })?;
    let recorded = receipt
        .payload
        .outputs
        .get(ARAZZO_RENDERED_YAML_REL_PATH)
        .ok_or_else(|| {
            CngRefusal::AuditMismatch(format!(
                "ggen receipt {} has no digest recorded for {ARAZZO_RENDERED_YAML_REL_PATH}",
                receipt_path.display()
            ))
        })?;
    if &recomputed != recorded {
        return Err(CngRefusal::AuditMismatch(format!(
            "arazzo render digest mismatch for {ARAZZO_RENDERED_YAML_REL_PATH} — \
             recomputed {recomputed} vs receipt {recorded}"
        )));
    }

    Ok(ArazzoRenderVerification {
        output_path: ARAZZO_RENDERED_YAML_REL_PATH.to_string(),
        digest: recomputed,
    })
}

#[cfg(test)]
#[path = "arazzo_test.rs"]
mod arazzo_test;
