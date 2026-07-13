//! Per-set manufacture through the real cng chain (import → classify →
//! role-infer → plan → project → validate → conformance → receipt), and the
//! graph-read classification helpers it depends on.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::model::{NamedNodeRef, Term};
use oxigraph::store::Store;

use crate::pipeline::{generate_plan, hierarchical_projection, import_artifacts, plan_id};
use crate::powl::{powl_to_turtle_with_phase_provenance, CngRefusal};
use crate::runner;
use crate::shape;

use super::roles::infer_lawful_next_action;
use super::RWAI_PREFIX;

#[derive(Default)]
pub(super) struct SetOutcome {
    pub(super) stage_ns: Vec<(&'static str, u64)>,
    pub(super) total_ns: u64,
    pub(super) transitions: usize,
    pub(super) powl_digest: String,
    pub(super) powl_bytes: u64,
    pub(super) refusal_code: Option<&'static str>,
    /// Triples in the classified artifact's parsed graph (store.len() —
    /// a real count of admitted graph state, never incremented by lookups).
    pub(super) graph_triples: usize,
    /// Number of graph classification lookups executed for this set.
    pub(super) classification_lookups: usize,
    /// Graph-derived category; consumed by role inference AND workflow
    /// selection (never decorative).
    pub(super) category: Option<String>,
    /// Graph-derived worker IRI (from the artifact's ex:worker triple).
    pub(super) worker_iri: Option<String>,
    /// Derived standing role (Mycin terminal conclusion premise).
    pub(super) inferred_role: Option<String>,
    /// Activity labels of the executed plan ops, in order (OCEL events).
    pub(super) activity_labels: Vec<String>,
    /// Plan id of the manufactured workflow.
    pub(super) plan_id: Option<String>,
    /// Tape length (planned ops).
    pub(super) tape_ops: usize,
}

/// One benchmark run: the artifact-set directory, its recursion depth,
/// the parent run IRI (recursive attachment), the socket attachments
/// derived from the admitted graph, and the outcome.
pub(super) struct RunRecord {
    pub(super) dir: PathBuf,
    /// (parent activity IRI, child workflow IRI) rows from
    /// attachments-with-parent.rq over this node's observation fragment.
    pub(super) attachments: Vec<(String, String)>,
    pub(super) outcome: SetOutcome,
}

/// Deterministic run IRI: content-addressed over the set directory path.
pub(super) fn run_iri(dir: &Path) -> String {
    let digest = blake3::hash(dir.display().to_string().as_bytes()).to_hex();
    format!("{RWAI_PREFIX}run-{}", &digest[..16])
}

/// Manufactures ONE artifact set through the real cng chain, returning
/// per-stage timings and the receipt digest. `export_dir`, when set,
/// receives the generated POWL artifact (storage is measured).
pub(super) fn manufacture_set(set_dir: &Path, export_dir: Option<&Path>) -> SetOutcome {
    let mut out = SetOutcome::default();
    let t_total = Instant::now();

    // Stage: import/admission (real oxigraph Turtle parse per artifact).
    let t = Instant::now();
    let artifacts = match import_artifacts(set_dir) {
        Ok(a) => a,
        Err(refusal) => {
            out.refusal_code = Some(refusal_code_static(&refusal));
            out.total_ns = t_total.elapsed().as_nanos() as u64;
            return out;
        }
    };
    out.stage_ns.push(("import", t.elapsed().as_nanos() as u64));

    // Stage: classification — a graph read (oxigraph pattern scan) over the
    // first admitted artifact's graph. The category is read from the graph,
    // not from Rust.
    let t = Instant::now();
    let (category, worker, parsed_triples) = match artifacts.first() {
        None => {
            out.refusal_code = Some("CNG_R01");
            out.total_ns = t_total.elapsed().as_nanos() as u64;
            return out;
        }
        Some(artifact) => match classify_artifact(&artifact.path) {
            Ok(Some(result)) => result,
            // Syntactically valid Turtle missing ex:category/ex:worker: not
            // an I/O or parse fault, but still nothing to classify against.
            Ok(None) => {
                out.refusal_code = Some("CNG_R01");
                out.total_ns = t_total.elapsed().as_nanos() as u64;
                return out;
            }
            Err(refusal) => {
                out.refusal_code = Some(refusal_code_static(&refusal));
                out.total_ns = t_total.elapsed().as_nanos() as u64;
                return out;
            }
        },
    };
    out.graph_triples += parsed_triples;
    out.classification_lookups += 1;
    out.category = Some(category.clone());
    out.worker_iri = Some(worker);
    out.stage_ns
        .push(("classify", t.elapsed().as_nanos() as u64));

    // Stage: role inference — old-AI (wasm4pm-cognition Mycin forward
    // chaining) derives the standing role and lawful next action from the
    // graph-extracted category. No derivation → typed refusal, no fallback.
    let t = Instant::now();
    match infer_lawful_next_action(&category) {
        Some(action) => out.inferred_role = Some(action),
        None => {
            out.refusal_code = Some("CNG_R05");
            out.total_ns = t_total.elapsed().as_nanos() as u64;
            return out;
        }
    }
    out.stage_ns
        .push(("role-infer", t.elapsed().as_nanos() as u64));

    // Stage: plan (merge fragments + bcinr-pddl ground + bounded BFS).
    let t = Instant::now();
    let (tape, surface) = match generate_plan(&artifacts) {
        Ok(v) => v,
        Err(refusal) => {
            out.refusal_code = Some(refusal_code_static(&refusal));
            out.total_ns = t_total.elapsed().as_nanos() as u64;
            return out;
        }
    };
    out.stage_ns.push(("plan", t.elapsed().as_nanos() as u64));

    // Workflow selection: the graph-derived category SELECTS the workflow
    // family; the admitted planning surface must belong to it. A mismatch is
    // a typed refusal — classification causally gates manufacture, it is
    // never decorative.
    if !surface.domain.name.starts_with(&format!("wf-{category}-")) {
        out.refusal_code = Some("CNG_R09");
        out.total_ns = t_total.elapsed().as_nanos() as u64;
        return out;
    }

    // Stage: hierarchical projection + per-phase provenance serialization.
    let t = Instant::now();
    let (model, phase_sources) = match hierarchical_projection(&tape, &surface) {
        Ok(v) => v,
        Err(refusal) => {
            out.refusal_code = Some(refusal_code_static(&refusal));
            out.total_ns = t_total.elapsed().as_nanos() as u64;
            return out;
        }
    };
    let base = format!("urn:rwai:powl:{}", plan_id(&tape));
    let turtle = match powl_to_turtle_with_phase_provenance(
        &model,
        &base,
        Some("urn:rwai:plan"),
        &phase_sources,
    ) {
        Ok(t) => t,
        Err(refusal) => {
            out.refusal_code = Some(refusal_code_static(&refusal));
            out.total_ns = t_total.elapsed().as_nanos() as u64;
            return out;
        }
    };
    out.stage_ns
        .push(("project", t.elapsed().as_nanos() as u64));

    // Stage: shape validation over the parsed generated graph.
    let t = Instant::now();
    let store = match Store::new() {
        Ok(s) => s,
        Err(_) => {
            out.refusal_code = Some("CNG_R10");
            out.total_ns = t_total.elapsed().as_nanos() as u64;
            return out;
        }
    };
    if store
        .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
        .is_err()
    {
        out.refusal_code = Some("CNG_R06");
        out.total_ns = t_total.elapsed().as_nanos() as u64;
        return out;
    }
    if let Err(refusal) = shape::validate_powl_store(&store, true) {
        out.refusal_code = Some(refusal_code_static(&refusal));
        out.total_ns = t_total.elapsed().as_nanos() as u64;
        return out;
    }
    out.stage_ns
        .push(("validate", t.elapsed().as_nanos() as u64));

    // Stage: bcinr-powl conformance execution over the hierarchical model
    // (linearized phases on the real branchless scheduler).
    let t = Instant::now();
    let run = match runner::validate_run_hierarchical(&tape, &model) {
        Ok(r) => r,
        Err(refusal) => {
            out.refusal_code = Some(refusal_code_static(&refusal));
            out.total_ns = t_total.elapsed().as_nanos() as u64;
            return out;
        }
    };
    out.transitions = run.executed_ops;
    out.activity_labels = tape.ops.iter().map(|op| op.label.clone()).collect();
    out.tape_ops = tape.ops.len();
    out.plan_id = Some(plan_id(&tape));
    out.stage_ns
        .push(("conformance", t.elapsed().as_nanos() as u64));

    // Stage: receipt (BLAKE3 over the generated bytes) + optional export.
    let t = Instant::now();
    out.powl_digest = blake3::hash(turtle.as_bytes()).to_hex().to_string();
    out.powl_bytes = turtle.len() as u64;
    if let Some(dir) = export_dir {
        let name = set_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "node".to_string());
        let _write = fs::create_dir_all(dir)
            .and_then(|()| fs::write(dir.join(format!("{name}.powl.ttl")), &turtle));
    }
    out.stage_ns
        .push(("receipt", t.elapsed().as_nanos() as u64));

    out.total_ns = t_total.elapsed().as_nanos() as u64;
    out
}

pub(super) fn refusal_code_static(refusal: &CngRefusal) -> &'static str {
    refusal.code()
}

/// Term to plain string: IRI text for named nodes, literal value otherwise.
pub(super) fn term_value(term: &Term) -> String {
    match term {
        Term::NamedNode(n) => n.as_str().to_string(),
        Term::Literal(l) => l.value().to_string(),
        other => other.to_string(),
    }
}

/// First object of `(any, <predicate>, ?o)` in `store`, as a term.
///
/// Distinguishes three outcomes a single `.ok()?` chain previously collapsed
/// into one `None`: an unparseable predicate IRI (`CNG_R01 MalformedTtl`,
/// defensive — `predicate` is always built from a compile-time
/// `RWAI_PREFIX` constant in this crate, but the check is real, not asserted
/// away); a genuine store-read fault on the pattern scan (`CNG_R10
/// IoRefused` — previously silently treated as "no such triple" via
/// `.ok()`); and an honest zero-match scan (`Ok(None)`), which is not a
/// failure at all.
pub(super) fn first_object(store: &Store, predicate: &str) -> Result<Option<Term>, CngRefusal> {
    let pred = NamedNodeRef::new(predicate)
        .map_err(|e| CngRefusal::MalformedTtl(format!("predicate IRI {predicate}: {e}")))?;
    match store.quads_for_pattern(None, Some(pred), None, None).next() {
        None => Ok(None),
        Some(Ok(quad)) => Ok(Some(quad.object)),
        Some(Err(e)) => Err(CngRefusal::IoRefused(format!(
            "quad scan for {predicate}: {e}"
        ))),
    }
}

/// Real classification: oxigraph pattern reads of `ex:category` and
/// `ex:worker` over the artifact's admitted graph (no SPARQL text — the
/// benchmark's only SPARQL comes from the on-disk query set).
///
/// The four fallible operations on this path each carry their own cause
/// instead of collapsing into one `CNG_R01 MalformedTtl` via a `.ok()?`
/// chain: reading the artifact off disk and constructing the scratch store
/// both refuse `CNG_R10 IoRefused` (a permission error or a path that
/// resolves to a directory is an I/O fault, not a Turtle defect); only an
/// actual Turtle parse failure refuses `CNG_R01 MalformedTtl`. A
/// syntactically valid artifact simply missing the `ex:category`/
/// `ex:worker` predicates is `Ok(None)` — not an error at all — leaving the
/// caller's pre-existing CNG_R01 "nothing to classify" fallback for that
/// case unchanged.
pub(super) fn classify_artifact(
    path: &Path,
) -> Result<Option<(String, String, usize)>, CngRefusal> {
    let turtle = fs::read_to_string(path)
        .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", path.display())))?;
    let store =
        Store::new().map_err(|e| CngRefusal::IoRefused(format!("store construction: {e}")))?;
    store
        .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
        .map_err(|e| CngRefusal::MalformedTtl(format!("{}: {e}", path.display())))?;
    let category = first_object(&store, &format!("{RWAI_PREFIX}category"))?;
    let worker = first_object(&store, &format!("{RWAI_PREFIX}worker"))?;
    let (category, worker) = match (category, worker) {
        (Some(c), Some(w)) => (c, w),
        _ => return Ok(None),
    };
    Ok(Some((
        term_value(&category),
        term_value(&worker),
        store.len().unwrap_or(0),
    )))
}

#[cfg(test)]
#[path = "manufacture_test.rs"]
mod manufacture_test;
