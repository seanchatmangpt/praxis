//! cng — Chatman Engine noun-verb CLI.
//!
//! Human-facing artifact handle for `A = μ(O*)`: many admitted PDDL Turtle
//! planning artifacts (`*.domain.ttl`, `*.problem.ttl`) become one POWL v2
//! Turtle workflow artifact with provenance, validation, runner/conformance
//! evidence, and a release manifest. The CLI imports, admits, plans,
//! projects, inspects, exports, validates, and receipts artifacts; it is
//! never the planning actor.
//!
//! Release law: for any admitted artifact set, every command either returns
//! its JSON result or emits exactly one typed refusal (`CNG_R01`–`CNG_R20`).
//! There is no third state — no silent fallback, no placeholder output, no
//! hand-authored generated POWL.

use cng::{pipeline, powl, shape};

use std::fs;
use std::path::{Path, PathBuf};

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::store::Store;
use serde::Serialize;

use pipeline::{generate_plan, import_artifacts, leaf_sources, plan_id, ImportedArtifact};
use powl::{powl_to_turtle_with_provenance, project_tape_to_powl, Powl};

const DEFAULT_BASE_IRI: &str = "urn:chatman:powl:cng";
#[cfg(feature = "bench")]
const DEFAULT_DECOMP_BASE_IRI: &str = "urn:chatman:decomp:cng";

#[derive(Debug, Serialize)]
struct ImportReport {
    imported_pddl_ttl_paths: Vec<String>,
    source_digests: Vec<String>,
    domain_fragments: usize,
    problem_fragments: usize,
}

#[derive(Debug, Serialize)]
struct AdmitReport {
    imported_pddl_ttl_paths: Vec<String>,
    domain_name: String,
    domain_fragments: usize,
    problem_fragments: usize,
    merged_actions: usize,
    merged_goal_atoms: usize,
}

#[derive(Debug, Serialize)]
struct PlanReport {
    imported_pddl_ttl_paths: Vec<String>,
    generated_plan_id: String,
    steps: Vec<String>,
}

/// Report of one real (non-test) no-LLM decomposition run (PROJ-741): the
/// PLANNING marker set (`queries/markers/marker-decomposition-*.rq`,
/// `marker-no-llm-authoring.rq`) is provable ONLY against evidence a real
/// run like this one produces on disk — never from in-process test state.
#[cfg(feature = "bench")]
#[derive(Debug, Serialize)]
struct DecomposeReport {
    domain_pddl_path: String,
    problem_pddl_path: String,
    out_dir: String,
    /// One of the three typed `DecompositionOutcome` values, never absent.
    outcome: String,
    selected_candidate_id: String,
    subworkflow_count: usize,
    candidate_receipt_count: usize,
    rejected_candidate_count: usize,
    interface_atom_count: usize,
    release_obligation_count: usize,
    cross_edge_count: usize,
    result_graph_path: String,
}

/// Human-readable rendering of a [`DecomposeReport`] for `--pretty`
/// (PROJ-dx-pretty-report). `DecomposeReport` lives in this binary crate
/// (not the `cng` library), so its renderer lives next to it here; it
/// reuses the shared ANSI/digest helpers from `cng::bench::report_pretty`
/// rather than duplicating color logic.
///
/// Per-candidate scores (`bench::decomp::CandidateReceipt::score`) are not
/// surfaced here: `DecomposeReport` only carries the aggregate counts
/// (`candidate_receipt_count`, `rejected_candidate_count`) that
/// `plan_decompose` already extracts from the decomposition result, so the
/// "comparison" below is candidates-examined vs. admitted vs. rejected,
/// not a per-candidate score table.
///
/// # Complexity
/// O(1) — fixed field count.
#[cfg(feature = "bench")]
fn render_decompose_report_human(report: &DecomposeReport) -> String {
    use cng::bench::report_pretty::{paint, use_color, Ansi};

    let color = use_color();
    let mut out = String::new();

    out.push_str(&paint("== cng plan decompose ==\n", Ansi::Bold, color));
    out.push_str(&format!("  domain  : {}\n", report.domain_pddl_path));
    out.push_str(&format!("  problem : {}\n", report.problem_pddl_path));
    out.push_str(&format!("  out_dir : {}\n\n", report.out_dir));

    let outcome_color = if report.outcome == "Selected" {
        Ansi::Green
    } else {
        Ansi::Yellow
    };
    out.push_str(&format!(
        "  {} {}\n",
        paint("OUTCOME", Ansi::Bold, color),
        paint(&report.outcome, outcome_color, color)
    ));
    out.push_str(&format!(
        "  selected_candidate_id : {}\n\n",
        report.selected_candidate_id
    ));

    out.push_str(&paint("CANDIDATE COMPARISON\n", Ansi::Bold, color));
    let admissible_or_selected = report
        .candidate_receipt_count
        .saturating_sub(report.rejected_candidate_count);
    out.push_str(&format!(
        "  {:<22} {}\n",
        "candidates examined", report.candidate_receipt_count
    ));
    out.push_str(&format!(
        "  {:<22} {}\n",
        "admissible/selected",
        paint(&admissible_or_selected.to_string(), Ansi::Green, color)
    ));
    out.push_str(&format!(
        "  {:<22} {}\n\n",
        "rejected",
        if report.rejected_candidate_count > 0 {
            paint(
                &report.rejected_candidate_count.to_string(),
                Ansi::Red,
                color,
            )
        } else {
            report.rejected_candidate_count.to_string()
        }
    ));

    out.push_str(&paint("STRUCTURE\n", Ansi::Bold, color));
    out.push_str(&format!(
        "  subworkflows        : {}\n",
        report.subworkflow_count
    ));
    out.push_str(&format!(
        "  interface_atoms     : {}\n",
        report.interface_atom_count
    ));
    out.push_str(&format!(
        "  release_obligations : {}\n",
        report.release_obligation_count
    ));
    out.push_str(&format!(
        "  cross_edges         : {}\n\n",
        report.cross_edge_count
    ));

    out.push_str(&format!(
        "  result_graph_path : {}\n",
        report.result_graph_path
    ));
    out
}

#[derive(Debug, Serialize)]
struct ProjectReport {
    generated_plan_id: String,
    activity_leaves: usize,
    order_pairs: usize,
    turtle: String,
}

#[derive(Debug, Serialize)]
struct ExportReport {
    imported_pddl_ttl_paths: Vec<String>,
    generated_plan_id: String,
    generated_powl_ttl_path: String,
    powl_digest: String,
    validation_result: String,
    activity_leaves: usize,
    order_pairs: usize,
}

#[derive(Debug, Serialize)]
struct InspectReport {
    file: String,
    triples: usize,
    validation_result: String,
    shape: shape::ShapeReport,
}

#[derive(Debug, Serialize)]
struct DoctorReport {
    cng_version: String,
    turtle_parser: String,
    planner: String,
    shape_validator: String,
    runtime_checks: Vec<String>,
}

/// Shared pipeline state: imported artifacts, plan tape, admitted surface,
/// projected model, and provenance-bearing Turtle.
struct Manufactured {
    artifacts: Vec<ImportedArtifact>,
    tape: bcinr_pddl::Pddl8Tape,
    model: Powl,
    turtle: String,
}

/// Runs import → admit/merge → plan → project → serialize-with-provenance.
/// Every element of the output carries the `urn:blake3:` IRI of the artifact
/// that contributed it.
fn manufacture(
    dir: &str,
    base_iri: Option<String>,
    derived_from: Option<String>,
) -> Result<Manufactured> {
    let artifacts = import_artifacts(Path::new(dir)).map_err(to_cli_error)?;
    let (tape, surface) = generate_plan(&artifacts).map_err(to_cli_error)?;
    let model = project_tape_to_powl(&tape).map_err(to_cli_error)?;
    let sources = leaf_sources(&tape, &surface).map_err(to_cli_error)?;
    let base = base_iri.unwrap_or_else(|| DEFAULT_BASE_IRI.to_string());
    let turtle = powl_to_turtle_with_provenance(&model, &base, derived_from.as_deref(), &sources)
        .map_err(to_cli_error)?;
    Ok(Manufactured {
        artifacts,
        tape,
        model,
        turtle,
    })
}

/// Parses Turtle into an in-memory store and shape-validates it.
fn parse_and_validate(
    turtle: &str,
    require_provenance: bool,
) -> Result<(usize, shape::ShapeReport)> {
    let store = Store::new()
        .map_err(|e| to_cli_error_msg(format!("oxigraph store construction failed: {e}")))?;
    store
        .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
        .map_err(|e| {
            to_cli_error(powl::CngRefusal::InvalidPowl(format!(
                "generated POWL failed to parse as Turtle: {e}"
            )))
        })?;
    let triples = store
        .len()
        .map_err(|e| to_cli_error_msg(format!("store.len() failed: {e}")))?;
    let report = shape::validate_powl_store(&store, require_provenance).map_err(to_cli_error)?;
    Ok((triples, report))
}

/// Writes `turtle` to `out` and returns the canonical path.
fn write_artifact(out: &str, turtle: &str) -> Result<PathBuf> {
    let out_path = PathBuf::from(out);
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| {
                to_cli_error_msg(format!("cannot create {}: {e}", parent.display()))
            })?;
        }
    }
    fs::write(&out_path, turtle)
        .map_err(|e| to_cli_error_msg(format!("cannot write {}: {e}", out_path.display())))?;
    out_path
        .canonicalize()
        .map_err(|e| to_cli_error_msg(format!("cannot canonicalize {}: {e}", out_path.display())))
}

fn artifact_paths(artifacts: &[ImportedArtifact]) -> Vec<String> {
    artifacts
        .iter()
        .map(|a| a.path.display().to_string())
        .collect()
}

/// Validates and lists the importable PDDL Turtle planning artifacts in a
/// directory (each artifact parses as Turtle and its PDDL literals are
/// selected exactly as the pipeline will consume them).
#[verb("import", "plan")]
fn plan_import(dir: String) -> Result<ImportReport> {
    let artifacts = import_artifacts(Path::new(&dir)).map_err(to_cli_error)?;
    Ok(ImportReport {
        domain_fragments: artifacts.iter().filter(|a| a.domain_text.is_some()).count(),
        problem_fragments: artifacts
            .iter()
            .filter(|a| a.problem_text.is_some())
            .count(),
        source_digests: artifacts.iter().map(|a| a.source_iri.clone()).collect(),
        imported_pddl_ttl_paths: artifact_paths(&artifacts),
    })
}

/// Admits the artifact set without planning: every fragment must parse and
/// the structural merge must succeed (shared domain name, no duplicate
/// actions). Reports the merged planning surface.
#[verb("admit", "plan")]
fn plan_admit(dir: String) -> Result<AdmitReport> {
    let artifacts = import_artifacts(Path::new(&dir)).map_err(to_cli_error)?;
    let surface = pipeline::merge_imported(&artifacts).map_err(to_cli_error)?;
    Ok(AdmitReport {
        imported_pddl_ttl_paths: artifact_paths(&artifacts),
        domain_name: surface.domain.name,
        domain_fragments: artifacts.iter().filter(|a| a.domain_text.is_some()).count(),
        problem_fragments: artifacts
            .iter()
            .filter(|a| a.problem_text.is_some())
            .count(),
        merged_actions: surface.domain.actions.len(),
        merged_goal_atoms: surface.problem.goal.len(),
    })
}

/// Merges every admitted fragment structurally and generates the ONE
/// combined plan (bounded BFS), returning its BLAKE3 plan id and step
/// labels.
#[verb("generate", "plan")]
fn plan_generate(dir: String) -> Result<PlanReport> {
    let artifacts = import_artifacts(Path::new(&dir)).map_err(to_cli_error)?;
    let (tape, _surface) = generate_plan(&artifacts).map_err(to_cli_error)?;
    Ok(PlanReport {
        imported_pddl_ttl_paths: artifact_paths(&artifacts),
        generated_plan_id: plan_id(&tape),
        steps: tape.ops.iter().map(|op| op.label.clone()).collect(),
    })
}

/// Runs the no-LLM multi-actor goal decomposition (PROJ-741,
/// `cng::bench::decomp::decompose`) on an admitted PDDL domain/problem file
/// pair (raw `.pddl` text, parsed via the unchanged `bcinr_pddl` parser —
/// the same two-file convention `tests/cng_ipc_corpus.rs`'s negative
/// fixtures use) and writes the full `decomposition-result.ttl` evidence
/// bundle under `out`. This is the ONLY non-test entrypoint that produces
/// the evidence the PROJ-739/740 planning markers
/// (`queries/markers/marker-decomposition-*.rq`,
/// `marker-no-llm-authoring.rq`) are provable against — everything else
/// this command does is a thin CLI wrapper over what
/// `bench::decomp::mod` already exposes; no new decomposition machinery.
/// `decompose()` is fully deterministic (no seed): the outcome depends only
/// on the admitted domain/problem content.
#[cfg(feature = "bench")]
#[verb("decompose", "plan")]
fn plan_decompose(
    domain: String,
    problem: String,
    out: String,
    base_iri: Option<String>,
    // PROJ-dx-pretty-report: prints `render_decompose_report_human` instead
    // of the JSON report; `main()` auto-adds `--format quiet` when this
    // flag is present, so the two never both print.
    pretty: bool,
) -> Result<DecomposeReport> {
    let domain_text = fs::read_to_string(&domain)
        .map_err(|e| to_cli_error_msg(format!("cannot read {domain}: {e}")))?;
    let problem_text = fs::read_to_string(&problem)
        .map_err(|e| to_cli_error_msg(format!("cannot read {problem}: {e}")))?;
    let parsed_domain = bcinr_pddl::parse::domain_from_pddl(&domain_text).map_err(|e| {
        to_cli_error(powl::CngRefusal::MalformedTtl(format!(
            "domain PDDL failed to parse: {e:?}"
        )))
    })?;
    let parsed_problem = bcinr_pddl::parse::problem_from_pddl(&problem_text).map_err(|e| {
        to_cli_error(powl::CngRefusal::MalformedTtl(format!(
            "problem PDDL failed to parse: {e:?}"
        )))
    })?;
    let base = base_iri.unwrap_or_else(|| DEFAULT_DECOMP_BASE_IRI.to_string());
    let result =
        cng::bench::decomp::decompose(&parsed_domain, &parsed_problem, Path::new(&out), &base)
            .map_err(to_cli_error)?;
    let rejected_candidate_count = result
        .candidate_receipts
        .iter()
        .filter(|r| r.status == cng::bench::decomp::CandidateStatus::Inadmissible)
        .count();
    let selected_candidate_id = match &result.outcome {
        cng::bench::decomp::DecompositionOutcome::Selected { candidate_id, .. } => {
            candidate_id.clone()
        }
        cng::bench::decomp::DecompositionOutcome::NoAdmissibleDecomposition { .. }
        | cng::bench::decomp::DecompositionOutcome::NoBeneficialDecomposition { .. } => {
            cng::bench::decomp::SINGLE_ACTOR_CANDIDATE_ID.to_string()
        }
    };
    let report = DecomposeReport {
        domain_pddl_path: domain,
        problem_pddl_path: problem,
        out_dir: out,
        outcome: result.outcome.as_str().to_string(),
        selected_candidate_id,
        subworkflow_count: result.subworkflows.len(),
        candidate_receipt_count: result.candidate_receipts.len(),
        rejected_candidate_count,
        interface_atom_count: result.interface_atoms.len(),
        release_obligation_count: result.release_obligations.len(),
        cross_edge_count: result.cross_edges.len(),
        result_graph_path: result.result_graph_path.display().to_string(),
    };
    println!("OUTCOME={}", report.outcome);
    println!("SELECTED_CANDIDATE_ID={}", report.selected_candidate_id);
    println!("SUBWORKFLOW_COUNT={}", report.subworkflow_count);
    println!("CANDIDATE_RECEIPT_COUNT={}", report.candidate_receipt_count);
    println!(
        "REJECTED_CANDIDATE_COUNT={}",
        report.rejected_candidate_count
    );
    println!("INTERFACE_ATOM_COUNT={}", report.interface_atom_count);
    println!(
        "RELEASE_OBLIGATION_COUNT={}",
        report.release_obligation_count
    );
    println!("RESULT_GRAPH_PATH={}", report.result_graph_path);
    if pretty {
        println!("{}", render_decompose_report_human(&report));
    }
    Ok(report)
}

/// Projects the combined plan into a POWL v2 model and returns its
/// provenance-bearing Turtle serialization inline (use `workflow export` to
/// write the artifact).
#[verb("project", "workflow")]
fn workflow_project(
    dir: String,
    base_iri: Option<String>,
    derived_from: Option<String>,
) -> Result<ProjectReport> {
    let m = manufacture(&dir, base_iri, derived_from)?;
    let (leaves, pairs) = model_shape(&m.model);
    Ok(ProjectReport {
        generated_plan_id: plan_id(&m.tape),
        activity_leaves: leaves,
        order_pairs: pairs,
        turtle: m.turtle,
    })
}

/// Runs the full manufacture and exports the ONE generated POWL v2 Turtle
/// workflow artifact to `--out`, shape-validating the exported bytes.
#[verb("export", "workflow")]
fn workflow_export(
    dir: String,
    out: String,
    base_iri: Option<String>,
    derived_from: Option<String>,
) -> Result<ExportReport> {
    let m = manufacture(&dir, base_iri, derived_from.clone())?;
    let (leaves, pairs) = model_shape(&m.model);
    let canonical = write_artifact(&out, &m.turtle)?;
    let exported = fs::read_to_string(&canonical)
        .map_err(|e| to_cli_error_msg(format!("cannot re-read {}: {e}", canonical.display())))?;
    let (_triples, shape_report) = parse_and_validate(&exported, derived_from.is_some())?;
    Ok(ExportReport {
        imported_pddl_ttl_paths: artifact_paths(&m.artifacts),
        generated_plan_id: plan_id(&m.tape),
        generated_powl_ttl_path: canonical.display().to_string(),
        powl_digest: format!("blake3:{}", blake3::hash(exported.as_bytes()).to_hex()),
        validation_result: format!(
            "shape-valid: {} leaves, {} bindings, {} precedes",
            shape_report.activity_leaves, shape_report.child_bindings, shape_report.precedes
        ),
        activity_leaves: leaves,
        order_pairs: pairs,
    })
}

/// Parses a POWL v2 Turtle artifact and validates it against the declared
/// structural shape (`shapes/powl2-shapes.ttl`); refuses `CNG_R06` on any
/// shape violation.
#[verb("inspect", "workflow")]
fn workflow_inspect(file: String) -> Result<InspectReport> {
    let turtle = fs::read_to_string(&file)
        .map_err(|e| to_cli_error_msg(format!("cannot read {file}: {e}")))?;
    let (triples, shape_report) = parse_and_validate(&turtle, false)?;
    Ok(InspectReport {
        file,
        triples,
        validation_result: "shape-valid".to_string(),
        shape: shape_report,
    })
}

/// Reports the toolchain surface this binary was built with and runs cheap
/// self-checks (in-memory Turtle store construction).
#[verb("doctor", "workflow")]
fn workflow_doctor() -> Result<DoctorReport> {
    let mut checks = Vec::new();
    match Store::new() {
        Ok(_) => checks.push("oxigraph in-memory store: ok".to_string()),
        Err(e) => checks.push(format!("oxigraph in-memory store: FAILED: {e}")),
    }
    Ok(DoctorReport {
        cng_version: env!("CARGO_PKG_VERSION").to_string(),
        turtle_parser: "oxigraph 0.5 (RdfParser::Turtle + SparqlEvaluator)".to_string(),
        planner: "bcinr-pddl 26.6.26 (GroundProblem bounded BFS)".to_string(),
        shape_validator: "shapes/powl2-shapes.ttl (SPARQL structural validator)".to_string(),
        runtime_checks: checks,
    })
}

#[cfg(feature = "runner")]
#[derive(Debug, Serialize)]
struct ValidateReport {
    generated_plan_id: String,
    runner: String,
    validated: bool,
    conformant: bool,
    executed_ops: usize,
    detail: String,
}

#[cfg(feature = "runner")]
#[derive(Debug, Serialize)]
struct EvidenceManifest {
    run_id: String,
    command: String,
    status: String,
    timestamp_utc: String,
    imported: Vec<ManifestInput>,
    generated_plan_id: String,
    generated_powl_ttl_path: String,
    powl_digest: String,
    validation_result: String,
    runner_result: String,
    pddl_fixture_seed: String,
}

#[cfg(feature = "runner")]
#[derive(Debug, Serialize)]
struct ManifestInput {
    path: String,
    digest: String,
}

#[cfg(feature = "runner")]
#[derive(Debug, Serialize)]
struct EvidenceReport {
    imported_pddl_ttl_paths: Vec<String>,
    generated_plan_id: String,
    generated_powl_ttl_path: String,
    powl_digest: String,
    validation_result: String,
    runner_result: String,
    pddl_fixture_seed: String,
    evidence_manifest_path: String,
    activity_leaves: usize,
    order_pairs: usize,
    parse_proof_triples: usize,
    determinism_proof: bool,
}

/// Runs the combined plan's projected workflow on the bcinr-powl runtime
/// (compile admission + branchless scheduler + order-conformance check) and
/// reports the computed verdict.
#[cfg(feature = "runner")]
#[verb("validate", "workflow")]
fn workflow_validate(dir: String) -> Result<ValidateReport> {
    let m = manufacture(&dir, None, None)?;
    let report = cng::runner::validate_run(&m.tape, &m.model).map_err(to_cli_error)?;
    Ok(ValidateReport {
        generated_plan_id: plan_id(&m.tape),
        runner: report.runner,
        validated: report.validated,
        conformant: report.conformant,
        executed_ops: report.executed_ops,
        detail: report.detail,
    })
}

/// Runs the full proof chain (import → admit/merge → plan → project →
/// export → parse-back → shape-validate → runner/conformance), prints every
/// evidence marker, and writes the release manifest binding inputs, output,
/// digests, results, command, run id, and timestamp.
#[cfg(feature = "runner")]
#[verb("evidence", "workflow")]
fn workflow_evidence(
    dir: String,
    out: String,
    base_iri: Option<String>,
    derived_from: Option<String>,
    // Fixture seed to echo in the evidence trail (fixture generation lives
    // in the test surface; pass-through only).
    seed: Option<String>,
) -> Result<EvidenceReport> {
    let base = base_iri.clone();
    let m = manufacture(&dir, base_iri, derived_from.clone())?;
    let (leaves, pairs) = model_shape(&m.model);

    // Determinism proof: a second full serialization must be byte-identical.
    let m2 = manufacture(&dir, base, derived_from.clone())?;
    if m.turtle != m2.turtle {
        return Err(to_cli_error(powl::CngRefusal::Nondeterminism(
            "repeated manufacture produced different POWL bytes".to_string(),
        )));
    }

    // Anti-hardcoding proof: every plan-step label must appear in the output.
    for op in &m.tape.ops {
        if !m.turtle.contains(&op.label) {
            return Err(to_cli_error(powl::CngRefusal::HardcodingSuspicion(
                format!(
                    "plan step {:?} does not appear in the generated POWL; output \
                 is detached from the admitted plan",
                    op.label
                ),
            )));
        }
    }

    let canonical = write_artifact(&out, &m.turtle)?;
    let exported = fs::read_to_string(&canonical)
        .map_err(|e| to_cli_error_msg(format!("cannot re-read {}: {e}", canonical.display())))?;
    let (parse_proof_triples, shape_report) =
        parse_and_validate(&exported, derived_from.is_some())?;
    let validation_result = format!(
        "shape-valid: {} leaves, {} bindings, {} precedes, {} derivedFrom",
        shape_report.activity_leaves,
        shape_report.child_bindings,
        shape_report.precedes,
        shape_report.derived_from
    );

    let runner = cng::runner::validate_run(&m.tape, &m.model).map_err(to_cli_error)?;
    let runner_result = format!(
        "{}: validated={} conformant={} executed_ops={}",
        runner.runner, runner.validated, runner.conformant, runner.executed_ops
    );

    let powl_digest = format!("blake3:{}", blake3::hash(exported.as_bytes()).to_hex());
    let generated_plan_id = plan_id(&m.tape);
    let imported: Vec<ManifestInput> = m
        .artifacts
        .iter()
        .map(|a| ManifestInput {
            path: a.path.display().to_string(),
            digest: a.digest.clone(),
        })
        .collect();
    let seed = seed.unwrap_or_else(|| "none".to_string());

    // Run id is deterministic: BLAKE3 over input digests + output digest.
    let mut run_material = String::new();
    for input in &imported {
        run_material.push_str(&input.digest);
        run_material.push('\n');
    }
    run_material.push_str(&powl_digest);
    let run_id = format!("blake3:{}", blake3::hash(run_material.as_bytes()).to_hex());
    // Timestamp is evidence metadata only — it is deliberately excluded from
    // run_id and all digests so determinism claims stay byte-exact.
    let timestamp_utc = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| format!("unix:{}", d.as_secs()))
        .unwrap_or_else(|_| "unix:unavailable".to_string());

    let manifest = EvidenceManifest {
        run_id: run_id.clone(),
        command: format!("cng workflow evidence --dir {dir} --out {out}"),
        status: "MANUFACTURED".to_string(),
        timestamp_utc,
        imported: imported
            .iter()
            .map(|i| ManifestInput {
                path: i.path.clone(),
                digest: i.digest.clone(),
            })
            .collect(),
        generated_plan_id: generated_plan_id.clone(),
        generated_powl_ttl_path: canonical.display().to_string(),
        powl_digest: powl_digest.clone(),
        validation_result: validation_result.clone(),
        runner_result: runner_result.clone(),
        pddl_fixture_seed: seed.clone(),
    };
    let manifest_path = format!("{}.manifest.json", canonical.display());
    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| to_cli_error_msg(format!("manifest serialization failed: {e}")))?;
    fs::write(&manifest_path, &manifest_json)
        .map_err(|e| to_cli_error_msg(format!("cannot write {manifest_path}: {e}")))?;

    let imported_paths: Vec<String> = imported.iter().map(|i| i.path.clone()).collect();
    println!("IMPORTED_PDDL_TTL_PATHS={}", imported_paths.join(","));
    println!("GENERATED_PLAN_ID={generated_plan_id}");
    println!("GENERATED_POWL_TTL_PATH={}", canonical.display());
    println!("POWL_DIGEST={powl_digest}");
    println!("VALIDATION_RESULT={validation_result}");
    println!("RUNNER_RESULT={runner_result}");
    println!("PDDL_FIXTURE_SEED={seed}");
    println!("EVIDENCE_MANIFEST_PATH={manifest_path}");

    Ok(EvidenceReport {
        imported_pddl_ttl_paths: imported_paths,
        generated_plan_id,
        generated_powl_ttl_path: canonical.display().to_string(),
        powl_digest,
        validation_result,
        runner_result,
        pddl_fixture_seed: seed,
        evidence_manifest_path: manifest_path,
        activity_leaves: leaves,
        order_pairs: pairs,
        parse_proof_triples,
        determinism_proof: true,
    })
}

/// Generates a deterministic Fortune-5 benchmark corpus: partitioned worker
/// roster RDF, worker-attributed workload artifact sets across 12 enterprise
/// categories, and the 8-ary recursion tree with graph-declared attachments.
#[cfg(feature = "bench")]
#[verb("generate", "benchmark")]
fn benchmark_generate(
    out: String,
    workers: u64,
    sets: Option<usize>,
    depth: Option<usize>,
    seed: Option<u64>,
    refusal_per_mille: Option<usize>,
) -> Result<cng::bench::GenerateReport> {
    let workers = workers as usize;
    let cfg = cng::bench::BenchConfig {
        workers,
        artifact_sets: sets.unwrap_or_else(|| (workers / 100).clamp(64, 50_000)),
        recursion_depth: depth.unwrap_or(5),
        seed: seed.unwrap_or(42),
        refusal_per_mille: refusal_per_mille.unwrap_or(10),
    };
    cng::bench::generate(Path::new(&out), &cfg).map_err(to_cli_error)
}

/// Runs the benchmark: admits every roster partition, manufactures every
/// workload set and recursion node through the real cng chain (import →
/// plan → project → validate → conformance → receipt), replays a sample,
/// and prints the evidence markers.
#[cfg(feature = "bench")]
#[verb("run", "benchmark")]
fn benchmark_run(
    dir: String,
    threads: Option<usize>,
    replay_per_mille: Option<usize>,
    queries_dir: Option<String>,
) -> Result<cng::bench::RunReport> {
    let threads = threads.unwrap_or_else(|| {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(8)
    });
    let queries_dir = queries_dir.map(PathBuf::from);
    let report = cng::bench::run(
        Path::new(&dir),
        threads,
        replay_per_mille.unwrap_or(20),
        queries_dir.as_deref(),
    )
    .map_err(to_cli_error)?;
    println!("MEASUREMENT_CLASS={}", report.measurement_class);
    println!("WORKERS_REPRESENTED={}", report.workers_represented);
    println!(
        "INPUT_TTL_ARTIFACTS={}",
        report.telemetry.input_ttl_artifacts
    );
    println!("INPUT_TRIPLES={}", report.telemetry.roster_triples);
    println!("RECURSION_DEPTH={}", report.recursion_depth);
    println!(
        "LOGICAL_WORKFLOW_NODES={}",
        report.telemetry.logical_workflow_nodes
    );
    println!(
        "MATERIALIZED_POWL_NODES={}",
        report.telemetry.materialized_powl_nodes
    );
    println!("EXECUTED_TRANSITIONS={}", report.executed_transitions);
    println!(
        "VALIDATION_RESULT=shape-valid:{}/{}",
        report.telemetry.validation_passes, report.telemetry.materialized_powl_nodes
    );
    println!(
        "CONFORMANCE_RESULT=conformant:{}/{}",
        report.telemetry.conformance_passes, report.telemetry.materialized_powl_nodes
    );
    println!("RECEIPTS_GENERATED={}", report.receipts);
    println!(
        "REPLAY_RESULT={}/{}",
        report.telemetry.replay_passes, report.telemetry.replay_checked
    );
    println!("POWL_DIGEST={}", report.evidence_chain_digest);
    println!("WORKFLOW_INSTANCES={}", report.workflow_instances);
    println!("RECURSIVE_ATTACHMENTS={}", report.recursive_attachments);
    println!("CONFORMANT_TRANSITIONS={}", report.conformance);
    println!("REFUSED_TRANSITIONS={}", report.refusals);
    println!(
        "DATALOG_DERIVED_ROLES={}",
        report.telemetry.datalog_derived_roles
    );
    println!("OCEL_GRAPH_DIGEST={}", report.ocel_graph_digest);
    println!("SPARQL_RESULT_DIGEST={}", report.sparql_result_digest);
    println!(
        "BENCHMARK_RESULT_PATH={}/results/results.json",
        report.bench_dir
    );
    println!(
        "MODELED_LLM_COMPARISON_PATH={}/results/modeled-llm-comparison.json",
        report.bench_dir
    );
    println!(
        "DERIVED_SCALE_PATH={}/results/derived-scale.json",
        report.bench_dir
    );
    println!(
        "EVIDENCE_MANIFEST_PATH={}/results/evidence-manifest.json",
        report.bench_dir
    );
    Ok(report)
}

/// Runs the single-operator workday benchmark: a roster of ONE operator
/// executes a deterministic logical-tick day (splitmix64-seeded, no wall
/// clock in any digest) through the real cng chain, with the
/// standing-next-action law (exactly one lawful action per tick, else
/// CNG_R12) and the bounded admission → resume loop.
#[cfg(feature = "bench")]
#[verb("workday", "benchmark")]
fn benchmark_workday(
    out: String,
    seed: Option<u64>,
    ticks: Option<usize>,
    refusal_per_mille: Option<usize>,
    // PROJ-dx-pretty-report: prints `render_workday_report_human` instead
    // of the JSON report; `main()` auto-adds `--format quiet` when this
    // flag is present, so the two never both print.
    pretty: bool,
) -> Result<cng::bench::WorkdayReport> {
    let cfg = cng::bench::WorkdayConfig {
        seed: seed.unwrap_or(42),
        ticks: ticks.unwrap_or(32),
        refusal_per_mille: refusal_per_mille.unwrap_or(125),
    };
    let report = cng::bench::workday(Path::new(&out), &cfg, None).map_err(to_cli_error)?;
    println!("MEASUREMENT_CLASS={}", report.measurement_class);
    println!("WORKDAY_SEED={}", report.seed);
    println!("WORKDAY_TICKS={}", report.ticks);
    println!("WORKERS_REPRESENTED={}", report.workers_represented);
    println!("WORKFLOW_INSTANCES={}", report.workflow_instances);
    println!("EXECUTED_TRANSITIONS={}", report.executed_transitions);
    println!("RECEIPTS_GENERATED={}", report.receipts);
    println!("REFUSED_TRANSITIONS={}", report.refusals);
    println!("ADMISSION_REQUESTS={}", report.admission_requests);
    println!("ADMISSIONS_GRANTED={}", report.admissions_granted);
    println!("RESUMED_WORKFLOWS={}", report.resumes);
    println!("DISPATCHES_SENT={}", report.dispatches_sent);
    println!("CONSEQUENCES_ADMITTED={}", report.consequences_admitted);
    println!("CONSEQUENCES_REFUSED={}", report.consequences_refused);
    println!("DISPATCH_TIMEOUTS={}", report.dispatch_timeouts);
    println!("REMEDIATIONS_MANUFACTURED={}", report.remediations);
    println!("REPLAY_VERIFIED={}", report.replay_verified);
    // PROJ-622: SPARQL-derived success markers (all true by construction —
    // a false marker refused CNG_R20 with a nonzero exit before this line).
    for (marker, value) in &report.markers {
        println!("MARKER_{marker}={value}");
    }
    println!("POWL_DIGEST={}", report.evidence_chain_digest);
    println!("OCEL_GRAPH_DIGEST={}", report.ocel_graph_digest);
    println!("OBS_DIGEST={}", report.obs_digest);
    println!(
        "WORKDAY_RESULT_PATH={}/results/workday-report.json",
        report.out_dir
    );
    if pretty {
        println!(
            "{}",
            cng::bench::report_pretty::render_workday_report_human(&report)
        );
    }
    Ok(report)
}

/// Independent verification: replays a deterministic sample against the
/// recorded digests and re-validates exported POWL artifacts from disk.
#[cfg(feature = "bench")]
#[verb("verify", "benchmark")]
fn benchmark_verify(
    dir: String,
    sample_every: Option<usize>,
    threads: Option<usize>,
) -> Result<cng::bench::VerifyReport> {
    let threads = threads.unwrap_or_else(|| {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(8)
    });
    let report = cng::bench::verify(Path::new(&dir), sample_every.unwrap_or(50), threads)
        .map_err(to_cli_error)?;
    println!("REPLAY_RESULT={}/{}", report.replay_passes, report.replayed);
    Ok(report)
}

/// Prints the standard engine serve/resume marker lines, or (PROJ-dx-
/// pretty-report) `render_engine_serve_report_human` when `pretty` is set
/// — `main()` auto-adds `--format quiet` whenever `--pretty` is present,
/// so the two never both print. O(1).
#[cfg(feature = "bench")]
fn print_engine_report(report: &cng::bench::EngineServeReport, pretty: bool) {
    println!("MEASUREMENT_CLASS={}", report.measurement_class);
    println!("ENGINE_ID={}", report.engine_id);
    println!("ENGINE_VERSION={}", report.engine_version);
    println!("ENGINE_INSTANCE_NONCE={}", report.instance_nonce);
    println!("ENGINE_RESUMED={}", report.resumed);
    println!("LEDGER_ENTRIES_VERIFIED={}", report.ledger_entries_verified);
    println!("ENGINE_POLLS={}", report.polls);
    println!("CONTRACTS_EXECUTED={}", report.contracts_executed);
    println!("ENGINE_QUIESCED={}", report.quiesced);
    println!("RECEIPT_CHAIN_DIGEST={}", report.receipt_chain_digest);
    if pretty {
        println!(
            "{}",
            cng::bench::report_pretty::render_engine_serve_report_human(report)
        );
    }
}

/// Runs one Chatman Engine worker process (PROJ-723): a bounded receipted
/// poll loop over `<root>/engines/<engine-id>/inbox/`; each admitted
/// dispatch contract executes through the real cng manufacture chain and
/// its consequence Turtle is written atomically to the engine's outbox;
/// the SHACL-validated `control/quiesce.ttl` ends the loop. Poll counts
/// are logical; the inter-poll sleep sits behind the RealTimeWait seam and
/// never enters any digest.
#[cfg(feature = "bench")]
#[verb("serve", "engine")]
fn engine_serve(
    root: String,
    engine_id: String,
    seed: Option<u64>,
    max_polls: Option<u64>,
    poll_wait_ms: Option<u64>,
    // PROJ-dx-pretty-report: see `print_engine_report`.
    pretty: bool,
) -> Result<cng::bench::EngineServeReport> {
    let report = cng::bench::engine_serve(
        Path::new(&root),
        &engine_id,
        seed.unwrap_or(42),
        max_polls.unwrap_or(64),
        poll_wait_ms,
    )
    .map_err(to_cli_error)?;
    print_engine_report(&report, pretty);
    Ok(report)
}

/// Resumes a Chatman Engine worker after a crash/kill (PROJ-724): reloads
/// the durable dispatch ledger tail + processed idempotency set, verifies
/// every per-dispatch receipt-chain prefix (a torn ledger tail refuses
/// CNG_R11), receipts the verification, and continues the serve loop.
#[cfg(feature = "bench")]
#[verb("resume", "engine")]
fn engine_resume(
    root: String,
    engine_id: String,
    seed: Option<u64>,
    max_polls: Option<u64>,
    poll_wait_ms: Option<u64>,
    // PROJ-dx-pretty-report: see `print_engine_report`.
    pretty: bool,
) -> Result<cng::bench::EngineServeReport> {
    let report = cng::bench::engine_resume(
        Path::new(&root),
        &engine_id,
        seed.unwrap_or(42),
        max_polls.unwrap_or(64),
        poll_wait_ms,
    )
    .map_err(to_cli_error)?;
    print_engine_report(&report, pretty);
    Ok(report)
}

/// Independent auditor replay from a self-contained evidence bundle.
#[cfg(feature = "bench")]
#[verb("replay", "evidence")]
fn evidence_replay(bundle: String) -> Result<cng::bench::AuditReplayReport> {
    let report = cng::bench::audit_replay(Path::new(&bundle)).map_err(to_cli_error)?;
    println!("AUDIT_OBS_DIGEST_MATCH={}", report.obs_digest_match);
    println!("AUDIT_QUERIES_VERIFIED={}", report.queries_verified);
    println!(
        "AUDIT_OCEL_GRAPH_DIGEST_MATCH={}",
        report.ocel_graph_digest_match
    );
    println!("AUDIT_RESULT=CONFORMANT");
    Ok(report)
}

/// Counts leaves and order pairs of the projected model.
///
/// # Complexity
/// O(1) — reads the already-built vectors' lengths.
fn model_shape(model: &Powl) -> (usize, usize) {
    match model {
        Powl::PartialOrder { children, order } => (children.len(), order.len()),
        Powl::Leaf(_) => (1, 0),
    }
}

fn to_cli_error(refusal: powl::CngRefusal) -> clap_noun_verb::NounVerbError {
    clap_noun_verb::NounVerbError::execution_error(format!("{refusal}\nhint: {}", refusal.hint()))
}

fn to_cli_error_msg(msg: String) -> clap_noun_verb::NounVerbError {
    clap_noun_verb::NounVerbError::execution_error(format!("CNG_R10: {msg}"))
}

/// PROJ-dx-pretty-report: `--pretty` is one user-facing flag, but getting
/// "human-readable INSTEAD of JSON, never both" out of the underlying
/// `clap-noun-verb` framework needs two effects — the per-verb `pretty:
/// bool` param (added to `plan_decompose`/`benchmark_workday`/
/// `engine_serve`/`engine_resume`) that prints the human-readable render,
/// and the framework's own `--format quiet` (already-global, pre-existing
/// flag) to suppress its automatic JSON dump of the verb's return value.
/// This wrapper sets `--format quiet` in argv whenever `--pretty` is
/// present so the user never has to pass both. An explicit `--format
/// <other>` given alongside `--pretty` has its value overwritten in place
/// (clap refuses a *second* `--format` occurrence outright — it does not
/// silently keep the last one — so this rewrites the existing occurrence
/// rather than appending a new one); pretty output always wins over JSON
/// when both are requested.
fn main() -> Result<()> {
    let mut args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--pretty") {
        if let Some(idx) = args.iter().position(|a| a == "--format") {
            match args.get_mut(idx + 1) {
                Some(value) => *value = "quiet".to_string(),
                None => args.push("quiet".to_string()),
            }
        } else if let Some(idx) = args.iter().position(|a| a.starts_with("--format=")) {
            args[idx] = "--format=quiet".to_string();
        } else {
            args.push("--format".to_string());
            args.push("quiet".to_string());
        }
    }
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!("failed to lock CLI registry: {e}"))
    })?;
    registry.run(args)
}

#[cfg(all(test, feature = "bench"))]
mod tests {
    use chicago_tdd_tools::prelude::*;

    use std::fs;
    use std::path::PathBuf;

    use super::{plan_decompose, render_decompose_report_human};

    fn scratch_dir(test_name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/chatman/cng-tests/main-report-pretty")
            .join(test_name)
    }

    // PROJ-dx-pretty-report: exercises the REAL `plan decompose` CLI verb
    // end to end (not a hand-built `DecomposeReport`) — a solvable IPC
    // blocksworld instance rendered to on-disk PDDL and admitted/decomposed
    // exactly as `cng plan decompose --pretty` would — and asserts the
    // human-readable render carries the outcome, the candidate comparison,
    // and the structural counts.
    test!(render_decompose_report_human_is_sensible_on_a_real_run, {
        // Arrange: a small, guaranteed-solvable blocksworld instance
        // written to on-disk .pddl files (plan_decompose reads paths, the
        // same convention as the CLI's `--domain`/`--problem` flags).
        let (problem, _gated_size) = cng::bench::ipc::generate_solvable("blocksworld", 0, 3)
            .expect("blocksworld has a solvable size <= 3");
        let dir = scratch_dir("decompose_pretty_smoke");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create scratch dir");
        let domain_path = dir.join("domain.pddl");
        let problem_path = dir.join("problem.pddl");
        fs::write(&domain_path, &problem.domain_pddl).expect("write domain fixture");
        fs::write(&problem_path, &problem.problem_pddl).expect("write problem fixture");
        let out_dir = dir.join("out");

        // Act: the real CLI verb function, pretty=true.
        let report = plan_decompose(
            domain_path.display().to_string(),
            problem_path.display().to_string(),
            out_dir.display().to_string(),
            None,
            true,
        )
        .expect("real plan decompose run");
        let rendered = render_decompose_report_human(&report);

        // Assert: non-empty, and carries the substance a human would look
        // for — outcome, candidate comparison, structure — not just a
        // re-serialization of the JSON.
        assert!(!rendered.is_empty());
        assert!(rendered.contains("cng plan decompose"));
        assert!(rendered.contains("OUTCOME"));
        assert!(rendered.contains(&report.outcome));
        assert!(rendered.contains("CANDIDATE COMPARISON"));
        assert!(rendered.contains("candidates examined"));
        assert!(rendered.contains("STRUCTURE"));
        assert!(rendered.contains(&report.result_graph_path));
    });
}
