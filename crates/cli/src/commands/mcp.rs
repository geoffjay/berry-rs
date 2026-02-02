//! MCP command - start the MCP server.

use anyhow::Result;

use crate::output::print_info;

/// MCP command arguments.
#[derive(Debug)]
pub struct McpArgs {
    pub server_url: Option<String>,
    pub verbose: bool,
}

/// Run the mcp command.
pub async fn run(args: McpArgs) -> Result<()> {
    print_info("Starting Berry MCP server...");

    let config = berry_mcp::McpConfig {
        server_url: args.server_url,
        verbose: args.verbose,
    };

    // Run the MCP server directly (blocks until stdin closes)
    berry_mcp::run_mcp(config).await
}
