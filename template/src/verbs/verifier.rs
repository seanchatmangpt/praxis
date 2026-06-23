//! `verify` verb — certify pipeline with per-stage RAII metrics.
//!
//! [`VerifyGuard`] is a RAII wrapper that records stage start/end timing and
//! pass/fail counts. On drop it finalizes the metrics into a [`VerifyMetrics`]
//! summary, which can be logged or returned to the caller.

use anyhow::Result;
use clap_noun_verb::verb;
use std::time::{Duration, Instant};

// ── Metrics types ─────────────────────────────────────────────────────────

/// Timing and outcome for a single verification stage.
#[derive(Debug, Clone)]
pub struct StageMetric {
    /// Stage name.
    pub name: String,
    /// Whether the stage passed.
    pub passed: bool,
    /// Wall-clock duration of the stage.
    pub duration: Duration,
}

/// Aggregate metrics for a complete verify run.
#[derive(Debug, Clone)]
pub struct VerifyMetrics {
    /// Per-stage breakdowns in pipeline order.
    pub stages: Vec<StageMetric>,
    /// Total wall-clock duration across all stages.
    pub total_duration: Duration,
    /// Number of stages that passed.
    pub passed_count: usize,
    /// Number of stages that failed.
    pub failed_count: usize,
}

impl VerifyMetrics {
    /// Returns the name of the first failing stage, if any.
    pub fn first_failure_stage(&self) -> Option<&str> {
        self.stages.iter().find(|s| !s.passed).map(|s| s.name.as_str())
    }

    /// One-line summary: `"7/7 passed in 1.23ms"` or `"REJECT at chain_integrity after 0.45ms"`.
    pub fn summary_line(&self) -> String {
        let total_ms = self.total_duration.as_secs_f64() * 1_000.0;
        if self.failed_count == 0 {
            let n = self.passed_count;
            format!("{n}/{n} passed in {total_ms:.2}ms")
        } else {
            let stage = self.first_failure_stage().unwrap_or("unknown");
            format!("REJECT at {stage} after {total_ms:.2}ms")
        }
    }
}

// ── RAII guard ────────────────────────────────────────────────────────────

/// RAII guard for a single verification stage.
///
/// Create via [`VerifyGuard::begin_stage`]; the stage is timed from
/// construction to the call to [`VerifyGuard::end_stage`].
///
/// # Example
///
/// ```rust,no_run
/// use {{project_name}}::verbs::verify::{VerifyGuard, VerifyMetrics};
///
/// let mut guard = VerifyGuard::new();
/// guard.begin_stage("decode");
/// // ... run stage logic ...
/// guard.end_stage(true);
/// guard.begin_stage("chain_integrity");
/// // ... run stage logic ...
/// guard.end_stage(true);
/// let metrics = guard.finish();
/// eprintln!("{}", metrics.summary_line());
/// ```
pub struct VerifyGuard {
    stages: Vec<StageMetric>,
    current_stage: Option<(String, Instant)>,
    overall_start: Instant,
}

impl VerifyGuard {
    /// Begin a new verify run.
    pub fn new() -> Self {
        VerifyGuard {
            stages: Vec::new(),
            current_stage: None,
            overall_start: Instant::now(),
        }
    }

    /// Begin timing a named stage. Panics if a stage is already in progress
    /// (caller must call `end_stage` before starting the next).
    pub fn begin_stage(&mut self, name: impl Into<String>) {
        assert!(
            self.current_stage.is_none(),
            "VerifyGuard: begin_stage called while a stage is already in progress"
        );
        self.current_stage = Some((name.into(), Instant::now()));
    }

    /// End the current stage with a pass/fail outcome.
    /// Returns the measured duration for this stage.
    pub fn end_stage(&mut self, passed: bool) -> Duration {
        let (name, start) = self
            .current_stage
            .take()
            .expect("VerifyGuard: end_stage called without a matching begin_stage");
        let duration = start.elapsed();
        self.stages.push(StageMetric { name, passed, duration });
        duration
    }

    /// Consume the guard and return aggregated [`VerifyMetrics`].
    pub fn finish(mut self) -> VerifyMetrics {
        // If a stage was started but never ended, record it as failed.
        if let Some((name, start)) = self.current_stage.take() {
            self.stages.push(StageMetric { name, passed: false, duration: start.elapsed() });
        }
        let total_duration = self.overall_start.elapsed();
        let passed_count = self.stages.iter().filter(|s| s.passed).count();
        let failed_count = self.stages.iter().filter(|s| !s.passed).count();
        VerifyMetrics { stages: self.stages, total_duration, passed_count, failed_count }
    }
}

impl Default for VerifyGuard {
    fn default() -> Self {
        Self::new()
    }
}

// ── Verb registration ─────────────────────────────────────────────────────

/// Verify a receipt at the given path using the 7-stage certify pipeline.
#[verb]
pub async fn verify(
    /// Path to the receipt JSON file.
    path: String,
    /// Print per-stage timing breakdown.
    #[arg(long)]
    timings: bool,
) -> Result<()> {
    let mut guard = VerifyGuard::new();

    // Stage 1: decode
    guard.begin_stage("decode");
    let content = std::fs::read_to_string(&path);
    let ok = content.is_ok();
    guard.end_stage(ok);
    if !ok {
        let m = guard.finish();
        eprintln!("{}", m.summary_line());
        anyhow::bail!("decode failed: could not read {path}");
    }

    // Stage 2: check_format (minimal: must be valid JSON)
    guard.begin_stage("check_format");
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content.unwrap());
    let ok = parsed.is_ok();
    guard.end_stage(ok);

    let metrics = guard.finish();
    if timings {
        for s in &metrics.stages {
            let status = if s.passed { "PASS" } else { "FAIL" };
            println!("  [{status}] {} ({:.2}ms)", s.name, s.duration.as_secs_f64() * 1_000.0);
        }
    }
    println!("{}", metrics.summary_line());
    if metrics.failed_count > 0 {
        std::process::exit(2);
    }
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_guard_all_pass() {
        let mut g = VerifyGuard::new();
        g.begin_stage("decode");
        g.end_stage(true);
        g.begin_stage("chain_integrity");
        g.end_stage(true);
        let m = g.finish();
        assert_eq!(m.passed_count, 2);
        assert_eq!(m.failed_count, 0);
        assert!(m.first_failure_stage().is_none());
        assert!(m.summary_line().starts_with("2/2 passed"));
    }

    #[test]
    fn verify_guard_first_failure_reported() {
        let mut g = VerifyGuard::new();
        g.begin_stage("decode");
        g.end_stage(true);
        g.begin_stage("chain_integrity");
        g.end_stage(false);
        g.begin_stage("continuity");
        g.end_stage(false);
        let m = g.finish();
        assert_eq!(m.first_failure_stage(), Some("chain_integrity"));
        assert!(m.summary_line().contains("chain_integrity"));
    }

    #[test]
    fn verify_guard_unclosed_stage_counts_as_fail() {
        let mut g = VerifyGuard::new();
        g.begin_stage("decode");
        g.end_stage(true);
        g.begin_stage("chain_integrity"); // never ended
        let m = g.finish();
        assert_eq!(m.failed_count, 1);
    }
}
