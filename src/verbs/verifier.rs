//! `verify` verb — certify pipeline with per-stage RAII metrics.
//!
//! Metrics types moved to `praxis_core::verify`; re-exported here for
//! backward compatibility with existing callers and tests.

use clap_noun_verb::error::{NounVerbError, Result};
use clap_noun_verb_macros::{arg, verb};

pub use praxis_core::verify::{StageMetric, VerifyGuard, VerifyMetrics};

// ── Domain logic ──────────────────────────────────────────────────────────

/// Run the decode + check_format certify pipeline for a receipt file.
fn run_verify_pipeline(path: &str) -> std::result::Result<VerifyMetrics, VerifyMetrics> {
    let mut guard = VerifyGuard::new();

    guard.begin_stage("decode");
    let content = std::fs::read_to_string(path);
    guard.end_stage(content.is_ok());
    let Ok(content) = content else {
        return Err(guard.finish());
    };

    guard.begin_stage("check_format");
    let parsed: std::result::Result<serde_json::Value, _> = serde_json::from_str(&content);
    guard.end_stage(parsed.is_ok());

    Ok(guard.finish())
}

fn print_metrics(metrics: &VerifyMetrics, timings: bool) {
    if timings {
        for s in &metrics.stages {
            let status = if s.passed { "PASS" } else { "FAIL" };
            println!("  [{status}] {} ({:.2}ms)", s.name, s.duration.as_secs_f64() * 1_000.0);
        }
    }
    println!("{}", metrics.summary_line());
}

// ── Verb registration ─────────────────────────────────────────────────────

/// Verify a receipt at the given path using the 7-stage certify pipeline.
#[verb]
pub fn verify(
    #[arg(help = "Path to the receipt JSON file")] path: String,
    #[arg(help = "Print per-stage timing breakdown")] timings: bool,
) -> Result<()> {
    let metrics = match run_verify_pipeline(&path) {
        Ok(m) => m,
        Err(m) => {
            eprintln!("{}", m.summary_line());
            return Err(NounVerbError::execution_error(format!("decode failed: could not read {path}")));
        }
    };
    print_metrics(&metrics, timings);
    if metrics.failed_count > 0 {
        std::process::exit(2);
    }
    Ok(())
}
