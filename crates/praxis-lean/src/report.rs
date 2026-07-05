use crate::index::LeanDeclarationIndex;
use crate::receipt::ReconcileRecord;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    pub total_index_records: usize,
    pub total_receipts: usize,
    pub status_counts: BTreeMap<String, usize>,
    /// Index entries with no corresponding receipt line (`OrphanLabel` in
    /// `crate::error::LeanRefusal` terms).
    pub missing_receipts: Vec<String>,
    /// Receipt lines whose label has no corresponding index entry
    /// (`OrphanReceipt`).
    pub orphan_receipts: Vec<String>,
    /// Index entries whose `file_path` does not exist on disk
    /// (`OrphanFile`) -- populated only when `build` is given a `root` to
    /// check against; empty otherwise.
    pub missing_files: Vec<String>,
    pub duplicate_index_labels: Vec<String>,
}

impl VerificationReport {
    pub fn build(index: &LeanDeclarationIndex, receipts: &[ReconcileRecord]) -> Self {
        let mut status_counts = BTreeMap::new();
        let mut receipt_labels = std::collections::BTreeSet::new();

        for r in receipts {
            *status_counts.entry(r.status.clone()).or_insert(0) += 1;
            receipt_labels.insert(r.statement_label.clone());
        }

        let index_labels: std::collections::BTreeSet<&str> = index
            .records
            .iter()
            .map(|r| r.statement_label.as_str())
            .collect();

        let missing_receipts = index
            .records
            .iter()
            .filter(|r| !receipt_labels.contains(&r.statement_label))
            .map(|r| r.statement_label.clone())
            .collect();

        let orphan_receipts = receipts
            .iter()
            .filter(|r| !index_labels.contains(r.statement_label.as_str()))
            .map(|r| r.statement_label.clone())
            .collect();

        Self {
            total_index_records: index.records.len(),
            total_receipts: receipts.len(),
            status_counts,
            missing_receipts,
            orphan_receipts,
            missing_files: Vec::new(),
            duplicate_index_labels: index.duplicate_labels(),
        }
    }

    /// Same as [`Self::build`], additionally populating `missing_files` by
    /// checking each index record's `file_path` against `root`.
    pub fn build_with_root(
        index: &LeanDeclarationIndex,
        receipts: &[ReconcileRecord],
        root: &camino::Utf8Path,
    ) -> Self {
        let mut report = Self::build(index, receipts);
        report.missing_files = index
            .missing_file_records(root)
            .into_iter()
            .map(|r| r.statement_label.clone())
            .collect();
        report
    }
}
