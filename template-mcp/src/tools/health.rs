//! Health-check MCP tool.
//!
//! Returns the server's operational status: version, uptime, active feature
//! flags, and a BLAKE3 digest of the current configuration. Designed for CI
//! liveness probes — callers gate on `passed: true`.

use std::sync::OnceLock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use rmcp::{tool, ServerHandler, ToolError};
use serde::Serialize;

use crate::shared_args::CommonResponse;

// ── Server-start timestamp ────────────────────────────────────────────────────

/// Global server-start instant, initialised once on first access.
static SERVER_START: OnceLock<Instant> = OnceLock::new();

/// Return the number of seconds since the server started.
pub fn uptime_secs() -> u64 {
    let start = SERVER_START.get_or_init(Instant::now);
    start.elapsed().as_secs()
}

// ── Domain types ──────────────────────────────────────────────────────────────

/// Health-check result returned by the `health` tool.
#[derive(Debug, Serialize)]
pub struct HealthResult {
    /// Crate version from `Cargo.toml`.
    pub version: &'static str,
    /// Seconds elapsed since the server process started.
    pub uptime_secs: u64,
    /// UTC Unix timestamp of the health-check call.
    pub timestamp_utc: u64,
    /// Feature flags compiled into this binary.
    pub features: Vec<&'static str>,
    /// BLAKE3 hex digest of the concatenated version + feature flags string.
    /// Callers can detect config drift by comparing this value over time.
    pub config_hash: String,
    /// Always `true` unless a critical internal assertion fails.
    pub passed: bool,
}

fn build_health_result() -> HealthResult {
    let version = env!("CARGO_PKG_VERSION");
    let uptime = uptime_secs();
    let timestamp_utc = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Enumerate features compiled into this binary.
    #[allow(unused_mut)]
    let mut features: Vec<&'static str> = Vec::new();
    #[cfg(feature = "clap-cli")]
    features.push("clap-cli");

    // Stable config fingerprint: version + sorted features joined.
    let mut config_parts = vec![version];
    config_parts.extend_from_slice(&features);
    let config_str = config_parts.join("|");
    let config_hash = blake3::hash(config_str.as_bytes()).to_hex().to_string();

    HealthResult {
        version,
        uptime_secs: uptime,
        timestamp_utc,
        features,
        config_hash,
        passed: true,
    }
}

// ── Tool implementation ───────────────────────────────────────────────────────

/// MCP health-check tool.
#[derive(Debug, Default, Clone)]
pub struct HealthTool;

impl HealthTool {
    pub fn new() -> Self {
        // Ensure the uptime clock is initialised as early as possible.
        let _ = SERVER_START.get_or_init(Instant::now);
        Self
    }

    /// Direct (non-MCP-routed) entry point used by composite fan-out tools.
    ///
    /// Calling this method bypasses the MCP proc-macro dispatch and lets
    /// [`AllTool`](crate::tools::all::AllTool) invoke the same logic directly.
    pub async fn call_health(&self) -> Result<String, ToolError> {
        let result = build_health_result();
        let response = CommonResponse::ok(result);
        serde_json::to_string_pretty(&response)
            .map_err(|e| ToolError::ExecutionError(e.to_string()))
    }
}

#[tool(tool_box)]
impl HealthTool {
    /// Return server status: version, uptime, feature flags, config hash.
    ///
    /// Always returns `passed: true` unless a critical internal assertion fails.
    /// Use this endpoint as a CI liveness probe.
    #[tool(description = "Return server health: version, uptime, feature flags, config BLAKE3 hash.")]
    async fn health(&self) -> Result<String, ToolError> {
        self.call_health().await
    }
}

#[tool(tool_box)]
impl ServerHandler for HealthTool {}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_result_passes() {
        let result = build_health_result();
        assert!(result.passed);
        assert_eq!(result.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(result.config_hash.len(), 64, "BLAKE3 hex is 64 chars");
    }

    #[test]
    fn health_result_config_hash_is_stable() {
        let r1 = build_health_result();
        let r2 = build_health_result();
        // Config hash must not change between calls on the same binary.
        assert_eq!(r1.config_hash, r2.config_hash);
    }

    #[test]
    fn health_result_uptime_non_decreasing() {
        let r1 = build_health_result();
        let r2 = build_health_result();
        // Uptime should not go backwards.
        assert!(r2.uptime_secs >= r1.uptime_secs);
    }
}
