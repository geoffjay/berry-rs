# Developing Berry

This guide covers setting up a local development environment for the Berry project.

## Prerequisites

- [Rust](https://www.rust-lang.org/) 1.75+ (install via rustup)
- [Docker](https://www.docker.com/) for running ChromaDB

## Setup

```bash
# Clone the repository
git clone https://github.com/geoffjay/berry-rs.git
cd berry

# Build all packages
cargo build

# Run tests
cargo test --all
```

## Project Structure

```
berry-rs/
├── Cargo.toml                    # Workspace manifest
├── crates/
│   ├── berry/                    # Shared library crate
│   │   └── src/
│   │       ├── lib.rs            # Library entry point
│   │       ├── types/            # Type definitions
│   │       │   ├── mod.rs
│   │       │   ├── memory.rs     # Memory struct
│   │       │   ├── memory_type.rs
│   │       │   ├── visibility.rs
│   │       │   └── requests.rs   # Request/response types
│   │       ├── store/            # Storage abstraction
│   │       │   ├── mod.rs
│   │       │   ├── traits.rs     # VectorStore trait
│   │       │   └── chroma.rs     # ChromaDB implementation
│   │       ├── config/           # Configuration
│   │       │   ├── mod.rs
│   │       │   ├── types.rs
│   │       │   └── loader.rs
│   │       ├── error.rs          # Error types
│   │       └── logging.rs        # Logging setup
│   ├── cli/                      # CLI binary (berry)
│   │   └── src/
│   │       ├── main.rs
│   │       ├── client.rs         # HTTP client
│   │       ├── output.rs         # Display formatting
│   │       └── commands/         # CLI commands
│   ├── server/                   # HTTP server (berry-server)
│   │   └── src/
│   │       ├── main.rs
│   │       ├── state.rs          # Application state
│   │       └── routes/           # HTTP handlers
│   └── mcp/                      # MCP server (berry-mcp)
│       └── src/
│           ├── main.rs
│           ├── handler.rs        # MCP client
│           └── tools/            # MCP tool definitions
├── docs/                         # Documentation
├── scripts/
│   └── install.sh                # Installation script
└── .github/workflows/
    ├── ci.yml                    # CI workflow
    └── release.yml               # Release workflow
```

## Running the Services

Berry requires two services to be running:

1. **ChromaDB** - Vector database for storing memories
2. **Berry Server** - HTTP API server

### Start ChromaDB

```bash
docker run -d -p 8000:8000 chromadb/chroma
```

### Start Berry Server

In development mode (with debug logging):

```bash
BERRY_LOG=debug cargo run -p berry-server -- --port 4114
```

Or build and run the release binary:

```bash
cargo build --release
./target/release/berry-server --port 4114
```

## CLI Development

### Configuration

Create `~/.config/berry/config.jsonc`:

```jsonc
{
  "server": {
    "url": "http://localhost:4114",
    "timeout": 5000
  },
  "defaults": {
    "type": "information",
    "createdBy": "user"
  },
  "chroma": {
    "url": "http://localhost:8000",
    "collection": "berry_memories"
  }
}
```

### Running the CLI

During development:

```bash
cargo run -p berry-cli -- --help
cargo run -p berry-cli -- remember "test memory"
cargo run -p berry-cli -- search "test"
```

Or build and use directly:

```bash
cargo build
./target/debug/berry --help
```

### Testing Commands

```bash
# Store a memory
cargo run -p berry-cli -- remember "The API uses JWT tokens"

# Search memories
cargo run -p berry-cli -- search "authentication"

# Recall by ID
cargo run -p berry-cli -- recall mem_1234567890_abcdef

# Delete a memory
cargo run -p berry-cli -- forget mem_1234567890_abcdef
```

## Building

### Debug Build

```bash
cargo build
```

Binaries are placed in `target/debug/`:
- `target/debug/berry`
- `target/debug/berry-server`
- `target/debug/berry-mcp`

### Release Build

```bash
cargo build --release
```

Binaries are placed in `target/release/`.

### Running Tests

```bash
# Run all tests
cargo test --all

# Run tests for a specific crate
cargo test -p berry

# Run tests with output
cargo test --all -- --nocapture
```

### Code Formatting

```bash
cargo fmt --all
```

### Linting

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

## Adding New Features

### Adding a New Memory Field

1. Update the `Memory` struct in `crates/berry/src/types/memory.rs`
2. Update `CreateMemoryRequest` in `crates/berry/src/types/requests.rs`
3. Update the ChromaDB metadata conversion in `crates/berry/src/store/chroma.rs`
4. Add CLI flags in `crates/cli/src/main.rs`
5. Update server routes as needed

### Adding a New CLI Command

1. Create a new module in `crates/cli/src/commands/`
2. Export the module in `crates/cli/src/commands/mod.rs`
3. Add the subcommand to the `Commands` enum in `crates/cli/src/main.rs`
4. Handle the command in the main match statement

### Adding a New API Endpoint

1. Create a handler in `crates/server/src/routes/`
2. Export the handler in `crates/server/src/routes/mod.rs`
3. Add the route in `crates/server/src/main.rs`

## Debugging

### Enable Debug Logging

```bash
BERRY_LOG=debug cargo run -p berry-server -- --port 4114
```

Available log levels: `error`, `warn`, `info`, `debug`, `trace`

### JSON Log Format

```bash
BERRY_LOG_FORMAT=json cargo run -p berry-server -- --port 4114
```

### Testing ChromaDB Connection

```bash
curl http://localhost:8000/api/v1/heartbeat
```

### Testing Server Health

```bash
curl http://localhost:4114/health
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test --all`
5. Run clippy: `cargo clippy --all-targets`
6. Format code: `cargo fmt --all`
7. Submit a pull request

## Troubleshooting

### Cargo build fails with "edition 2024"

Ensure you have Rust 1.75+ installed:

```bash
rustup update stable
```

### ChromaDB connection refused

Ensure ChromaDB is running:

```bash
docker ps | grep chroma
```

Start it if needed:

```bash
docker run -d -p 8000:8000 chromadb/chroma
```

### Tests fail with network errors

Some tests may require ChromaDB to be running. Integration tests that require external services should be marked appropriately.
