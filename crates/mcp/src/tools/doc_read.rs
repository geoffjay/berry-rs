//! Document read tool for MCP.

use serde::{Deserialize, Serialize};

/// Document read tool input.
#[derive(Debug, Deserialize)]
pub struct DocReadInput {
    /// Document ID (slug) to retrieve
    pub id: String,
}

/// Document read tool output.
#[derive(Debug, Serialize)]
pub struct DocReadOutput {
    /// Success status
    pub success: bool,
    /// Whether the document was found
    pub found: bool,
    /// Document title
    pub title: Option<String>,
    /// Document content (markdown)
    pub content: Option<String>,
    /// Document tags
    pub tags: Option<Vec<String>>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Document read tool definition.
pub struct DocReadTool;

impl DocReadTool {
    /// Tool name.
    pub const NAME: &'static str = "doc_read";

    /// Tool description.
    pub const DESCRIPTION: &'static str =
        "Retrieve a document by its ID from the Berry document store.";
}
