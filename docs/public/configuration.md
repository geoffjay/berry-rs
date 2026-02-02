# Configuration

Berry can be configured through a configuration file and environment variables.

## Configuration File

The configuration file is located at `~/.config/berry/config.jsonc`. This file uses JSONC format, which allows comments
and trailing commas.

### Configuration File Location

```
~/.config/berry/config.jsonc
```

If the file doesn't exist, Berry uses default values. Create it with `berry init`.

### Full Configuration Example

```jsonc
{
  // Berry Configuration
  // See https://github.com/berry-rs/berry for documentation

  // Server connection settings
  "server": {
    // URL of the Berry server
    "url": "http://localhost:4114",

    // Request timeout in milliseconds
    "timeout": 5000
  },

  // Default values for new memories
  "defaults": {
    // Default memory type: "question", "request", or "information"
    "type": "information",

    // Default creator identifier
    "createdBy": "user",

    // Default visibility: "private", "shared", or "public"
    "visibility": "public"
  },

  // ChromaDB configuration
  "chroma": {
    // ChromaDB server URL
    "url": "http://localhost:8000",

    // Collection name for storing memories
    "collection": "berry_memories"

    // Optional: authentication provider
    // "provider": "token",

    // Optional: API key for authentication
    // "apiKey": "your-api-key"
  }
}
```

### Configuration Options

#### `server.url`

The URL where the Berry server is running.

- **Type:** `string`
- **Default:** `"http://localhost:4114"`
- **Example:** `"http://192.168.1.100:4114"`

#### `server.timeout`

Request timeout in milliseconds. If the server doesn't respond within this time, the request fails.

- **Type:** `number`
- **Default:** `5000`
- **Example:** `10000` (10 seconds)

#### `defaults.type`

Default memory type when creating new memories without specifying a type.

- **Type:** `string`
- **Default:** `"information"`
- **Options:**
  - `"question"` - Questions or queries
  - `"request"` - Requests or tasks
  - `"information"` - General information

#### `defaults.createdBy`

Default creator identifier for new memories.

- **Type:** `string`
- **Default:** `"user"`
- **Example:** `"claude"`, `"system"`, or a username

#### `defaults.visibility`

Default visibility level for new memories.

- **Type:** `string`
- **Default:** `"public"`
- **Options:**
  - `"private"` - Only visible to the creator
  - `"shared"` - Visible to specific actors
  - `"public"` - Visible to everyone

#### `chroma.url`

The URL of the ChromaDB instance for vector storage.

- **Type:** `string`
- **Default:** `"http://localhost:8000"`
- **Example:** `"https://your-instance.trychroma.com"`

#### `chroma.collection`

The name of the ChromaDB collection to use for storing memories.

- **Type:** `string`
- **Default:** `"berry_memories"`

#### `chroma.provider`

Authentication provider for ChromaDB (optional).

- **Type:** `string` (optional)
- **Example:** `"token"`

#### `chroma.apiKey`

API key for ChromaDB authentication (optional).

- **Type:** `string` (optional)

## Environment Variables

Environment variables override configuration file values.

### CLI/Server Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `BERRY_SERVER_URL` | Berry server URL | `http://localhost:4114` |
| `BERRY_TIMEOUT` | Request timeout (ms) | `5000` |
| `BERRY_CREATED_BY` | Default creator | `user` |
| `BERRY_DEFAULT_TYPE` | Default memory type | `information` |
| `BERRY_LOG` | Log level | `info` |
| `BERRY_LOG_FORMAT` | Log format (`text` or `json`) | `text` |

### ChromaDB Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `CHROMA_URL` | ChromaDB server URL | `http://localhost:8000` |
| `CHROMA_COLLECTION` | Collection name | `berry_memories` |
| `CHROMA_PROVIDER` | Auth provider | (none) |
| `CHROMA_API_KEY` | API key | (none) |

### Setting Environment Variables

#### Using shell export

```bash
export BERRY_SERVER_URL=http://localhost:4114
export CHROMA_URL=http://localhost:8000
berry serve
```

#### Inline with command

```bash
BERRY_LOG=debug berry serve --foreground
```

## Configuration Precedence

1. Command-line flags (highest priority)
2. Environment variables
3. Configuration file (`~/.config/berry/config.jsonc`)
4. Default values (lowest priority)

## Validating Configuration

If the configuration file has syntax errors, Berry will display a warning and fall back to default values:

```
Warning: Config file has parse errors: missing field `url`
Using default configuration.
```

To validate your configuration file, run any Berry command and check for warnings, or use a JSON/JSONC linter.

## Server-Specific Configuration

The Berry server (`berry-server`) accepts command-line arguments:

```bash
berry-server --help
```

```
Berry memory system HTTP server

Usage: berry-server [OPTIONS]

Options:
  -p, --port <PORT>  Port to listen on [default: 4114]
      --host <HOST>  Host to bind to [default: 127.0.0.1]
  -h, --help         Print help
  -V, --version      Print version
```

Example:

```bash
berry-server --port 8080 --host 0.0.0.0
```
