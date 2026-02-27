//! Document update tool for MCP.

use serde::{Deserialize, Serialize};

/// Document update tool input.
#[derive(Debug, Deserialize)]
pub struct DocUpdateInput {
    /// Document ID (slug) to update
    pub id: String,
    /// New title (optional)
    pub title: Option<String>,
    /// New content (optional)
    pub content: Option<String>,
    /// New tags (optional, replaces existing)
    pub tags: Option<Vec<String>>,
}

/// Document update tool output.
#[derive(Debug, Serialize)]
pub struct DocUpdateOutput {
    /// Success status
    pub success: bool,
    /// Updated document ID
    pub id: Option<String>,
    /// Updated document title
    pub title: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Document update tool definition.
pub struct DocUpdateTool;

impl DocUpdateTool {
    /// Tool name.
    pub const NAME: &'static str = "doc_update";

    /// Tool description.
    pub const DESCRIPTION: &'static str =
        "Update an existing document in the Berry document store. All fields are optional for partial updates.";
}
