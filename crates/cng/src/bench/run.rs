//! Top of the benchmark dependency graph: `run()` — roster admission,
//! Datalog role derivation, parallel manufacture (workload sets + recursion
//! tree BFS), replay sampling, observation emission, OCEL materialization,
//! graph-derived metrics, the telemetry/evidence reconcile gate, and the
//! self-contained evidence bundle (manifest + modeled/derived artifacts).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Instant;

use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::store::Store;

use crate::powl::CngRefusal;

use super::manufacture::{manufacture_set, run_iri, RunRecord, SetOutcome};
use super::report::{
    latency_stats, DerivedScaleExtrapolation, EvidenceManifest, ModeledLlmAssumptions,
    ModeledLlmComparison, RunReport, SparqlMetrics, TelemetryCounters, SET_CAP,
};
use super::roles::{
    derive_attachments, derive_roles_datalog, roster_workers, run_construct, select_rows, ObsWriter,
};
use super::templates::{load_templates, QuerySet};
use super::verify::{count_ttl, count_ttl_recursive, dir_bytes, dir_bytes_recursive};
use super::{parallel_chunks, rwai_local, splitmix64};

/// The OCEL CONSTRUCT query stems, in the fixed order `run()` and
/// `audit_replay()` both materialize them — order is part of the contract
/// only insofar as it's identical between producer and auditor (the final
/// serialization is sorted, so materialization order does not affect the
/// digest, but a fixed list keeps both paths byte-for-byte the same code).
pub(super) const OCEL_CONSTRUCT_STEMS: [&str; 10] = [
    "ocel-events.construct",
    "ocel-objects.construct",
    "ocel-e2o.construct",
    "ocel-o2o-sockets.construct",
    "ocel-receipts.construct",
    "ocel-log.construct",
    // PROJ-611: admission_requested/admission_granted/resumed events from
    // the workday bounded-admission → resume loop. Yields zero triples on a
    // Fortune-5 corpus (no such obs kinds), so existing digests stay
    // producer/auditor-consistent.
    "ocel-admissions.construct",
    // PROJ-612/613: hook_receipt/hook_standing events from the workday
    // hook broker. Zero triples on a Fortune-5 corpus (no such obs kinds),
    // so existing digests stay producer/auditor-consistent.
    "ocel-hook-receipts.construct",
    // PROJ-619/620: dispatch_sent/acknowledged/poll, consequence_returned/
    // admitted/refused, dispatch_timed_out, remediation_manufactured events
    // from the external-dispatch broker surface. Zero triples on a
    // Fortune-5 corpus (no such obs kinds), so existing digests stay
    // producer/auditor-consistent.
    "ocel-dispatches.construct",
    // PROJ-614: replay_verified events (replay re-manufacture reproduced
    // the recorded POWL digest) carrying the `replay.verified` ocel
    // attribute metric-replay.rq counts. Zero triples on corpora without
    // replay_verified obs, so pre-existing digests stay consistent.
    "ocel-replays.construct",
];

/// Serializes an evidence store as sorted, deduplicated N-Triples and
/// BLAKE3-hashes the result. Shared by `run()` (which computes
/// `ocel_graph_digest`) and `audit_replay()` (which recomputes it from a
/// bundle to compare against the manifest) — one implementation, zero drift.
///
/// # Complexity
/// O(t log t) in evidence triples for the sort.
pub(super) fn evidence_digest(store: &Store) -> Result<(String, String), CngRefusal> {
    let mut lines: Vec<String> = Vec::new();
    for quad in store.iter() {
        let quad = quad.map_err(|e| CngRefusal::IoRefused(format!("evidence iteration: {e}")))?;
        lines.push(format!(
            "{} {} {} .",
            quad.subject, quad.predicate, quad.object
        ));
    }
    lines.sort();
    lines.dedup();
    let nt = lines.join("\n");
    let digest = format!("blake3:{}", blake3::hash(nt.as_bytes()).to_hex());
    Ok((nt, digest))
}

/// BLAKE3 digest over every `.ttl` file under `<bench_dir>/obs/`, sorted by
/// bench-dir-relative path. Feeds the digest with, per file: the relative
/// path bytes, a single `0u8` separator, then the file bytes.
///
/// # Complexity
/// O(obs bytes log(files)) for the sort.
pub(super) fn obs_dir_digest(bench_dir: &Path) -> Result<String, CngRefusal> {
    let obs_dir = bench_dir.join("obs");
    let mut paths = Vec::new();
    super::roles::collect_ttl_paths_recursive(&obs_dir, &mut paths)?;
    let mut rel_paths: Vec<(String, PathBuf)> = paths
        .into_iter()
        .map(|p| {
            let rel = p
                .strip_prefix(bench_dir)
                .map_err(|_| {
                    CngRefusal::IoRefused(format!(
                        "obs file {} is not under bench dir {}",
                        p.display(),
                        bench_dir.display()
                    ))
                })?
                .display()
                .to_string();
            Ok((rel, p))
        })
        .collect::<Result<_, CngRefusal>>()?;
    rel_paths.sort_by(|a, b| a.0.cmp(&b.0));
    let mut hasher = blake3::Hasher::new();
    for (rel, path) in &rel_paths {
        hasher.update(rel.as_bytes());
        hasher.update(&[0u8]);
        let bytes = fs::read(path)
            .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", path.display())))?;
        hasher.update(&bytes);
    }
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

/// Emits the standard per-record observation sequence for one manufacture
/// outcome (imported → refused | planned/projected/shape-validated/
/// transition-fired.../socket-attached.../role-derived/receipted). Shared
/// by `run()` (Fortune-5, wf_id = run IRI local, tick = 0 — the Fortune-5
/// path has no workday clock, so its `ex:logicalTick` is the constant 0)
/// and `workday()` (wf_id = tick set id, tick = the logical workday tick)
/// — one implementation, zero drift.
///
/// # Complexity
/// O(transitions + attachments) template emissions per record.
pub(super) fn emit_record_observations(
    writer: &mut ObsWriter<'_>,
    wf_id: &str,
    outcome: &SetOutcome,
    attachments: &[(String, String)],
    tick: usize,
) -> Result<(), CngRefusal> {
    let tick_text = tick.to_string();
    writer.emit("imported", &[("SET_ID", wf_id), ("WORKFLOW_ID", wf_id)])?;
    if let Some(code) = outcome.refusal_code {
        writer.emit(
            "refused",
            &[
                ("SET_ID", wf_id),
                ("WORKFLOW_ID", wf_id),
                ("REFUSAL_CODE", code),
            ],
        )?;
        return Ok(());
    }
    let plan = outcome.plan_id.as_deref().ok_or_else(|| {
        CngRefusal::HardcodingSuspicion(format!(
            "manufactured set {wf_id} has no plan id; receipt would be detached"
        ))
    })?;
    let tape_ops = outcome.tape_ops.to_string();
    let executed = outcome.transitions.to_string();
    writer.emit(
        "planned",
        &[
            ("SET_ID", wf_id),
            ("WORKFLOW_ID", wf_id),
            ("PLAN_ID", plan),
            ("TAPE_OPS", tape_ops.as_str()),
            ("TICK", tick_text.as_str()),
        ],
    )?;
    writer.emit(
        "projected",
        &[("SET_ID", wf_id), ("WORKFLOW_ID", wf_id), ("PLAN_ID", plan)],
    )?;
    writer.emit(
        "shape-validated",
        &[("SET_ID", wf_id), ("WORKFLOW_ID", wf_id)],
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
                ("SET_ID", wf_id),
                ("WORKFLOW_ID", wf_id),
                ("WORKER_ID", worker_local),
                ("ACTIVITY_LABEL", label.as_str()),
                ("TICK", tick_text.as_str()),
            ],
        )?;
    }
    for (parent_activity, child_iri) in attachments {
        writer.emit(
            "socket-attached",
            &[
                ("SET_ID", wf_id),
                ("WORKFLOW_ID", wf_id),
                ("PARENT_ACTIVITY", rwai_local(parent_activity)),
                ("CHILD_WORKFLOW", rwai_local(child_iri)),
            ],
        )?;
    }
    if let Some(role) = &outcome.inferred_role {
        writer.emit(
            "role-derived",
            &[
                ("SET_ID", wf_id),
                ("WORKER_ID", worker_local),
                ("ROLE", role.as_str()),
                ("TICK", tick_text.as_str()),
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
            ("SET_ID", wf_id),
            ("WORKFLOW_ID", wf_id),
            ("ACTIVITY_LABEL", last_label),
            ("DIGEST", outcome.powl_digest.as_str()),
            ("TAPE_OPS", tape_ops.as_str()),
            ("EXECUTED_OPS", executed.as_str()),
            ("TICK", tick_text.as_str()),
        ],
    )?;
    Ok(())
}

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
    let cfg: super::BenchConfig = serde_json::from_str(
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
                    // Fortune-5 has no workday clock; logical tick 0.
                    ("TICK", "0"),
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
    // Passing replays are recorded per set (dir + reproduced digest) so the
    // observation emission below can receipt each verification as a
    // replay_verified obs fact (PROJ-614: metric-replay.rq then counts them
    // from the evidence graph — the graph, not this Vec, is the authority).
    let replay_pass_records: Mutex<Vec<(PathBuf, String)>> = Mutex::new(Vec::new());
    // Per-set digest map keyed RELATIVE to bench_dir so the whole directory
    // is relocatable: `benchmark verify` rejoins keys against its own --dir.
    // Sets outside bench_dir would be a generator bug; refuse loudly.
    let replay_digests: BTreeMap<String, String> = outcomes
        .iter()
        .filter(|r| r.outcome.refusal_code.is_none())
        .map(|r| {
            let rel = r.dir.strip_prefix(bench_dir).map_err(|_| {
                CngRefusal::HardcodingSuspicion(format!(
                    "set dir {} is not under bench dir {}; digests.json keys must be \
                     bench-dir-relative for portable replay",
                    r.dir.display(),
                    bench_dir.display()
                ))
            })?;
            Ok((rel.display().to_string(), r.outcome.powl_digest.clone()))
        })
        .collect::<Result<_, CngRefusal>>()?;
    parallel_chunks(&replay_dirs, threads, |dir| {
        let replay = manufacture_set(dir, None);
        let rel_key = dir
            .strip_prefix(bench_dir)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| dir.display().to_string());
        if let Some(expected) = replay_digests.get(&rel_key) {
            if &replay.powl_digest == expected {
                if let Ok(mut guard) = replay_pass_records.lock() {
                    guard.push((dir.clone(), expected.clone()));
                }
            }
        } else if let Some(code) = replay.refusal_code {
            // Refused sets must refuse identically on replay; the reproduced
            // "digest" recorded for the obs fact is the refusal code.
            if let Ok(mut guard) = replay_pass_records.lock() {
                guard.push((dir.clone(), format!("refused:{code}")));
            }
        }
    });
    // Canonical order: parallel completion order must never leak into the
    // obs sequence (and thus into any digest).
    let mut replay_pass_records: Vec<(PathBuf, String)> =
        replay_pass_records.into_inner().unwrap_or_default();
    replay_pass_records.sort_by(|a, b| a.0.cmp(&b.0));
    let replay_pass_count = replay_pass_records.len();

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
            emit_record_observations(&mut writer, &wf_id, &record.outcome, &record.attachments, 0)?;
        }
        // PROJ-614: every passing replay is receipted as a replay_verified
        // observation; ocel-replays.construct.rq projects these into
        // `replay.verified`-attributed events that metric-replay.rq counts.
        //
        // # Complexity
        // O(passing replays) template emissions.
        for (dir, digest) in &replay_pass_records {
            let wf_id = rwai_local(&run_iri(dir)).to_string();
            writer.emit(
                "replay-verified",
                &[("SET_ID", wf_id.as_str()), ("DIGEST", digest.as_str())],
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
    for construct in OCEL_CONSTRUCT_STEMS {
        run_construct(&obs_store, queries.get(construct)?, &evidence_store)?;
    }
    let (evidence_nt, ocel_graph_digest) = evidence_digest(&evidence_store)?;
    let evidence_dir = bench_dir.join("evidence");
    fs::create_dir_all(&evidence_dir)
        .map_err(|e| CngRefusal::IoRefused(format!("mkdir evidence: {e}")))?;
    fs::write(evidence_dir.join("ocel.nt"), &evidence_nt)
        .map_err(|e| CngRefusal::IoRefused(format!("write ocel.nt: {e}")))?;

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
    // metric-derived-roles.rq is pack-generated (ocel-bench-pack) and runs
    // over the OBS graph; it ships on disk at HEAD (PROJ-614), but a bundle
    // built before it landed may not carry it → explicitly None, never a
    // silent zero.
    let derived_roles = match queries.get("metric-derived-roles") {
        Ok(query) => {
            let rows = select_rows(&obs_store, query)?;
            metric_rows.insert("metric-derived-roles".to_string(), rows);
            Some(super::roles::metric_count(
                &obs_store,
                query,
                "metric-derived-roles",
            )?)
        }
        Err(_) => None,
    };
    let count_of = |name: &str| -> Result<u64, CngRefusal> {
        super::roles::metric_count(&evidence_store, queries.get(name)?, name)
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
        // PROJ-614: the replay headline is graph-derived (metric-replay.rq
        // over replay_verified events); the Rust pass counter is telemetry.
        || sparql.replay_verified as usize != replay_pass_count
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
        replay_passes: replay_pass_count,
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

    // --- Evidence bundle manifest (PROJ-603): self-contained copies of
    // every query, ontology, and rule file the run consumed, plus their
    // digests, so an independent auditor can replay from `bench_dir` alone.
    let obs_digest = obs_dir_digest(bench_dir)?;
    let query_digests = queries.digests();
    let bundled_queries_dir = bench_dir.join("queries");
    fs::create_dir_all(&bundled_queries_dir).map_err(|e| {
        CngRefusal::IoRefused(format!("mkdir {}: {e}", bundled_queries_dir.display()))
    })?;
    for (stem, text) in &queries.queries {
        let path = bundled_queries_dir.join(format!("{stem}.rq"));
        fs::write(&path, text)
            .map_err(|e| CngRefusal::IoRefused(format!("write {}: {e}", path.display())))?;
    }

    let ontology_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("praxis-graphlaw")
        .join("ontologies")
        .join("core");
    let bundled_ontology_dir = bench_dir.join("ontology");
    fs::create_dir_all(&bundled_ontology_dir).map_err(|e| {
        CngRefusal::IoRefused(format!("mkdir {}: {e}", bundled_ontology_dir.display()))
    })?;
    let mut ontology_digests: BTreeMap<String, String> = BTreeMap::new();
    for name in ["ocel2.ttl", "bench-obs.ttl"] {
        let src = ontology_dir.join(name);
        let bytes = fs::read(&src)
            .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", src.display())))?;
        ontology_digests.insert(
            name.to_string(),
            format!("blake3:{}", blake3::hash(&bytes).to_hex()),
        );
        let dst = bundled_ontology_dir.join(name);
        fs::write(&dst, &bytes)
            .map_err(|e| CngRefusal::IoRefused(format!("write {}: {e}", dst.display())))?;
    }

    let rules_digest = format!("blake3:{}", blake3::hash(rules_text.as_bytes()).to_hex());
    let bundled_rules_dir = bench_dir.join("rules");
    fs::create_dir_all(&bundled_rules_dir).map_err(|e| {
        CngRefusal::IoRefused(format!("mkdir {}: {e}", bundled_rules_dir.display()))
    })?;
    let bundled_rules_path = bundled_rules_dir.join("bench-roles.dl");
    fs::write(&bundled_rules_path, &rules_text).map_err(|e| {
        CngRefusal::IoRefused(format!("write {}: {e}", bundled_rules_path.display()))
    })?;

    let manifest = EvidenceManifest {
        measurement_class: RunReport::MEASUREMENT_CLASS.to_string(),
        schema_version: 1,
        obs_digest,
        query_digests,
        ontology_digests,
        rules_digest,
        ocel_graph_digest: report.ocel_graph_digest.clone(),
        sparql_result_digest: report.sparql_result_digest.clone(),
        evidence_chain_digest: report.evidence_chain_digest.clone(),
        replay_command: "cng evidence replay --bundle <this directory>".to_string(),
        signatures: Vec::new(),
    };
    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| CngRefusal::IoRefused(format!("evidence manifest serialize: {e}")))?;
    fs::write(results_dir.join("evidence-manifest.json"), &manifest_json)
        .map_err(|e| CngRefusal::IoRefused(format!("write evidence-manifest.json: {e}")))?;

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
