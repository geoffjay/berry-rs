# Document Store

Berry includes a document store for managing markdown documents alongside memories. While memories are short-form vector-indexed snippets, documents are longer-form markdown files stored on the filesystem with sidecar metadata. This gives AI agents a consistent, configure-once interface for document CRUD without needing to know filesystem paths.

## How It Works

Documents are stored as plain markdown files at a configured directory path. Metadata (title, tags, creator, timestamps) lives in sidecar JSON files within a hidden `.berry/` subdirectory.

```
<documents_path>/
  architecture-decisions.md        # markdown content
  onboarding-guide.md
  .berry/
    architecture-decisions.json    # metadata (title, tags, timestamps)
    onboarding-guide.json
```

Each document gets a slug ID generated from its title (e.g. "Architecture Decisions" becomes `architecture-decisions`). If a slug already exists, Berry appends `-2`, `-3`, etc.

## Configuration

Document management is enabled by default. Configure it in your config file or with environment variables.

### Configuration File

```jsonc
{
  "documents": {
    // Path to the documents directory
    "path": "/path/to/documents",

    // Set to false to disable document management
    "enabled": true
  }
}
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `BERRY_DOCUMENTS_PATH` | Path to the documents directory | Platform-specific (see below) |
| `BERRY_DOCUMENTS_ENABLED` | Enable/disable documents (`true`/`false`) | `true` |

### Default Path

The default documents directory is platform-specific:

| Platform | Default Path |
|----------|-------------|
| Linux    | `~/.local/share/berry/documents` |
| macOS    | `~/Library/Application Support/berry/documents` |
| Windows  | `%APPDATA%\berry\data\documents` |

## CLI Usage

All document operations are under the `berry doc` subcommand.

### Create a Document

```bash
berry doc create "Architecture Decisions" --content "# Architecture Decisions\n\nThis document tracks key decisions." --by architect
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `TITLE` | Document title (used to generate the slug ID) |

**Options:**

| Option | Description |
|--------|-------------|
| `-c, --content <CONTENT>` | Markdown body (prompts interactively if omitted) |
| `--tags <TAGS>` | Comma-separated tags |
| `--by <CREATOR>` | Creator identifier (default: `user`) |

**Examples:**

```bash
# With tags
berry doc create "API Reference" --content "# API Reference" --tags "api,reference"

# Interactive mode (prompts for content)
berry doc create "Meeting Notes"
```

### Read a Document

```bash
berry doc read <ID>
```

**Examples:**

```bash
berry doc read architecture-decisions

# JSON output
berry --format json doc read architecture-decisions
```

### Update a Document

```bash
berry doc update <ID> [OPTIONS]
```

All fields are optional — only provided fields are updated.

**Options:**

| Option | Description |
|--------|-------------|
| `-t, --title <TITLE>` | New title |
| `-c, --content <CONTENT>` | New markdown content |
| `--tags <TAGS>` | New tags (comma-separated, replaces existing) |

**Examples:**

```bash
# Update content only
berry doc update architecture-decisions --content "# Updated content"

# Update tags
berry doc update architecture-decisions --tags "architecture,decisions,adr"

# Update title (ID remains the same)
berry doc update architecture-decisions --title "Architecture Decision Records"
```

### Delete a Document

```bash
berry doc delete <ID> [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `-f, --force` | Skip confirmation prompt |

**Examples:**

```bash
# With confirmation
berry doc delete architecture-decisions

# Skip confirmation
berry doc delete -f architecture-decisions
```

### List Documents

```bash
berry doc list [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--tags <TAGS>` | Filter by tags (comma-separated, any match) |
| `--by <CREATOR>` | Filter by creator |

**Examples:**

```bash
# List all documents
berry doc list

# Filter by tag
berry doc list --tags architecture

# Filter by creator
berry doc list --by architect

# JSON output
berry --format json doc list
```

## HTTP API

The Berry server exposes document endpoints when the document store is enabled. All endpoints return JSON.

### Create Document

```
POST /v1/documents
Content-Type: application/json

{
  "title": "Architecture Decisions",
  "content": "# Architecture Decisions\n\nKey decisions...",
  "tags": ["architecture"],
  "created_by": "user"
}
```

**Response:**

```json
{
  "success": true,
  "document": {
    "id": "architecture-decisions",
    "title": "Architecture Decisions",
    "content": "# Architecture Decisions\n\nKey decisions...",
    "tags": ["architecture"],
    "created_by": "user",
    "created_at": "2025-01-15T10:30:00Z",
    "updated_at": "2025-01-15T10:30:00Z"
  }
}
```

### Get Document

```
GET /v1/documents/{id}
```

Returns `404` if the document does not exist.

### Update Document

```
PATCH /v1/documents/{id}
Content-Type: application/json

{
  "content": "# Updated content"
}
```

All fields are optional. Only provided fields are updated.

### Delete Document

```
DELETE /v1/documents/{id}
```

**Response:**

```json
{
  "success": true,
  "deleted": true
}
```

### List Documents

```
GET /v1/documents
GET /v1/documents?tags=architecture&created_by=user
```

**Response:**

```json
{
  "success": true,
  "documents": [...],
  "total": 2
}
```

## MCP Tools

When using Berry as an MCP server (`berry mcp`), the following document tools are available:

| Tool | Description |
|------|-------------|
| `doc_create` | Create a new document |
| `doc_read` | Retrieve a document by ID |
| `doc_update` | Update an existing document |
| `doc_delete` | Delete a document |
| `doc_list` | List documents with optional filters |

### doc_create

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `title` | string | Yes | Document title |
| `content` | string | Yes | Markdown body content |
| `tags` | string[] | No | Tags for the document |
| `created_by` | string | Yes | Creator identifier |

### doc_read

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Document ID (slug) |

### doc_update

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Document ID (slug) |
| `title` | string | No | New title |
| `content` | string | No | New markdown content |
| `tags` | string[] | No | New tags (replaces existing) |

### doc_delete

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Document ID (slug) |

### doc_list

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `tags` | string[] | No | Filter by tags |
| `created_by` | string | No | Filter by creator |

## Document vs Memory

| | Memories | Documents |
|--|---------|-----------|
| **Content** | Short text snippets | Full markdown files |
| **Storage** | Vector database (LanceDB/ChromaDB) | Filesystem |
| **Search** | Semantic similarity search | Tag and creator filters |
| **IDs** | Auto-generated (`mem_<timestamp>_<random>`) | Slug from title (`architecture-decisions`) |
| **Use case** | Quick facts, Q&A, context | Longer-form reference material |

## Disabling Documents

If you don't need document management, disable it to skip directory creation and route registration:

```bash
export BERRY_DOCUMENTS_ENABLED=false
```

Or in the config file:

```jsonc
{
  "documents": {
    "enabled": false
  }
}
```

When disabled, the document API endpoints return `501 Not Implemented`.

## See Also

- [CLI Reference](./cli.md) - Full CLI documentation
- [Configuration](./configuration.md) - All configuration options
- [Getting Started](./getting-started.md) - Installation and setup
