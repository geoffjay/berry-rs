//! Berry HTTP Server
//!
//! Provides RESTful API endpoints for memory operations.

use clap::Parser;

use berry_server::{run_server, ServerConfig};

/// Berry HTTP Server
#[derive(Parser)]
#[command(name = "berry-server")]
#[command(version, about = "Berry memory system HTTP server")]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "4114")]
    port: u16,

    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    berry::logging::init();

    let args = Args::parse();

    let config = ServerConfig {
        port: args.port,
        host: args.host,
    };

    run_server(config).await
}
