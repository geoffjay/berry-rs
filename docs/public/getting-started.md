# Getting Started

This guide walks you through installing and using Berry for the first time.

## Prerequisites

Before installing Berry, ensure you have the following:

- [Ollama](https://ollama.com) - For running local embeddings (or an OpenAI API key)

For building from source:

- [Rust](https://www.rust-lang.org/) 1.75+ (with cargo)

## Installation

### Installation Script (Recommended)

The easiest way to install Berry is via the install script:

```bash
curl -fsSL https://raw.githubusercontent.com/geoffjay/berry-rs/main/scripts/install.sh | bash
```

This downloads pre-built binaries for your platform to `~/.local/bin`.

### From Source

For development or contributing, install from source:

```bash
# Install Rust if you don't have it
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/geoffjay/berry-rs.git
cd berry
cargo build --release

# The binaries are now in target/release/
# - target/release/berry
# - target/release/berry-server
# - target/release/berry-mcp

# Optionally install to ~/.cargo/bin
cargo install --path crates/cli
cargo install --path crates/server
cargo install --path crates/mcp
```

Ensure `~/.cargo/bin` or `~/.local/bin` is in your PATH:

```bash
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"
```

Add this line to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.) to make it permanent.

### Initialize Configuration

Berry uses a common configuration file for settings. The configuration file location is platform-specific:

| Platform | Configuration Path |
|----------|-------------------|
| Linux    | `~/.config/berry/config.jsonc` |
| macOS    | `~/Library/Application Support/berry/config.jsonc` |
| Windows  | `%APPDATA%\berry\config.jsonc` |

Create the configuration file:

```bash
berry init
```

This creates the file in the platform dependent location with default settings. You can also create it manually:

```bash
mkdir -p ~/.config/berry
cat > ~/.config/berry/config.jsonc << 'EOF'
{
  "store": "lance",
  "server": {
    "url": "http://localhost:4114",
    "timeout": 5000
  },
  "defaults": {
    "type": "information",
    "createdBy": "user",
    "visibility": "public"
  },
  "lance": {
    "table": "berry_memories"
  }
}
EOF
```

## Starting the Services

Berry uses LanceDB by default, which is an embedded database — no additional services are needed. If you use Ollama for embeddings, ensure it is running:

- [Ollama](./services/ollama.md)

If you opt to use ChromaDB instead, see [ChromaDB setup](./services/chromadb.md).

Start the Berry server:

```bash
berry serve
```

You should see:

```
Starting Berry server in background on port 4114...
Server started with PID 12345. Use `berry serve --foreground` to run in foreground.
```

### Running in Foreground

To run the server in the foreground (useful for debugging):

```bash
berry serve --foreground
```

Or run the server binary directly:

```bash
berry-server --port 4114
```

## Basic Usage

### Store a Memory

```bash
berry remember "The API endpoint for users is /api/v1/users"
```

With type and tags:

```bash
berry remember "How do I reset my password?" --type question --tags "auth,faq"
```

### Search Memories

```bash
berry search "API endpoint"
```

### Recall a Specific Memory

If you know the memory ID:

```bash
berry recall mem_1234567890_abcdef
```

### Remove a Memory

```bash
berry forget mem_1234567890_abcdef
```

## Verify Installation

Check that everything is working:

```bash
# Check version
berry --version

# Check server health
curl http://localhost:4114/health
```

Expected health response:

```json
{"status":"healthy","version":"0.1.0","database":"connected"}
```

## Next Steps

- [CLI Reference](./cli.md) - Complete command reference
- [Configuration](./configuration.md) - Customize Berry settings
- [Development](./develop.md) - Set up a development environment

## Troubleshooting

### Command not found: berry

Ensure your installation directory is in your PATH:

```bash
# For cargo install
export PATH="$HOME/.cargo/bin:$PATH"

# For install script
export PATH="$HOME/.local/bin:$PATH"
```

Add this line to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.) to make it permanent.

### Server failed to start

If using the default LanceDB store, ensure the data directory is writable. Check the server logs for details.

If using ChromaDB, ensure it is running:

```bash
docker ps | grep chroma
```

### Connection refused

Check that the server URL in your configuration matches where the server is running:

```bash
cat ~/.config/berry/config.jsonc
```

The default configuration expects the server at `http://localhost:4114`.

### ChromaDB connection issues (if using ChromaDB)

Verify ChromaDB is accessible:

```bash
curl http://localhost:8000/api/v1/heartbeat
```

If using a cloud ChromaDB instance, ensure you have set the environment variables:

```bash
export CHROMA_URL=https://your-instance.trychroma.com
export CHROMA_API_KEY=your-api-key
```
