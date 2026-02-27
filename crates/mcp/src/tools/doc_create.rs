//! Document create tool for MCP.

use serde::{Deserialize, Serialize};

/// Document create tool input.
#[derive(Debug, Deserialize)]
pub struct DocCreateInput {
    /// Document title
    pub title: String,
    /// Markdown body content
    pub content: String,
    /// Tags for the document
    pub tags: Option<Vec<String>>,
    /// Who is creating this document
    pub created_by: String,
}

/// Document create tool output.
#[derive(Debug, Serialize)]
pub struct DocCreateOutput {
    /// Success status
    pub success: bool,
    /// Created document ID (slug)
    pub id: Option<String>,
    /// Created document title
    pub title: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Document create tool definition.
pub struct DocCreateTool;

impl DocCreateTool {
    /// Tool name.
    pub const NAME: &'static str = "doc_create";

    /// Tool description.
    pub const DESCRIPTION: &'static str =
        "Create a new markdown document in the Berry document store.";
}
