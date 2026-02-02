//! MCP command - start the MCP server.

use std::process::Command;

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
    let mcp_binary = find_mcp_binary()?;

    print_info("Starting Berry MCP server...");

    let mut cmd = Command::new(&mcp_binary);

    if let Some(url) = args.server_url {
        cmd.args(["--server-url", &url]);
    }

    if args.verbose {
        cmd.arg("--verbose");
    }

    // MCP servers communicate over stdio, so we don't redirect them
    let status = cmd.status()?;

    if !status.success() {
        anyhow::bail!("MCP server exited with status: {}", status);
    }

    Ok(())
}

/// Find the MCP server binary.
fn find_mcp_binary() -> Result<String> {
    let candidates = [
        "berry-mcp",
        "./berry-mcp",
        "../mcp/target/release/berry-mcp",
        "../mcp/target/debug/berry-mcp",
    ];

    for candidate in candidates {
        if which::which(candidate).is_ok() {
            return Ok(candidate.to_string());
        }
    }

    Ok("berry-mcp".to_string())
}
