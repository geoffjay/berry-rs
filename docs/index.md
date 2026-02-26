# Berry

**A semantic memory system for AI assistants**

Berry provides persistent, searchable memory storage that bridges the gap between you and your AI tooling. Store information, questions, and requests, then retrieve them using natural language search powered by vector embeddings.

<div class="grid cards" markdown>

-   :material-clock-fast:{ .lg .middle } __Quick to Set Up__

    ---

    Install Berry with a single command and start storing memories in minutes.

    [:octicons-arrow-right-24: Getting Started](public/getting-started.md)

-   :material-brain:{ .lg .middle } __Semantic Search__

    ---

    Find memories using natural language. Berry understands meaning, not just keywords.

    [:octicons-arrow-right-24: CLI Reference](public/cli.md)

-   :material-cog:{ .lg .middle } __Flexible Configuration__

    ---

    Use embedded LanceDB or cloud ChromaDB, with OpenAI or local Ollama embeddings.

    [:octicons-arrow-right-24: Configuration](public/configuration.md)

-   :material-robot:{ .lg .middle } __AI Integration__

    ---

    Native MCP server for Claude, OpenCode, and other AI assistants.

    [:octicons-arrow-right-24: MCP Setup](#mcp-integration)

</div>

## Why Berry?

AI assistants are powerful but stateless. Each conversation starts fresh, losing valuable context from previous sessions. Berry solves this by providing:

- **Persistent Memory**: Store important information that survives between sessions
- **Semantic Search**: Find relevant memories using natural language queries
- **Structured Data**: Categorize memories as questions, requests, or information
- **Access Control**: Control visibility with private, shared, and public memories
- **AI-Native**: Built-in MCP server for seamless AI assistant integration

## Quick Start

### Install

```bash
curl -fsSL https://raw.githubusercontent.com/geoffjay/berry-rs/main/scripts/install.sh | bash
```

### Configure

```bash
# Initialize configuration
berry init

# That's it! Berry uses LanceDB by default (embedded, no setup needed).

# Optionally, to use ChromaDB instead:
# docker run -d -p 8000:8000 chromadb/chroma
# export BERRY_STORE=chroma
```

### Start the Server

```bash
berry serve --foreground
```

### Store and Search

```bash
# Store a memory
berry remember "The API uses JWT tokens for authentication"

# Search memories
berry search "authentication"

# Store with metadata
berry remember "How do I reset a password?" --type question --tags "auth,faq"
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Clients                               │
├─────────────┬─────────────┬─────────────┬───────────────────┤
│  berry CLI  │  MCP Server │  HTTP API   │  Your Application │
└──────┬──────┴──────┬──────┴──────┬──────┴─────────┬─────────┘
       │             │             │                │
       └─────────────┴──────┬──────┴────────────────┘
                            │
                    ┌───────▼───────┐
                    │ Berry Server  │
                    │  (HTTP API)   │
                    └───────┬───────┘
                            │
              ┌─────────────┼─────────────┐
              │             │             │
      ┌───────▼───────┐ ┌───▼───┐ ┌───────▼───────┐
      ┌───────▼───────┐    │             │
      │   LanceDB     │    │             │
      │  (Default)    │    │             │
      └───────────────┘    │             │
      ┌───────▼───────┐ ┌───▼───┐ ┌───────▼───────┐
      │   ChromaDB    │ │Ollama │ │    OpenAI     │
      │ (Alternative) │ │(Local)│ │  (Embeddings) │
      └───────────────┘ └───────┘ └───────────────┘
```

## MCP Integration

Berry includes a native MCP (Model Context Protocol) server for AI assistant integration.

### Claude Code

Add to your Claude Code MCP configuration:

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

### Available Tools

| Tool | Description |
|------|-------------|
| `remember` | Store a new memory with optional type, tags, and visibility |
| `recall` | Retrieve a specific memory by ID |
| `forget` | Delete a memory |
| `search` | Search memories using semantic similarity |

## Memory Types

Berry supports three memory types to help organize information:

| Type | Use Case | Example |
|------|----------|---------|
| `question` | Questions to answer later | "What's the deployment schedule?" |
| `request` | Tasks or action items | "Update the documentation" |
| `information` | Facts and knowledge | "The API rate limit is 100 req/min" |

## Embedding Providers

Berry supports multiple embedding providers for semantic search:

### OpenAI (Cloud)

```bash
export EMBEDDING_PROVIDER=openai
export EMBEDDING_MODEL=text-embedding-3-small
export OPENAI_API_KEY=sk-your-key
```

### Ollama (Local)

```bash
ollama pull nomic-embed-text

export EMBEDDING_PROVIDER=openai
export EMBEDDING_MODEL=nomic-embed-text
export EMBEDDING_BASE_URL=http://localhost:11434/v1
```

## Documentation

<div class="grid" markdown>

[:material-rocket-launch: **Getting Started**](public/getting-started.md)

Installation, setup, and first steps.

[:material-console: **CLI Reference**](public/cli.md)

Complete command-line interface documentation.

[:material-cog: **Configuration**](public/configuration.md)

Configuration file and environment variables.

[:material-code-braces: **Development**](public/develop.md)

Contributing and development setup.

</div>

## Requirements

- **Embedding Provider**: OpenAI API or local Ollama
- **Rust 1.75+**: Only required for building from source
- **ChromaDB** (optional): Only if using ChromaDB instead of the default LanceDB

## Status

> **Beta**: Berry is functional and safe to use, but breaking changes may occur before v1.0.

## License

Berry is open source software. See the repository for license details.

## Links

- [GitHub Repository](https://github.com/geoffjay/berry-rs)
- [ChromaDB](https://trychroma.com)
- [Ollama](https://ollama.ai)
