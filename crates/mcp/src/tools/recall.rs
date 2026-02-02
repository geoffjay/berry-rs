//! Recall tool for MCP.

use serde::{Deserialize, Serialize};

/// Recall tool input.
#[derive(Debug, Deserialize)]
pub struct RecallInput {
    /// Memory ID to retrieve
    pub id: String,
    /// Actor performing the recall (for visibility filtering)
    pub as_actor: Option<String>,
}

/// Recall tool output.
#[derive(Debug, Serialize)]
pub struct RecallOutput {
    /// Success status
    pub success: bool,
    /// Whether the memory was found
    pub found: bool,
    /// Memory content (if found and visible)
    pub content: Option<String>,
    /// Memory type
    pub memory_type: Option<String>,
    /// Tags
    pub tags: Option<Vec<String>>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Recall tool definition.
pub struct RecallTool;

impl RecallTool {
    /// Tool name.
    pub const NAME: &'static str = "recall";

    /// Tool description.
    pub const DESCRIPTION: &'static str =
        "Retrieve a memory by its ID from the Berry memory system.";
}
