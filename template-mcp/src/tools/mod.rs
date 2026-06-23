//! MCP tool implementations.
//!
//! Add each tool as a module here and re-export the structs you want to
//! expose from the `ToolSet`.
//!
//! ## Registered tools
//!
//! | Tool struct   | MCP name        | Purpose                                      |
//! |---------------|-----------------|----------------------------------------------|
//! | `ExampleTool` | `analyse`       | Template example — replace with domain logic |
//! | `HealthTool`  | `health`        | Server liveness probe for CI                 |
//! | `AllTool`     | `analyse_all`   | Fan-out: runs all tools in parallel          |

pub mod all;
pub mod example;
pub mod health;

use rmcp::{tool, ServerHandler};

/// Aggregate tool set — collects all tools into a single MCP service.
///
/// Add new tools by:
/// 1. Creating `src/tools/<name>.rs` with a `#[tool(tool_box)]` impl.
/// 2. Adding a `pub mod <name>;` entry above.
/// 3. Adding a field here.
/// 4. Adding the new tool to `AllTool` in `src/tools/all.rs` so it participates
///    in the composite fan-out.
#[derive(Debug, Clone)]
pub struct ToolSet {
    pub example: example::ExampleTool,
    pub health: health::HealthTool,
    pub all: all::AllTool,
}

impl ToolSet {
    pub fn new() -> Self {
        Self {
            example: example::ExampleTool::new(),
            health: health::HealthTool::new(),
            all: all::AllTool::new(),
        }
    }
}

impl Default for ToolSet {
    fn default() -> Self {
        Self::new()
    }
}

#[tool(tool_box)]
impl ToolSet {
    // Delegate to member tools by forwarding their tool_box impls.
    // rmcp will merge their tool registrations into this service.
}

#[tool(tool_box)]
impl ServerHandler for ToolSet {}
