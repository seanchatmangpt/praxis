//! Turtle observation templates (`.template.ttl` files) and the on-disk
//! `.rq` query set. Zero inline SPARQL/Turtle: every template and query is
//! loaded from disk and filled via `{{KEY}}` placeholder substitution.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::powl::CngRefusal;

use super::OBS_KINDS;

pub(super) struct Templates {
    pub(super) domain: String,
    pub(super) problem: String,
    /// Observation templates keyed by kind suffix (see [`OBS_KINDS`]).
    pub(super) obs: BTreeMap<&'static str, String>,
    /// Content-bearing category fragments (PROJ-609/621, and the Stage 2
    /// "soc2-audit" addition below), keyed by category name ("interruption",
    /// "planning", "api-orchestration", "soc2-audit"); appended to the
    /// domain fragment.
    pub(super) category_content: BTreeMap<&'static str, String>,
    /// ex:AdmissionRequest artifact template (PROJ-611).
    pub(super) admission_request: String,
}

pub(super) fn load_templates() -> Result<Templates, CngRefusal> {
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
    let mut category_content = BTreeMap::new();
    for category in [
        "interruption",
        "planning",
        "api-orchestration",
        "soc2-audit",
    ] {
        category_content.insert(
            category,
            read(&format!("bench-category-{category}.template.ttl"))?,
        );
    }
    Ok(Templates {
        domain: read("bench-domain-fragment.template.ttl")?,
        problem: read("bench-problem.template.ttl")?,
        obs,
        category_content,
        admission_request: read("bench-admission-request.template.ttl")?,
    })
}

/// All SPARQL text the benchmark executes, loaded from `.rq` files on disk.
/// No SPARQL string is ever embedded in this module.
pub struct QuerySet {
    pub(super) queries: BTreeMap<String, String>,
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

    /// BLAKE3 digest per loaded query, keyed by stem.
    ///
    /// # Complexity
    /// O(total bytes) across all loaded query texts.
    pub fn digests(&self) -> BTreeMap<String, String> {
        self.queries
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    format!("blake3:{}", blake3::hash(v.as_bytes()).to_hex()),
                )
            })
            .collect()
    }
}

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
