//! Shared argument types reused across all MCP tools.
//!
//! ## House rules
//!
//! - Every tool that operates over a historical window embeds [`TimeWindowArgs`].
//! - Every tool response is wrapped in [`CommonResponse<T>`].
//! - The `passed` field is **always** present, even on error — it is the CI gate field.
//! - The `result_hash` field is the BLAKE3 hex digest of the serialised `data`.

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
