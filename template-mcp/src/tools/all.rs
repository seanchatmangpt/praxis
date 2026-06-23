//! Composite fan-out tool: `analyse_all`.
//!
//! Calls every registered analysis tool in parallel via [`tokio::join!`] and
//! returns an aggregated [`AllResults`] struct wrapped in the standard
//! [`CommonResponse`] envelope.
//!
//! This is the "umbrella" tool that MCP clients should call when they want a
//! full picture without issuing individual tool calls. The hash in the response
//! covers the entire aggregated payload, so callers can detect partial failures
//! by checking individual `passed` sub-fields.

use rmcp::{tool, ServerHandler, ToolError};
use serde::Serialize;

use crate::shared_args::{CommonResponse, TimeWindowArgs};
use crate::tools::{example::ExampleTool, health::HealthTool};

// ── Aggregate result type ─────────────────────────────────────────────────────

/// Aggregated results from all analysis tools, returned by `analyse_all`.
///
/// Each field holds the parsed JSON output produced by the corresponding tool.
/// This keeps the aggregate response self-contained — callers can decode each
/// field independently without an extra round-trip.
#[derive(Debug, Serialize)]
pub struct AllResults {
    /// JSON output of `ExampleTool::analyse`.
    pub example: serde_json::Value,
    /// JSON output of `HealthTool::health`.
    pub health: serde_json::Value,
    /// `true` only when every sub-tool returned `passed: true`.
    pub all_passed: bool,
}

// ── Composite tool ────────────────────────────────────────────────────────────

/// Composite MCP tool that fans out to all registered tools in parallel.
///
/// Add new tools by:
/// 1. Adding a field of the new tool type here.
/// 2. Adding a parallel branch in the `tokio::join!` call inside `run_all`.
/// 3. Adding the result field to [`AllResults`].
#[derive(Debug, Clone)]
pub struct AllTool {
    example: ExampleTool,
    health: HealthTool,
}

impl AllTool {
    pub fn new() -> Self {
        Self {
            example: ExampleTool::new(),
            health: HealthTool::new(),
        }
    }

    /// Direct (non-MCP-routed) entry point — shared by the MCP handler and tests.
    pub async fn run_all(&self, args: TimeWindowArgs) -> Result<String, ToolError> {
        // Fan-out: run all tools concurrently.
        let (example_raw, health_raw) = tokio::join!(
            self.example.call_analyse(args.clone()),
            self.health.call_health(),
        );

        // Parse each sub-result into a JSON Value so we can inspect `passed`.
        let example_val = parse_tool_output(example_raw)?;
        let health_val = parse_tool_output(health_raw)?;

        let all_passed = sub_passed(&example_val) && sub_passed(&health_val);

        let aggregate = AllResults {
            example: example_val,
            health: health_val,
            all_passed,
        };

        let response = CommonResponse::ok(aggregate);
        serde_json::to_string_pretty(&response)
            .map_err(|e| ToolError::ExecutionError(e.to_string()))
    }
}

impl Default for AllTool {
    fn default() -> Self {
        Self::new()
    }
}

#[tool(tool_box)]
impl AllTool {
    /// Run all analysis tools in parallel and return aggregated results.
    ///
    /// Each sub-tool is called concurrently via `tokio::join!`. The response
    /// `passed` field is `true` only when every sub-tool succeeded. The
    /// `result_hash` covers the entire aggregated payload.
    #[tool(description = "Run every analysis tool in parallel and return aggregated results.")]
    async fn analyse_all(
        &self,
        #[tool(description = "Time window (hours) and result limit — forwarded to all analysis tools")]
        args: TimeWindowArgs,
    ) -> Result<String, ToolError> {
        self.run_all(args).await
    }
}

#[tool(tool_box)]
impl ServerHandler for AllTool {}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Parse a raw tool JSON string into a `serde_json::Value`.
///
/// If the sub-tool call returned an error, wraps it in a JSON object with
/// `passed: false` so the aggregate can still complete.
fn parse_tool_output(raw: Result<String, ToolError>) -> Result<serde_json::Value, ToolError> {
    match raw {
        Ok(s) => serde_json::from_str(&s).map_err(|e| ToolError::ExecutionError(e.to_string())),
        Err(e) => {
            // Sub-tool error: produce a `passed: false` sentinel value so the
            // aggregate can still complete rather than short-circuiting.
            Ok(serde_json::json!({
                "passed": false,
                "error": e.to_string(),
                "data": null,
                "result_hash": blake3::hash(b"tool-error").to_hex().to_string(),
            }))
        }
    }
}

/// Extract the `passed` boolean from a sub-tool JSON response.
fn sub_passed(val: &serde_json::Value) -> bool {
    val.get("passed")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn analyse_all_returns_all_passed() {
        let tool = AllTool::new();
        let raw = tool.run_all(TimeWindowArgs::default()).await.unwrap();
        let resp: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(resp["passed"], true, "outer CommonResponse must be passed");
        assert_eq!(resp["data"]["all_passed"], true, "all sub-tools should pass");
        assert!(resp["data"]["example"].is_object(), "example field present");
        assert!(resp["data"]["health"].is_object(), "health field present");
    }

    #[tokio::test]
    async fn analyse_all_result_hash_present() {
        let tool = AllTool::new();
        let raw = tool.run_all(TimeWindowArgs::default()).await.unwrap();
        let resp: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let hash = resp["result_hash"].as_str().unwrap_or("");
        assert_eq!(hash.len(), 64, "BLAKE3 hex is 64 chars");
    }

    #[test]
    fn sub_passed_true_when_field_true() {
        let val = serde_json::json!({ "passed": true });
        assert!(sub_passed(&val));
    }

    #[test]
    fn sub_passed_false_when_field_false() {
        let val = serde_json::json!({ "passed": false });
        assert!(!sub_passed(&val));
    }

    #[test]
    fn sub_passed_false_when_field_missing() {
        let val = serde_json::json!({ "other": 1 });
        assert!(!sub_passed(&val));
    }
}
