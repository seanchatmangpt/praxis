//! Independent auditor replay from a self-contained evidence bundle: no
//! producer-process state or repo checkout is consulted, only the files a
//! `run()` bundle wrote to `bundle_dir`.

use std::fs;
use std::path::{Path, PathBuf};

use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::store::Store;

use crate::powl::CngRefusal;

use super::report::EvidenceManifest;
use super::roles::{collect_ttl_paths_recursive, run_construct};
use super::run::{evidence_digest, obs_dir_digest, OCEL_CONSTRUCT_STEMS};
use super::templates::QuerySet;

/// Report of an independent auditor replay from a self-contained bundle.
#[derive(Debug, serde::Serialize)]
pub struct AuditReplayReport {
    pub bundle_dir: String,
    pub obs_files_hashed: usize,
    pub obs_digest_match: bool,
    pub queries_verified: usize,
    pub ocel_graph_digest_match: bool,
    pub recomputed_ocel_graph_digest: String,
    pub expected_ocel_graph_digest: String,
}

/// Independent auditor replay: recomputes evidence from bundle files only.
///
/// A party holding ONLY a copied bundle directory (produced by `run()`)
/// re-derives the OCEL evidence graph from the bundled observations and
/// bundled queries and compares digests against the bundled manifest. No
/// repo checkout state, no producer memory is consulted.
///
/// Steps: (1) parse `results/evidence-manifest.json`; (2) re-hash
/// `obs/*.ttl` and compare `obs_digest`; (3) re-hash `queries/*.rq` against
/// `manifest.query_digests`; (4) load `obs/*.ttl` into a fresh store, run
/// the bundled `ocel-*.construct` queries, serialize sorted N-Triples,
/// BLAKE3, compare to `ocel_graph_digest`. Any disagreement or missing
/// input refuses `CNG_R11 AuditMismatch`.
///
/// # Complexity
/// O(obs bytes + evidence triples log-sorted).
pub fn audit_replay(bundle_dir: &Path) -> Result<AuditReplayReport, CngRefusal> {
    let manifest_path = bundle_dir.join("results").join("evidence-manifest.json");
    let manifest_text = fs::read_to_string(&manifest_path).map_err(|_| {
        CngRefusal::AuditMismatch(format!(
            "bundle is not auditable — expected manifest at {}",
            manifest_path.display()
        ))
    })?;
    let manifest: EvidenceManifest = serde_json::from_str(&manifest_text).map_err(|e| {
        CngRefusal::AuditMismatch(format!(
            "bundle is not auditable — cannot parse manifest {}: {e}",
            manifest_path.display()
        ))
    })?;

    // Step 2: obs digest.
    let recomputed_obs_digest = obs_dir_digest(bundle_dir)?;
    if recomputed_obs_digest != manifest.obs_digest {
        return Err(CngRefusal::AuditMismatch(format!(
            "obs digest mismatch — recomputed {recomputed_obs_digest} vs manifest {}",
            manifest.obs_digest
        )));
    }
    let mut obs_paths = Vec::new();
    collect_ttl_paths_recursive(&bundle_dir.join("obs"), &mut obs_paths)?;
    let obs_files_hashed = obs_paths.len();

    // Step 3: query digests.
    let bundled_queries_dir = bundle_dir.join("queries");
    let queries = QuerySet::load(&bundled_queries_dir)?;
    let query_digests = queries.digests();
    for (stem, expected_digest) in &manifest.query_digests {
        let actual_digest = query_digests.get(stem).ok_or_else(|| {
            CngRefusal::AuditMismatch(format!(
                "query {stem}.rq is present in the manifest but missing from the bundle"
            ))
        })?;
        if actual_digest != expected_digest {
            return Err(CngRefusal::AuditMismatch(format!(
                "query {stem}.rq digest mismatch — recomputed {actual_digest} vs manifest {expected_digest}"
            )));
        }
    }
    let queries_verified = manifest.query_digests.len();

    // Step 4: rebuild the observation store from the bundled obs files.
    let mut rel_obs_paths: Vec<(String, PathBuf)> = obs_paths
        .into_iter()
        .map(|p| {
            let rel = p
                .strip_prefix(bundle_dir)
                .map_err(|_| {
                    CngRefusal::AuditMismatch(format!(
                        "obs file {} is not under bundle dir {}",
                        p.display(),
                        bundle_dir.display()
                    ))
                })?
                .display()
                .to_string();
            Ok((rel, p))
        })
        .collect::<Result<_, CngRefusal>>()?;
    rel_obs_paths.sort_by(|a, b| a.0.cmp(&b.0));
    let obs_store = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("observation store construction: {e}")))?;
    // `run()` builds its observation store from roster admission facts
    // (`<bench_dir>/roster/*.ttl`, loaded first) plus the obs/ partitions
    // (obs_dir_digest covers only obs/, matching PROJ-603's manifest, but
    // OCEL materialization needs both — mirror run()'s exact construction
    // so the replayed evidence graph is comparable at all).
    let mut roster_paths: Vec<PathBuf> = fs::read_dir(bundle_dir.join("roster"))
        .map_err(|e| CngRefusal::IoRefused(format!("read roster: {e}")))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("ttl"))
        .collect();
    roster_paths.sort();
    for path in &roster_paths {
        let turtle = fs::read_to_string(path)
            .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", path.display())))?;
        obs_store
            .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
            .map_err(|e| {
                CngRefusal::AuditMismatch(format!(
                    "tampered bundle — cannot parse {}: {e}",
                    path.display()
                ))
            })?;
    }
    for (_, path) in &rel_obs_paths {
        let turtle = fs::read_to_string(path)
            .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", path.display())))?;
        obs_store
            .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
            .map_err(|e| {
                CngRefusal::AuditMismatch(format!(
                    "tampered bundle — cannot parse {}: {e}",
                    path.display()
                ))
            })?;
    }

    // Step 5: materialize OCEL from the bundled queries, in fixed order.
    let evidence_store = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("evidence store construction: {e}")))?;
    for construct in OCEL_CONSTRUCT_STEMS {
        run_construct(&obs_store, queries.get(construct)?, &evidence_store)?;
    }

    // Step 6/7: serialize and compare.
    let (_, recomputed_ocel_graph_digest) = evidence_digest(&evidence_store)?;
    let ocel_graph_digest_match = recomputed_ocel_graph_digest == manifest.ocel_graph_digest;
    if !ocel_graph_digest_match {
        return Err(CngRefusal::AuditMismatch(format!(
            "OCEL graph digest mismatch — recomputed {recomputed_ocel_graph_digest} vs manifest {}",
            manifest.ocel_graph_digest
        )));
    }

    Ok(AuditReplayReport {
        bundle_dir: bundle_dir.display().to_string(),
        obs_files_hashed,
        obs_digest_match: true,
        queries_verified,
        ocel_graph_digest_match,
        recomputed_ocel_graph_digest,
        expected_ocel_graph_digest: manifest.ocel_graph_digest,
    })
}
