//! Berry MCP Server
//!
//! Model Context Protocol server for the Berry memory system.
//! Communicates over stdio using JSON-RPC 2.0.

use clap::Parser;

use berry_mcp::{McpConfig, run_mcp};

/// Berry MCP Server
#[derive(Parser)]
#[command(name = "berry-mcp")]
#[command(version, about = "Berry memory system MCP server")]
struct Args {
    /// Berry server URL
    #[arg(short = 's', long = "server-url")]
    server_url: Option<String>,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let config = McpConfig {
        server_url: args.server_url,
        verbose: args.verbose,
    };

    run_mcp(config).await
}
