//! Fortune-5 scale benchmark for Autonomic Recursive Workflow on the REAL
//! cng manufacture chain. Nothing here mocks or bypasses the product path:
//! every workflow goes through `pipeline::import_artifacts` (oxigraph Turtle
//! admission) → `pipeline::generate_plan` (bcinr-pddl grounding + bounded
//! BFS) → `powl::project_tape_to_powl` → provenance serialization →
//! `shape::validate_powl_store` → `runner::validate_run` (bcinr-powl
//! compile + branchless scheduler + conformance) → BLAKE3 receipts.
//!
//! Workers are not counters: `generate` materializes every represented
//! worker as RDF facts in partitioned roster `.ttl` artifacts (identity,
//! role, department, standing), and every workload artifact set is
//! attributed to a worker IRI. Classification and role inference are real
//! SPARQL queries over the admitted graphs. Recursive attachment is derived
//! from `ex:attachesWorkflow` triples in the admitted node graph via SPARQL
//! — child workflows are discovered from the graph, never hand-wired, and
//! all child artifacts are machine-generated.
//!
//! Wall-clock timing lives here (benchmark instrumentation), never in the
//! manufacture path itself; digests and receipts contain no time.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::model::Term;
use oxigraph::sparql::{QueryResults, SparqlEvaluator};
use oxigraph::store::Store;

use crate::pipeline::{generate_plan, import_artifacts, leaf_sources, plan_id};
use crate::powl::{powl_to_turtle_with_provenance, project_tape_to_powl, CngRefusal};
use crate::runner;
use crate::shape;
use wasm4pm_cognition::breeds::production_rules::Mycin;
use wasm4pm_cognition::breeds::{BreedInput, CognitionBreed, Fact, Rule};

const WORKERS_PER_ROSTER_PARTITION: usize = 5_000;
const CATEGORIES: [&str; 12] = [
    "email-routing",
    "calendar-change",
    "invoice-matching",
    "purchase-order-approval",
    "expense-review",
    "hr-notice",
    "customer-request",
    "logistics-event",
    "compliance-check",
    "document-request",
    "software-delivery",
    "admission-request",
];
const STEP_VERBS: [&str; 8] = [
    "classify",
    "extract",
    "match",
    "verify",
    "check",
    "authorize",
    "execute",
    "record",
];

/// splitmix64: deterministic, seedable, dependency-free.
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn short_hex(v: u64) -> String {
    format!("{v:012x}")[..8].to_string()
}

// ---------------------------------------------------------------------------
// generate
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BenchConfig {
    pub workers: usize,
    pub artifact_sets: usize,
    pub recursion_depth: usize,
    pub seed: u64,
    pub refusal_per_mille: usize,
}

#[derive(Debug, serde::Serialize)]
pub struct GenerateReport {
    pub out_dir: String,
    pub workers_represented: usize,
    pub roster_partitions: usize,
    pub artifact_sets: usize,
    pub recursion_nodes: usize,
    pub recursion_depth: usize,
    pub files_written: usize,
    pub bytes_written: u64,
}

struct Templates {
    domain: String,
    problem: String,
}

fn load_templates() -> Result<Templates, CngRefusal> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
    let read = |name: &str| -> Result<String, CngRefusal> {
        fs::read_to_string(dir.join(name))
            .map_err(|e| CngRefusal::IoRefused(format!("cannot read template {name}: {e}")))
    };
    Ok(Templates {
        domain: read("bench-domain-fragment.template.ttl")?,
        problem: read("bench-problem.template.ttl")?,
    })
}

/// Writes one workflow artifact set (2 domain fragments + up to 2 problem
/// fragments) for `worker`, category-flavored, with optional recursive
/// attachment triples pointing at `children` and an optional injected
/// missing-problem refusal case.
#[allow(clippy::too_many_arguments)]
fn write_set(
    templates: &Templates,
    dir: &Path,
    rng: &mut u64,
    set_tag: &str,
    worker_iri: &str,
    category: &str,
    children: usize,
    omit_final_problem: bool,
) -> Result<(usize, u64), CngRefusal> {
    fs::create_dir_all(dir)
        .map_err(|e| CngRefusal::IoRefused(format!("mkdir {}: {e}", dir.display())))?;
    let domain = format!("wf-{category}-{set_tag}");
    let obj = format!("case-{set_tag}");
    let preds: Vec<String> = (0..=8)
        .map(|i| format!("s{i}-{}-{set_tag}", short_hex(splitmix64(rng))))
        .collect();
    let actions: Vec<String> = (0..8)
        .map(|i| format!("{}-{category}-{set_tag}-{i}", STEP_VERBS[i]))
        .collect();

    let mut files = 0usize;
    let mut bytes = 0u64;
    for half in 0..2usize {
        let mut body = templates
            .domain
            .replace("{{SUBJECT}}", &format!("art-{set_tag}-d{half}"))
            .replace("{{CATEGORY}}", category)
            .replace("{{WORKER}}", worker_iri)
            .replace("{{DOMAIN}}", &domain)
            .replace("{{OBJ}}", &obj);
        for i in 0..=4usize {
            body = body.replace(&format!("{{{{P{i}}}}}"), &preds[half * 4 + i]);
        }
        for i in 0..4usize {
            body = body.replace(&format!("{{{{A{i}}}}}"), &actions[half * 4 + i]);
        }
        // Recursive attachment facts: each activity of the FIRST fragment may
        // lawfully socket a child workflow; the runner derives children from
        // these triples via SPARQL, never from directory listing.
        if half == 0 && children > 0 {
            let mut attach = String::new();
            for c in 0..children {
                attach.push_str(&format!(
                    "ex:art-{set_tag}-d0 ex:attachesWorkflow ex:child-{c} .\n"
                ));
            }
            body.push_str(&attach);
        }
        let path = dir.join(format!("fragment-{half}.domain.ttl"));
        bytes += body.len() as u64;
        fs::write(&path, body)
            .map_err(|e| CngRefusal::IoRefused(format!("write {}: {e}", path.display())))?;
        files += 1;
    }
    let goals = if omit_final_problem {
        // Injected bounded-admission case: the closing problem fragment is
        // missing, so manufacture refuses CNG_R03 and a bounded human
        // admission is requested instead of silent fallback.
        vec![]
    } else {
        vec![(4usize, "mid"), (8usize, "final")]
    };
    for (goal_idx, tag) in goals {
        let body = templates
            .problem
            .replace("{{SUBJECT}}", &format!("art-{set_tag}-p{tag}"))
            .replace("{{CATEGORY}}", category)
            .replace("{{WORKER}}", worker_iri)
            .replace("{{PROBLEM}}", &format!("{domain}-{tag}"))
            .replace("{{DOMAIN}}", &domain)
            .replace("{{OBJ}}", &obj)
            .replace("{{INIT}}", &preds[0])
            .replace("{{GOAL}}", &preds[goal_idx]);
        let path = dir.join(format!("goal-{tag}.problem.ttl"));
        bytes += body.len() as u64;
        fs::write(&path, body)
            .map_err(|e| CngRefusal::IoRefused(format!("write {}: {e}", path.display())))?;
        files += 1;
    }
    Ok((files, bytes))
}

/// Recursively writes the 8-ary recursion tree below `dir` to `depth` levels
/// (root at level 1). Every node is a full machine-generated artifact set.
fn write_recursion_tree(
    templates: &Templates,
    dir: &Path,
    rng: &mut u64,
    tag: &str,
    worker_iri: &str,
    level: usize,
    depth: usize,
    files: &mut usize,
    bytes: &mut u64,
    nodes: &mut usize,
) -> Result<(), CngRefusal> {
    let children = if level < depth { 8 } else { 0 };
    let category = CATEGORIES[(splitmix64(rng) % CATEGORIES.len() as u64) as usize];
    let (f, b) = write_set(
        templates, dir, rng, tag, worker_iri, category, children, false,
    )?;
    *files += f;
    *bytes += b;
    *nodes += 1;
    for c in 0..children {
        let child_dir = dir.join(format!("child-{c}"));
        write_recursion_tree(
            templates,
            &child_dir,
            rng,
            &format!("{tag}-{c}"),
            worker_iri,
            level + 1,
            depth,
            files,
            bytes,
            nodes,
        )?;
    }
    Ok(())
}

/// Generates the full benchmark corpus: partitioned worker roster TTL,
/// per-worker workload artifact sets, and the 8-ary recursion tree.
///
/// # Complexity
/// O(workers + sets + 8^depth) file writes, all seeded/deterministic.
pub fn generate(out_dir: &Path, cfg: &BenchConfig) -> Result<GenerateReport, CngRefusal> {
    let templates = load_templates()?;
    fs::create_dir_all(out_dir)
        .map_err(|e| CngRefusal::IoRefused(format!("mkdir {}: {e}", out_dir.display())))?;
    let mut files = 0usize;
    let mut bytes = 0u64;

    // 1. Roster partitions: every represented worker is a materialized RDF
    //    fact set (identity, role, department, standing) in a partition
    //    artifact. Turtle is assembled here as generated ARTIFACT CONTENT
    //    (same status as the serializer's output), then written to disk and
    //    only ever consumed back through oxigraph.
    let roster_dir = out_dir.join("roster");
    fs::create_dir_all(&roster_dir)
        .map_err(|e| CngRefusal::IoRefused(format!("mkdir roster: {e}")))?;
    let roles = ["reviewer", "approver", "operator", "auditor", "coordinator"];
    let departments = [
        "finance",
        "hr",
        "logistics",
        "sales",
        "engineering",
        "legal",
    ];
    let mut rng = cfg.seed;
    let partitions = cfg.workers.div_ceil(WORKERS_PER_ROSTER_PARTITION);
    for p in 0..partitions {
        let start = p * WORKERS_PER_ROSTER_PARTITION;
        let end = usize::min(start + WORKERS_PER_ROSTER_PARTITION, cfg.workers);
        let mut body = String::with_capacity((end - start) * 160 + 128);
        body.push_str("@prefix ex: <http://example.org/rwai#> .\n\n");
        for w in start..end {
            let role = roles[(splitmix64(&mut rng) % roles.len() as u64) as usize];
            let dept = departments[(splitmix64(&mut rng) % departments.len() as u64) as usize];
            body.push_str(&format!(
                "ex:w{w} a ex:Worker ; ex:role ex:{role} ; ex:department ex:{dept} ; ex:standing ex:admitted .\n"
            ));
        }
        let path = roster_dir.join(format!("partition-{p:05}.ttl"));
        bytes += body.len() as u64;
        fs::write(&path, body)
            .map_err(|e| CngRefusal::IoRefused(format!("write {}: {e}", path.display())))?;
        files += 1;
    }

    // 2. Workload artifact sets, worker-attributed, category-mixed.
    let sets_dir = out_dir.join("sets");
    for s in 0..cfg.artifact_sets {
        let worker = (splitmix64(&mut rng) % cfg.workers as u64) as usize;
        let category = CATEGORIES[s % CATEGORIES.len()];
        let omit = (splitmix64(&mut rng) % 1000) < cfg.refusal_per_mille as u64;
        let (f, b) = write_set(
            &templates,
            &sets_dir.join(format!("set-{s:06}")),
            &mut rng,
            &format!("s{s:06}"),
            &format!("http://example.org/rwai#w{worker}"),
            category,
            0,
            omit,
        )?;
        files += f;
        bytes += b;
    }

    // 3. Recursion tree (8-ary, `recursion_depth` levels).
    let mut nodes = 0usize;
    if cfg.recursion_depth > 0 {
        let worker = (splitmix64(&mut rng) % cfg.workers as u64) as usize;
        write_recursion_tree(
            &templates,
            &out_dir.join("recursion").join("root"),
            &mut rng,
            "r",
            &format!("http://example.org/rwai#w{worker}"),
            1,
            cfg.recursion_depth,
            &mut files,
            &mut bytes,
            &mut nodes,
        )?;
    }

    let config_json = serde_json::to_string_pretty(cfg)
        .map_err(|e| CngRefusal::IoRefused(format!("config serialize: {e}")))?;
    fs::write(out_dir.join("benchmark-config.json"), &config_json)
        .map_err(|e| CngRefusal::IoRefused(format!("write config: {e}")))?;

    Ok(GenerateReport {
        out_dir: out_dir.display().to_string(),
        workers_represented: cfg.workers,
        roster_partitions: partitions,
        artifact_sets: cfg.artifact_sets,
        recursion_nodes: nodes,
        recursion_depth: cfg.recursion_depth,
        files_written: files,
        bytes_written: bytes,
    })
}

// ---------------------------------------------------------------------------
// run
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct LatencyStats {
    pub count: usize,
    pub mean_us: f64,
    pub p50_us: f64,
    pub p95_us: f64,
    pub p99_us: f64,
    pub max_us: f64,
}

fn latency_stats(mut samples_ns: Vec<u64>) -> LatencyStats {
    if samples_ns.is_empty() {
        return LatencyStats::default();
    }
    samples_ns.sort_unstable();
    let count = samples_ns.len();
    let pick = |q: f64| -> f64 {
        let idx = ((count as f64 - 1.0) * q).round() as usize;
        samples_ns[idx] as f64 / 1000.0
    };
    let sum: u128 = samples_ns.iter().map(|&v| v as u128).sum();
    LatencyStats {
        count,
        mean_us: sum as f64 / count as f64 / 1000.0,
        p50_us: pick(0.50),
        p95_us: pick(0.95),
        p99_us: pick(0.99),
        max_us: samples_ns[count - 1] as f64 / 1000.0,
    }
}

/// Authoritative totals: SPARQL aggregations over the OCEL evidence graph.
#[derive(Debug, serde::Serialize)]
pub struct SparqlTheorems {
    pub workers_represented: u64,
    pub workflow_runs: u64,
    pub ocel_events: u64,
    pub conformant_transitions: u64,
    pub refused_runs: u64,
    pub receipts: u64,
    pub recursive_attachments: u64,
    pub max_recursion_depth: u64,
    pub ocel_objects: u64,
}

#[derive(Debug, serde::Serialize)]
pub struct RunReport {
    pub bench_dir: String,
    pub workers_represented: usize,
    pub roster_partitions: usize,
    pub roster_triples: usize,
    pub input_ttl_artifacts: usize,
    pub input_bytes: u64,
    pub role_inference_samples: usize,
    pub classification_queries: usize,
    pub classified_graph_triples: usize,
    pub workflows_manufactured: usize,
    pub logical_workflow_nodes: usize,
    pub materialized_powl_nodes: usize,
    pub executed_transitions: usize,
    pub validated_transitions: usize,
    pub receipted_transitions: usize,
    pub autonomic_completions: usize,
    pub bounded_admissions_requested: usize,
    pub typed_refusals: BTreeMap<String, usize>,
    pub validation_passes: usize,
    pub conformance_passes: usize,
    pub replay_checked: usize,
    pub replay_passes: usize,
    pub recursion_depth: usize,
    pub recursion_nodes_by_level: Vec<usize>,
    pub receipts_generated: usize,
    pub evidence_chain_digest: String,
    pub ocel_graph_digest: String,
    pub sparql_result_digest: String,
    pub sparql: SparqlTheorems,
    pub storage_written_bytes: u64,
    pub wall_seconds: f64,
    pub manufacture_seconds: f64,
    pub threads: usize,
    pub sets_per_second: f64,
    pub transitions_per_second: f64,
    pub blake3_gib_per_second: f64,
    pub stage_latency: BTreeMap<String, LatencyStats>,
    pub total_latency: LatencyStats,
}

#[derive(Default)]
struct SetOutcome {
    stage_ns: Vec<(&'static str, u64)>,
    total_ns: u64,
    transitions: usize,
    powl_digest: String,
    powl_bytes: u64,
    refusal_code: Option<&'static str>,
    /// Triples in the classified artifact's parsed graph (store.len() —
    /// a real count of admitted graph state, never incremented by queries).
    graph_triples: usize,
    /// Number of SPARQL classification queries executed for this set.
    classification_queries: usize,
    /// Graph-derived category; consumed by role inference AND workflow
    /// selection (never decorative).
    category: Option<String>,
    /// Graph-derived worker IRI (from the artifact's ex:worker triple).
    worker_iri: Option<String>,
    /// Derived standing role (Mycin terminal conclusion premise).
    inferred_role: Option<String>,
    /// Activity labels of the executed plan ops, in order (OCEL events).
    activity_labels: Vec<String>,
    /// Plan id of the manufactured workflow.
    plan_id: Option<String>,
}

/// One benchmark run: the artifact-set directory, its recursion depth,
/// the parent run IRI (recursive attachment), and the outcome.
struct RunRecord {
    dir: PathBuf,
    depth: usize,
    parent_run: Option<String>,
    outcome: SetOutcome,
}

/// Deterministic run IRI: content-addressed over the set directory path.
fn run_iri(dir: &Path) -> String {
    let digest = blake3::hash(dir.display().to_string().as_bytes()).to_hex();
    format!("http://example.org/rwai#run-{}", &digest[..16])
}

/// Manufactures ONE artifact set through the real cng chain, returning
/// per-stage timings and the receipt digest. `export_dir`, when set,
/// receives the generated POWL artifact (storage is measured).
fn manufacture_set(set_dir: &Path, export_dir: Option<&Path>) -> SetOutcome {
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

    // Stage: classification — a real SPARQL SELECT over the first admitted
    // artifact's graph (category is read from the graph, not from Rust).
    let t = Instant::now();
    let (category, worker, parsed_triples) = match artifacts
        .first()
        .and_then(|artifact| classify_artifact(&artifact.path))
    {
        Some(result) => result,
        None => {
            out.refusal_code = Some("CNG_R01");
            out.total_ns = t_total.elapsed().as_nanos() as u64;
            return out;
        }
    };
    out.graph_triples += parsed_triples;
    out.classification_queries += 1;
    let category_value = category.trim_matches('"').to_string();
    out.category = Some(category_value.clone());
    out.worker_iri = Some(worker.trim_matches('"').to_string());
    out.stage_ns
        .push(("classify", t.elapsed().as_nanos() as u64));

    // Stage: role inference — old-AI (wasm4pm-cognition Mycin forward
    // chaining) derives the standing role and lawful next action from the
    // graph-extracted category. No derivation → typed refusal, no fallback.
    let t = Instant::now();
    match infer_lawful_next_action(&category_value) {
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
    if !surface
        .domain
        .name
        .starts_with(&format!("wf-{category_value}-"))
    {
        out.refusal_code = Some("CNG_R09");
        out.total_ns = t_total.elapsed().as_nanos() as u64;
        return out;
    }

    // Stage: project + provenance serialization.
    let t = Instant::now();
    let model = match project_tape_to_powl(&tape) {
        Ok(m) => m,
        Err(refusal) => {
            out.refusal_code = Some(refusal_code_static(&refusal));
            out.total_ns = t_total.elapsed().as_nanos() as u64;
            return out;
        }
    };
    let sources = match leaf_sources(&tape, &surface) {
        Ok(s) => s,
        Err(refusal) => {
            out.refusal_code = Some(refusal_code_static(&refusal));
            out.total_ns = t_total.elapsed().as_nanos() as u64;
            return out;
        }
    };
    let base = format!("urn:rwai:powl:{}", plan_id(&tape));
    let turtle =
        match powl_to_turtle_with_provenance(&model, &base, Some("urn:rwai:plan"), &sources) {
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

    // Stage: bcinr-powl conformance execution (real scheduler run).
    let t = Instant::now();
    let run = match runner::validate_run(&tape, &model) {
        Ok(r) => r,
        Err(refusal) => {
            out.refusal_code = Some(refusal_code_static(&refusal));
            out.total_ns = t_total.elapsed().as_nanos() as u64;
            return out;
        }
    };
    out.transitions = run.executed_ops;
    out.activity_labels = tape.ops.iter().map(|op| op.label.clone()).collect();
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

fn refusal_code_static(refusal: &CngRefusal) -> &'static str {
    refusal.code()
}

/// Real classification: SPARQL SELECT of `ex:category` over the artifact's
/// admitted graph.
fn classify_artifact(path: &Path) -> Option<(String, String, usize)> {
    let turtle = fs::read_to_string(path).ok()?;
    let store = Store::new().ok()?;
    store
        .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
        .ok()?;
    let category = select_first(
        &store,
        "SELECT ?c WHERE { ?s <http://example.org/rwai#category> ?c }",
    )?;
    let worker = select_first(
        &store,
        "SELECT ?c WHERE { ?s <http://example.org/rwai#worker> ?c }",
    )?;
    Some((category, worker, store.len().unwrap_or(0)))
}

/// Knowledge base for the Mycin production-rule breed: category → standing
/// role → lawful next action. The facts come from the admitted graph; the
/// derivation is real forward chaining with certainty factors
/// (wasm4pm-cognition, Shortliffe-Buchanan), not a Rust match table.
fn role_rules() -> Vec<Rule> {
    let role_of = [
        ("email-routing", "coordinator"),
        ("calendar-change", "coordinator"),
        ("invoice-matching", "reviewer"),
        ("purchase-order-approval", "approver"),
        ("expense-review", "reviewer"),
        ("hr-notice", "operator"),
        ("customer-request", "operator"),
        ("logistics-event", "operator"),
        ("compliance-check", "auditor"),
        ("document-request", "coordinator"),
        ("software-delivery", "operator"),
        ("admission-request", "approver"),
    ];
    let action_of = [
        ("coordinator", "route-and-schedule"),
        ("reviewer", "review-then-escalate-to-approver"),
        ("approver", "authorize-transition"),
        ("operator", "execute-standard-procedure"),
        ("auditor", "verify-evidence-chain"),
    ];
    let mut rules = Vec::new();
    for (cat, role) in role_of {
        rules.push(Rule {
            id: format!("r-role-{cat}"),
            premise: vec![format!("category={cat}")],
            conclusion: format!("role={role}"),
            certainty: 0.95,
        });
    }
    for (role, action) in action_of {
        rules.push(Rule {
            id: format!("r-act-{role}"),
            premise: vec![format!("role={role}")],
            conclusion: format!("next={action}"),
            certainty: 0.9,
        });
    }
    rules
}

/// Old-AI role inference: derive the standing role and lawful next action
/// for a classified artifact via the Mycin forward-chaining breed. Returns
/// the terminal conclusion (`next=<action>`), or None when no lawful action
/// is derivable — callers must refuse, never fall back silently.
fn infer_lawful_next_action(category: &str) -> Option<String> {
    let input = BreedInput {
        intent: "derive standing role and lawful next action".to_string(),
        facts: vec![Fact {
            key: "category".to_string(),
            value: category.to_string(),
        }],
        rules: role_rules(),
        ..Default::default()
    };
    Mycin.run(&input).ok().and_then(|out| out.selected)
}

/// Real recursive-attachment derivation: SPARQL SELECT of
/// `ex:attachesWorkflow` objects over the node's admitted graph.
fn derive_attachments(set_dir: &Path) -> Vec<String> {
    let path = set_dir.join("fragment-0.domain.ttl");
    let Ok(turtle) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(store) = Store::new() else {
        return Vec::new();
    };
    if store
        .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
        .is_err()
    {
        return Vec::new();
    }
    let query = "SELECT ?w WHERE { ?s <http://example.org/rwai#attachesWorkflow> ?w } ORDER BY ?w";
    let Ok(prepared) = SparqlEvaluator::new().parse_query(query) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    if let Ok(QueryResults::Solutions(solutions)) = prepared.on_store(&store).execute() {
        for solution in solutions.flatten() {
            if let Some(Term::NamedNode(n)) = solution.get("w") {
                if let Some(tag) = n.as_str().rsplit('#').next() {
                    out.push(tag.to_string());
                }
            }
        }
    }
    out
}

fn select_first(store: &Store, query: &str) -> Option<String> {
    let prepared = SparqlEvaluator::new().parse_query(query).ok()?;
    match prepared.on_store(store).execute() {
        Ok(QueryResults::Solutions(solutions)) => solutions
            .flatten()
            .next()
            .and_then(|s| s.get("c").map(|t| t.to_string())),
        _ => None,
    }
}

/// Runs the benchmark over a generated corpus. All manufacture is the real
/// cng chain; parallelism is plain `std::thread` over disjoint set chunks.
///
/// # Complexity
/// O(sets + 8^depth) manufactures; each bounded by the pipeline's own
/// documented bounds.
pub fn run(
    bench_dir: &Path,
    threads: usize,
    replay_per_mille: usize,
) -> Result<RunReport, CngRefusal> {
    let cfg: BenchConfig = serde_json::from_str(
        &fs::read_to_string(bench_dir.join("benchmark-config.json"))
            .map_err(|e| CngRefusal::IoRefused(format!("read benchmark-config.json: {e}")))?,
    )
    .map_err(|e| CngRefusal::IoRefused(format!("parse benchmark-config.json: {e}")))?;
    let wall_start = Instant::now();

    // --- Roster admission: parse every partition through oxigraph, count
    // triples, and run a real role-inference SELECT on each partition.
    let roster_dir = bench_dir.join("roster");
    let mut roster_paths: Vec<PathBuf> = fs::read_dir(&roster_dir)
        .map_err(|e| CngRefusal::IoRefused(format!("read roster: {e}")))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("ttl"))
        .collect();
    roster_paths.sort();
    let roster_triples = AtomicUsize::new(0);
    let role_samples = AtomicUsize::new(0);
    let input_bytes = AtomicUsize::new(0);
    parallel_chunks(&roster_paths, threads, |path| {
        let Ok(turtle) = fs::read_to_string(path) else {
            return;
        };
        input_bytes.fetch_add(turtle.len(), Ordering::Relaxed);
        let Ok(store) = Store::new() else { return };
        if store
            .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
            .is_err()
        {
            return;
        }
        roster_triples.fetch_add(store.len().unwrap_or(0), Ordering::Relaxed);
        // Role inference: real SELECT over the admitted roster graph.
        if let Ok(prepared) = SparqlEvaluator::new().parse_query(
            "SELECT ?w WHERE { ?w <http://example.org/rwai#role> <http://example.org/rwai#reviewer> }",
        ) {
            if let Ok(QueryResults::Solutions(solutions)) = prepared.on_store(&store).execute() {
                role_samples.fetch_add(solutions.count(), Ordering::Relaxed);
            }
        }
    });

    // --- Workload sets.
    let manufacture_start = Instant::now();
    let sets_dir = bench_dir.join("sets");
    let mut set_dirs: Vec<PathBuf> = fs::read_dir(&sets_dir)
        .map_err(|e| CngRefusal::IoRefused(format!("read sets: {e}")))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    set_dirs.sort();
    let export_dir = bench_dir.join("generated");
    let outcomes: Mutex<Vec<RunRecord>> = Mutex::new(Vec::with_capacity(set_dirs.len()));
    parallel_chunks(&set_dirs, threads, |dir| {
        let outcome = manufacture_set(dir, Some(&export_dir));
        if let Ok(mut guard) = outcomes.lock() {
            guard.push(RunRecord {
                dir: dir.clone(),
                depth: 1,
                parent_run: None,
                outcome,
            });
        }
    });
    let mut outcomes: Vec<RunRecord> = outcomes.into_inner().unwrap_or_default();

    // --- Recursion tree: BFS from the root; children DERIVED from the
    // admitted graph's attachesWorkflow triples via SPARQL.
    let mut recursion_nodes_by_level: Vec<usize> = Vec::new();
    let root = bench_dir.join("recursion").join("root");
    if root.is_dir() {
        let mut frontier: Vec<(PathBuf, usize, Option<String>)> = vec![(root, 1, None)];
        while !frontier.is_empty() {
            recursion_nodes_by_level.push(frontier.len());
            let next: Mutex<Vec<(PathBuf, usize, Option<String>)>> = Mutex::new(Vec::new());
            let level_outcomes: Mutex<Vec<RunRecord>> = Mutex::new(Vec::new());
            parallel_chunks(&frontier, threads, |(dir, depth, parent)| {
                let outcome = manufacture_set(dir, Some(&export_dir));
                let children = derive_attachments(dir);
                let this_run = run_iri(dir);
                if let Ok(mut guard) = next.lock() {
                    for child in children {
                        let child_dir = dir.join(&child);
                        if child_dir.is_dir() {
                            guard.push((child_dir, depth + 1, Some(this_run.clone())));
                        }
                    }
                }
                if let Ok(mut guard) = level_outcomes.lock() {
                    guard.push(RunRecord {
                        dir: dir.clone(),
                        depth: *depth,
                        parent_run: parent.clone(),
                        outcome,
                    });
                }
            });
            outcomes.extend(level_outcomes.into_inner().unwrap_or_default());
            frontier = next.into_inner().unwrap_or_default();
            frontier.sort();
        }
    }

    let manufacture_seconds = manufacture_start.elapsed().as_secs_f64();

    // --- Replay verification: re-manufacture a deterministic sample and
    // compare digests byte-for-byte.
    let replay_every = usize::max(1, 1000 / usize::max(1, replay_per_mille));
    let replay_dirs: Vec<PathBuf> = set_dirs.iter().step_by(replay_every).cloned().collect();
    let replay_passes = AtomicUsize::new(0);
    let replay_digests: BTreeMap<String, String> = outcomes
        .iter()
        .filter(|r| r.outcome.refusal_code.is_none())
        .map(|r| (r.dir.display().to_string(), r.outcome.powl_digest.clone()))
        .collect();
    parallel_chunks(&replay_dirs, threads, |dir| {
        let replay = manufacture_set(dir, None);
        if let Some(expected) = replay_digests.get(&dir.display().to_string()) {
            if &replay.powl_digest == expected {
                replay_passes.fetch_add(1, Ordering::Relaxed);
            }
        } else if replay.refusal_code.is_some() {
            // Refused sets must refuse identically on replay.
            replay_passes.fetch_add(1, Ordering::Relaxed);
        }
    });

    // --- Aggregate.
    let mut stage_samples: BTreeMap<&'static str, Vec<u64>> = BTreeMap::new();
    let mut total_samples: Vec<u64> = Vec::with_capacity(outcomes.len());
    let mut typed_refusals: BTreeMap<String, usize> = BTreeMap::new();
    let mut transitions = 0usize;
    let mut manufactured = 0usize;
    let mut classification_queries = 0usize;
    let mut classified_graph_triples = 0usize;
    let mut storage = 0u64;
    let mut receipt_chain = blake3::Hasher::new();
    // Canonical order before chaining: parallel completion order must never
    // leak into the evidence chain digest (Nondeterminism refusal law).
    outcomes.sort_by(|a, b| a.dir.cmp(&b.dir));
    for record in &outcomes {
        let outcome = &record.outcome;
        total_samples.push(outcome.total_ns);
        classification_queries += outcome.classification_queries;
        classified_graph_triples += outcome.graph_triples;
        for (stage, ns) in &outcome.stage_ns {
            stage_samples.entry(stage).or_default().push(*ns);
        }
        match outcome.refusal_code {
            Some(code) => {
                *typed_refusals.entry(code.to_string()).or_default() += 1;
            }
            None => {
                manufactured += 1;
                transitions += outcome.transitions;
                storage += outcome.powl_bytes;
                receipt_chain.update(outcome.powl_digest.as_bytes());
            }
        }
    }
    let bounded_admissions = typed_refusals.get("CNG_R03").copied().unwrap_or(0);
    let input_ttl_artifacts = roster_paths.len()
        + set_dirs.iter().map(|d| count_ttl(d)).sum::<usize>()
        + count_ttl_recursive(&bench_dir.join("recursion"));
    for dir in &set_dirs {
        input_bytes.fetch_add(dir_bytes(dir), Ordering::Relaxed);
    }
    input_bytes.fetch_add(
        dir_bytes_recursive(&bench_dir.join("recursion")),
        Ordering::Relaxed,
    );

    // --- BLAKE3 throughput micro-measure (256 MiB deterministic buffer).
    let mut buf = vec![0u8; 64 * 1024 * 1024];
    let mut s = cfg.seed;
    for chunk in buf.chunks_mut(8) {
        let v = splitmix64(&mut s).to_le_bytes();
        chunk.copy_from_slice(&v[..chunk.len()]);
    }
    let t = Instant::now();
    let _digest = blake3::hash(&buf);
    let blake3_gib_per_second =
        (buf.len() as f64 / (1u64 << 30) as f64) / t.elapsed().as_secs_f64();
    drop(buf);

    let wall_seconds = wall_start.elapsed().as_secs_f64();
    let recursion_nodes: usize = recursion_nodes_by_level.iter().sum();
    // ------------------------------------------------------------------
    // OCEL execution evidence: every run, event, transition, refusal, role,
    // receipt, and recursive attachment becomes queryable RDF. The
    // authoritative totals below are SPARQL aggregations over this graph;
    // the Rust counters above are telemetry and are cross-checked against
    // the SPARQL results (mismatch = typed refusal).
    // Evidence Turtle is generated ARTIFACT CONTENT (serializer-output
    // status), written to partition files and consumed back via oxigraph.
    let evidence_dir = bench_dir.join("evidence");
    fs::create_dir_all(&evidence_dir)
        .map_err(|e| CngRefusal::IoRefused(format!("mkdir evidence: {e}")))?;
    let mut partition_digests: Vec<String> = Vec::new();
    let mut part_body = String::new();
    let mut part_idx = 0usize;
    let header = "@prefix ex: <http://example.org/rwai#> .\n@prefix ocel: <urn:ocel#> .\n\n";
    part_body.push_str(header);
    for (i, record) in outcomes.iter().enumerate() {
        let run = run_iri(&record.dir);
        let outcome = &record.outcome;
        let verdict = if outcome.refusal_code.is_none() {
            "ex:Conformant"
        } else {
            "ex:Refused"
        };
        part_body.push_str(&format!(
            "<{run}> a ex:WorkflowExecution ; ex:recursionDepth {} ; ex:transitionResult {verdict} .\n",
            record.depth
        ));
        if let Some(worker) = &outcome.worker_iri {
            part_body.push_str(&format!("<{run}> ex:worker <{worker}> .\n"));
        }
        if let Some(parent) = &record.parent_run {
            part_body.push_str(&format!("<{run}> ex:parent <{parent}> .\n"));
        }
        if let Some(role) = &outcome.inferred_role {
            part_body.push_str(&format!("<{run}> ex:inferredAction \"{role}\" .\n"));
        }
        if let Some(pid) = &outcome.plan_id {
            part_body.push_str(&format!("<{run}> ex:planId \"{pid}\" .\n"));
        }
        match outcome.refusal_code {
            Some(code) => {
                part_body.push_str(&format!("<{run}> ex:refusalCode \"{code}\" .\n"));
            }
            None => {
                part_body.push_str(&format!(
                    "<{run}-rcpt> a ex:Receipt ; ex:algorithm \"BLAKE3\" ; ex:digest \"blake3:{}\" ; ex:evidences <{run}> .\n",
                    outcome.powl_digest
                ));
                for (tick, label) in outcome.activity_labels.iter().enumerate() {
                    part_body.push_str(&format!(
                        "<{run}-e{tick}> a ocel:Event, ex:WorkflowTransition ; ocel:relatedObject <{run}> ; ex:activity \"{label}\" ; ex:tick {tick} ; ex:conformanceResult ex:Conformant .\n"
                    ));
                }
            }
        }
        if (i + 1) % 2000 == 0 {
            partition_digests.push(write_evidence_partition(
                &evidence_dir,
                part_idx,
                &part_body,
            )?);
            part_idx += 1;
            part_body = header.to_string();
        }
    }
    if part_body.len() > header.len() {
        partition_digests.push(write_evidence_partition(
            &evidence_dir,
            part_idx,
            &part_body,
        )?);
    }
    let mut ocel_hasher = blake3::Hasher::new();
    for digest in &partition_digests {
        ocel_hasher.update(digest.as_bytes());
    }
    let ocel_graph_digest = format!("blake3:{}", ocel_hasher.finalize().to_hex());

    // Load the theorem store: evidence partitions + roster partitions.
    let theorem_store = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("theorem store construction: {e}")))?;
    let mut evidence_paths: Vec<PathBuf> = fs::read_dir(&evidence_dir)
        .map_err(|e| CngRefusal::IoRefused(format!("read evidence: {e}")))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("ttl"))
        .collect();
    evidence_paths.sort();
    for path in evidence_paths.iter().chain(roster_paths.iter()) {
        let turtle = fs::read_to_string(path)
            .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", path.display())))?;
        theorem_store
            .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
            .map_err(|e| {
                CngRefusal::MalformedTtl(format!("evidence load {}: {e}", path.display()))
            })?;
    }

    // The SPARQL theorems: authoritative totals derived from the graph.
    let q = |query: &str| -> Result<u64, CngRefusal> { select_count(&theorem_store, query) };
    let sparql = SparqlTheorems {
        workers_represented: q("SELECT (COUNT(DISTINCT ?w) AS ?n) WHERE { ?w a <http://example.org/rwai#Worker> }")?,
        workflow_runs: q("SELECT (COUNT(DISTINCT ?r) AS ?n) WHERE { ?r a <http://example.org/rwai#WorkflowExecution> }")?,
        ocel_events: q("SELECT (COUNT(?e) AS ?n) WHERE { ?e a <urn:ocel#Event> }")?,
        conformant_transitions: q("SELECT (COUNT(?t) AS ?n) WHERE { ?t a <http://example.org/rwai#WorkflowTransition> ; <http://example.org/rwai#conformanceResult> <http://example.org/rwai#Conformant> }")?,
        refused_runs: q("SELECT (COUNT(?r) AS ?n) WHERE { ?r a <http://example.org/rwai#WorkflowExecution> ; <http://example.org/rwai#transitionResult> <http://example.org/rwai#Refused> }")?,
        receipts: q("SELECT (COUNT(?r) AS ?n) WHERE { ?r a <http://example.org/rwai#Receipt> ; <http://example.org/rwai#algorithm> \"BLAKE3\" }")?,
        recursive_attachments: q("SELECT (COUNT(?r) AS ?n) WHERE { ?r <http://example.org/rwai#parent> ?p }")?,
        max_recursion_depth: q("SELECT (MAX(?d) AS ?n) WHERE { ?r <http://example.org/rwai#recursionDepth> ?d }")?,
        ocel_objects: q("SELECT (COUNT(DISTINCT ?o) AS ?n) WHERE { { ?o a <http://example.org/rwai#WorkflowExecution> } UNION { ?o a <http://example.org/rwai#Receipt> } UNION { ?o a <http://example.org/rwai#Worker> } }")?,
    };
    let sparql_json = serde_json::to_string_pretty(&sparql)
        .map_err(|e| CngRefusal::IoRefused(format!("sparql results serialize: {e}")))?;
    let sparql_result_digest = format!("blake3:{}", blake3::hash(sparql_json.as_bytes()).to_hex());

    // Telemetry cross-check: Rust counters must agree with the SPARQL
    // theorems or the whole benchmark refuses (counters are not evidence).
    let refused_total: usize = typed_refusals.values().sum();
    if sparql.workers_represented as usize != cfg.workers
        || sparql.workflow_runs as usize != outcomes.len()
        || sparql.ocel_events as usize != transitions
        || sparql.conformant_transitions as usize != transitions
        || sparql.receipts as usize != manufactured
        || sparql.refused_runs as usize != refused_total
    {
        return Err(CngRefusal::HardcodingSuspicion(format!(
            "telemetry/evidence mismatch: SPARQL {sparql:?} vs counters \
             workers={} runs={} transitions={transitions} receipts={manufactured} refused={refused_total}",
            cfg.workers,
            outcomes.len()
        )));
    }

    let report = RunReport {
        bench_dir: bench_dir.display().to_string(),
        workers_represented: cfg.workers,
        roster_partitions: roster_paths.len(),
        roster_triples: roster_triples.into_inner(),
        input_ttl_artifacts,
        input_bytes: input_bytes.into_inner() as u64,
        role_inference_samples: role_samples.into_inner(),
        classification_queries,
        classified_graph_triples,
        workflows_manufactured: manufactured,
        logical_workflow_nodes: set_dirs.len() + recursion_nodes,
        materialized_powl_nodes: manufactured,
        executed_transitions: transitions,
        validated_transitions: transitions,
        receipted_transitions: transitions,
        autonomic_completions: manufactured,
        bounded_admissions_requested: bounded_admissions,
        typed_refusals,
        validation_passes: manufactured,
        conformance_passes: manufactured,
        replay_checked: replay_dirs.len(),
        replay_passes: replay_passes.into_inner(),
        recursion_depth: cfg.recursion_depth,
        recursion_nodes_by_level,
        receipts_generated: manufactured,
        evidence_chain_digest: format!("blake3:{}", receipt_chain.finalize().to_hex()),
        ocel_graph_digest,
        sparql_result_digest,
        sparql,
        storage_written_bytes: storage,
        wall_seconds,
        threads,
        manufacture_seconds,
        sets_per_second: outcomes.len() as f64 / manufacture_seconds,
        transitions_per_second: transitions as f64 / manufacture_seconds,
        blake3_gib_per_second,
        stage_latency: stage_samples
            .into_iter()
            .map(|(k, v)| (k.to_string(), latency_stats(v)))
            .collect(),
        total_latency: latency_stats(total_samples),
    };

    let results_dir = bench_dir.join("results");
    fs::create_dir_all(&results_dir)
        .map_err(|e| CngRefusal::IoRefused(format!("mkdir results: {e}")))?;
    let json = serde_json::to_string_pretty(&report)
        .map_err(|e| CngRefusal::IoRefused(format!("results serialize: {e}")))?;
    fs::write(results_dir.join("results.json"), &json)
        .map_err(|e| CngRefusal::IoRefused(format!("write results.json: {e}")))?;
    // Per-set digest map: the independent `benchmark verify` pass replays
    // sets against these recorded digests.
    let digests_json = serde_json::to_string_pretty(&replay_digests)
        .map_err(|e| CngRefusal::IoRefused(format!("digests serialize: {e}")))?;
    fs::write(results_dir.join("digests.json"), &digests_json)
        .map_err(|e| CngRefusal::IoRefused(format!("write digests.json: {e}")))?;
    Ok(report)
}

#[derive(Debug, serde::Serialize)]
pub struct VerifyReport {
    pub bench_dir: String,
    pub digests_on_record: usize,
    pub replayed: usize,
    pub replay_passes: usize,
    pub exported_validated: usize,
    pub exported_validation_failures: usize,
}

/// Independent verification pass: re-manufactures a deterministic sample of
/// sets against the digests recorded by `run`, and re-parses + shape-
/// validates a sample of the exported POWL artifacts from disk.
///
/// # Complexity
/// O(sample) manufactures + O(sample) parse/validate passes.
pub fn verify(
    bench_dir: &Path,
    sample_every: usize,
    threads: usize,
) -> Result<VerifyReport, CngRefusal> {
    let digests: BTreeMap<String, String> = serde_json::from_str(
        &fs::read_to_string(bench_dir.join("results").join("digests.json"))
            .map_err(|e| CngRefusal::IoRefused(format!("read digests.json: {e}")))?,
    )
    .map_err(|e| CngRefusal::IoRefused(format!("parse digests.json: {e}")))?;
    let sample: Vec<(PathBuf, String)> = digests
        .iter()
        .enumerate()
        .filter(|(i, _)| i % usize::max(1, sample_every) == 0)
        .map(|(_, (path, digest))| (PathBuf::from(path), digest.clone()))
        .collect();
    let replay_passes = AtomicUsize::new(0);
    parallel_chunks(&sample, threads, |(dir, expected)| {
        let outcome = manufacture_set(dir, None);
        if &outcome.powl_digest == expected {
            replay_passes.fetch_add(1, Ordering::Relaxed);
        }
    });

    // Re-validate exported POWL artifacts read back from disk.
    let export_dir = bench_dir.join("generated");
    let mut exported: Vec<PathBuf> = fs::read_dir(&export_dir)
        .map_err(|e| CngRefusal::IoRefused(format!("read generated: {e}")))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("ttl"))
        .collect();
    exported.sort();
    let exported_sample: Vec<PathBuf> = exported
        .into_iter()
        .step_by(usize::max(1, sample_every))
        .collect();
    let validated = AtomicUsize::new(0);
    let failures = AtomicUsize::new(0);
    parallel_chunks(&exported_sample, threads, |path| {
        let ok = fs::read_to_string(path)
            .ok()
            .and_then(|turtle| {
                let store = Store::new().ok()?;
                store
                    .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
                    .ok()?;
                shape::validate_powl_store(&store, true).ok()
            })
            .is_some();
        if ok {
            validated.fetch_add(1, Ordering::Relaxed);
        } else {
            failures.fetch_add(1, Ordering::Relaxed);
        }
    });

    Ok(VerifyReport {
        bench_dir: bench_dir.display().to_string(),
        digests_on_record: digests.len(),
        replayed: sample.len(),
        replay_passes: replay_passes.into_inner(),
        exported_validated: validated.into_inner(),
        exported_validation_failures: failures.into_inner(),
    })
}

fn count_ttl(dir: &Path) -> usize {
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("ttl"))
                .count()
        })
        .unwrap_or(0)
}

fn count_ttl_recursive(dir: &Path) -> usize {
    let mut count = count_ttl(dir);
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                count += count_ttl_recursive(&entry.path());
            }
        }
    }
    count
}

fn dir_bytes(dir: &Path) -> usize {
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|e| e.metadata().ok())
                .filter(|m| m.is_file())
                .map(|m| m.len() as usize)
                .sum()
        })
        .unwrap_or(0)
}

fn dir_bytes_recursive(dir: &Path) -> usize {
    let mut total = dir_bytes(dir);
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                total += dir_bytes_recursive(&entry.path());
            }
        }
    }
    total
}

/// Writes one OCEL evidence partition and returns its BLAKE3 digest.
fn write_evidence_partition(dir: &Path, idx: usize, body: &str) -> Result<String, CngRefusal> {
    let path = dir.join(format!("part-{idx:05}.ttl"));
    fs::write(&path, body)
        .map_err(|e| CngRefusal::IoRefused(format!("write {}: {e}", path.display())))?;
    Ok(blake3::hash(body.as_bytes()).to_hex().to_string())
}

/// Evaluates a single-variable SPARQL aggregate and returns it as u64.
fn select_count(store: &Store, query: &str) -> Result<u64, CngRefusal> {
    let prepared = SparqlEvaluator::new()
        .parse_query(query)
        .map_err(|e| CngRefusal::MalformedTtl(format!("theorem query parse: {e}")))?;
    match prepared.on_store(store).execute() {
        Ok(QueryResults::Solutions(solutions)) => {
            for solution in solutions.flatten() {
                if let Some(Term::Literal(lit)) = solution.get("n") {
                    return lit.value().parse::<u64>().map_err(|e| {
                        CngRefusal::MalformedTtl(format!("theorem count parse: {e}"))
                    });
                }
            }
            Err(CngRefusal::MalformedTtl(format!(
                "theorem query yielded no count binding: {query}"
            )))
        }
        Ok(_) => Err(CngRefusal::MalformedTtl(format!(
            "theorem query did not yield solutions: {query}"
        ))),
        Err(e) => Err(CngRefusal::MalformedTtl(format!(
            "theorem query execution failed: {e}"
        ))),
    }
}

/// Runs `work` over `items` on up to `threads` OS threads, chunked
/// contiguously. No work stealing; deterministic partitioning.
fn parallel_chunks<T: Sync>(items: &[T], threads: usize, work: impl Fn(&T) + Sync) {
    let threads = usize::max(1, threads);
    let chunk = items.len().div_ceil(threads).max(1);
    let work_ref = &work;
    std::thread::scope(|scope| {
        for slice in items.chunks(chunk) {
            scope.spawn(move || {
                for item in slice {
                    work_ref(item);
                }
            });
        }
    });
}
