//! Remember tool for MCP.

use serde::{Deserialize, Serialize};

use berry::{MemoryType, VisibilityLevel};

/// Remember tool input.
#[derive(Debug, Deserialize)]
pub struct RememberInput {
    /// The content to remember
    pub content: String,
    /// Memory type (question, request, information)
    #[serde(rename = "type")]
    pub memory_type: Option<String>,
    /// Tags for the memory
    pub tags: Option<Vec<String>>,
    /// Who is creating this memory
    pub created_by: String,
    /// Visibility level (private, shared, public)
    pub visibility: Option<String>,
    /// Actors to share with
    pub shared_with: Option<Vec<String>>,
}

/// Remember tool output.
#[derive(Debug, Serialize)]
pub struct RememberOutput {
    /// Success status
    pub success: bool,
    /// Created memory ID
    pub id: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Remember tool definition.
pub struct RememberTool;

impl RememberTool {
    /// Tool name.
    pub const NAME: &'static str = "remember";

    /// Tool description.
    pub const DESCRIPTION: &'static str = "Store a new memory in the Berry memory system. Use this to remember important information, questions, or requests.";

    /// Parse memory type from string.
    pub fn parse_memory_type(s: &str) -> Option<MemoryType> {
        s.parse().ok()
    }

    /// Parse visibility level from string.
    pub fn parse_visibility(s: &str) -> Option<VisibilityLevel> {
        s.parse().ok()
    }
}
