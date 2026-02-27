# CLI Reference

Complete reference for the Berry command-line interface.

## Synopsis

```
berry [OPTIONS] <COMMAND>
```

## Global Options

| Option | Description |
|--------|-------------|
| `--format <FORMAT>` | Output format: `text` (default) or `json` |
| `--help` | Print help information |
| `--version` | Print version information |

## Commands

### remember

Store a new memory.

```bash
berry remember [OPTIONS] [CONTENT]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `CONTENT` | Content of the memory (optional, prompts if not provided) |

**Options:**

| Option | Description |
|--------|-------------|
| `-t, --type <TYPE>` | Memory type: `question`, `request`, or `information` (default) |
| `--tags <TAGS>` | Comma-separated tags |
| `--by <CREATOR>` | Creator identifier (default: `user`) |
| `-r, --references <IDS>` | Comma-separated memory IDs to reference |
| `-v, --visibility <LEVEL>` | Visibility: `private`, `shared`, or `public` (default) |
| `--shared-with <ACTORS>` | Comma-separated actor IDs to share with |

**Examples:**

```bash
# Simple memory
berry remember "The API endpoint for users is /api/v1/users"

# With type and tags
berry remember "How do I reset my password?" --type question --tags "auth,faq"

# With visibility
berry remember "Personal note" --visibility private --by "alice"

# Interactive mode (prompts for content)
berry remember
```

### recall

Retrieve a memory by ID.

```bash
berry recall <ID>
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `ID` | Memory ID to retrieve |

**Examples:**

```bash
berry recall mem_1234567890_abcdef

# JSON output
berry --format json recall mem_1234567890_abcdef
```

### forget

Delete a memory.

```bash
berry forget [OPTIONS] <ID>
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `ID` | Memory ID to delete |

**Options:**

| Option | Description |
|--------|-------------|
| `-f, --force` | Skip confirmation prompt |

**Examples:**

```bash
# With confirmation
berry forget mem_1234567890_abcdef

# Skip confirmation
berry forget -f mem_1234567890_abcdef
```

### search

Search for memories using semantic similarity.

```bash
berry search [OPTIONS] <QUERY>
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `QUERY` | Search query |

**Options:**

| Option | Description |
|--------|-------------|
| `-a, --as-actor <ACTOR>` | Actor for visibility filtering |
| `-t, --type <TYPE>` | Filter by memory type |
| `--tags <TAGS>` | Comma-separated tags to filter by |
| `-l, --limit <N>` | Maximum results (default: 10) |
| `--from <DATE>` | Start date filter (ISO 8601) |
| `--to <DATE>` | End date filter (ISO 8601) |

**Examples:**

```bash
# Basic search
berry search "API endpoint"

# With actor for visibility filtering
berry search "project notes" --as-actor alice

# Filter by type and limit
berry search "questions" --type question --limit 5

# Date range
berry search "meeting notes" --from 2024-01-01T00:00:00Z --to 2024-12-31T23:59:59Z
```

### serve

Start the HTTP server.

```bash
berry serve [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `-p, --port <PORT>` | Port to listen on (default: 4114) |
| `--host <HOST>` | Host to bind to (default: 127.0.0.1) |
| `-f, --foreground` | Run in foreground |

**Examples:**

```bash
# Start server (foreground)
berry serve --foreground

# Custom port
berry serve --port 8080 --foreground

# Bind to all interfaces
berry serve --host 0.0.0.0 --foreground
```

### mcp

Start the MCP (Model Context Protocol) server for AI assistant integration.

```bash
berry mcp [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `-s, --server-url <URL>` | Berry server URL (overrides config) |
| `-v, --verbose` | Enable verbose logging |

**Examples:**

```bash
# Start MCP server
berry mcp

# With custom server URL
berry mcp --server-url http://localhost:8080

# Verbose mode for debugging
berry mcp --verbose
```

### doc

Manage markdown documents. See [Document Store](./documents.md) for full details.

```bash
berry doc <COMMAND>
```

**Subcommands:**

| Command | Description |
|---------|-------------|
| `create <TITLE>` | Create a new document |
| `read <ID>` | Read a document by slug ID |
| `update <ID>` | Update a document |
| `delete <ID>` | Delete a document |
| `list` | List documents |

**Examples:**

```bash
# Create a document
berry doc create "Architecture Decisions" --content "# ADRs" --tags "architecture"

# Read a document
berry doc read architecture-decisions

# Update content
berry doc update architecture-decisions --content "# Updated ADRs"

# List all documents
berry doc list

# Delete with confirmation skip
berry doc delete -f architecture-decisions
```

### init

Initialize configuration file.

```bash
berry init [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--force` | Overwrite existing configuration |

**Examples:**

```bash
# Create default config
berry init

# Overwrite existing config
berry init --force
```

### migrate

Migrate memories to a new collection with the current embedding model. This is useful when changing embedding models or dimensions.

```bash
berry migrate [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--dry-run` | Show what would be migrated without making changes |
| `--collection <NAME>` | Target collection name (default: `<current>_migrated`) |

**Examples:**

```bash
# Preview migration
berry migrate --dry-run

# Migrate to default collection name
berry migrate

# Migrate to specific collection
berry migrate --collection memories_v2
```

**Use Cases:**

1. **Changing embedding models**: When switching from one embedding model to another (e.g., `all-minilm` to `nomic-embed-text`), the embedding dimensions may differ. Migration re-embeds all documents with the new model.

2. **Upgrading embedding quality**: Newer embedding models often provide better semantic search quality. Migration allows upgrading without losing data.

3. **Dimension mismatch errors**: If you see errors like "Collection expecting embedding with dimension of X, got Y", migration resolves this by creating a new collection with the correct dimensions.

**Migration Workflow:**

```bash
# 1. Update embedding configuration in .env
export EMBEDDING_MODEL=nomic-embed-text
export EMBEDDING_BASE_URL=http://localhost:11434/v1

# 2. Preview the migration
berry migrate --dry-run

# 3. Run the migration
berry migrate --collection memories_new

# 4. Update CHROMA_COLLECTION in .env
export CHROMA_COLLECTION=memories_new

# 5. Restart the server
berry serve --foreground

# 6. Verify search works
berry search "test query" --as-actor user

# 7. Delete old collection via ChromaDB dashboard (optional)
```

## Environment Variables

The CLI respects the following environment variables (see [Configuration](./configuration.md) for details):

| Variable | Description |
|----------|-------------|
| `BERRY_SERVER_URL` | Berry server URL |
| `BERRY_TIMEOUT` | Request timeout in milliseconds |
| `BERRY_LOG` | Log level (error, warn, info, debug, trace) |
| `CHROMA_URL` | ChromaDB server URL |
| `CHROMA_COLLECTION` | ChromaDB collection name |
| `EMBEDDING_PROVIDER` | Embedding provider (openai) |
| `EMBEDDING_MODEL` | Embedding model name |
| `EMBEDDING_BASE_URL` | Embedding API base URL |
| `EMBEDDING_API_KEY` | Embedding API key (optional for local services) |
| `BERRY_DOCUMENTS_PATH` | Path to the documents directory |
| `BERRY_DOCUMENTS_ENABLED` | Enable/disable document management |

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |

## See Also

- [Getting Started](./getting-started.md) - Installation and setup
- [Configuration](./configuration.md) - Configuration options
- [Document Store](./documents.md) - Document management guide
