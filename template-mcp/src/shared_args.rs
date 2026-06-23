//! Shared argument types reused across all MCP tools.
//!
//! ## House rules
//!
//! - Every tool that operates over a historical window embeds [`TimeWindowArgs`].
//! - Every tool response is wrapped in [`CommonResponse<T>`].
//! - The `passed` field is **always** present, even on error — it is the CI gate field.
//! - The `result_hash` field is the BLAKE3 hex digest of the serialised `data`.
//!
//! ## CLI mirror
//!
//! When the same service is exposed as both MCP and a binary CLI, use
//! [`TimeWindowCliArgs`] (feature `clap-cli`) as the `clap::Args` struct and
//! convert it into [`TimeWindowArgs`] with `Into::into`. The two types are
//! intentionally kept separate so that MCP-only builds do not pull in `clap`.

use serde::{Deserialize, Serialize};

/// Standard time-window arguments shared by analysis tools.
///
/// Embed in any tool struct that operates over historical data:
/// ```rust,ignore
/// #[tool(description = "Analyse the last N hours")]
/// async fn analyse(&self, #[tool(description = "Window + limit")] args: TimeWindowArgs) -> ... { }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindowArgs {
    /// How many hours of history to include (default: 24).
    #[serde(default = "default_hours")]
    pub hours: u32,
    /// Maximum number of results to return (default: 1000).
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_hours() -> u32 { 24 }
fn default_limit() -> usize { 1000 }

impl Default for TimeWindowArgs {
    fn default() -> Self {
        Self { hours: default_hours(), limit: default_limit() }
    }
}

/// CLI mirror of [`TimeWindowArgs`] that derives [`clap::Args`].
///
/// Enable with the `clap-cli` feature. Convert to [`TimeWindowArgs`] via
/// `Into::into` before passing to tool implementations so that both MCP and
/// CLI entry-points share the same business logic.
///
/// ## Usage
///
/// ```rust,ignore
/// #[cfg(feature = "clap-cli")]
/// use crate::shared_args::TimeWindowCliArgs;
///
/// #[derive(clap::Parser)]
/// struct Cli {
///     #[command(flatten)]
///     window: TimeWindowCliArgs,
/// }
///
/// let args: TimeWindowArgs = cli.window.into();
/// ```
#[cfg(feature = "clap-cli")]
#[derive(Debug, Clone, clap::Args)]
pub struct TimeWindowCliArgs {
    /// How many hours of history to include.
    #[arg(long, default_value_t = 24, help = "Hours of history to analyse")]
    pub hours: u32,
    /// Maximum number of results to return.
    #[arg(long, default_value_t = 1000, help = "Maximum result count")]
    pub limit: usize,
}

#[cfg(feature = "clap-cli")]
impl From<TimeWindowCliArgs> for TimeWindowArgs {
    fn from(cli: TimeWindowCliArgs) -> Self {
        Self { hours: cli.hours, limit: cli.limit }
    }
}

#[cfg(feature = "clap-cli")]
impl From<TimeWindowArgs> for TimeWindowCliArgs {
    fn from(args: TimeWindowArgs) -> Self {
        Self { hours: args.hours, limit: args.limit }
    }
}

/// Standard response envelope for every tool call.
///
/// `passed` is always present — it enables CI scripts to gate on tool success
/// without parsing the full payload:
/// ```bash
/// result=$(mcp call analyse); [ "$(jq -r .passed <<< "$result")" = "true" ] || exit 1
/// ```
///
/// `result_hash` is the BLAKE3 hex digest of the serialised `data` field.
/// It proves the response body was not modified after generation.
#[derive(Debug, Serialize)]
pub struct CommonResponse<T: Serialize> {
    /// True when the tool completed successfully.
    pub passed: bool,
    /// BLAKE3 hex digest of `serde_json::to_vec(&data)`. Always present.
    pub result_hash: String,
    /// Tool payload, or `null` on error.
    pub data: Option<T>,
    /// Human-readable error message, or `null` on success.
    pub error: Option<String>,
}

impl<T: Serialize> CommonResponse<T> {
    /// Construct a successful response. Computes `result_hash` automatically.
    pub fn ok(data: T) -> Self {
        let json = serde_json::to_vec(&data).unwrap_or_default();
        let result_hash = blake3::hash(&json).to_hex().to_string();
        Self { passed: true, result_hash, data: Some(data), error: None }
    }

    /// Construct an error response.
    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            passed: false,
            result_hash: blake3::hash(b"error").to_hex().to_string(),
            data: None,
            error: Some(msg.into()),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_window_defaults() {
        let args = TimeWindowArgs::default();
        assert_eq!(args.hours, 24);
        assert_eq!(args.limit, 1000);
    }

    #[cfg(feature = "clap-cli")]
    #[test]
    fn cli_args_round_trips_to_time_window() {
        let cli = TimeWindowCliArgs { hours: 48, limit: 500 };
        let args: TimeWindowArgs = cli.into();
        assert_eq!(args.hours, 48);
        assert_eq!(args.limit, 500);
    }

    #[cfg(feature = "clap-cli")]
    #[test]
    fn time_window_round_trips_to_cli() {
        let args = TimeWindowArgs { hours: 6, limit: 200 };
        let cli: TimeWindowCliArgs = args.into();
        assert_eq!(cli.hours, 6);
        assert_eq!(cli.limit, 200);
    }
}
