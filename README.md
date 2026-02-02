> [!WARNING]
> Berry is in a Beta state, it's safe to use but breaking changes are possible.

# Berry

![Member?][logo]

A memory storage system that exists between you and your AI tooling.

This is the Rust implementation of Berry, providing improved performance and native binaries.

## Installation

### Installation Script (Recommended)

Install Berry using the installation script:

```bash
curl -fsSL https://raw.githubusercontent.com/berry-rs/berry/main/scripts/install.sh | bash
```

### From Source

For development or if you prefer building from source:

```bash
# Install Rust if you don't have it
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build berry
git clone https://github.com/berry-rs/berry.git
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

The recommended way to use Berry is to create a new ChromaDB instance in the cloud. To do this visit the
[ChromaDB website](https://trychroma.com) and create a user and a new instance, the free version has a usage limit but
is sufficient to get started. Once you have created an instance, you need to generate an API key, save it as well as the
tenant ID and database name. The server will use the following environment variables to connect to the database:

```
CHROMA_PROVIDER=cloud
CHROMA_API_KEY=<insert_chroma_api_key>
CHROMA_TENANT=<insert_chroma_tenant_id>
CHROMA_DATABASE=<insert_chroma_database_name>
```

If you want to use a local instance of ChromaDB, you can use the following environment variables:

```
CHROMA_PROVIDER=local
CHROMA_URL=http://localhost:8000
```

Depending on how you want to run the server these should be set. If you only want to run this for a single project that
could be managed using any number of task runners, if you want it for many projects and conversational sessions it would
be better to use a launch system.

### Initialize Configuration

The CLI and MCP server both use a common configuration file for settings. The configuration file is located at
`~/.config/berry/config.jsonc` and is created using the command `berry init`.

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
        <key>CHROMA_URL</key>
        <string>http://localhost:8000</string>
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

The CLI uses a configuration file located at `~/.config/berry/config.jsonc`, create it with `berry init` if you haven't
already.

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

The MCP server uses a configuration file located at `~/.config/berry/config.jsonc`, create it with `berry init` if you
haven't already.

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
