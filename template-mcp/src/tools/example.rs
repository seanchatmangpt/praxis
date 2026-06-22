//! Example MCP tool demonstrating the house pattern.
//!
//! Replace this with your actual domain logic. The structure to keep:
//! - [`TimeWindowArgs`] for shared time-window parameters
//! - [`CommonResponse<T>`] with mandatory `passed` + `result_hash` fields
//! - BLAKE3 integrity stamp computed by `CommonResponse::ok()`

use rmcp::{tool, ServerHandler, ToolError};
use serde::{Deserialize, Serialize};

use crate::shared_args::{CommonResponse, TimeWindowArgs};

// ── Domain types ─────────────────────────────────────────────────────────────

/// Replace with your actual result type.
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub summary: String,
    pub item_count: usize,
    pub window_hours: u32,
}

// ── Tool implementation ───────────────────────────────────────────────────────

#[derive(Debug, Default, Clone)]
pub struct ExampleTool;

impl ExampleTool {
    pub fn new() -> Self { Self }
}

#[tool(tool_box)]
impl ExampleTool {
    /// Analyse events within the specified time window.
    ///
    /// Returns a [`CommonResponse`] with `passed: true` on success,
    /// `passed: false` on error. The `result_hash` field is always a BLAKE3
    /// hex digest of the serialised payload.
    #[tool(description = "Analyse events in the given time window and return a summary.")]
    async fn analyse(
        &self,
        #[tool(description = "Time window (hours) and result limit")] args: TimeWindowArgs,
    ) -> Result<String, ToolError> {
        let response = match run_analysis(&args) {
            Ok(data) => CommonResponse::ok(data),
            Err(e)   => CommonResponse::<AnalysisResult>::err(e.to_string()),
        };
        serde_json::to_string_pretty(&response)
            .map_err(|e| ToolError::ExecutionError(e.to_string()))
    }
}

#[tool(tool_box)]
impl ServerHandler for ExampleTool {}

// ── Private implementation ────────────────────────────────────────────────────

fn run_analysis(args: &TimeWindowArgs) -> anyhow::Result<AnalysisResult> {
    // Replace with real domain logic.
    Ok(AnalysisResult {
        summary: format!("Analysed {} hours of history", args.hours),
        item_count: 0,
        window_hours: args.hours,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_analysis_defaults() {
        let args = TimeWindowArgs::default();
        let result = run_analysis(&args).unwrap();
        assert_eq!(result.window_hours, 24);
        assert_eq!(result.item_count, 0);
    }

    #[test]
    fn common_response_ok_has_hash() {
        let data = AnalysisResult { summary: "test".into(), item_count: 1, window_hours: 1 };
        let resp = CommonResponse::ok(data);
        assert!(resp.passed);
        assert_eq!(resp.result_hash.len(), 64, "BLAKE3 hex is 64 chars");
        assert!(resp.error.is_none());
    }

    #[test]
    fn common_response_err_not_passed() {
        let resp = CommonResponse::<AnalysisResult>::err("something went wrong");
        assert!(!resp.passed);
        assert!(resp.data.is_none());
        assert!(resp.error.is_some());
    }
}
