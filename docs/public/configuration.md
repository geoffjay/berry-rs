# Configuration

Berry can be configured through a configuration file and environment variables.

## Configuration File

The configuration file uses JSONC format, which allows comments and trailing commas.

### Configuration File Location

The configuration file location is platform-specific:

| Platform | Configuration Path |
|----------|-------------------|
| Linux    | `~/.config/berry/config.jsonc` |
| macOS    | `~/Library/Application Support/berry/config.jsonc` |
| Windows  | `%APPDATA%\berry\config.jsonc` |

If the file doesn't exist, Berry uses default values. Create it with `berry init`.

> **Note for macOS users**: If you previously used Berry with a config file at `~/.config/berry/config.jsonc`,
> Berry will detect and use that file while displaying a migration warning. Move your config to the new location
> to silence the warning.

### Full Configuration Example

```jsonc
{
  // Berry Configuration
  // See https://github.com/geoffjay/berry-rs for documentation

  // Vector store backend: "lance" (default, embedded) or "chroma" (requires server)
  "store": "lance",

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

  // ChromaDB configuration (used when store is "chroma")
  "chroma": {
    // ChromaDB server URL
    "url": "http://localhost:8000",

    // Collection name for storing memories
    "collection": "berry_memories"

    // Optional: authentication provider ("token", "basic", "x-chroma-token")
    // "provider": "token",

    // Optional: API key for authentication
    // "apiKey": "your-api-key",

    // Optional: Tenant name (for ChromaDB Cloud or multi-tenant setups)
    // "tenant": "your-tenant",

    // Optional: Database name (for ChromaDB Cloud or multi-tenant setups)
    // "database": "your-database"
  },

  // LanceDB configuration (used when store is "lance")
  "lance": {
    // Path to the LanceDB database directory
    // Default is platform-specific (see LanceDB section below)
    "path": "/path/to/lancedb",

    // Table name for storing memories
    "table": "berry_memories"
  }
}
```

### Configuration Options

#### `store`

The vector store backend to use for storing memories.

- **Type:** `string`
- **Default:** `"lance"`
- **Options:**
  - `"lance"` - LanceDB (embedded, no server required) — recommended
  - `"chroma"` - ChromaDB (requires a running ChromaDB server)

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

#### `chroma.tenant`

Tenant name for ChromaDB Cloud or multi-tenant deployments (optional).

- **Type:** `string` (optional)
- **Example:** `"my-tenant-id"`

#### `chroma.database`

Database name for ChromaDB Cloud or multi-tenant deployments (optional).

- **Type:** `string` (optional)
- **Example:** `"my-database"`

#### `lance.path`

The filesystem path to the LanceDB database directory. LanceDB is an embedded database, so no server is required — data is stored directly on disk at this path.

- **Type:** `string`
- **Default:** Platform-specific:
  | Platform | Default Path |
  |----------|-------------|
  | Linux    | `~/.local/share/berry/lancedb` |
  | macOS    | `~/Library/Application Support/berry/lancedb` |
  | Windows  | `%APPDATA%\berry\data\lancedb` |
- **Example:** `"/path/to/my/lancedb"`

#### `lance.table`

The name of the LanceDB table to use for storing memories.

- **Type:** `string`
- **Default:** `"berry_memories"`

## Environment Variables

Environment variables override configuration file values.

### CLI/Server Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `BERRY_STORE` | Vector store backend (`lance`, `lancedb`, `chroma`, `chromadb`) | `lance` |
| `BERRY_SERVER_URL` | Berry server URL | `http://localhost:4114` |
| `BERRY_TIMEOUT` | Request timeout (ms) | `5000` |
| `BERRY_CREATED_BY` | Default creator | `user` |
| `BERRY_DEFAULT_TYPE` | Default memory type | `information` |
| `BERRY_LOG` | Log level | `info` |
| `BERRY_LOG_FORMAT` | Log format (`text` or `json`) | `text` |

### LanceDB Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `LANCE_DB_PATH` | Path to LanceDB database directory | Platform-specific (see above) |
| `LANCE_TABLE` | Table name for memories | `berry_memories` |

### ChromaDB Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `CHROMA_URL` | ChromaDB server URL | `http://localhost:8000` |
| `CHROMA_COLLECTION` | Collection name | `berry_memories` |
| `CHROMA_PROVIDER` | Auth provider (`token`, `basic`, `x-chroma-token`) | (none) |
| `CHROMA_API_KEY` | API key or token | (none) |
| `CHROMA_TENANT` | Tenant name (for ChromaDB Cloud) | (none) |
| `CHROMA_DATABASE` | Database name (for ChromaDB Cloud) | (none) |

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
3. Configuration file (platform-specific path)
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

## LanceDB Configuration

LanceDB is an embedded vector database — no server is required. Data is stored directly on the local filesystem, making it ideal for single-node deployments, development, and environments where running a separate database server is undesirable. LanceDB is the default store backend.

### Using Environment Variables

```bash
export LANCE_DB_PATH=/path/to/lancedb
export LANCE_TABLE=berry_memories

berry serve
```

### Using Configuration File

```jsonc
{
  "store": "lance",
  "lance": {
    "path": "/path/to/lancedb",
    "table": "berry_memories"
  }
}
```

## ChromaDB Cloud Configuration

To connect to ChromaDB Cloud, you need to provide the URL, tenant, database, and authentication credentials:

### Using Environment Variables

```bash
export CHROMA_URL=https://api.trychroma.com
export CHROMA_TENANT=your-tenant-id
export CHROMA_DATABASE=your-database-name
export CHROMA_PROVIDER=token
export CHROMA_API_KEY=your-api-key

berry serve
```

### Using Configuration File

```jsonc
{
  "chroma": {
    "url": "https://api.trychroma.com",
    "collection": "berry_memories",
    "provider": "token",
    "apiKey": "your-api-key",
    "tenant": "your-tenant-id",
    "database": "your-database-name"
  }
}
```

### Authentication Providers

Berry supports the following authentication providers for ChromaDB:

| Provider | Header | Description |
|----------|--------|-------------|
| `token` or `bearer` | `Authorization: Bearer <key>` | Standard bearer token (ChromaDB Cloud default) |
| `basic` | `Authorization: Basic <key>` | HTTP Basic authentication |
| `x-chroma-token` | `X-Chroma-Token: <key>` | Legacy ChromaDB token header |

If no provider is specified but an API key is provided, Berry defaults to bearer token authentication.

## Embedding Configuration

Berry requires an embedding service for semantic search. Embeddings convert text into vectors that enable similarity-based retrieval.

### Embedding Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `EMBEDDING_PROVIDER` | Embedding provider (`openai`, `local`*) | `openai` |
| `EMBEDDING_MODEL` | Model name | `text-embedding-3-small` |
| `EMBEDDING_BASE_URL` | API base URL (not used for `local`) | `https://api.openai.com/v1` |
| `EMBEDDING_API_KEY` | API key (not needed for `local`) | (none) |
| `OPENAI_API_KEY` | Fallback API key if `EMBEDDING_API_KEY` not set | (none) |

\* The `local` provider requires Berry to be compiled with the `local-embeddings` feature.

### Using OpenAI Embeddings

```bash
export EMBEDDING_PROVIDER=openai
export EMBEDDING_MODEL=text-embedding-3-small
export OPENAI_API_KEY=sk-your-api-key
```

Available OpenAI models:

| Model | Dimensions | Notes |
|-------|------------|-------|
| `text-embedding-3-small` | 1536 | Good balance of quality and cost |
| `text-embedding-3-large` | 3072 | Higher quality, more expensive |
| `text-embedding-ada-002` | 1536 | Legacy model |

### Using Ollama (Local Embeddings)

For local embeddings without API costs, use [Ollama](https://ollama.ai/):

```bash
# Install an embedding model
ollama pull nomic-embed-text

# Configure Berry to use Ollama
export EMBEDDING_PROVIDER=openai
export EMBEDDING_MODEL=nomic-embed-text
export EMBEDDING_BASE_URL=http://localhost:11434/v1
# No API key needed for local Ollama
```

Supported Ollama embedding models:

| Model | Dimensions | Notes |
|-------|------------|-------|
| `nomic-embed-text` | 768 | Good quality, popular choice |
| `mxbai-embed-large` | 1024 | Higher quality |
| `all-minilm` | 384 | Smaller, faster |
| `snowflake-arctic-embed` | 1024 | High quality |

### Embedding Dimension Mismatch

If you change embedding models, the dimensions may differ. ChromaDB collections are created with a fixed embedding dimension. If you see errors like:

```
Collection expecting embedding with dimension of 384, got 768
```

You need to migrate your data to a new collection. See the [migrate command](./cli.md#migrate) for details.

### Using Local Embeddings (embed_anything)

Berry supports fully local, offline embedding generation using the `embed_anything` crate. This requires no external API calls or services.

> **Note**: Local embeddings require Berry to be compiled with the `local-embeddings` feature flag. Pre-built releases may not include this feature.

#### Enabling Local Embeddings

When building from source, enable the feature on the `berry` crate:

```bash
# CPU-only (works on all platforms)
cargo build --release --features berry/local-embeddings

# With Metal GPU acceleration (macOS only)
cargo build --release --features berry/local-embeddings-metal

# With CUDA GPU acceleration (requires NVIDIA GPU + CUDA toolkit)
cargo build --release --features berry/local-embeddings-cuda

# With Accelerate framework (macOS only)
cargo build --release --features berry/local-embeddings-accelerate
```

This builds the entire workspace with local embedding support enabled.

#### Configuration

```bash
export EMBEDDING_PROVIDER=local
export EMBEDDING_MODEL=minilm
```

Or in the config file:

```jsonc
{
  "embedding": {
    "provider": "local",
    "model": "minilm"
  }
}
```

#### Supported Models

You can use shorthand aliases or full HuggingFace model IDs:

| Alias | Full Model ID | Dimensions |
|-------|---------------|------------|
| `jina-small` | `jinaai/jina-embeddings-v2-small-en` | 512 |
| `jina-base` | `jinaai/jina-embeddings-v2-base-en` | 768 |
| `minilm` | `sentence-transformers/all-MiniLM-L6-v2` | 384 |
| `bge-small` | `BAAI/bge-small-en-v1.5` | 384 |
| `bge-base` | `BAAI/bge-base-en-v1.5` | 768 |

You can also specify any HuggingFace model ID directly:

```bash
export EMBEDDING_MODEL=sentence-transformers/all-mpnet-base-v2
```

#### Model Cache

Models are downloaded from HuggingFace on first use and cached locally:

| Platform | Default Cache Location |
|----------|----------------------|
| All | `~/.cache/huggingface/` |

Override the cache location with the `HF_HOME` environment variable:

```bash
export HF_HOME=/path/to/cache
```

#### Comparison with Other Providers

| Provider | Requires API Key | Requires Network | GPU Support |
|----------|-----------------|------------------|-------------|
| `openai` | Yes | Yes | N/A (cloud) |
| `openai` + Ollama | No | No (after model download) | Yes |
| `local` | No | No (after model download) | Yes (Metal/CUDA) |

Local embeddings are ideal for:
- Air-gapped or offline environments
- Avoiding API costs
- Privacy-sensitive applications
- Development and testing

### Configuration File

Embedding configuration can also be set in the config file:

```jsonc
{
  "embedding": {
    "provider": "openai",
    "model": "nomic-embed-text",
    "baseUrl": "http://localhost:11434/v1"
    // "apiKey": "optional-for-local-services"
  }
}
```
