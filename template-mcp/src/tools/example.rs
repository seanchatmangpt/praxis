//! Example MCP tool demonstrating the house pattern.
//!
//! Replace this with your actual domain logic. The structure to keep:
//! - [`TimeWindowArgs`] for shared time-window parameters
//! - [`CommonResponse<T>`] with mandatory `passed` + `result_hash` fields
//! - BLAKE3 integrity stamp computed by `CommonResponse::ok()`
//! - [`ToolResultCache`] get-before/insert-after around any tool that is a
//!   pure function of its input (see `analyse` below) — the same idiom
//!   praxis's `mcp_lawobject_server` and bcinr-mcp use. Never wrap a
//!   side-effecting or non-deterministic tool this way.

use rmcp::{tool, ServerHandler, ToolError};
use serde::{Deserialize, Serialize};

use crate::cache::ToolResultCache;
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
pub struct ExampleTool {
    /// Cache for `analyse`'s output, keyed on `(tool name, BLAKE3(canonical
    /// input))`. `analyse` is a pure function of `TimeWindowArgs` today
    /// (`run_analysis` reads no external mutable state), so caching it is
    /// safe; if you replace `run_analysis` with something that reads a
    /// database, file, or clock, either drop the cache or key in an
    /// `environment_digest` capturing that external state (see
    /// `ToolCacheKey` in `crate::cache` for that richer key shape).
    cache: ToolResultCache,
}

impl ExampleTool {
    pub fn new() -> Self { Self::default() }
}

#[tool(tool_box)]
impl ExampleTool {
    /// Analyse events within the specified time window.
    ///
    /// Returns a [`CommonResponse`] with `passed: true` on success,
    /// `passed: false` on error. The `result_hash` field is always a BLAKE3
    /// hex digest of the serialised payload.
    ///
    /// Demonstrates the cache house pattern: canonicalize the input,
    /// build the key, check the cache before doing any work, and only
    /// insert on a successful (`passed: true`) result — an error response
    /// is never cached, since a transient failure shouldn't be replayed
    /// forever.
    #[tool(description = "Analyse events in the given time window and return a summary.")]
    async fn analyse(
        &self,
        #[tool(description = "Time window (hours) and result limit")] args: TimeWindowArgs,
    ) -> Result<String, ToolError> {
        let canonical_input = serde_json::to_vec(&args).unwrap_or_default();
        let key = ToolResultCache::key("analyse", &canonical_input);

        if let Some(cached) = self.cache.get(&key).await {
            return Ok(cached);
        }

        let response = match run_analysis(&args) {
            Ok(data) => CommonResponse::ok(data),
            Err(e)   => CommonResponse::<AnalysisResult>::err(e.to_string()),
        };
        let text = serde_json::to_string_pretty(&response)
            .map_err(|e| ToolError::ExecutionError(e.to_string()))?;

        if response.passed {
            self.cache.insert(key, text.clone()).await;
        }
        Ok(text)
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

    #[tokio::test]
    async fn analyse_second_call_hits_cache() {
        let tool = ExampleTool::new();
        let args = TimeWindowArgs { hours: 12, limit: 5 };

        let first = tool.analyse(args.clone()).await.expect("first call");
        let second = tool.analyse(args).await.expect("second call should hit cache");
        assert_eq!(first, second);
    }

    #[tokio::test]
    async fn analyse_different_input_misses_cache() {
        let tool = ExampleTool::new();
        let a = tool.analyse(TimeWindowArgs { hours: 1, limit: 5 }).await.expect("call a");
        let b = tool.analyse(TimeWindowArgs { hours: 2, limit: 5 }).await.expect("call b");
        assert_ne!(a, b);
    }
}
