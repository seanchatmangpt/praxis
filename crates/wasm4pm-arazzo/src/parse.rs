use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use wasm4pm_compat::arazzo::ArazzoDescription;

use crate::Refusal;

/// An index of parsed Arazzo documents, keyed by their absolute base URI.
#[derive(Debug, Default)]
pub struct DocumentIndex {
    pub documents: HashMap<String, ArazzoDescription>,
}

impl DocumentIndex {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    /// Parses an Arazzo document from a JSON string, resolving its base URI.
    /// `fallback_base_uri` is used if the document does not define a `$self` URI.
    pub fn add_document(&mut self, content: &str, fallback_base_uri: &str) -> Result<(), Refusal> {
        let doc: ArazzoDescription = serde_json::from_str(content)
            .map_err(|e| Refusal::Parse(format!("Failed to parse Arazzo document: {}", e)))?;

        // Support exactly Arazzo 1.1.x series for strict admission
        if !doc.arazzo.starts_with("1.1.") {
            return Err(Refusal::InvalidVersion(doc.arazzo.clone()));
        }

        let base_uri = if let Some(self_uri) = &doc.self_uri {
            self_uri.clone()
        } else {
            fallback_base_uri.to_string()
        };

        if self.documents.contains_key(&base_uri) {
            return Err(Refusal::Parse(format!(
                "Duplicate document base URI: {}",
                base_uri
            )));
        }

        self.documents.insert(base_uri, doc);
        Ok(())
    }

    /// Parses multiple Arazzo documents in parallel, resolving their base URIs,
    /// and inserts them into the index.
    pub fn add_documents_par(&mut self, docs: &[(&str, &str)]) -> Result<(), Refusal> {
        let parsed_results: Result<Vec<(String, ArazzoDescription)>, Refusal> = docs
            .par_iter()
            .map(|(content, fallback_base_uri)| {
                let doc: ArazzoDescription = serde_json::from_str(content).map_err(|e| {
                    Refusal::Parse(format!("Failed to parse Arazzo document: {}", e))
                })?;

                if !doc.arazzo.starts_with("1.1.") {
                    return Err(Refusal::InvalidVersion(doc.arazzo.clone()));
                }

                let base_uri = if let Some(self_uri) = &doc.self_uri {
                    self_uri.clone()
                } else {
                    fallback_base_uri.to_string()
                };

                Ok((base_uri, doc))
            })
            .collect();

        let parsed = parsed_results?;

        self.documents.reserve(parsed.len());

        for (base_uri, doc) in parsed {
            if self.documents.contains_key(&base_uri) {
                return Err(Refusal::Parse(format!(
                    "Duplicate document base URI: {}",
                    base_uri
                )));
            }
            self.documents.insert(base_uri, doc);
        }

        Ok(())
    }

    /// Loads and parses an Arazzo document from a file.
    ///
    /// Swarm audit wnl2yhbgm finding #11: this previously used `unsafe {
    /// MmapOptions::new().map(&file) }` for "zero-copy" throughput -- exactly the "this is
    /// faster" rationale `.claude/rules/rust-agi-core-team.md`'s invariant #7 forbids (unsafe is
    /// permitted only for cryptographic verification or FFI memory-layout guarantees, neither of
    /// which applies here). A concurrent writer truncating or overwriting the file mid-mmap would
    /// have produced UB/SIGBUS instead of a clean I/O error. Removed entirely rather than
    /// justified with a `// SAFETY:` comment, since no valid justification exists: a plain
    /// `fs::read` is safe and functionally equivalent.
    pub fn add_document_from_file(
        &mut self,
        path: &Path,
        fallback_base_uri: &str,
    ) -> Result<(), Refusal> {
        let bytes =
            fs::read(path).map_err(|e| Refusal::Parse(format!("Failed to read file: {}", e)))?;

        let doc: ArazzoDescription = serde_json::from_slice(&bytes)
            .map_err(|e| Refusal::Parse(format!("Failed to parse Arazzo document: {}", e)))?;

        if !doc.arazzo.starts_with("1.1.") {
            return Err(Refusal::InvalidVersion(doc.arazzo.clone()));
        }

        let base_uri = if let Some(self_uri) = &doc.self_uri {
            self_uri.clone()
        } else {
            fallback_base_uri.to_string()
        };

        if self.documents.contains_key(&base_uri) {
            return Err(Refusal::Parse(format!(
                "Duplicate document base URI: {}",
                base_uri
            )));
        }

        self.documents.insert(base_uri, doc);
        Ok(())
    }

    /// Loads and parses multiple Arazzo documents in parallel.
    ///
    /// See [`Self::add_document_from_file`]'s doc comment (swarm audit wnl2yhbgm finding #11) for
    /// why this reads via `fs::read` rather than the `unsafe` mmap it previously used.
    pub fn add_documents_from_files_par(&mut self, paths: &[(&Path, &str)]) -> Result<(), Refusal> {
        let parsed_results: Result<Vec<(String, ArazzoDescription)>, Refusal> = paths
            .par_iter()
            .map(|(path, fallback_base_uri)| {
                let bytes = fs::read(path)
                    .map_err(|e| Refusal::Parse(format!("Failed to read file: {}", e)))?;

                let doc: ArazzoDescription = serde_json::from_slice(&bytes).map_err(|e| {
                    Refusal::Parse(format!("Failed to parse Arazzo document: {}", e))
                })?;

                if !doc.arazzo.starts_with("1.1.") {
                    return Err(Refusal::InvalidVersion(doc.arazzo.clone()));
                }

                let base_uri = if let Some(self_uri) = &doc.self_uri {
                    self_uri.clone()
                } else {
                    fallback_base_uri.to_string()
                };

                Ok((base_uri, doc))
            })
            .collect();

        let parsed = parsed_results?;
        self.documents.reserve(parsed.len());

        for (base_uri, doc) in parsed {
            if self.documents.contains_key(&base_uri) {
                return Err(Refusal::Parse(format!(
                    "Duplicate document base URI: {}",
                    base_uri
                )));
            }
            self.documents.insert(base_uri, doc);
        }

        Ok(())
    }
}
