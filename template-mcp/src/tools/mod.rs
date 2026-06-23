//! MCP tool implementations.
//!
//! Add each tool as a module here and re-export the structs you want to
//! expose from the `ToolSet`.

pub mod example;

use rmcp::{tool, ServerHandler};

/// Aggregate tool set — collects all tools into a single MCP service.
///
/// Add new tools by:
/// 1. Creating `src/tools/<name>.rs` with a `#[tool(tool_box)]` impl.
/// 2. Adding a field here and wiring it in the `#[tool(tool_box)]` impl block.
#[derive(Debug, Default, Clone)]
pub struct ToolSet {
    pub example: example::ExampleTool,
}

impl ToolSet {
    pub fn new() -> Self {
        Self::default()
    }
}

#[tool(tool_box)]
impl ToolSet {
    // Delegate to member tools by forwarding their tool_box impls.
    // rmcp will merge their tool registrations into this service.
}

#[tool(tool_box)]
impl ServerHandler for ToolSet {}
