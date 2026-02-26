[![CI][ci-badge]][ci-url]
[![Release][release-badge]][release-url]
[![codecov][codecov-badge]][codecov-url]
[![MIT licensed][mit-badge]][mit-url]
[![Apache licensed][apache-badge]][apache-url]

[ci-badge]: https://github.com/geoffjay/berry-rs/actions/workflows/ci.yml/badge.svg
[ci-url]: https://github.com/geoffjay/berry-rs/actions/workflows/ci.yml
[release-badge]: https://github.com/geoffjay/berry-rs/actions/workflows/release.yml/badge.svg
[release-url]: https://github.com/geoffjay/berry-rs/actions/workflows/release.yml
[codecov-badge]: https://codecov.io/gh/geoffjay/berry-rs/graph/badge.svg?token=knPW8TUmoJ
[codecov-url]: https://codecov.io/gh/geoffjay/berry-rs
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/geoffjay/berry-rs/blob/main/LICENSE-MIT
[apache-badge]: https://img.shields.io/badge/License-Apache_2.0-yellowgreen.svg
[apache-url]: https://github.com/geoffjay/berry-rs/blob/main/LICENSE-APACHE

# Berry

> [!WARNING]
> Berry is in a Beta state, it's safe to use but breaking changes are possible.

![Member?][logo]

A memory storage system that exists between you and your AI tooling.

This is the Rust implementation of Berry, providing improved performance and native binaries.

## Installation

### Installation Script (Recommended)

Install Berry using the installation script:

```bash
curl -fsSL https://raw.githubusercontent.com/geoffjay/berry-rs/main/scripts/install.sh | bash
```

### From Source

For development or if you prefer building from source:

```bash
# Install Rust if you don't have it
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build berry
git clone https://github.com/geoffjay/berry-rs.git
cd berry
cargo build --release

# Install binaries (optional)
cargo install --path crates/cli
cargo install --path crates/server
cargo install --path crates/mcp
```

### Verify Installation

Once installed, verify the installation:

```bash
berry --version
```

## Setup

Berry uses **LanceDB** by default — an embedded vector database that requires no setup. Data is stored locally on disk, so you can start using Berry immediately after installation.

If you prefer to use ChromaDB instead, set `BERRY_STORE=chroma` and configure the ChromaDB connection:

```bash
# Local ChromaDB
export BERRY_STORE=chroma
export CHROMA_URL=http://localhost:8000

# Or ChromaDB Cloud
export BERRY_STORE=chroma
export CHROMA_PROVIDER=cloud
export CHROMA_API_KEY=<insert_chroma_api_key>
export CHROMA_TENANT=<insert_chroma_tenant_id>
export CHROMA_DATABASE=<insert_chroma_database_name>
```

### Initialize Configuration

The CLI and MCP server both use a common configuration file for settings. The configuration file location is
platform-specific:

| Platform | Configuration Path |
|----------|-------------------|
| Linux    | `~/.config/berry/config.jsonc` |
| macOS    | `~/Library/Application Support/berry/config.jsonc` |
| Windows  | `%APPDATA%\berry\config.jsonc` |

Create the configuration file using `berry init`.

### Launchd (macOS)

#### Configuration

Create a file called `com.berry.server.plist` in `~/Library/LaunchAgents` with the following contents:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.berry.server</string>

    <key>ProgramArguments</key>
    <array>
        <string>/Users/username/.local/bin/berry-server</string>
        <string>--port</string>
        <string>4114</string>
    </array>

    <key>EnvironmentVariables</key>
    <dict>
        <!-- LanceDB is the default store; no additional env vars needed -->
        <!-- Uncomment below to use ChromaDB instead: -->
        <!-- <key>BERRY_STORE</key> -->
        <!-- <string>chroma</string> -->
        <!-- <key>CHROMA_URL</key> -->
        <!-- <string>http://localhost:8000</string> -->
    </dict>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <true/>

    <key>StandardOutPath</key>
    <string>/Users/username/.local/state/berry/server.log</string>

    <key>StandardErrorPath</key>
    <string>/Users/username/.local/state/berry/server.error.log</string>

    <key>WorkingDirectory</key>
    <string>/Users/username</string>
</dict>
</plist>
```

> [!IMPORTANT]
> Replace `ProgramArguments` with the actual path to your `berry-server` binary

> [!IMPORTANT]
> Replace "username" appropriately

#### Installation

Perform the following steps to install the `launchd` service:

```bash
# Create log directory
mkdir -p ~/.local/state/berry

# Copy to LaunchAgents
cp com.berry.server.plist ~/Library/LaunchAgents/

# Load the service
launchctl load ~/Library/LaunchAgents/com.berry.server.plist
```

Management commands:

```bash
# Check status
launchctl list | grep berry

# Test health
curl http://localhost:4114/health

# Stop
launchctl stop com.berry.server

# Start
launchctl start com.berry.server

# Unload (disable)
launchctl unload ~/Library/LaunchAgents/com.berry.server.plist

# View logs
tail -f ~/.local/state/berry/server.log
```

## CLI

### Configuration

The CLI uses a platform-specific configuration file (see [Initialize Configuration](#initialize-configuration) above).
Create it with `berry init` if you haven't already.

### Sample Commands

```bash
# Store some memories
berry remember "The API uses JWT tokens for authentication"
berry remember "Database backups run at 3am daily" --type information --tags "ops,database"
berry remember "How do I reset a user's password?" --type question --tags "auth,faq"

# Search memories
berry search "authentication"
berry search "database" --limit 5
berry search "password" --type question

# Recall a specific memory by ID
berry recall mem_abc123

# Remove a memory
berry forget mem_abc123

# Interactive mode (guided prompts)
berry

# Remember with all options
berry remember "Deploy process requires approval" \
  --type request \
  --tags "deploy,process" \
  --by "engineering"

# Search with filters
berry search "meeting" \
  --type information \
  --tags "notes" \
  --limit 20 \
  --from "2024-01-01T00:00:00Z" \
  --to "2024-12-31T23:59:59Z"
```

## MCP Server

### Configuration

The MCP server uses a platform-specific configuration file (see [Initialize Configuration](#initialize-configuration)
above). Create it with `berry init` if you haven't already.

### Claude Code

```json
{
  "mcpServers": {
    "berry": {
      "type": "stdio",
      "command": "berry",
      "args": ["mcp"]
    }
  }
}
```

### OpenCode

```json
{
  "mcp": {
    "berry": {
      "type": "local",
      "command": ["berry", "mcp"],
      "enabled": true
    }
  }
}
```

## Documentation

- [Getting Started](docs/public/getting-started.md)
- [Configuration](docs/public/configuration.md)
- [Development](docs/public/develop.md)

<!-- links -->

[logo]: docs/assets/member.png
