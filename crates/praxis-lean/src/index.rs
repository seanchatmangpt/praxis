use crate::error::{LeanRefusal, Result};
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeanDeclRecord {
    pub statement_label: String,
    pub lean_declaration: String,
    pub file_path: Utf8PathBuf,
    pub dependency_labels: Vec<String>,
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LeanDeclarationIndex {
    pub records: Vec<LeanDeclRecord>,
}

impl LeanDeclarationIndex {
    pub fn load(path: &Utf8Path) -> Result<Self> {
        let text = fs::read_to_string(path).map_err(|source| LeanRefusal::Io {
            path: path.to_path_buf(),
            source,
        })?;
        serde_json::from_str(&text).map_err(|source| LeanRefusal::Json {
            path: path.to_path_buf(),
            source,
        })
    }

    pub fn save(&self, path: &Utf8Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| LeanRefusal::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let text = serde_json::to_string_pretty(self).map_err(|source| LeanRefusal::Json {
            path: path.to_path_buf(),
            source,
        })?;
        fs::write(path, text).map_err(|source| LeanRefusal::Io {
            path: path.to_path_buf(),
            source,
        })
    }

    pub fn by_label(&self) -> BTreeMap<&str, &LeanDeclRecord> {
        self.records
            .iter()
            .map(|r| (r.statement_label.as_str(), r))
            .collect()
    }

    pub fn duplicate_labels(&self) -> Vec<String> {
        let mut seen = BTreeSet::new();
        let mut dup = BTreeSet::new();
        for r in &self.records {
            if !seen.insert(r.statement_label.clone()) {
                dup.insert(r.statement_label.clone());
            }
        }
        dup.into_iter().collect()
    }

    pub fn missing_file_records(&self, root: &Utf8Path) -> Vec<&LeanDeclRecord> {
        self.records
            .iter()
            .filter(|r| !root.join(&r.file_path).exists())
            .collect()
    }

    /// Build an index from the real Praxis corpus by shelling out to a
    /// small Python/SPARQL script reusing
    /// `tools/paper-factory/paper_factory_engine.py`'s existing
    /// `sparql_select` helper, rather than adding a second Rust RDF/SPARQL
    /// dependency for one query. Reuses the exact `(label, kind,
    /// dependsOn)` extraction pattern already proven in this session's
    /// thesis/autoformalization work. `lean_pilot_dir` is where
    /// `<sanitized-label>.lean` files are expected to live, matching the
    /// existing `tools/paper-factory/lean-pilot/` naming convention
    /// (non-alphanumeric characters, e.g. `:`, replaced with `_`).
    pub fn build_from_corpus(
        repo_root: &Utf8Path,
        corpus_ttl: &Utf8Path,
        lean_pilot_dir: &Utf8Path,
    ) -> Result<Self> {
        let script = format!(
            r#"import sys, json
sys.path.insert(0, "tools/paper-factory")
import paper_factory_engine as pfe
from rdflib import Graph

corpus = Graph()
corpus.parse("{corpus_ttl}", format="turtle")
MATH = "http://seanchatmangpt.github.io/packs/tex-math#"

rows = pfe.sparql_select(corpus, f"""
PREFIX math: <{{MATH}}>
SELECT ?s ?label ?kind WHERE {{{{ ?s a math:Statement ; math:label ?label ; math:kind ?kind . }}}}
""")
by_iri = {{r["s"]: {{"label": r["label"], "kind": r["kind"]}} for r in rows}}

seen_labels = set()
out = []
for iri, v in by_iri.items():
    if v["label"] in seen_labels:
        continue
    seen_labels.add(v["label"])
    deps = pfe.sparql_select(corpus, f"""
    PREFIX math: <{{MATH}}>
    SELECT ?dep WHERE {{{{ <{{iri}}> math:dependsOn ?dep . }}}}
    """)
    dep_labels = [by_iri[d["dep"]]["label"] for d in deps if d["dep"] in by_iri]
    out.append({{"label": v["label"], "kind": v["kind"], "dependsOn": dep_labels}})

print(json.dumps(out))
"#,
            corpus_ttl = corpus_ttl
        );

        let output = std::process::Command::new("python3")
            .arg("-c")
            .arg(&script)
            .current_dir(repo_root)
            .output()
            .map_err(|source| LeanRefusal::Io {
                path: corpus_ttl.to_path_buf(),
                source,
            })?;

        if !output.status.success() {
            return Err(LeanRefusal::Io {
                path: corpus_ttl.to_path_buf(),
                source: std::io::Error::other(String::from_utf8_lossy(&output.stderr).to_string()),
            });
        }

        #[derive(Deserialize)]
        struct Row {
            label: String,
            kind: String,
            #[serde(rename = "dependsOn")]
            depends_on: Vec<String>,
        }
        let rows: Vec<Row> =
            serde_json::from_slice(&output.stdout).map_err(|source| LeanRefusal::Json {
                path: corpus_ttl.to_path_buf(),
                source,
            })?;

        let records = rows
            .into_iter()
            .map(|r| {
                let sanitized = sanitize_label(&r.label);
                LeanDeclRecord {
                    statement_label: r.label,
                    lean_declaration: sanitized.clone(),
                    file_path: lean_pilot_dir.join(format!("{sanitized}.lean")),
                    dependency_labels: r.depends_on,
                    kind: Some(r.kind),
                }
            })
            .collect();

        Ok(Self { records })
    }
}

fn sanitize_label(label: &str) -> String {
    label
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}
