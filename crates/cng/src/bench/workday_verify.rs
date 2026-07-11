//! Workday verification harness (PROJ-616): independent replay and bundle
//! manifest assembly over a finished `workday()` output directory.
//!
//! Nothing here consults producer-process state: every input is a file the
//! workday run wrote to `out_dir` (obs partitions, roster admission facts,
//! `evidence/ocel.nt`, `results/workday-report.json`) plus the on-disk
//! query set. The replay recomputes evidence from those files alone and
//! refuses, typed, on any disagreement:
//!
//! - `CNG_R13 UnreceiptedActuation` — a recorded `hook_receipt` observation
//!   lost its `ex:hookDeltaHash` (zero-unreceipted-actuation law re-checked
//!   at replay time, BEFORE any digest comparison, so a stripped receipt is
//!   named as the actuation-law violation it is, not as generic tamper).
//! - `CNG_R11 AuditMismatch` — the recomputed obs digest, the recomputed
//!   OCEL graph digest, or the on-disk `evidence/ocel.nt` serialization
//!   disagrees with what the run recorded (third-party integrity failure).
//!
//! Bundle manifest: [`assemble_workday_manifest`] lists every evidence file
//! under the fixed bundle subdirectories with its BLAKE3 digest, in
//! deterministic (`BTreeMap`, bundle-relative path) order, and writes it to
//! `results/workday-bundle-manifest.json`.
//!
//! Wiring note (PROJ-616 seam): these functions take a finished workday
//! output directory; calling them from the `workday()` run path itself is
//! left to the orchestrator (it would touch `workday.rs`, which this module
//! deliberately does not). Only an independent pass like this one may ever
//! claim the REPLAYABLE hook-standing rung (see `workday.rs` module docs).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::model::{LiteralRef, NamedNodeRef, TermRef};
use oxigraph::store::Store;

use crate::powl::CngRefusal;

use super::manufacture::term_value;
use super::roles::{collect_ttl_paths_recursive, run_construct};
use super::run::{evidence_digest, obs_dir_digest, OCEL_CONSTRUCT_STEMS};
use super::templates::QuerySet;

/// The bench-obs vocabulary prefix (bench-obs.ttl; same IRIs the
/// observation templates render).
const OBS_PREFIX: &str = "https://ggen.io/ontology/bench-obs#";
/// The rwai example prefix (observation templates' `ex:`).
const EX_PREFIX: &str = "http://example.org/rwai#";

/// Fixed bundle subdirectories the manifest covers, in canonical order.
/// `results/` is deliberately excluded: `workday-report.json` carries the
/// run's `out_dir` (path-derived by design), so it can never be part of a
/// path-independent byte-comparison surface.
const BUNDLE_DIRS: [&str; 7] = [
    "admissions",
    "dispatch",
    "evidence",
    "generated",
    "obs",
    "roster",
    "ticks",
];

/// Report of one independent workday replay pass. Every boolean is `true`
/// by construction on `Ok` (a `false` would have been a typed refusal);
/// they are carried so downstream JSON consumers see the checks performed.
#[derive(Debug, serde::Serialize)]
pub struct WorkdayReplayReport {
    pub out_dir: String,
    /// `hook_receipt` observations reconciled against the
    /// zero-unreceipted-actuation law (each carries `ex:hookDeltaHash`).
    pub hook_receipt_observations: usize,
    pub obs_digest_match: bool,
    pub ocel_graph_digest_match: bool,
    pub ocel_serialization_match: bool,
    pub recomputed_obs_digest: String,
    pub recomputed_ocel_graph_digest: String,
}

/// Reads one recorded string field out of `results/workday-report.json`.
/// The report struct is producer-only (`Serialize`), so the replay side
/// reads it as untyped JSON — the recorded digests are data, not API.
///
/// # Complexity
/// O(report bytes) parse (parsed once per field by the caller's `Value`).
fn report_field(report: &serde_json::Value, field: &str) -> Result<String, CngRefusal> {
    report
        .get(field)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            CngRefusal::AuditMismatch(format!(
                "workday-report.json is missing the recorded `{field}` digest; \
                 the bundle is not replayable"
            ))
        })
}

/// Rebuilds the workday observation store exactly as `workday()` built it:
/// roster admission facts first (`roster/*.ttl`, sorted), then every obs
/// partition (`obs/**/*.ttl`, sorted by bundle-relative path). A file that
/// no longer parses is a tampered bundle (`CNG_R11`).
///
/// # Complexity
/// O(roster + obs bytes) parse, O(files log files) for the sorts.
fn load_workday_obs_store(out_dir: &Path) -> Result<Store, CngRefusal> {
    let store = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("observation store construction: {e}")))?;
    let mut roster_paths: Vec<PathBuf> = fs::read_dir(out_dir.join("roster"))
        .map_err(|e| CngRefusal::IoRefused(format!("read roster: {e}")))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("ttl"))
        .collect();
    roster_paths.sort();
    let mut obs_paths = Vec::new();
    collect_ttl_paths_recursive(&out_dir.join("obs"), &mut obs_paths)?;
    obs_paths.sort();
    for path in roster_paths.iter().chain(obs_paths.iter()) {
        let turtle = fs::read_to_string(path)
            .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", path.display())))?;
        store
            .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
            .map_err(|e| {
                CngRefusal::AuditMismatch(format!(
                    "tampered bundle — cannot parse {}: {e}",
                    path.display()
                ))
            })?;
    }
    Ok(store)
}

/// Re-checks the zero-unreceipted-actuation law over the recorded
/// observations: every `hook_receipt` observation must still carry its
/// `ex:hookDeltaHash`. A receipt whose delta hash was stripped is a
/// `CNG_R13 UnreceiptedActuation` naming the workflow (obsSetId) and
/// category (hookName) — the actuation-law violation, not generic tamper.
/// Returns the number of receipts reconciled.
///
/// # Complexity
/// O(hook receipts) typed pattern scans (each scan O(1) lookups).
fn reconcile_hook_receipts(store: &Store) -> Result<usize, CngRefusal> {
    let iri = |suffix: &str| -> Result<oxigraph::model::NamedNode, CngRefusal> {
        oxigraph::model::NamedNode::new(suffix.to_string())
            .map_err(|e| CngRefusal::MalformedTtl(format!("{suffix}: {e}")))
    };
    let kind_pred = iri(&format!("{OBS_PREFIX}obsKind"))?;
    let set_pred = iri(&format!("{OBS_PREFIX}obsSetId"))?;
    let delta_pred = iri(&format!("{EX_PREFIX}hookDeltaHash"))?;
    let name_pred = iri(&format!("{EX_PREFIX}hookName"))?;
    let kind_lit = LiteralRef::new_simple_literal("hook_receipt");
    let mut reconciled = 0usize;
    for quad in store.quads_for_pattern(
        None,
        Some(kind_pred.as_ref()),
        Some(TermRef::from(kind_lit)),
        None,
    ) {
        let quad = quad.map_err(|e| CngRefusal::MalformedTtl(format!("hook receipt scan: {e}")))?;
        let subject = quad.subject;
        let first_object = |pred: NamedNodeRef<'_>| -> Result<Option<String>, CngRefusal> {
            match store
                .quads_for_pattern(Some(subject.as_ref()), Some(pred), None, None)
                .next()
            {
                Some(Ok(q)) => Ok(Some(term_value(&q.object))),
                Some(Err(e)) => Err(CngRefusal::MalformedTtl(format!(
                    "hook receipt field scan: {e}"
                ))),
                None => Ok(None),
            }
        };
        if first_object(delta_pred.as_ref())?.is_none() {
            let workflow = first_object(set_pred.as_ref())?.unwrap_or_else(|| subject.to_string());
            let category =
                first_object(name_pred.as_ref())?.unwrap_or_else(|| "unknown".to_string());
            return Err(CngRefusal::UnreceiptedActuation { workflow, category });
        }
        reconciled += 1;
    }
    Ok(reconciled)
}

/// Independent workday replay: rebuilds the observation store from the
/// bundle files, re-checks the actuation-receipt law, recomputes the obs
/// digest and the OCEL evidence graph (same fixed CONSTRUCT order as the
/// producer), and compares everything against what the run recorded in
/// `results/workday-report.json` and `evidence/ocel.nt`.
///
/// # Errors
/// `CNG_R13 UnreceiptedActuation` (stripped receipt evidence) checked
/// first; `CNG_R11 AuditMismatch` on any digest/serialization disagreement
/// or unparseable bundle input; `CNG_R10` for plain I/O failures.
///
/// # Complexity
/// O(obs bytes) parse + O(t log t) evidence serialization + fixed CONSTRUCT
/// set over O(obs facts).
pub fn workday_replay(
    out_dir: &Path,
    queries_dir: Option<&Path>,
) -> Result<WorkdayReplayReport, CngRefusal> {
    let report_path = out_dir.join("results").join("workday-report.json");
    let report_text = fs::read_to_string(&report_path).map_err(|_| {
        CngRefusal::AuditMismatch(format!(
            "bundle is not replayable — expected recorded report at {}",
            report_path.display()
        ))
    })?;
    let report: serde_json::Value = serde_json::from_str(&report_text).map_err(|e| {
        CngRefusal::AuditMismatch(format!(
            "bundle is not replayable — cannot parse {}: {e}",
            report_path.display()
        ))
    })?;
    let recorded_obs_digest = report_field(&report, "obs_digest")?;
    let recorded_ocel_digest = report_field(&report, "ocel_graph_digest")?;

    // 1. Rebuild the observation store; a tampered file refuses here.
    let obs_store = load_workday_obs_store(out_dir)?;

    // 2. Actuation-receipt law FIRST: a stripped ex:hookDeltaHash is named
    //    as the CNG_R13 law violation before any generic digest check.
    let hook_receipt_observations = reconcile_hook_receipts(&obs_store)?;

    // 3. Obs digest against the recorded one.
    let recomputed_obs_digest = obs_dir_digest(out_dir)?;
    if recomputed_obs_digest != recorded_obs_digest {
        return Err(CngRefusal::AuditMismatch(format!(
            "obs digest mismatch — recomputed {recomputed_obs_digest} vs recorded \
             {recorded_obs_digest}"
        )));
    }

    // 4. Re-materialize OCEL in the producer's fixed CONSTRUCT order.
    // Seam (reported honestly): the workday bundle does not carry its own
    // queries/ copy, so replay loads the same on-disk query set the
    // producer used; bundling queries into the workday output is
    // orchestrator wiring (it would touch workday.rs).
    let query_dir_owned;
    let query_dir = match queries_dir {
        Some(dir) => dir,
        None => {
            query_dir_owned = QuerySet::default_dir();
            &query_dir_owned
        }
    };
    let queries = QuerySet::load(query_dir)?;
    let evidence_store = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("evidence store construction: {e}")))?;
    for construct in OCEL_CONSTRUCT_STEMS {
        run_construct(&obs_store, queries.get(construct)?, &evidence_store)?;
    }
    let (recomputed_nt, recomputed_ocel_digest) = evidence_digest(&evidence_store)?;
    if recomputed_ocel_digest != recorded_ocel_digest {
        return Err(CngRefusal::AuditMismatch(format!(
            "OCEL graph digest mismatch — recomputed {recomputed_ocel_digest} vs recorded \
             {recorded_ocel_digest}"
        )));
    }

    // 5. The on-disk serialization must be exactly the recomputed one.
    let ocel_path = out_dir.join("evidence").join("ocel.nt");
    let recorded_nt = fs::read_to_string(&ocel_path).map_err(|_| {
        CngRefusal::AuditMismatch(format!(
            "bundle is not replayable — expected OCEL serialization at {}",
            ocel_path.display()
        ))
    })?;
    if recorded_nt != recomputed_nt {
        return Err(CngRefusal::AuditMismatch(format!(
            "evidence/ocel.nt does not match the recomputed OCEL serialization \
             ({} recorded bytes vs {} recomputed bytes)",
            recorded_nt.len(),
            recomputed_nt.len()
        )));
    }

    Ok(WorkdayReplayReport {
        out_dir: out_dir.display().to_string(),
        hook_receipt_observations,
        obs_digest_match: true,
        ocel_graph_digest_match: true,
        ocel_serialization_match: true,
        recomputed_obs_digest,
        recomputed_ocel_graph_digest: recomputed_ocel_digest,
    })
}

/// Report of one PARTIAL (prefix) workday replay pass (PROJ-724, G13
/// machinery): an UNFINISHED bundle — no recorded report, no serialized
/// OCEL — is lawful input; only observed INCONSISTENCY refuses. The
/// `*_compared` flags say which recorded artifacts existed and were
/// checked (`false` = absent, honestly skipped, never inferred).
#[derive(Debug, serde::Serialize)]
pub struct WorkdayPartialReplayReport {
    pub out_dir: String,
    /// `hook_receipt` observations reconciled against the actuation law.
    pub hook_receipt_observations: usize,
    /// Whether `results/workday-report.json` existed and its digests were
    /// compared (a mismatch refused before this struct existed).
    pub report_compared: bool,
    /// Whether `evidence/ocel.nt` existed and was byte-compared.
    pub ocel_serialization_compared: bool,
    pub recomputed_obs_digest: String,
    pub recomputed_ocel_graph_digest: String,
}

/// Partial (prefix) workday replay (PROJ-724): verifies whatever prefix of
/// a workday bundle exists — obs partitions parse, the actuation-receipt
/// law holds, and the OCEL evidence graph rematerializes — WITHOUT
/// requiring the finished-bundle artifacts (`results/workday-report.json`,
/// `evidence/ocel.nt`). Where a recorded artifact IS present it must agree
/// (same refusals as [`workday_replay`]); absence is skipped and reported,
/// never refused. This is the crash-resume falsifier's verification entry:
/// a killed producer leaves a lawful prefix, not a tamper.
///
/// # Errors
/// `CNG_R13 UnreceiptedActuation` (stripped receipt); `CNG_R11
/// AuditMismatch` on any disagreement with a PRESENT recorded artifact or
/// an unparseable bundle input; `CNG_R10` for plain I/O failures.
///
/// # Complexity
/// O(obs bytes) parse + O(t log t) evidence serialization + fixed CONSTRUCT
/// set over O(obs facts).
pub fn workday_replay_partial(
    out_dir: &Path,
    queries_dir: Option<&Path>,
) -> Result<WorkdayPartialReplayReport, CngRefusal> {
    // 1. Rebuild the observation store from whatever prefix exists.
    let obs_store = load_workday_obs_store(out_dir)?;

    // 2. Actuation-receipt law over the recorded prefix.
    let hook_receipt_observations = reconcile_hook_receipts(&obs_store)?;

    // 3. Recompute digests; compare only against artifacts that exist.
    let recomputed_obs_digest = obs_dir_digest(out_dir)?;
    let query_dir_owned;
    let query_dir = match queries_dir {
        Some(dir) => dir,
        None => {
            query_dir_owned = QuerySet::default_dir();
            &query_dir_owned
        }
    };
    let queries = QuerySet::load(query_dir)?;
    let evidence_store = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("evidence store construction: {e}")))?;
    for construct in OCEL_CONSTRUCT_STEMS {
        run_construct(&obs_store, queries.get(construct)?, &evidence_store)?;
    }
    let (recomputed_nt, recomputed_ocel_digest) = evidence_digest(&evidence_store)?;

    // 4. Recorded report, IF present: its digests must agree.
    let report_path = out_dir.join("results").join("workday-report.json");
    let report_compared = if report_path.is_file() {
        let report_text = fs::read_to_string(&report_path)
            .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", report_path.display())))?;
        let report: serde_json::Value = serde_json::from_str(&report_text).map_err(|e| {
            CngRefusal::AuditMismatch(format!("cannot parse {}: {e}", report_path.display()))
        })?;
        let recorded_obs = report_field(&report, "obs_digest")?;
        let recorded_ocel = report_field(&report, "ocel_graph_digest")?;
        if recomputed_obs_digest != recorded_obs {
            return Err(CngRefusal::AuditMismatch(format!(
                "obs digest mismatch — recomputed {recomputed_obs_digest} vs recorded \
                 {recorded_obs}"
            )));
        }
        if recomputed_ocel_digest != recorded_ocel {
            return Err(CngRefusal::AuditMismatch(format!(
                "OCEL graph digest mismatch — recomputed {recomputed_ocel_digest} vs \
                 recorded {recorded_ocel}"
            )));
        }
        true
    } else {
        false
    };

    // 5. Serialized OCEL, IF present: must be byte-identical.
    let ocel_path = out_dir.join("evidence").join("ocel.nt");
    let ocel_serialization_compared = if ocel_path.is_file() {
        let recorded_nt = fs::read_to_string(&ocel_path)
            .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", ocel_path.display())))?;
        if recorded_nt != recomputed_nt {
            return Err(CngRefusal::AuditMismatch(format!(
                "evidence/ocel.nt does not match the recomputed OCEL serialization \
                 ({} recorded bytes vs {} recomputed bytes)",
                recorded_nt.len(),
                recomputed_nt.len()
            )));
        }
        true
    } else {
        false
    };

    Ok(WorkdayPartialReplayReport {
        out_dir: out_dir.display().to_string(),
        hook_receipt_observations,
        report_compared,
        ocel_serialization_compared,
        recomputed_obs_digest,
        recomputed_ocel_graph_digest: recomputed_ocel_digest,
    })
}

/// Collects every file under `dir`, recursively, as bundle-relative paths.
///
/// # Complexity
/// O(files) directory walk (recursion depth bounded by the directory tree).
fn collect_files_recursive(
    root: &Path,
    dir: &Path,
    out: &mut Vec<(String, PathBuf)>,
) -> Result<(), CngRefusal> {
    let entries = fs::read_dir(dir)
        .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", dir.display())))?;
    for entry in entries {
        let entry = entry.map_err(|e| CngRefusal::IoRefused(format!("read dir entry: {e}")))?;
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(root, &path, out)?;
        } else {
            let rel = path
                .strip_prefix(root)
                .map_err(|_| {
                    CngRefusal::IoRefused(format!(
                        "bundle file {} is not under {}",
                        path.display(),
                        root.display()
                    ))
                })?
                .display()
                .to_string();
            out.push((rel, path));
        }
    }
    Ok(())
}

/// Assembles the workday evidence-bundle manifest: every file under the
/// fixed bundle subdirectories ([`BUNDLE_DIRS`]) mapped to its BLAKE3
/// digest, keyed by bundle-relative path (`BTreeMap` — canonical order for
/// serialization and comparison; nothing time- or path-content-derived).
/// The manifest is written to `results/workday-bundle-manifest.json` and
/// returned. Two same-seed workday runs must yield byte-identical manifest
/// maps — that equality IS the determinism gate.
///
/// # Errors
/// `CNG_R10 IoRefused` on unreadable bundle files or an unwritable
/// results directory.
///
/// # Complexity
/// O(bundle bytes) hashing + O(files log files) map construction.
pub fn assemble_workday_manifest(out_dir: &Path) -> Result<BTreeMap<String, String>, CngRefusal> {
    let mut files: Vec<(String, PathBuf)> = Vec::new();
    for sub in BUNDLE_DIRS {
        let dir = out_dir.join(sub);
        if dir.is_dir() {
            collect_files_recursive(out_dir, &dir, &mut files)?;
        }
    }
    let mut manifest: BTreeMap<String, String> = BTreeMap::new();
    for (rel, path) in files {
        let bytes = fs::read(&path)
            .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", path.display())))?;
        manifest.insert(rel, format!("blake3:{}", blake3::hash(&bytes).to_hex()));
    }
    let results_dir = out_dir.join("results");
    fs::create_dir_all(&results_dir)
        .map_err(|e| CngRefusal::IoRefused(format!("mkdir results: {e}")))?;
    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| CngRefusal::IoRefused(format!("manifest serialize: {e}")))?;
    fs::write(results_dir.join("workday-bundle-manifest.json"), json)
        .map_err(|e| CngRefusal::IoRefused(format!("write workday-bundle-manifest.json: {e}")))?;
    Ok(manifest)
}

#[cfg(test)]
#[path = "workday_verify_test.rs"]
mod workday_verify_test;
