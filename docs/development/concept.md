# Project Concept

This is a pre-planning document meant to provide high-level context for the intention of the project.

## Project

"Berry" is a memory system that exists between you and your AI tooling. This is the Rust implementation, providing
improved performance and native binaries compared to the original TypeScript version.

### Structure

This project is built as a Cargo workspace with the following crates:

- `berry` - Shared library containing types, store abstraction, and utilities
- `cli` - Command-line interface binary
- `server` - HTTP API server binary
- `mcp` - Model Context Protocol server binary

## Problem

A system that tracks information for AI-assisted workflows. The goal is to enable AI tooling to better direct and guide
behaviors and decisions through a custom memory system that stores:

- Questions and their answers
- Requests and their resolutions
- Information generated without prompts

## Solution

A program running on the user's machine provides an API to a ChromaDB vector database. The API provides endpoints for
typical operations for interacting with the data, as well as search functionality.

### API Endpoints

- `GET /health` - Health check
- `GET /v1/memory/<id>` - Get a single memory by ID
- `POST /v1/memory` - Create a memory
- `DELETE /v1/memory/<id>` - Delete a memory by ID
- `PATCH /v1/memory/<id>/visibility` - Update memory visibility
- `POST /v1/search` - Search for memories matching criteria
- `GET /schema` - Get API schema (for MCP integration)

### Memory Schema

The memory collection includes:

- `id` - Unique identifier (format: `mem_<timestamp>_<random>`)
- `content` - The memory content
- `type` - Type of information: question, request, or information
- `tags` - Metadata tags
- `created_by` - Who created the memory
- `created_at` - When the memory was created
- `updated_at` - When the memory was last updated
- `owner` - Optional owner
- `visibility` - Visibility level: private, shared, or public
- `shared_with` - List of actors the memory is shared with

### Technologies

The following technologies are used:

- **Rust** - Primary language for all components
- **Cargo** - Build system and package manager
- **ChromaDB** - Vector database for storing memory records
- **Axum** - HTTP server framework
- **Clap** - CLI argument parsing
- **Inquire** - Interactive prompts
- **Tokio** - Async runtime
- **Tracing** - Structured logging

### User Interfaces

Berry supports the following interfaces:

- **CLI** (`berry`) - Command-line interface with interactive prompts
- **HTTP API** (`berry-server`) - RESTful API server
- **MCP** (`berry-mcp`) - Model Context Protocol server for AI tool integration

### CLI Commands

- `berry` - Interactive mode with guided prompts
- `berry remember` - Add a memory to the database
- `berry search` - Search for memories using vector search
- `berry forget` - Remove a memory by ID
- `berry recall` - Retrieve a single memory by ID
- `berry serve` - Start the HTTP server
- `berry mcp` - Start the MCP server
- `berry init` - Initialize configuration

### Future Considerations

- Additional storage backends (Milvus, Qdrant)
- Web interface
- Mobile clients
- Notifications for AI-generated questions
