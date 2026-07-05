use crate::error::{LeanRefusal, Result};
use crate::hash::{self, file_blake3_hex};
use crate::lean::{LeanCheck, LeanToolchain};
use crate::no_sorry::AuditFinding;
use crate::status::{FailureClass, VerificationStatus};
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use std::fs::{self as stdfs, OpenOptions};
use std::io::{BufRead, BufReader, Write};

/// Receipt schema v2. One line per disposition.
///
/// Deliberately has NO wall-clock field: this repo's stated invariant is
/// "no wall clock in any hash/receipt path" -- ordering is append-order in
/// the ledger file, exactly like the existing (schema v1)
/// `formalization_receipts.jsonl`, not a timestamp. `chain_hash`/
/// `prev_chain_hash` provide genesis-folded tamper-evidence instead
/// (mirrors the `chain_predecessor`/`"genesis"` convention documented in
/// `src/receipt_shacl.rs`, reimplemented locally -- see `hash::chain_hash`
/// doc comment for why this doesn't depend on `praxis-core::ReceiptRecord`
/// directly).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReceipt {
    pub schema_version: u32,
    pub statement_label: String,
    pub lean_declaration: Option<String>,
    pub file_path: Utf8PathBuf,
    pub file_hash: Option<String>,
    pub dependency_labels: Vec<String>,
    pub status: VerificationStatus,
    pub failure_class: FailureClass,
    pub attempt_count: u32,
    pub lean: Option<LeanToolchain>,
    pub kernel_exit_code: Option<i32>,
    pub stdout_hash: Option<String>,
    pub stderr_hash: Option<String>,
    pub audit_findings: Vec<AuditFinding>,
    pub prev_chain_hash: String,
    pub chain_hash: String,
}

impl VerificationReceipt {
    #[allow(clippy::too_many_arguments)]
    pub fn from_check(
        statement_label: String,
        lean_declaration: Option<String>,
        dependency_labels: Vec<String>,
        attempt_count: u32,
        toolchain: LeanToolchain,
        check: LeanCheck,
        audit_findings: Vec<AuditFinding>,
        prev_chain_hash: &str,
    ) -> Self {
        let file_hash = file_blake3_hex(&check.file_path).ok();
        let status = if !audit_findings.is_empty() {
            if audit_findings.iter().any(|f| f.kind == "axiom") {
                VerificationStatus::AxiomUnauthorized
            } else {
                VerificationStatus::NoSorryFailed
            }
        } else if check.success {
            VerificationStatus::Verified
        } else {
            VerificationStatus::KernelRejected
        };
        let failure_class = if !audit_findings.is_empty() {
            if audit_findings.iter().any(|f| f.kind == "axiom") {
                FailureClass::UnauthorizedAxiom
            } else {
                FailureClass::ContainsSorry
            }
        } else if check.success {
            FailureClass::None
        } else {
            classify_failure(&check.stderr_preview)
        };

        // Chain hash covers every field that determines the disposition,
        // computed from the canonical JSON of a hashable projection -- not
        // the whole struct (which would be circular, since chain_hash is a
        // field of it).
        let payload = serde_json::json!({
            "statement_label": statement_label,
            "lean_declaration": lean_declaration,
            "file_path": check.file_path,
            "file_hash": file_hash,
            "dependency_labels": dependency_labels,
            "status": status,
            "failure_class": failure_class,
            "attempt_count": attempt_count,
            "kernel_exit_code": check.exit_code,
            "stdout_hash": check.stdout_hash,
            "stderr_hash": check.stderr_hash,
        });
        let payload_bytes = serde_json::to_vec(&payload).unwrap_or_default();
        let chain_hash_hex = hash::chain_hash(prev_chain_hash, &payload_bytes);

        Self {
            schema_version: 2,
            statement_label,
            lean_declaration,
            file_path: check.file_path,
            file_hash,
            dependency_labels,
            status,
            failure_class,
            attempt_count,
            lean: Some(toolchain),
            kernel_exit_code: check.exit_code,
            stdout_hash: Some(check.stdout_hash),
            stderr_hash: Some(check.stderr_hash),
            audit_findings,
            prev_chain_hash: prev_chain_hash.to_string(),
            chain_hash: chain_hash_hex,
        }
    }
}

fn classify_failure(stderr: &str) -> FailureClass {
    let s = stderr.to_lowercase();
    if s.contains("unknown identifier") || s.contains("unknown constant") {
        FailureClass::UnknownIdentifier
    } else if s.contains("type mismatch") {
        FailureClass::TypeMismatch
    } else if s.contains("function expected") {
        FailureClass::TypeMismatch
    } else if s.contains("unknown tactic") || s.contains("tactic") {
        FailureClass::TacticFailure
    } else if s.contains("unexpected token") || s.contains("invalid") {
        FailureClass::ParseError
    } else {
        FailureClass::Unknown
    }
}

#[derive(Debug, Clone)]
pub struct ReceiptLedger {
    path: Utf8PathBuf,
}

impl ReceiptLedger {
    pub fn new(path: impl Into<Utf8PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// The chain hash to fold the next receipt onto: the last line's own
    /// `chain_hash`, or [`hash::GENESIS_CHAIN_HASH_HEX`] if the ledger is
    /// empty or doesn't exist yet.
    pub fn tip_chain_hash(&self) -> Result<String> {
        if !self.path.exists() {
            return Ok(hash::GENESIS_CHAIN_HASH_HEX.to_string());
        }
        let receipts = self.read_all()?;
        Ok(receipts
            .last()
            .map(|r| r.chain_hash.clone())
            .unwrap_or_else(|| hash::GENESIS_CHAIN_HASH_HEX.to_string()))
    }

    pub fn append(&self, receipt: &VerificationReceipt) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            stdfs::create_dir_all(parent).map_err(|source| LeanRefusal::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|source| LeanRefusal::Io {
                path: self.path.clone(),
                source,
            })?;
        serde_json::to_writer(&mut f, receipt).map_err(|source| LeanRefusal::Json {
            path: self.path.clone(),
            source,
        })?;
        writeln!(f).map_err(|source| LeanRefusal::Io {
            path: self.path.clone(),
            source,
        })?;
        Ok(())
    }

    pub fn read_all(&self) -> Result<Vec<VerificationReceipt>> {
        let f = stdfs::File::open(&self.path).map_err(|source| LeanRefusal::Io {
            path: self.path.clone(),
            source,
        })?;
        let reader = BufReader::new(f);
        let mut out = Vec::new();
        for line in reader.lines() {
            let line = line.map_err(|source| LeanRefusal::Io {
                path: self.path.clone(),
                source,
            })?;
            if line.trim().is_empty() {
                continue;
            }
            let receipt: VerificationReceipt =
                serde_json::from_str(&line).map_err(|source| LeanRefusal::Json {
                    path: self.path.clone(),
                    source,
                })?;
            out.push(receipt);
        }
        Ok(out)
    }

    pub fn path(&self) -> &Utf8Path {
        &self.path
    }

    /// Read the ledger tolerating EITHER schema: this crate's own schema v2
    /// (`VerificationReceipt`) or the pre-existing schema v1 shape already
    /// used by `tools/paper-factory/lean-pilot/formalization_receipts.jsonl`
    /// (`label`/`kind`/`status`/`attempts`/`last_error`/`depends_on`/
    /// `kernel_version` -- no `schema_version`, no chain hash, different
    /// field names throughout). Each line is tried as v2 first, falling back
    /// to v1 -- a line that matches neither is a real parse error, not
    /// silently skipped.
    pub fn read_all_any_schema(&self) -> Result<Vec<ReconcileRecord>> {
        let f = stdfs::File::open(&self.path).map_err(|source| LeanRefusal::Io {
            path: self.path.clone(),
            source,
        })?;
        let reader = BufReader::new(f);
        let mut out = Vec::new();
        for line in reader.lines() {
            let line = line.map_err(|source| LeanRefusal::Io {
                path: self.path.clone(),
                source,
            })?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(v2) = serde_json::from_str::<VerificationReceipt>(trimmed) {
                out.push(ReconcileRecord {
                    statement_label: v2.statement_label,
                    status: format!("{:?}", v2.status).to_lowercase(),
                    schema_version: 2,
                });
                continue;
            }
            let v1: LegacyReceiptV1 =
                serde_json::from_str(trimmed).map_err(|source| LeanRefusal::Json {
                    path: self.path.clone(),
                    source,
                })?;
            out.push(ReconcileRecord {
                statement_label: v1.label,
                status: v1.status,
                schema_version: 1,
            });
        }
        Ok(out)
    }
}

/// The pre-existing schema v1 shape, matching
/// `tools/paper-factory/lean-pilot/formalization_receipts.jsonl` exactly
/// as it exists on disk today -- read-only compatibility shim, not a
/// schema this crate writes.
#[derive(Debug, Clone, Deserialize)]
pub struct LegacyReceiptV1 {
    pub label: String,
    pub kind: String,
    pub status: String,
    pub attempts: u32,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    pub kernel_version: String,
}

/// A schema-agnostic view of one receipt line, normalized from either v1
/// or v2, sufficient for reconciliation and reporting (which only need the
/// label and status, not every schema-specific field).
#[derive(Debug, Clone, Serialize)]
pub struct ReconcileRecord {
    pub statement_label: String,
    pub status: String,
    pub schema_version: u32,
}
