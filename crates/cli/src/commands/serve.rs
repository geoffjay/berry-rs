//! Serve command - start the HTTP server.

use anyhow::Result;

use crate::output::print_info;

/// Serve command arguments.
#[derive(Debug)]
pub struct ServeArgs {
    pub port: u16,
    pub host: String,
    /// Kept for CLI compatibility, but server always runs in foreground now.
    #[allow(dead_code)]
    pub foreground: bool,
}

/// Run the serve command.
pub async fn run(args: ServeArgs) -> Result<()> {
    // Initialize logging
    berry::logging::init();

    print_info(&format!(
        "Starting Berry server on {}:{}...",
        args.host, args.port
    ));

    let config = berry_server::ServerConfig {
        port: args.port,
        host: args.host,
    };

    // Run the server directly (blocks until shutdown)
    berry_server::run_server(config).await
}
