//! Search tool for MCP.

use serde::{Deserialize, Serialize};

/// Search tool input.
#[derive(Debug, Deserialize)]
pub struct SearchInput {
    /// Search query
    pub query: String,
    /// Actor performing the search (for visibility filtering)
    pub as_actor: Option<String>,
    /// Filter by memory type
    #[serde(rename = "type")]
    pub memory_type: Option<String>,
    /// Filter by tags
    pub tags: Option<Vec<String>>,
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Start date filter (ISO 8601)
    pub from: Option<String>,
    /// End date filter (ISO 8601)
    pub to: Option<String>,
}

/// Search result.
#[derive(Debug, Serialize)]
pub struct SearchResult {
    /// Memory ID
    pub id: String,
    /// Memory content
    pub content: String,
    /// Memory type
    pub memory_type: String,
    /// Tags
    pub tags: Vec<String>,
}

/// Search tool output.
#[derive(Debug, Serialize)]
pub struct SearchOutput {
    /// Success status
    pub success: bool,
    /// Search results
    pub results: Vec<SearchResult>,
    /// Total count
    pub total: usize,
    /// Error message if failed
    pub error: Option<String>,
}

/// Search tool definition.
pub struct SearchTool;

impl SearchTool {
    /// Tool name.
    pub const NAME: &'static str = "search";

    /// Tool description.
    pub const DESCRIPTION: &'static str =
        "Search for memories in the Berry memory system using semantic search.";
}
