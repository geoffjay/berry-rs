//! Berry MCP Server Library
//!
//! Model Context Protocol server for the Berry memory system.
//! Communicates over stdio using JSON-RPC 2.0.
//!
//! Can be used as a library or run as a standalone binary.

use std::io::{BufRead, Write};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use berry::config::load_config;

pub mod handler;
pub mod tools;

use handler::BerryMcpClient;
use tools::{
    forget::{ForgetInput, ForgetOutput, ForgetTool},
    recall::{RecallInput, RecallOutput, RecallTool},
    remember::{RememberInput, RememberOutput, RememberTool},
    search::{SearchInput, SearchOutput, SearchResult, SearchTool},
};

/// MCP server configuration options.
#[derive(Debug, Clone, Default)]
pub struct McpConfig {
    /// Berry server URL.
    pub server_url: Option<String>,
    /// Enable verbose logging.
    pub verbose: bool,
}

/// JSON-RPC 2.0 request.
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

/// JSON-RPC 2.0 response.
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 error.
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

/// MCP tool definition.
#[derive(Debug, Serialize)]
struct McpTool {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

/// MCP server info.
#[derive(Debug, Serialize)]
struct McpServerInfo {
    name: String,
    version: String,
}

/// MCP capabilities.
#[derive(Debug, Serialize)]
struct McpCapabilities {
    tools: Value,
}

/// Initialize result.
#[derive(Debug, Serialize)]
struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    protocol_version: String,
    #[serde(rename = "serverInfo")]
    server_info: McpServerInfo,
    capabilities: McpCapabilities,
}

/// Tools list result.
#[derive(Debug, Serialize)]
struct ListToolsResult {
    tools: Vec<McpTool>,
}

/// Tool call result.
#[derive(Debug, Serialize)]
struct CallToolResult {
    content: Vec<ToolContent>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    is_error: Option<bool>,
}

/// Tool content.
#[derive(Debug, Serialize)]
struct ToolContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

impl JsonRpcResponse {
    fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(id: Option<Value>, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
        }
    }
}

/// MCP server state.
struct McpServer {
    client: BerryMcpClient,
}

impl McpServer {
    fn new(server_url: &str) -> anyhow::Result<Self> {
        Ok(Self {
            client: BerryMcpClient::new(server_url)?,
        })
    }

    /// Handle a JSON-RPC request.
    async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.id),
            "initialized" => JsonRpcResponse::success(request.id, Value::Null),
            "tools/list" => self.handle_list_tools(request.id),
            "tools/call" => self.handle_call_tool(request.id, request.params).await,
            _ => JsonRpcResponse::error(
                request.id,
                -32601,
                format!("Method not found: {}", request.method),
            ),
        }
    }

    fn handle_initialize(&self, id: Option<Value>) -> JsonRpcResponse {
        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            server_info: McpServerInfo {
                name: "berry-mcp".to_string(),
                version: berry::VERSION.to_string(),
            },
            capabilities: McpCapabilities {
                tools: serde_json::json!({}),
            },
        };

        JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
    }

    fn handle_list_tools(&self, id: Option<Value>) -> JsonRpcResponse {
        let tools = vec![
            McpTool {
                name: RememberTool::NAME.to_string(),
                description: RememberTool::DESCRIPTION.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "The content to remember"
                        },
                        "type": {
                            "type": "string",
                            "enum": ["question", "request", "information"],
                            "description": "Type of memory"
                        },
                        "tags": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Tags for the memory"
                        },
                        "created_by": {
                            "type": "string",
                            "description": "Who is creating this memory"
                        },
                        "visibility": {
                            "type": "string",
                            "enum": ["private", "shared", "public"],
                            "description": "Visibility level"
                        },
                        "shared_with": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Actors to share with"
                        }
                    },
                    "required": ["content", "created_by"]
                }),
            },
            McpTool {
                name: RecallTool::NAME.to_string(),
                description: RecallTool::DESCRIPTION.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Memory ID to retrieve"
                        },
                        "as_actor": {
                            "type": "string",
                            "description": "Actor performing the recall"
                        }
                    },
                    "required": ["id"]
                }),
            },
            McpTool {
                name: ForgetTool::NAME.to_string(),
                description: ForgetTool::DESCRIPTION.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Memory ID to delete"
                        },
                        "as_actor": {
                            "type": "string",
                            "description": "Actor performing the deletion"
                        }
                    },
                    "required": ["id"]
                }),
            },
            McpTool {
                name: SearchTool::NAME.to_string(),
                description: SearchTool::DESCRIPTION.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query"
                        },
                        "as_actor": {
                            "type": "string",
                            "description": "Actor performing the search"
                        },
                        "type": {
                            "type": "string",
                            "enum": ["question", "request", "information"],
                            "description": "Filter by memory type"
                        },
                        "tags": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Filter by tags"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum results"
                        },
                        "from": {
                            "type": "string",
                            "description": "Start date (ISO 8601)"
                        },
                        "to": {
                            "type": "string",
                            "description": "End date (ISO 8601)"
                        }
                    },
                    "required": ["query"]
                }),
            },
        ];

        let result = ListToolsResult { tools };
        JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
    }

    async fn handle_call_tool(&self, id: Option<Value>, params: Value) -> JsonRpcResponse {
        let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);

        match tool_name {
            "remember" => self.call_remember(id, arguments).await,
            "recall" => self.call_recall(id, arguments).await,
            "forget" => self.call_forget(id, arguments).await,
            "search" => self.call_search(id, arguments).await,
            _ => JsonRpcResponse::error(id, -32602, format!("Unknown tool: {}", tool_name)),
        }
    }

    async fn call_remember(&self, id: Option<Value>, arguments: Value) -> JsonRpcResponse {
        let input: RememberInput = match serde_json::from_value(arguments) {
            Ok(i) => i,
            Err(e) => {
                return JsonRpcResponse::error(id, -32602, format!("Invalid arguments: {}", e));
            }
        };

        let result = self
            .client
            .remember(
                input.content,
                input
                    .memory_type
                    .and_then(|t| RememberTool::parse_memory_type(&t)),
                input.tags,
                input.created_by,
                input
                    .visibility
                    .and_then(|v| RememberTool::parse_visibility(&v)),
                input.shared_with,
            )
            .await;

        match result {
            Ok(memory) => {
                let output = RememberOutput {
                    success: true,
                    id: Some(memory.id),
                    error: None,
                };
                self.tool_response(id, output)
            }
            Err(e) => {
                let output = RememberOutput {
                    success: false,
                    id: None,
                    error: Some(e.to_string()),
                };
                self.tool_response_error(id, output)
            }
        }
    }

    async fn call_recall(&self, id: Option<Value>, arguments: Value) -> JsonRpcResponse {
        let input: RecallInput = match serde_json::from_value(arguments) {
            Ok(i) => i,
            Err(e) => {
                return JsonRpcResponse::error(id, -32602, format!("Invalid arguments: {}", e));
            }
        };

        let result = self
            .client
            .recall(&input.id, input.as_actor.as_deref())
            .await;

        match result {
            Ok(Some(memory)) => {
                let output = RecallOutput {
                    success: true,
                    found: true,
                    content: Some(memory.content),
                    memory_type: Some(memory.memory_type.to_string()),
                    tags: Some(memory.tags),
                    error: None,
                };
                self.tool_response(id, output)
            }
            Ok(None) => {
                let output = RecallOutput {
                    success: true,
                    found: false,
                    content: None,
                    memory_type: None,
                    tags: None,
                    error: None,
                };
                self.tool_response(id, output)
            }
            Err(e) => {
                let output = RecallOutput {
                    success: false,
                    found: false,
                    content: None,
                    memory_type: None,
                    tags: None,
                    error: Some(e.to_string()),
                };
                self.tool_response_error(id, output)
            }
        }
    }

    async fn call_forget(&self, id: Option<Value>, arguments: Value) -> JsonRpcResponse {
        let input: ForgetInput = match serde_json::from_value(arguments) {
            Ok(i) => i,
            Err(e) => {
                return JsonRpcResponse::error(id, -32602, format!("Invalid arguments: {}", e));
            }
        };

        let result = self
            .client
            .forget(&input.id, input.as_actor.as_deref())
            .await;

        match result {
            Ok(deleted) => {
                let output = ForgetOutput {
                    success: true,
                    deleted,
                    error: None,
                };
                self.tool_response(id, output)
            }
            Err(e) => {
                let output = ForgetOutput {
                    success: false,
                    deleted: false,
                    error: Some(e.to_string()),
                };
                self.tool_response_error(id, output)
            }
        }
    }

    async fn call_search(&self, id: Option<Value>, arguments: Value) -> JsonRpcResponse {
        let input: SearchInput = match serde_json::from_value(arguments) {
            Ok(i) => i,
            Err(e) => {
                return JsonRpcResponse::error(id, -32602, format!("Invalid arguments: {}", e));
            }
        };

        let result = self
            .client
            .search(
                input.query,
                input.as_actor,
                input.memory_type.and_then(|t| t.parse().ok()),
                input.tags,
                input.limit,
                input.from,
                input.to,
            )
            .await;

        match result {
            Ok(memories) => {
                let results: Vec<SearchResult> = memories
                    .iter()
                    .map(|m| SearchResult {
                        id: m.id.clone(),
                        content: m.content.clone(),
                        memory_type: m.memory_type.to_string(),
                        tags: m.tags.clone(),
                    })
                    .collect();
                let total = results.len();
                let output = SearchOutput {
                    success: true,
                    results,
                    total,
                    error: None,
                };
                self.tool_response(id, output)
            }
            Err(e) => {
                let output = SearchOutput {
                    success: false,
                    results: vec![],
                    total: 0,
                    error: Some(e.to_string()),
                };
                self.tool_response_error(id, output)
            }
        }
    }

    fn tool_response<T: Serialize>(&self, id: Option<Value>, output: T) -> JsonRpcResponse {
        let text = serde_json::to_string_pretty(&output).unwrap_or_default();
        let result = CallToolResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text,
            }],
            is_error: None,
        };
        JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
    }

    fn tool_response_error<T: Serialize>(&self, id: Option<Value>, output: T) -> JsonRpcResponse {
        let text = serde_json::to_string_pretty(&output).unwrap_or_default();
        let result = CallToolResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text,
            }],
            is_error: Some(true),
        };
        JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
    }
}

/// Run the Berry MCP server.
///
/// This function blocks, reading from stdin and writing to stdout.
pub async fn run_mcp(config: McpConfig) -> anyhow::Result<()> {
    // Set up logging to stderr if verbose
    if config.verbose {
        berry::logging::init();
    }

    // Load app config
    let app_config = load_config().unwrap_or_default();

    // Get server URL
    let server_url = config
        .server_url
        .unwrap_or_else(|| app_config.server.url.clone());

    // Create MCP server
    let server = McpServer::new(&server_url)?;

    // Read from stdin, write to stdout
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let response = JsonRpcResponse::error(None, -32700, format!("Parse error: {}", e));
                writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
                stdout.flush()?;
                continue;
            }
        };

        let response = server.handle_request(request).await;
        writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
        stdout.flush()?;
    }

    Ok(())
}
