//! Document list tool for MCP.

use serde::{Deserialize, Serialize};

/// Document list tool input.
#[derive(Debug, Deserialize)]
pub struct DocListInput {
    /// Filter by tags
    pub tags: Option<Vec<String>>,
    /// Filter by creator
    pub created_by: Option<String>,
}

/// Document list result entry.
#[derive(Debug, Serialize)]
pub struct DocListEntry {
    /// Document ID (slug)
    pub id: String,
    /// Document title
    pub title: String,
    /// Document tags
    pub tags: Vec<String>,
    /// Document creator
    pub created_by: String,
}

/// Document list tool output.
#[derive(Debug, Serialize)]
pub struct DocListOutput {
    /// Success status
    pub success: bool,
    /// List of documents
    pub documents: Vec<DocListEntry>,
    /// Total count
    pub total: usize,
    /// Error message if failed
    pub error: Option<String>,
}

/// Document list tool definition.
pub struct DocListTool;

impl DocListTool {
    /// Tool name.
    pub const NAME: &'static str = "doc_list";

    /// Tool description.
    pub const DESCRIPTION: &'static str =
        "List documents in the Berry document store with optional tag and creator filters.";
}
