//! Schema endpoint for MCP integration.

use axum::Json;
use schemars::schema_for;
use serde::Serialize;

use berry::{CreateMemoryRequest, Memory, SearchRequest};

/// Server schema response.
#[derive(Debug, Serialize)]
pub struct SchemaResponse {
    /// API version
    pub version: String,
    /// Schema for Memory type
    pub memory: serde_json::Value,
    /// Schema for CreateMemoryRequest
    pub create_request: serde_json::Value,
    /// Schema for SearchRequest
    pub search_request: serde_json::Value,
}

/// Schema handler.
///
/// GET /schema
pub async fn schema_handler() -> Json<SchemaResponse> {
    Json(SchemaResponse {
        version: berry::VERSION.to_string(),
        memory: serde_json::to_value(schema_for!(Memory)).unwrap_or_default(),
        create_request: serde_json::to_value(schema_for!(CreateMemoryRequest)).unwrap_or_default(),
        search_request: serde_json::to_value(schema_for!(SearchRequest)).unwrap_or_default(),
    })
}
