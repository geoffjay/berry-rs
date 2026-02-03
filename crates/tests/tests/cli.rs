//! CLI integration tests.
//!
//! Tests the Berry CLI binary for correct help output and argument parsing.
//!
//! **Note**: These tests require the `berry` binary to be built first.
//! Run `cargo build` or `cargo build -p berry-cli` before running these tests.

use std::process::Command;

#[allow(deprecated)]
use assert_cmd::cargo::cargo_bin;
use assert_cmd::prelude::*;
use predicates::prelude::*;

/// Helper to get the CLI binary.
fn berry_cmd() -> Command {
    #[allow(deprecated)]
    Command::new(cargo_bin("berry"))
}

/// Test CLI help output.
#[test]
fn test_cli_help() {
    berry_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Berry"))
        .stdout(predicate::str::contains("semantic memory system"));
}

/// Test CLI version output.
#[test]
fn test_cli_version() {
    berry_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("berry"));
}

/// Test remember subcommand help.
#[test]
fn test_remember_help() {
    berry_cmd()
        .args(["remember", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Store a new memory"))
        .stdout(predicate::str::contains("--tags"));
}

/// Test recall subcommand help.
#[test]
fn test_recall_help() {
    berry_cmd()
        .args(["recall", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Retrieve a memory by ID"));
}

/// Test forget subcommand help.
#[test]
fn test_forget_help() {
    berry_cmd()
        .args(["forget", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Delete a memory"));
}

/// Test search subcommand help.
#[test]
fn test_search_help() {
    berry_cmd()
        .args(["search", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Search for memories"))
        .stdout(predicate::str::contains("--limit"));
}

/// Test init subcommand help.
#[test]
fn test_init_help() {
    berry_cmd()
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialize configuration"));
}

/// Test serve subcommand help.
#[test]
fn test_serve_help() {
    berry_cmd()
        .args(["serve", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Start the HTTP server"))
        .stdout(predicate::str::contains("--port"));
}

/// Test mcp subcommand help.
#[test]
fn test_mcp_help() {
    berry_cmd()
        .args(["mcp", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Start the MCP server"));
}

/// Test migrate subcommand help.
#[test]
fn test_migrate_help() {
    berry_cmd()
        .args(["migrate", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Migrate memories"));
}

/// Test invalid subcommand.
#[test]
fn test_invalid_subcommand() {
    berry_cmd().arg("invalid_command").assert().failure();
}

/// Test JSON output format flag.
#[test]
fn test_format_flag() {
    berry_cmd()
        .args(["--format", "json", "--help"])
        .assert()
        .success();
}

// NOTE: The following tests require a running server and are marked as ignored
// by default. They demonstrate the pattern for full integration tests.
//
// To run these tests:
// 1. Start ChromaDB: docker run -p 8000:8000 chromadb/chroma:0.5.23
// 2. Start berry server: cargo run -p server
// 3. Run tests: cargo test -p berry-integration-tests --test cli -- --ignored

/// Test creating a memory via CLI (requires running server).
#[test]
#[ignore = "Requires running server"]
fn test_cli_remember() {
    berry_cmd()
        .args([
            "remember",
            "Test memory from CLI integration test",
            "--tags",
            "cli,test,integration",
            "--by",
            "cli_test",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("mem_"));
}

/// Test recalling a memory via CLI (requires running server).
#[test]
#[ignore = "Requires running server"]
fn test_cli_recall() {
    // First create a memory
    let output = berry_cmd()
        .args(["--format", "json", "remember", "Memory to recall"])
        .output()
        .expect("Failed to create memory");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let response: serde_json::Value =
        serde_json::from_str(&stdout).expect("Failed to parse JSON response");

    let memory_id = response["memory"]["id"]
        .as_str()
        .expect("No memory ID in response");

    // Now recall it
    berry_cmd()
        .args(["recall", memory_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Memory to recall"));
}

/// Test searching via CLI (requires running server).
#[test]
#[ignore = "Requires running server"]
fn test_cli_search() {
    berry_cmd()
        .args(["search", "test query", "--limit", "5"])
        .assert()
        .success();
}

/// Test deleting a memory via CLI (requires running server).
#[test]
#[ignore = "Requires running server"]
fn test_cli_forget() {
    // First create a memory
    let output = berry_cmd()
        .args(["--format", "json", "remember", "Memory to delete"])
        .output()
        .expect("Failed to create memory");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let response: serde_json::Value =
        serde_json::from_str(&stdout).expect("Failed to parse JSON response");

    let memory_id = response["memory"]["id"]
        .as_str()
        .expect("No memory ID in response");

    // Delete it with force flag
    berry_cmd()
        .args(["forget", memory_id, "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted"));
}
