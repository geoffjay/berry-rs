//! Serve command - start the HTTP server.

use std::process::{Command, Stdio};

use anyhow::Result;

use crate::output::{print_info, print_success};

/// Serve command arguments.
#[derive(Debug)]
pub struct ServeArgs {
    pub port: u16,
    pub foreground: bool,
}

/// Run the serve command.
pub async fn run(args: ServeArgs) -> Result<()> {
    let server_binary = find_server_binary()?;

    if args.foreground {
        print_info(&format!("Starting Berry server on port {}...", args.port));

        // Run in foreground - replace current process
        let status = Command::new(&server_binary)
            .args(["--port", &args.port.to_string()])
            .status()?;

        if !status.success() {
            anyhow::bail!("Server exited with status: {}", status);
        }
    } else {
        print_info(&format!(
            "Starting Berry server in background on port {}...",
            args.port
        ));

        // Spawn in background
        let child = Command::new(&server_binary)
            .args(["--port", &args.port.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        print_success(&format!(
            "Server started with PID {}. Use `berry serve --foreground` to run in foreground.",
            child.id()
        ));
    }

    Ok(())
}

/// Find the server binary.
fn find_server_binary() -> Result<String> {
    // Try to find berry-server in PATH or relative to current binary
    let candidates = [
        "berry-server",
        "./berry-server",
        "../server/target/release/berry-server",
        "../server/target/debug/berry-server",
    ];

    for candidate in candidates {
        if which::which(candidate).is_ok() {
            return Ok(candidate.to_string());
        }
    }

    // If not found, assume it's in PATH and let the OS handle it
    Ok("berry-server".to_string())
}
