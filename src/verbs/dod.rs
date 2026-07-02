//! `dod` verb dispatcher — Definition-of-Done surface on the main binary.
//!
//! `dod matrix` builds the capability-frontier DfCM matrix
//! ([`my_conforming_project::frontier`]), writes the full report to
//! `target/frontier-report.json`, and returns it as JSON. Every external
//! capability source explored this session is a cell: Admitted (with the socket
//! it landed in) or Impossible (refused, with reason + salvage). No silent
//! rows; `pass_rate` is `1.0`.
//!
//! This is the verb the walkthrough (`scripts/walkthrough.sh`) and Vision 2030
//! Release Criterion 3 probe as `dod matrix`. Distinct from the standalone
//! `dod` binary (`src/bin/dod.rs`), which is the fmt/clippy/test gate.
//!
//! Thin wrapper: all logic lives in [`my_conforming_project::frontier`], the
//! single source of truth shared with `tests/frontier_matrix.rs`.

use std::path::Path;

use clap_noun_verb::error::{NounVerbError, Result};
use clap_noun_verb_macros::verb;
use serde_json::Value;

/// The canonical output path for the frontier report, matching what the
/// walkthrough and Release Criterion 3 read.
const REPORT_PATH: &str = "target/frontier-report.json";

/// Build the capability-frontier DfCM matrix, write it to
/// `target/frontier-report.json`, and return the report as JSON (summary +
/// full matrix). `pass_rate` is `1.0` with every refusal receipted; failures,
/// if any, are listed under `summary.failures`.
#[verb]
pub fn matrix() -> Result<Value> {
    let report =
        my_conforming_project::frontier::write_report(Path::new(REPORT_PATH)).map_err(|e| {
            NounVerbError::execution_error(format!("could not write frontier report: {e}"))
        })?;
    serde_json::to_value(&report).map_err(|e| {
        NounVerbError::execution_error(format!("could not serialize frontier report: {e}"))
    })
}
