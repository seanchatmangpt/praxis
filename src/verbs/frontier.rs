//! `frontier` verb — the Lane 10 combinatorial-maximalist frontier receipt.
//!
//! Thin wrapper over [`my_conforming_project::frontier`]: all matrix
//! construction and evidence lives there (shared with
//! `tests/frontier_matrix.rs`) so the CLI and the test suite can never
//! drift on what counts as admitted, refused, or unevaluated.

use clap_noun_verb::error::{NounVerbError, Result};
use clap_noun_verb_macros::verb;
use my_conforming_project::frontier;
use serde_json::{json, Value};

/// Build the frontier `capability_source` × `praxis_socket` DfCM matrix and
/// return it as JSON (summary + full per-cell disposition).
///
/// When `out` is given, also writes the full report to that path (creating
/// parent directories); otherwise nothing is written to disk.
#[verb]
pub fn matrix(
    #[arg(
        default_value = "",
        help = "Optional path to also write the full report JSON to"
    )]
    out: String,
) -> Result<Value> {
    let report = if out.is_empty() {
        frontier::full_report().map_err(|e| NounVerbError::argument_error(e.to_string()))?
    } else {
        frontier::write_report(std::path::Path::new(&out))
            .map_err(|e| NounVerbError::argument_error(e.to_string()))?
    };
    serde_json::to_value(&report).map_err(|e| NounVerbError::argument_error(e.to_string()))
}

/// Print just the summary counts (total / evaluated / passing / coverage /
/// pass_rate / failures) without the full per-cell matrix.
#[verb]
pub fn summary() -> Result<Value> {
    let report = frontier::frontier_report()
        .map_err(|e| NounVerbError::argument_error(e.to_string()))?;
    serde_json::to_value(&report).map_err(|e| NounVerbError::argument_error(e.to_string()))
}

/// Report the counts of admitted/executed vs. refused cells and confirm
/// they partition the evaluated set (no third, silently-dropped state).
#[verb]
pub fn counts() -> Result<Value> {
    let matrix = frontier::build_frontier_matrix()
        .map_err(|e| NounVerbError::argument_error(e.to_string()))?;
    let evaluated = matrix.evaluated();
    let refused = frontier::refused_count()
        .map_err(|e| NounVerbError::argument_error(e.to_string()))?;
    let admitted = evaluated.saturating_sub(refused);
    Ok(json!({
        "total_cells": matrix.total(),
        "evaluated_cells": evaluated,
        "admitted_or_executed": admitted,
        "refused": refused,
        "coverage": matrix.coverage(),
        "pass_rate": matrix.pass_rate(),
    }))
}
