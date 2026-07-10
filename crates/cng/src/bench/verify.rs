//! Independent verification pass (`benchmark verify`): re-manufactures a
//! deterministic sample of sets against digests recorded by `run()`, and
//! re-parses + shape-validates a sample of the exported POWL artifacts.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::store::Store;

use crate::powl::CngRefusal;
use crate::shape;

use super::manufacture::manufacture_set;
use super::parallel_chunks;

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
        .map(|(_, (path, digest))| {
            let candidate = PathBuf::from(path);
            // v26.7.10 digests are bench_dir-relative; pre-v26.7.10 files may
            // hold absolute or CWD-relative keys. Rejoin relative keys against
            // bench_dir; leave absolute keys as-is (legacy compatibility).
            let resolved = if candidate.is_absolute() {
                candidate
            } else {
                bench_dir.join(&candidate)
            };
            (resolved, digest.clone())
        })
        .collect();
    let missing: Vec<String> = sample
        .iter()
        .filter(|(dir, _)| !dir.is_dir())
        .map(|(dir, _)| dir.display().to_string())
        .collect();
    if !missing.is_empty() {
        return Err(CngRefusal::AuditMismatch(format!(
            "digest keys resolve to {} missing set dir(s) under {}; first: {}",
            missing.len(),
            bench_dir.display(),
            missing[0]
        )));
    }
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

pub(super) fn count_ttl(dir: &Path) -> usize {
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("ttl"))
                .count()
        })
        .unwrap_or(0)
}

pub(super) fn count_ttl_recursive(dir: &Path) -> usize {
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

pub(super) fn dir_bytes(dir: &Path) -> usize {
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

pub(super) fn dir_bytes_recursive(dir: &Path) -> usize {
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
