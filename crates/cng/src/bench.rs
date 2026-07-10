//! Fortune-5 scale benchmark for Autonomic Recursive Workflow on the REAL
//! cng manufacture chain. Nothing here mocks or bypasses the product path:
//! every workflow goes through `pipeline::import_artifacts` (oxigraph Turtle
//! admission) → `pipeline::generate_plan` (bcinr-pddl grounding + bounded
//! BFS) → `pipeline::hierarchical_projection` → provenance serialization →
//! `shape::validate_powl_store` → `runner::validate_run_hierarchical`
//! (bcinr-powl compile + branchless scheduler + conformance) → BLAKE3
//! receipts.
//!
//! Evidence discipline (Phases 1–4 of the Recursive Workflow benchmark
//! plan):
//! - Every fact the benchmark asserts about itself is first emitted as an
//!   observation (`obs:` vocabulary,
//!   `crates/praxis-graphlaw/ontologies/core/bench-obs.ttl`) rendered from
//!   the `.template.ttl` files — never inline `format!` Turtle.
//! - The OCEL evidence graph is materialized by running the on-disk
//!   `queries/ocel-*.construct.rq` CONSTRUCTs over the observation store;
//!   `ocel_graph_digest` hashes its sorted N-Triples serialization.
//! - Headline `RunReport` numbers are ASSIGNED from the on-disk
//!   `queries/metric-*.rq` SELECTs over the evidence graph; the in-process
//!   Rust counters live in `RunReport.telemetry` and a mismatch with the
//!   graph-derived numbers is a typed refusal (the graph is the authority).
//! - Zero inline SPARQL: classification and attachment discovery are
//!   oxigraph pattern scans over admitted graphs; every SPARQL string is
//!   loaded from the queries directory.
//!
//! Wall-clock timing lives here (benchmark instrumentation), never in the
//! manufacture path itself; digests, receipts, and observation facts
//! contain no time — `obs:obsSeq` is a logical counter.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::model::{GraphName, LiteralRef, NamedNodeRef, Term, TermRef};
use oxigraph::sparql::{QueryResults, SparqlEvaluator};
use oxigraph::store::Store;

use crate::pipeline::{generate_plan, hierarchical_projection, import_artifacts, plan_id};
use crate::powl::{powl_to_turtle_with_phase_provenance, CngRefusal};
use crate::runner;
use crate::shape;
use wasm4pm_cognition::breeds::production_rules::Mycin;
use wasm4pm_cognition::breeds::{BreedInput, CognitionBreed, Fact, Rule};

const WORKERS_PER_ROSTER_PARTITION: usize = 5_000;
const OBS_PER_PARTITION: usize = 4_000;
const RWAI_PREFIX: &str = "http://example.org/rwai#";
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

/// Observation-template kinds, keyed by the `bench-observation-<kind>`
/// template file suffix. `roster` is the generate-time roster variant.
const OBS_KINDS: [&str; 11] = [
    "imported",
    "planned",
    "projected",
    "shape-validated",
    "transition-fired",
    "receipted",
    "refused",
    "roster-admitted",
    "socket-attached",
    "role-derived",
    "roster",
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

/// Fills `{{KEY}}` placeholders in a template. Same mechanism as the PDDL
/// domain/problem template substitution in `write_set`.
///
/// # Complexity
/// O(|template| * |pairs|).
fn fill_template(template: &str, pairs: &[(&str, &str)]) -> String {
    let mut body = template.to_string();
    for (key, value) in pairs {
        body = body.replace(&format!("{{{{{key}}}}}"), value);
    }
    body
}

/// Strips the rwai `ex:` prefix from a full IRI, yielding the local name
/// the observation templates re-prefix as `ex:{{...}}`.
fn rwai_local(iri: &str) -> &str {
    iri.strip_prefix(RWAI_PREFIX).unwrap_or(iri)
}

// ---------------------------------------------------------------------------
// Templates and query set (G-A)
// ---------------------------------------------------------------------------

struct Templates {
    domain: String,
    problem: String,
    /// Observation templates keyed by kind suffix (see [`OBS_KINDS`]).
    obs: BTreeMap<&'static str, String>,
}

fn load_templates() -> Result<Templates, CngRefusal> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
    let read = |name: &str| -> Result<String, CngRefusal> {
        fs::read_to_string(dir.join(name))
            .map_err(|e| CngRefusal::IoRefused(format!("cannot read template {name}: {e}")))
    };
    let mut obs = BTreeMap::new();
    for kind in OBS_KINDS {
        obs.insert(
            kind,
            read(&format!("bench-observation-{kind}.template.ttl"))?,
        );
    }
    Ok(Templates {
        domain: read("bench-domain-fragment.template.ttl")?,
        problem: read("bench-problem.template.ttl")?,
        obs,
    })
}

/// All SPARQL text the benchmark executes, loaded from `.rq` files on disk.
/// No SPARQL string is ever embedded in this module.
pub struct QuerySet {
    queries: BTreeMap<String, String>,
}

impl QuerySet {
    /// Loads every `.rq` file under `dir`, keyed by file stem.
    ///
    /// # Errors
    /// `CNG_R10 IoRefused` when the directory or a file is unreadable.
    ///
    /// # Complexity
    /// O(files) reads.
    pub fn load(dir: &Path) -> Result<QuerySet, CngRefusal> {
        let mut queries = BTreeMap::new();
        let entries = fs::read_dir(dir).map_err(|e| {
            CngRefusal::IoRefused(format!("read queries dir {}: {e}", dir.display()))
        })?;
        for entry in entries {
            let entry =
                entry.map_err(|e| CngRefusal::IoRefused(format!("read queries dir entry: {e}")))?;
            let path = entry.path();
            if path.extension().and_then(|x| x.to_str()) != Some("rq") {
                continue;
            }
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| {
                    CngRefusal::IoRefused(format!("non-UTF8 query filename: {}", path.display()))
                })?
                .to_string();
            let text = fs::read_to_string(&path)
                .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", path.display())))?;
            queries.insert(stem, text);
        }
        Ok(QuerySet { queries })
    }

    /// Query text by file stem.
    ///
    /// # Errors
    /// `CNG_R05 UnsupportedConstruct` naming the missing file.
    pub fn get(&self, name: &str) -> Result<&str, CngRefusal> {
        self.queries.get(name).map(String::as_str).ok_or_else(|| {
            CngRefusal::UnsupportedConstruct(format!(
                "required query {name}.rq is not present in the loaded query set"
            ))
        })
    }

    /// Default queries directory: `<CARGO_MANIFEST_DIR>/queries`.
    pub fn default_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("queries")
    }
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
        // these triples in the admitted graph, never from directory listing.
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
#[allow(clippy::too_many_arguments)]
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

/// Generates the full benchmark corpus: partitioned worker roster as
/// `roster_admitted` observation facts (rendered from the roster
/// observation template — no inline Turtle), per-worker workload artifact
/// sets, and the 8-ary recursion tree.
///
/// # Complexity
/// O(workers + sets + 8^depth) file writes, all seeded/deterministic.
pub fn generate(out_dir: &Path, cfg: &BenchConfig) -> Result<GenerateReport, CngRefusal> {
    let templates = load_templates()?;
    fs::create_dir_all(out_dir)
        .map_err(|e| CngRefusal::IoRefused(format!("mkdir {}: {e}", out_dir.display())))?;
    let mut files = 0usize;
    let mut bytes = 0u64;

    // 1. Roster partitions: every represented worker is a materialized
    //    roster_admitted observation fact set (identity, role, department,
    //    standing) rendered from the roster observation template, written to
    //    disk, and only ever consumed back through oxigraph.
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
    let roster_template = templates.obs.get("roster").ok_or_else(|| {
        CngRefusal::IoRefused("roster observation template missing from loaded set".to_string())
    })?;
    let mut rng = cfg.seed;
    let partitions = cfg.workers.div_ceil(WORKERS_PER_ROSTER_PARTITION);
    for p in 0..partitions {
        let start = p * WORKERS_PER_ROSTER_PARTITION;
        let end = usize::min(start + WORKERS_PER_ROSTER_PARTITION, cfg.workers);
        let mut body = String::with_capacity((end - start) * 360 + 128);
        for w in start..end {
            let role = roles[(splitmix64(&mut rng) % roles.len() as u64) as usize];
            let dept = departments[(splitmix64(&mut rng) % departments.len() as u64) as usize];
            let seq = w.to_string();
            let worker_id = format!("w{w}");
            body.push_str(&fill_template(
                roster_template,
                &[
                    ("SUBJECT", format!("obs-roster-{worker_id}").as_str()),
                    ("SEQ", seq.as_str()),
                    ("SET_ID", "roster"),
                    ("WORKER_ID", worker_id.as_str()),
                    ("ROLE", role),
                    ("DEPARTMENT", dept),
                    ("STANDING", "admitted"),
                ],
            ));
            body.push('\n');
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
            &format!("{RWAI_PREFIX}w{worker}"),
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
            &format!("{RWAI_PREFIX}w{worker}"),
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

/// Authoritative totals: results of the on-disk `metric-*.rq` SELECTs over
/// the materialized OCEL evidence graph (plus the two obs-graph SELECTs).
/// These ASSIGN the `RunReport` headline numbers; the Rust counters are
/// telemetry only.
#[derive(Debug, serde::Serialize)]
pub struct SparqlMetrics {
    /// metric-workers.rq: DISTINCT Worker objects.
    pub workers: u64,
    /// metric-workflow-instances.rq: DISTINCT WorkflowExecution objects.
    pub workflow_instances: u64,
    /// metric-recursive-attachments.rq: DISTINCT attachesWorkflow O2O rels.
    pub recursive_attachments: u64,
    /// metric-transitions.rq: event count per eventTypeName.
    pub transitions_by_type: BTreeMap<String, u64>,
    /// metric-conformance.rq: DISTINCT events with conformant=true.
    pub conformance: u64,
    /// metric-refusals.rq: DISTINCT refused events.
    pub refusals: u64,
    /// metric-receipts.rq: DISTINCT events carrying a receipt.digest.
    pub receipts: u64,
    /// metric-replay.rq: DISTINCT events carrying replay.verified. The pack
    /// emits no such attribute yet, so this is deterministically 0 (see the
    /// query header); replay telemetry lives in `TelemetryCounters`.
    pub replay_verified: u64,
    /// Datalog-derived role count. `None` when the pack provides no
    /// metric-derived-roles.rq (the query file is pack-generated; absent at
    /// HEAD — see the reported seam). The graph-side facts exist either way
    /// as role_derived observations; the telemetry counter is
    /// `datalog_derived_roles`.
    pub derived_roles: Option<u64>,
}

/// In-process Rust counters. NOT authoritative: cross-checked against
/// `SparqlMetrics` (mismatch = typed refusal) and reported for latency /
/// storage / replay context only.
#[derive(Debug, serde::Serialize)]
pub struct TelemetryCounters {
    pub workers_represented: usize,
    pub roster_partitions: usize,
    pub roster_triples: usize,
    pub input_ttl_artifacts: usize,
    pub input_bytes: u64,
    pub datalog_derived_roles: usize,
    pub datalog_derived_facts: usize,
    pub classification_lookups: usize,
    pub classified_graph_triples: usize,
    pub workflows_manufactured: usize,
    pub logical_workflow_nodes: usize,
    pub materialized_powl_nodes: usize,
    pub executed_transitions: usize,
    pub validated_transitions: usize,
    pub receipted_transitions: usize,
    pub socket_attachments: usize,
    pub autonomic_completions: usize,
    pub bounded_admissions_requested: usize,
    pub typed_refusals: BTreeMap<String, usize>,
    pub validation_passes: usize,
    pub conformance_passes: usize,
    pub replay_checked: usize,
    pub replay_passes: usize,
    pub recursion_nodes_by_level: Vec<usize>,
    pub receipts_generated: usize,
    pub storage_written_bytes: u64,
}

#[derive(Debug, serde::Serialize)]
pub struct RunReport {
    /// Measurement class marker: every headline number in this struct was
    /// measured by executing the real cng chain and read back from the
    /// evidence graph via SELECT.
    pub measurement_class: &'static str,
    pub bench_dir: String,
    // --- Headline numbers: ASSIGNED FROM SparqlMetrics (graph authority).
    pub workers_represented: u64,
    pub workflow_instances: u64,
    pub recursive_attachments: u64,
    pub executed_transitions: u64,
    pub validated_transitions: u64,
    pub receipted_transitions: u64,
    pub conformance: u64,
    pub refusals: u64,
    pub receipts: u64,
    pub replay_verified: u64,
    // --- Structure and digests.
    pub recursion_depth: usize,
    pub evidence_chain_digest: String,
    pub ocel_graph_digest: String,
    pub sparql_result_digest: String,
    pub sparql: SparqlMetrics,
    pub telemetry: TelemetryCounters,
    // --- Wall-clock instrumentation (benchmark harness only).
    pub wall_seconds: f64,
    pub manufacture_seconds: f64,
    pub threads: usize,
    pub sets_per_second: f64,
    pub transitions_per_second: f64,
    pub blake3_gib_per_second: f64,
    pub stage_latency: BTreeMap<String, LatencyStats>,
    pub total_latency: LatencyStats,
}

impl RunReport {
    /// The one measurement class this struct may carry.
    pub const MEASUREMENT_CLASS: &'static str = "MEASURED_CNG_RESULT";
}

/// MODELED LLM-agent cost comparison. Never merged into `RunReport`: the
/// only measured inputs are the SELECT-sourced node counts; everything else
/// is a declared assumption (ported from BENCHMARK.md prose into data).
#[derive(Debug, serde::Serialize)]
pub struct ModeledLlmComparison {
    pub measurement_class: &'static str,
    pub assumptions: ModeledLlmAssumptions,
    /// calls * (tokens_in*usd_in + tokens_out*usd_out) / 1e6.
    pub modeled_llm_usd_total: f64,
    pub modeled_llm_usd_per_million_workflows: f64,
    /// measured manufacture CPU-seconds (wall * threads) at usd_per_vcpu_hour.
    pub rwai_measured_cpu_usd_total: f64,
    pub rwai_measured_cpu_usd_per_million_workflows: f64,
}

#[derive(Debug, serde::Serialize)]
pub struct ModeledLlmAssumptions {
    pub llm_calls_per_workflow_step: u64,
    pub tokens_per_call_in: u64,
    pub tokens_per_call_out: u64,
    pub usd_per_mtok_in: f64,
    pub usd_per_mtok_out: f64,
    pub usd_per_vcpu_hour: f64,
    /// SELECT-sourced: metric-transitions transition_fired count × calls-per-step.
    pub calls: u64,
    /// SELECT-sourced: metric-workflow-instances count.
    pub workflow_instances: u64,
}

/// DERIVED arithmetic extrapolation past the generation cap. Never merged
/// into `RunReport`; the cap logic itself (main.rs benchmark_generate)
/// is unchanged.
#[derive(Debug, serde::Serialize)]
pub struct DerivedScaleExtrapolation {
    pub measurement_class: &'static str,
    pub set_cap: usize,
    pub requested_sets: usize,
    pub capped_sets: usize,
    /// Measured sets actually present in this corpus.
    pub measured_sets: usize,
    pub extrapolated_workflow_instances: f64,
    pub extrapolated_transitions: f64,
}

/// Generation-cap constant (mirrors main.rs benchmark_generate clamp).
pub const SET_CAP: usize = 50_000;

#[derive(Default)]
struct SetOutcome {
    stage_ns: Vec<(&'static str, u64)>,
    total_ns: u64,
    transitions: usize,
    powl_digest: String,
    powl_bytes: u64,
    refusal_code: Option<&'static str>,
    /// Triples in the classified artifact's parsed graph (store.len() —
    /// a real count of admitted graph state, never incremented by lookups).
    graph_triples: usize,
    /// Number of graph classification lookups executed for this set.
    classification_lookups: usize,
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
    /// Tape length (planned ops).
    tape_ops: usize,
}

/// One benchmark run: the artifact-set directory, its recursion depth,
/// the parent run IRI (recursive attachment), the socket attachments
/// derived from the admitted graph, and the outcome.
struct RunRecord {
    dir: PathBuf,
    /// (parent activity IRI, child workflow IRI) rows from
    /// attachments-with-parent.rq over this node's observation fragment.
    attachments: Vec<(String, String)>,
    outcome: SetOutcome,
}

/// Deterministic run IRI: content-addressed over the set directory path.
fn run_iri(dir: &Path) -> String {
    let digest = blake3::hash(dir.display().to_string().as_bytes()).to_hex();
    format!("{RWAI_PREFIX}run-{}", &digest[..16])
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

    // Stage: classification — a graph read (oxigraph pattern scan) over the
    // first admitted artifact's graph. The category is read from the graph,
    // not from Rust.
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

fn refusal_code_static(refusal: &CngRefusal) -> &'static str {
    refusal.code()
}

/// Term to plain string: IRI text for named nodes, literal value otherwise.
fn term_value(term: &Term) -> String {
    match term {
        Term::NamedNode(n) => n.as_str().to_string(),
        Term::Literal(l) => l.value().to_string(),
        other => other.to_string(),
    }
}

/// First object of `(any, <predicate>, ?o)` in `store`, as a plain string.
fn first_object(store: &Store, predicate: &str) -> Option<Term> {
    let pred = NamedNodeRef::new(predicate).ok()?;
    store
        .quads_for_pattern(None, Some(pred), None, None)
        .next()?
        .ok()
        .map(|q| q.object)
}

/// Real classification: oxigraph pattern reads of `ex:category` and
/// `ex:worker` over the artifact's admitted graph (no SPARQL text — the
/// benchmark's only SPARQL comes from the on-disk query set).
fn classify_artifact(path: &Path) -> Option<(String, String, usize)> {
    let turtle = fs::read_to_string(path).ok()?;
    let store = Store::new().ok()?;
    store
        .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
        .ok()?;
    let category = first_object(&store, &format!("{RWAI_PREFIX}category"))?;
    let worker = first_object(&store, &format!("{RWAI_PREFIX}worker"))?;
    Some((
        term_value(&category),
        term_value(&worker),
        store.len().unwrap_or(0),
    ))
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

/// Recursive-attachment derivation (G-D): pattern-scans the node's admitted
/// fragment graph for `ex:attachesWorkflow`, projects the pairs into
/// socket_attached observation facts (template-rendered), and reads them
/// back through the on-disk `attachments-with-parent.rq` SELECT over that
/// observation store — KEEPING the `?parentActivity` binding.
///
/// Returns `(parent activity IRI, child workflow IRI)` rows, ordered by
/// child IRI (the query's ORDER BY).
///
/// # Errors
/// Typed refusals for unreadable fragments, template/store failures, or a
/// query that does not yield solutions.
///
/// # Complexity
/// O(attachments) scan + one SELECT over O(attachments) facts.
fn derive_attachments(
    set_dir: &Path,
    templates: &Templates,
    attach_query: &str,
) -> Result<Vec<(String, String)>, CngRefusal> {
    let path = set_dir.join("fragment-0.domain.ttl");
    let turtle = fs::read_to_string(&path)
        .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", path.display())))?;
    let fragment_store =
        Store::new().map_err(|e| CngRefusal::IoRefused(format!("store construction: {e}")))?;
    fragment_store
        .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
        .map_err(|e| CngRefusal::MalformedTtl(format!("fragment load {}: {e}", path.display())))?;

    let attaches_iri = format!("{RWAI_PREFIX}attachesWorkflow");
    let attaches = NamedNodeRef::new(&attaches_iri)
        .map_err(|e| CngRefusal::MalformedTtl(format!("attachesWorkflow IRI: {e}")))?;
    let mut pairs: Vec<(String, String)> = Vec::new();
    for quad in fragment_store.quads_for_pattern(None, Some(attaches), None, None) {
        let quad =
            quad.map_err(|e| CngRefusal::MalformedTtl(format!("fragment pattern scan: {e}")))?;
        let subject = quad.subject.to_string();
        let subject = subject.trim_matches(|c| c == '<' || c == '>').to_string();
        if let Term::NamedNode(child) = quad.object {
            pairs.push((subject, child.as_str().to_string()));
        }
    }
    if pairs.is_empty() {
        return Ok(Vec::new());
    }

    // Project the pairs as socket_attached observation facts and read them
    // back through the on-disk query (preserves the parentActivity binding).
    let socket_template = templates.obs.get("socket-attached").ok_or_else(|| {
        CngRefusal::IoRefused("socket-attached observation template missing".to_string())
    })?;
    let obs_store =
        Store::new().map_err(|e| CngRefusal::IoRefused(format!("store construction: {e}")))?;
    let set_id = rwai_local(&run_iri(set_dir)).to_string();
    for (i, (parent, child)) in pairs.iter().enumerate() {
        let seq = i.to_string();
        let body = fill_template(
            socket_template,
            &[
                ("SUBJECT", format!("obs-attach-{set_id}-{i}").as_str()),
                ("SEQ", seq.as_str()),
                ("SET_ID", set_id.as_str()),
                ("WORKFLOW_ID", set_id.as_str()),
                ("PARENT_ACTIVITY", rwai_local(parent)),
                ("CHILD_WORKFLOW", rwai_local(child)),
            ],
        );
        obs_store
            .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), body.as_bytes())
            .map_err(|e| CngRefusal::MalformedTtl(format!("socket obs load: {e}")))?;
    }
    let rows = select_rows(&obs_store, attach_query)?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let parent = row.get("parentActivity").cloned().ok_or_else(|| {
            CngRefusal::MalformedTtl(
                "attachments-with-parent.rq row missing ?parentActivity".to_string(),
            )
        })?;
        let child = row.get("childWorkflow").cloned().ok_or_else(|| {
            CngRefusal::MalformedTtl(
                "attachments-with-parent.rq row missing ?childWorkflow".to_string(),
            )
        })?;
        out.push((parent, child));
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// SPARQL execution helpers (query text always from the on-disk QuerySet)
// ---------------------------------------------------------------------------

/// Executes a SELECT and returns rows as `variable name → plain value`
/// maps, in the engine's (query-ordered) row order.
fn select_rows(store: &Store, query: &str) -> Result<Vec<BTreeMap<String, String>>, CngRefusal> {
    let prepared = SparqlEvaluator::new()
        .parse_query(query)
        .map_err(|e| CngRefusal::MalformedTtl(format!("query parse: {e}")))?;
    match prepared.on_store(store).execute() {
        Ok(QueryResults::Solutions(solutions)) => {
            let mut rows = Vec::new();
            for solution in solutions {
                let solution =
                    solution.map_err(|e| CngRefusal::MalformedTtl(format!("query eval: {e}")))?;
                let mut row = BTreeMap::new();
                for (var, term) in solution.iter() {
                    row.insert(var.as_str().to_string(), term_value(&term.clone()));
                }
                rows.push(row);
            }
            Ok(rows)
        }
        Ok(_) => Err(CngRefusal::MalformedTtl(
            "query did not yield solutions".to_string(),
        )),
        Err(e) => Err(CngRefusal::MalformedTtl(format!(
            "query execution failed: {e}"
        ))),
    }
}

/// Executes a single-`?count` metric SELECT.
fn metric_count(store: &Store, query: &str, name: &str) -> Result<u64, CngRefusal> {
    let rows = select_rows(store, query)?;
    let row = rows
        .first()
        .ok_or_else(|| CngRefusal::MalformedTtl(format!("{name}: metric query yielded no rows")))?;
    let value = row.get("count").ok_or_else(|| {
        CngRefusal::MalformedTtl(format!("{name}: metric query row has no ?count binding"))
    })?;
    value
        .parse::<u64>()
        .map_err(|e| CngRefusal::MalformedTtl(format!("{name}: count parse: {e}")))
}

/// Executes a CONSTRUCT over `source` and inserts the produced triples
/// into `sink` (default graph). Returns the triple count produced.
fn run_construct(source: &Store, query: &str, sink: &Store) -> Result<usize, CngRefusal> {
    let prepared = SparqlEvaluator::new()
        .parse_query(query)
        .map_err(|e| CngRefusal::MalformedTtl(format!("construct parse: {e}")))?;
    match prepared.on_store(source).execute() {
        Ok(QueryResults::Graph(triples)) => {
            let mut n = 0usize;
            for triple in triples {
                let triple =
                    triple.map_err(|e| CngRefusal::MalformedTtl(format!("construct eval: {e}")))?;
                let quad = triple.in_graph(GraphName::DefaultGraph);
                sink.insert(&quad)
                    .map_err(|e| CngRefusal::IoRefused(format!("evidence insert: {e}")))?;
                n += 1;
            }
            Ok(n)
        }
        Ok(_) => Err(CngRefusal::MalformedTtl(
            "construct query did not yield a graph".to_string(),
        )),
        Err(e) => Err(CngRefusal::MalformedTtl(format!(
            "construct execution failed: {e}"
        ))),
    }
}

// ---------------------------------------------------------------------------
// Observation emission (G-A)
// ---------------------------------------------------------------------------

/// Appends template-rendered observation facts into partitioned `.ttl`
/// artifacts (replay/audit input) and mirrors every fact into the given
/// observation store. `obs:obsSeq` is the writer's monotone logical
/// counter — deterministic, no wall clock.
struct ObsWriter<'a> {
    templates: &'a Templates,
    store: &'a Store,
    dir: PathBuf,
    prefix: &'static str,
    seq: u64,
    buf: String,
    in_buf: usize,
    part_idx: usize,
}

impl<'a> ObsWriter<'a> {
    fn new(
        templates: &'a Templates,
        store: &'a Store,
        dir: &Path,
        prefix: &'static str,
    ) -> Result<Self, CngRefusal> {
        fs::create_dir_all(dir)
            .map_err(|e| CngRefusal::IoRefused(format!("mkdir {}: {e}", dir.display())))?;
        Ok(ObsWriter {
            templates,
            store,
            dir: dir.to_path_buf(),
            prefix,
            seq: 0,
            buf: String::new(),
            in_buf: 0,
            part_idx: 0,
        })
    }

    /// Emits one observation of `kind` with the extra placeholder pairs;
    /// SUBJECT and SEQ are supplied by the writer's monotone counter.
    fn emit(&mut self, kind: &'static str, extra: &[(&str, &str)]) -> Result<(), CngRefusal> {
        let template = self.templates.obs.get(kind).ok_or_else(|| {
            CngRefusal::IoRefused(format!(
                "observation template {kind} missing from loaded set"
            ))
        })?;
        let seq = self.seq;
        self.seq += 1;
        let subject = format!("obs-{}-{seq}", self.prefix);
        let seq_text = seq.to_string();
        let mut pairs: Vec<(&str, &str)> =
            vec![("SUBJECT", subject.as_str()), ("SEQ", seq_text.as_str())];
        pairs.extend_from_slice(extra);
        let body = fill_template(template, &pairs);
        self.store
            .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), body.as_bytes())
            .map_err(|e| CngRefusal::MalformedTtl(format!("observation load ({kind}): {e}")))?;
        self.buf.push_str(&body);
        self.buf.push('\n');
        self.in_buf += 1;
        if self.in_buf >= OBS_PER_PARTITION {
            self.flush()?;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), CngRefusal> {
        if self.in_buf == 0 {
            return Ok(());
        }
        let path = self
            .dir
            .join(format!("{}-part-{:05}.ttl", self.prefix, self.part_idx));
        fs::write(&path, &self.buf)
            .map_err(|e| CngRefusal::IoRefused(format!("write {}: {e}", path.display())))?;
        self.part_idx += 1;
        self.buf.clear();
        self.in_buf = 0;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Datalog role layer (Phase 4)
// ---------------------------------------------------------------------------

/// One roster worker as read back from the admitted observation graph.
struct RosterWorker {
    worker_id: String,
    role: String,
    department: String,
}

/// Reads every roster_admitted observation back out of the observation
/// store via pattern scans (worker id, declared role, department), sorted
/// by worker id.
fn roster_workers(obs_store: &Store) -> Result<Vec<RosterWorker>, CngRefusal> {
    const OBS_PREFIX: &str = "https://ggen.io/ontology/bench-obs#";
    let kind_iri = format!("{OBS_PREFIX}obsKind");
    let worker_iri = format!("{OBS_PREFIX}obsWorkerId");
    let role_iri = format!("{OBS_PREFIX}obsRole");
    let dept_iri = format!("{OBS_PREFIX}obsDepartment");
    fn pred(iri: &str) -> Result<NamedNodeRef<'_>, CngRefusal> {
        NamedNodeRef::new(iri).map_err(|e| CngRefusal::MalformedTtl(format!("{iri}: {e}")))
    }
    let kind_pred = pred(&kind_iri)?;
    let kind_lit = LiteralRef::new_simple_literal("roster_admitted");
    let worker_pred = pred(&worker_iri)?;
    let role_pred = pred(&role_iri)?;
    let dept_pred = pred(&dept_iri)?;

    let mut workers = Vec::new();
    for quad in
        obs_store.quads_for_pattern(None, Some(kind_pred), Some(TermRef::from(kind_lit)), None)
    {
        let quad = quad.map_err(|e| CngRefusal::MalformedTtl(format!("roster scan: {e}")))?;
        let subject = quad.subject;
        let read = |pred: NamedNodeRef<'_>, what: &str| -> Result<String, CngRefusal> {
            obs_store
                .quads_for_pattern(Some(subject.as_ref()), Some(pred), None, None)
                .next()
                .transpose()
                .map_err(|e| CngRefusal::MalformedTtl(format!("roster scan ({what}): {e}")))?
                .map(|q| term_value(&q.object))
                .ok_or_else(|| {
                    CngRefusal::MalformedTtl(format!(
                        "roster_admitted observation {subject} has no {what}"
                    ))
                })
        };
        workers.push(RosterWorker {
            worker_id: read(worker_pred, "obsWorkerId")?,
            role: read(role_pred, "obsRole")?,
            department: read(dept_pred, "obsDepartment")?,
        });
    }
    workers.sort_by(|a, b| a.worker_id.cmp(&b.worker_id));
    Ok(workers)
}

/// Result of the Datalog role derivation layer.
struct DatalogRoles {
    /// worker id → Datalog-derived role.
    derived: BTreeMap<String, String>,
    /// Total derived facts (roles + obligations + custody + closure).
    derived_facts: usize,
}

/// Runs the praxis-graphlaw Datalog engine over the roster fact base with
/// the rules in `rules_text` (`crates/cng/rules/bench-roles.dl`), deriving
/// role/obligation/custody/closure facts. A worker whose Datalog-derived
/// role differs from the roster-declared role is a typed refusal.
///
/// # Complexity
/// O(workers) facts; semi-naive materialization over 8 linear rules.
fn derive_roles_datalog(
    workers: &[RosterWorker],
    rules_text: &str,
) -> Result<DatalogRoles, CngRefusal> {
    use praxis_graphlaw::parser::Parser;
    use praxis_graphlaw::TripleStore;

    let mut doc = String::with_capacity(workers.len() * 64 + rules_text.len());
    for w in workers {
        doc.push_str(&format!(":{} :declaredRole :{}.\n", w.worker_id, w.role));
        doc.push_str(&format!(
            ":{} :department :{}.\n",
            w.worker_id, w.department
        ));
    }
    for line in rules_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        doc.push_str(trimmed);
        doc.push('\n');
    }

    let (facts, rules) = Parser::parse(doc);
    if rules.is_empty() {
        return Err(CngRefusal::UnsupportedConstruct(
            "bench-roles.dl yielded zero parsed Datalog rules".to_string(),
        ));
    }
    let mut store = TripleStore::new();
    for fact in facts {
        store.add(fact);
    }
    store.add_rules(rules).map_err(|e| {
        CngRefusal::UnsupportedConstruct(format!("Datalog rule validation refused: {e}"))
    })?;
    let inferred = store.materialize().map_err(|e| {
        CngRefusal::UnsupportedConstruct(format!("Datalog materialization refused: {e}"))
    })?;

    let decode = |encoded: usize| -> Result<String, CngRefusal> {
        praxis_graphlaw::encoding::Encoder::decode(&encoded)
            .ok_or_else(|| CngRefusal::MalformedTtl("Datalog term failed to decode".to_string()))
    };
    let mut derived: BTreeMap<String, String> = BTreeMap::new();
    for triple in &inferred {
        let predicate = decode(triple.p.to_encoded())?;
        if predicate == ":derivedRole" {
            let worker = decode(triple.s.to_encoded())?;
            let role = decode(triple.o.to_encoded())?;
            derived.insert(
                worker.trim_start_matches(':').to_string(),
                role.trim_start_matches(':').to_string(),
            );
        }
    }
    for w in workers {
        match derived.get(&w.worker_id) {
            Some(role) if role == &w.role => {}
            Some(role) => {
                return Err(CngRefusal::HardcodingSuspicion(format!(
                    "Datalog-derived role {role} for worker {} contradicts the \
                     roster-declared role {}; the roster graph is the admitted input",
                    w.worker_id, w.role
                )));
            }
            None => {
                return Err(CngRefusal::HardcodingSuspicion(format!(
                    "Datalog derived no role for roster worker {}; derivation must \
                     cover every admitted worker",
                    w.worker_id
                )));
            }
        }
    }
    Ok(DatalogRoles {
        derived,
        derived_facts: inferred.len(),
    })
}

// ---------------------------------------------------------------------------
// run
// ---------------------------------------------------------------------------

/// Runs the benchmark over a generated corpus. All manufacture is the real
/// cng chain; parallelism is plain `std::thread` over disjoint set chunks.
/// `queries_dir` overrides the default on-disk query set location.
///
/// # Complexity
/// O(sets + 8^depth) manufactures; each bounded by the pipeline's own
/// documented bounds. Evidence materialization is O(obs facts) per
/// CONSTRUCT; serialization is O(t log t) in evidence triples.
pub fn run(
    bench_dir: &Path,
    threads: usize,
    replay_per_mille: usize,
    queries_dir: Option<&Path>,
) -> Result<RunReport, CngRefusal> {
    let cfg: BenchConfig = serde_json::from_str(
        &fs::read_to_string(bench_dir.join("benchmark-config.json"))
            .map_err(|e| CngRefusal::IoRefused(format!("read benchmark-config.json: {e}")))?,
    )
    .map_err(|e| CngRefusal::IoRefused(format!("parse benchmark-config.json: {e}")))?;
    let templates = load_templates()?;
    let query_dir_owned;
    let query_dir = match queries_dir {
        Some(dir) => dir,
        None => {
            query_dir_owned = QuerySet::default_dir();
            &query_dir_owned
        }
    };
    let queries = QuerySet::load(query_dir)?;
    let wall_start = Instant::now();

    // --- Roster admission: parse every partition (roster_admitted
    // observation facts) through oxigraph into the observation store.
    let roster_dir = bench_dir.join("roster");
    let mut roster_paths: Vec<PathBuf> = fs::read_dir(&roster_dir)
        .map_err(|e| CngRefusal::IoRefused(format!("read roster: {e}")))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("ttl"))
        .collect();
    roster_paths.sort();
    let obs_store = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("observation store construction: {e}")))?;
    let mut input_bytes = 0u64;
    for path in &roster_paths {
        let turtle = fs::read_to_string(path)
            .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", path.display())))?;
        input_bytes += turtle.len() as u64;
        obs_store
            .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
            .map_err(|e| {
                CngRefusal::MalformedTtl(format!("roster load {}: {e}", path.display()))
            })?;
    }
    let roster_triples = obs_store
        .len()
        .map_err(|e| CngRefusal::IoRefused(format!("observation store len: {e}")))?;

    // --- Datalog role layer (Phase 4): derive role/obligation/custody/
    // closure facts from the admitted roster graph; emit role_derived
    // observation facts from the derivations.
    let workers = roster_workers(&obs_store)?;
    let rules_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("rules")
        .join("bench-roles.dl");
    let rules_text = fs::read_to_string(&rules_path)
        .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", rules_path.display())))?;
    let datalog = derive_roles_datalog(&workers, &rules_text)?;
    let obs_dir = bench_dir.join("obs");
    {
        let mut role_writer = ObsWriter::new(&templates, &obs_store, &obs_dir, "role")?;
        for (worker_id, role) in &datalog.derived {
            role_writer.emit(
                "role-derived",
                &[
                    ("SET_ID", "datalog"),
                    ("WORKER_ID", worker_id.as_str()),
                    ("ROLE", role.as_str()),
                ],
            )?;
        }
        role_writer.flush()?;
    }

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
                attachments: Vec::new(),
                outcome,
            });
        }
    });
    let mut outcomes: Vec<RunRecord> = outcomes.into_inner().unwrap_or_default();

    // --- Recursion tree: BFS from the root; children DERIVED from the
    // admitted graph's attachesWorkflow triples via the on-disk
    // attachments-with-parent.rq (parent activity binding preserved).
    let attach_query = queries.get("attachments-with-parent")?.to_string();
    let mut recursion_nodes_by_level: Vec<usize> = Vec::new();
    let root = bench_dir.join("recursion").join("root");
    if root.is_dir() {
        let mut frontier: Vec<PathBuf> = vec![root];
        while !frontier.is_empty() {
            recursion_nodes_by_level.push(frontier.len());
            let next: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());
            let level_outcomes: Mutex<Vec<RunRecord>> = Mutex::new(Vec::new());
            let attach_refusals: Mutex<Vec<CngRefusal>> = Mutex::new(Vec::new());
            parallel_chunks(&frontier, threads, |dir| {
                let outcome = manufacture_set(dir, Some(&export_dir));
                let attachments = match derive_attachments(dir, &templates, &attach_query) {
                    Ok(a) => a,
                    Err(refusal) => {
                        if let Ok(mut guard) = attach_refusals.lock() {
                            guard.push(refusal);
                        }
                        Vec::new()
                    }
                };
                if let Ok(mut guard) = next.lock() {
                    for (_, child_iri) in &attachments {
                        let child_dir = dir.join(rwai_local(child_iri));
                        if child_dir.is_dir() {
                            guard.push(child_dir);
                        }
                    }
                }
                if let Ok(mut guard) = level_outcomes.lock() {
                    guard.push(RunRecord {
                        dir: dir.clone(),
                        attachments,
                        outcome,
                    });
                }
            });
            if let Some(refusal) = attach_refusals.into_inner().unwrap_or_default().pop() {
                return Err(refusal);
            }
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

    // --- Aggregate telemetry counters (canonical dir order first: parallel
    // completion order must never leak into any digest or obs sequence).
    outcomes.sort_by(|a, b| a.dir.cmp(&b.dir));
    let mut stage_samples: BTreeMap<&'static str, Vec<u64>> = BTreeMap::new();
    let mut total_samples: Vec<u64> = Vec::with_capacity(outcomes.len());
    let mut typed_refusals: BTreeMap<String, usize> = BTreeMap::new();
    let mut transitions = 0usize;
    let mut manufactured = 0usize;
    let mut socket_attachments = 0usize;
    let mut classification_lookups = 0usize;
    let mut classified_graph_triples = 0usize;
    let mut storage = 0u64;
    let mut receipt_chain = blake3::Hasher::new();
    for record in &outcomes {
        let outcome = &record.outcome;
        total_samples.push(outcome.total_ns);
        classification_lookups += outcome.classification_lookups;
        classified_graph_triples += outcome.graph_triples;
        socket_attachments += record.attachments.len();
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
    let refused_total: usize = typed_refusals.values().sum();
    let input_ttl_artifacts = roster_paths.len()
        + set_dirs.iter().map(|d| count_ttl(d)).sum::<usize>()
        + count_ttl_recursive(&bench_dir.join("recursion"));
    let mut input_bytes = input_bytes;
    for dir in &set_dirs {
        input_bytes += dir_bytes(dir) as u64;
    }
    input_bytes += dir_bytes_recursive(&bench_dir.join("recursion")) as u64;

    // --- Observation emission (G-A): every run fact becomes an obs:
    // template-rendered fact in partitioned .ttl artifacts (replay input)
    // and in the observation store. Global monotone obsSeq.
    {
        let mut writer = ObsWriter::new(&templates, &obs_store, &obs_dir, "run")?;
        for record in &outcomes {
            let wf_id = rwai_local(&run_iri(&record.dir)).to_string();
            let outcome = &record.outcome;
            writer.emit(
                "imported",
                &[("SET_ID", wf_id.as_str()), ("WORKFLOW_ID", wf_id.as_str())],
            )?;
            if let Some(code) = outcome.refusal_code {
                writer.emit(
                    "refused",
                    &[
                        ("SET_ID", wf_id.as_str()),
                        ("WORKFLOW_ID", wf_id.as_str()),
                        ("REFUSAL_CODE", code),
                    ],
                )?;
                continue;
            }
            let plan = outcome.plan_id.as_deref().ok_or_else(|| {
                CngRefusal::HardcodingSuspicion(format!(
                    "manufactured set {} has no plan id; receipt would be detached",
                    record.dir.display()
                ))
            })?;
            let tape_ops = outcome.tape_ops.to_string();
            let executed = outcome.transitions.to_string();
            writer.emit(
                "planned",
                &[
                    ("SET_ID", wf_id.as_str()),
                    ("WORKFLOW_ID", wf_id.as_str()),
                    ("PLAN_ID", plan),
                    ("TAPE_OPS", tape_ops.as_str()),
                ],
            )?;
            writer.emit(
                "projected",
                &[
                    ("SET_ID", wf_id.as_str()),
                    ("WORKFLOW_ID", wf_id.as_str()),
                    ("PLAN_ID", plan),
                ],
            )?;
            writer.emit(
                "shape-validated",
                &[("SET_ID", wf_id.as_str()), ("WORKFLOW_ID", wf_id.as_str())],
            )?;
            let worker_local = outcome
                .worker_iri
                .as_deref()
                .map(rwai_local)
                .unwrap_or("unattributed");
            for label in &outcome.activity_labels {
                writer.emit(
                    "transition-fired",
                    &[
                        ("SET_ID", wf_id.as_str()),
                        ("WORKFLOW_ID", wf_id.as_str()),
                        ("WORKER_ID", worker_local),
                        ("ACTIVITY_LABEL", label.as_str()),
                    ],
                )?;
            }
            for (parent_activity, child_iri) in &record.attachments {
                writer.emit(
                    "socket-attached",
                    &[
                        ("SET_ID", wf_id.as_str()),
                        ("WORKFLOW_ID", wf_id.as_str()),
                        ("PARENT_ACTIVITY", rwai_local(parent_activity)),
                        ("CHILD_WORKFLOW", rwai_local(child_iri)),
                    ],
                )?;
            }
            if let Some(role) = &outcome.inferred_role {
                writer.emit(
                    "role-derived",
                    &[
                        ("SET_ID", wf_id.as_str()),
                        ("WORKER_ID", worker_local),
                        ("ROLE", role.as_str()),
                    ],
                )?;
            }
            let last_label = outcome
                .activity_labels
                .last()
                .map(String::as_str)
                .unwrap_or("receipt");
            writer.emit(
                "receipted",
                &[
                    ("SET_ID", wf_id.as_str()),
                    ("WORKFLOW_ID", wf_id.as_str()),
                    ("ACTIVITY_LABEL", last_label),
                    ("DIGEST", outcome.powl_digest.as_str()),
                    ("TAPE_OPS", tape_ops.as_str()),
                    ("EXECUTED_OPS", executed.as_str()),
                ],
            )?;
        }
        writer.flush()?;
    }

    // --- BLAKE3 throughput micro-measure (64 MiB deterministic buffer).
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

    // --- OCEL materialization (G-A step 3): run every ocel-*.construct.rq
    // over the observation store into a separate evidence store; serialize
    // deterministically (sorted N-Triples) — that serialization is what
    // ocel_graph_digest hashes.
    let evidence_store = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("evidence store construction: {e}")))?;
    for construct in [
        "ocel-events.construct",
        "ocel-objects.construct",
        "ocel-e2o.construct",
        "ocel-o2o-sockets.construct",
        "ocel-receipts.construct",
        "ocel-log.construct",
    ] {
        run_construct(&obs_store, queries.get(construct)?, &evidence_store)?;
    }
    let mut evidence_lines: Vec<String> = Vec::new();
    for quad in evidence_store.iter() {
        let quad = quad.map_err(|e| CngRefusal::IoRefused(format!("evidence iteration: {e}")))?;
        evidence_lines.push(format!(
            "{} {} {} .",
            quad.subject, quad.predicate, quad.object
        ));
    }
    evidence_lines.sort();
    evidence_lines.dedup();
    let evidence_nt = evidence_lines.join("\n");
    let evidence_dir = bench_dir.join("evidence");
    fs::create_dir_all(&evidence_dir)
        .map_err(|e| CngRefusal::IoRefused(format!("mkdir evidence: {e}")))?;
    fs::write(evidence_dir.join("ocel.nt"), &evidence_nt)
        .map_err(|e| CngRefusal::IoRefused(format!("write ocel.nt: {e}")))?;
    let ocel_graph_digest = format!("blake3:{}", blake3::hash(evidence_nt.as_bytes()).to_hex());

    // --- Graph-derived metrics (G-C): the on-disk metric SELECTs are the
    // authority for every headline number.
    let mut metric_rows: BTreeMap<String, Vec<BTreeMap<String, String>>> = BTreeMap::new();
    for name in [
        "metric-workers",
        "metric-workflow-instances",
        "metric-recursive-attachments",
        "metric-transitions",
        "metric-conformance",
        "metric-refusals",
        "metric-receipts",
        "metric-replay",
    ] {
        metric_rows.insert(
            name.to_string(),
            select_rows(&evidence_store, queries.get(name)?)?,
        );
    }
    // metric-derived-roles.rq is pack-generated and runs over the OBS graph;
    // absent at HEAD → explicitly None, never a silent zero.
    let derived_roles = match queries.get("metric-derived-roles") {
        Ok(query) => {
            let rows = select_rows(&obs_store, query)?;
            metric_rows.insert("metric-derived-roles".to_string(), rows);
            Some(metric_count(&obs_store, query, "metric-derived-roles")?)
        }
        Err(_) => None,
    };
    let count_of = |name: &str| -> Result<u64, CngRefusal> {
        metric_count(&evidence_store, queries.get(name)?, name)
    };
    let mut transitions_by_type: BTreeMap<String, u64> = BTreeMap::new();
    for row in metric_rows
        .get("metric-transitions")
        .map(Vec::as_slice)
        .unwrap_or(&[])
    {
        let kind = row.get("type").cloned().ok_or_else(|| {
            CngRefusal::MalformedTtl("metric-transitions row missing ?type".to_string())
        })?;
        let count = row
            .get("count")
            .ok_or_else(|| {
                CngRefusal::MalformedTtl("metric-transitions row missing ?count".to_string())
            })?
            .parse::<u64>()
            .map_err(|e| CngRefusal::MalformedTtl(format!("metric-transitions count: {e}")))?;
        transitions_by_type.insert(kind, count);
    }
    let sparql = SparqlMetrics {
        workers: count_of("metric-workers")?,
        workflow_instances: count_of("metric-workflow-instances")?,
        recursive_attachments: count_of("metric-recursive-attachments")?,
        transitions_by_type,
        conformance: count_of("metric-conformance")?,
        refusals: count_of("metric-refusals")?,
        receipts: count_of("metric-receipts")?,
        replay_verified: count_of("metric-replay")?,
        derived_roles,
    };
    // sparql_result_digest hashes the full ordered SELECT results.
    let mut digest_text = String::new();
    for (name, rows) in &metric_rows {
        digest_text.push_str(name);
        digest_text.push('\n');
        for row in rows {
            for (var, value) in row {
                digest_text.push_str(&format!("{var}={value};"));
            }
            digest_text.push('\n');
        }
    }
    let sparql_result_digest = format!("blake3:{}", blake3::hash(digest_text.as_bytes()).to_hex());

    // --- Reconcile gate: telemetry counters must agree with the
    // graph-derived numbers, or the whole benchmark refuses. The evidence
    // graph is the authority; the counters are only telemetry.
    let fired = sparql
        .transitions_by_type
        .get("transition_fired")
        .copied()
        .unwrap_or(0);
    let derived_roles_agree = sparql
        .derived_roles
        .map(|n| n as usize == datalog.derived.len())
        .unwrap_or(true);
    if sparql.workers as usize != cfg.workers
        || sparql.workflow_instances as usize != outcomes.len()
        || fired as usize != transitions
        || sparql.receipts as usize != manufactured
        || sparql.conformance as usize != manufactured
        || sparql.refusals as usize != refused_total
        || sparql.recursive_attachments as usize != socket_attachments
        || !derived_roles_agree
    {
        return Err(CngRefusal::HardcodingSuspicion(format!(
            "telemetry/evidence mismatch — the SPARQL evidence graph is the authority: \
             graph {sparql:?} vs telemetry counters workers={} runs={} transitions={transitions} \
             receipts={manufactured} refused={refused_total} sockets={socket_attachments} \
             datalog_roles={}",
            cfg.workers,
            outcomes.len(),
            datalog.derived.len()
        )));
    }

    let wall_seconds = wall_start.elapsed().as_secs_f64();
    let telemetry = TelemetryCounters {
        workers_represented: cfg.workers,
        roster_partitions: roster_paths.len(),
        roster_triples,
        input_ttl_artifacts,
        input_bytes,
        datalog_derived_roles: datalog.derived.len(),
        datalog_derived_facts: datalog.derived_facts,
        classification_lookups,
        classified_graph_triples,
        workflows_manufactured: manufactured,
        logical_workflow_nodes: set_dirs.len() + recursion_nodes_by_level.iter().sum::<usize>(),
        materialized_powl_nodes: manufactured,
        executed_transitions: transitions,
        validated_transitions: transitions,
        receipted_transitions: transitions,
        socket_attachments,
        autonomic_completions: manufactured,
        bounded_admissions_requested: bounded_admissions,
        typed_refusals,
        validation_passes: manufactured,
        conformance_passes: manufactured,
        replay_checked: replay_dirs.len(),
        replay_passes: replay_passes.into_inner(),
        recursion_nodes_by_level,
        receipts_generated: manufactured,
        storage_written_bytes: storage,
    };
    let report = RunReport {
        measurement_class: RunReport::MEASUREMENT_CLASS,
        bench_dir: bench_dir.display().to_string(),
        workers_represented: sparql.workers,
        workflow_instances: sparql.workflow_instances,
        recursive_attachments: sparql.recursive_attachments,
        executed_transitions: fired,
        validated_transitions: fired,
        receipted_transitions: fired,
        conformance: sparql.conformance,
        refusals: sparql.refusals,
        receipts: sparql.receipts,
        replay_verified: sparql.replay_verified,
        recursion_depth: cfg.recursion_depth,
        evidence_chain_digest: format!("blake3:{}", receipt_chain.finalize().to_hex()),
        ocel_graph_digest,
        sparql_result_digest,
        sparql,
        telemetry,
        wall_seconds,
        manufacture_seconds,
        threads,
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
    // sets against these recorded digests. (Field shape unchanged.)
    let digests_json = serde_json::to_string_pretty(&replay_digests)
        .map_err(|e| CngRefusal::IoRefused(format!("digests serialize: {e}")))?;
    fs::write(results_dir.join("digests.json"), &digests_json)
        .map_err(|e| CngRefusal::IoRefused(format!("write digests.json: {e}")))?;

    // --- MODELED_LLM_COMPARISON (G-B): assumptions + arithmetic ported
    // from BENCHMARK.md prose into machine-readable data. Calls are the
    // SELECT-sourced transition count; never merged into RunReport.
    let assumptions = ModeledLlmAssumptions {
        llm_calls_per_workflow_step: 3,
        tokens_per_call_in: 2_000,
        tokens_per_call_out: 500,
        usd_per_mtok_in: 3.0,
        usd_per_mtok_out: 15.0,
        usd_per_vcpu_hour: 0.05,
        calls: report.executed_transitions * 3,
        workflow_instances: report.workflow_instances,
    };
    let usd_per_call = (assumptions.tokens_per_call_in as f64 * assumptions.usd_per_mtok_in
        + assumptions.tokens_per_call_out as f64 * assumptions.usd_per_mtok_out)
        / 1_000_000.0;
    let modeled_llm_usd_total = assumptions.calls as f64 * usd_per_call;
    let rwai_cpu_usd_total =
        report.manufacture_seconds * report.threads as f64 / 3600.0 * assumptions.usd_per_vcpu_hour;
    let per_million = |usd_total: f64| -> f64 {
        if report.workflow_instances == 0 {
            0.0
        } else {
            usd_total / report.workflow_instances as f64 * 1_000_000.0
        }
    };
    let modeled = ModeledLlmComparison {
        measurement_class: "MODELED_LLM_COMPARISON",
        modeled_llm_usd_per_million_workflows: per_million(modeled_llm_usd_total),
        rwai_measured_cpu_usd_per_million_workflows: per_million(rwai_cpu_usd_total),
        modeled_llm_usd_total,
        rwai_measured_cpu_usd_total: rwai_cpu_usd_total,
        assumptions,
    };
    let modeled_json = serde_json::to_string_pretty(&modeled)
        .map_err(|e| CngRefusal::IoRefused(format!("modeled comparison serialize: {e}")))?;
    fs::write(
        results_dir.join("modeled-llm-comparison.json"),
        &modeled_json,
    )
    .map_err(|e| CngRefusal::IoRefused(format!("write modeled-llm-comparison.json: {e}")))?;

    // --- DERIVED_ARITHMETIC scale extrapolation (G-G): the generation cap
    // (main.rs benchmark_generate clamp) is unchanged; this file records
    // what pure arithmetic says about the uncapped request.
    let requested_sets = usize::max(64, cfg.workers / 100);
    let capped_sets = usize::min(requested_sets, SET_CAP);
    let measured_sets = set_dirs.len();
    let ratio = if measured_sets == 0 {
        0.0
    } else {
        requested_sets as f64 / measured_sets as f64
    };
    let derived_scale = DerivedScaleExtrapolation {
        measurement_class: "DERIVED_ARITHMETIC",
        set_cap: SET_CAP,
        requested_sets,
        capped_sets,
        measured_sets,
        extrapolated_workflow_instances: report.workflow_instances as f64 * ratio,
        extrapolated_transitions: report.executed_transitions as f64 * ratio,
    };
    let scale_json = serde_json::to_string_pretty(&derived_scale)
        .map_err(|e| CngRefusal::IoRefused(format!("derived scale serialize: {e}")))?;
    fs::write(results_dir.join("derived-scale.json"), &scale_json)
        .map_err(|e| CngRefusal::IoRefused(format!("write derived-scale.json: {e}")))?;

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

#[cfg(test)]
mod tests {
    use super::*;

    /// The Phase-0 fixture (one observation per kind) must satisfy every
    /// CONSTRUCT + metric SELECT contract end to end.
    #[test]
    fn fixture_obs_materialize_and_count() {
        let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/bench-obs/sample-observations.ttl");
        let turtle = fs::read_to_string(&fixture).expect("fixture readable");
        let obs = Store::new().expect("store");
        obs.load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
            .expect("fixture parses");
        let evidence = Store::new().expect("store");
        for construct in [
            "ocel-events.construct",
            "ocel-objects.construct",
            "ocel-e2o.construct",
            "ocel-o2o-sockets.construct",
            "ocel-receipts.construct",
            "ocel-log.construct",
        ] {
            run_construct(&obs, queries.get(construct).expect("query"), &evidence)
                .expect("construct runs");
        }
        // Fixture: 1 worker, 3 workflow ids (wf-A, wf-B via socket, wf-C).
        let count = |name: &str| {
            metric_count(&evidence, queries.get(name).expect("query"), name).expect("count")
        };
        assert_eq!(count("metric-workers"), 1);
        assert_eq!(count("metric-recursive-attachments"), 1);
        assert_eq!(count("metric-receipts"), 1);
        assert_eq!(count("metric-refusals"), 1);
        assert_eq!(count("metric-conformance"), 1);
        assert_eq!(count("metric-replay"), 0);
        // attachments-with-parent runs over the OBS graph and keeps the
        // parentActivity binding.
        let rows = select_rows(&obs, queries.get("attachments-with-parent").expect("query"))
            .expect("rows");
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].get("parentActivity").map(String::as_str),
            Some("http://example.org/rwai#activity-step-1")
        );
    }
}
