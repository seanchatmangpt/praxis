//! MCP server entry point for {{project-name}}.
//!
//! Transport: stdio (stdin/stdout). The MCP host (e.g. Claude Desktop) launches
//! this binary and communicates over its stdio streams.
//!
//! Add more tools by creating modules under `src/tools/` and wiring them into
//! the service in `main()`.

use anyhow::Result;
use rmcp::{transport::io::stdio, ServiceExt};
use tracing_subscriber::{fmt, EnvFilter};

mod shared_args;
mod tools;

#[tokio::main]
async fn main() -> Result<()> {
    // Log to stderr (stdout is the MCP transport)
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("{{project-name}}-mcp starting");

    let service = tools::ToolSet::new().serve(stdio()).await?;
    service.waiting().await?;

    tracing::info!("{{project-name}}-mcp shutting down");
    Ok(())
}
