//! Document delete tool for MCP.

use serde::{Deserialize, Serialize};

/// Document delete tool input.
#[derive(Debug, Deserialize)]
pub struct DocDeleteInput {
    /// Document ID (slug) to delete
    pub id: String,
}

/// Document delete tool output.
#[derive(Debug, Serialize)]
pub struct DocDeleteOutput {
    /// Success status
    pub success: bool,
    /// Whether the document was deleted
    pub deleted: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Document delete tool definition.
pub struct DocDeleteTool;

impl DocDeleteTool {
    /// Tool name.
    pub const NAME: &'static str = "doc_delete";

    /// Tool description.
    pub const DESCRIPTION: &'static str = "Delete a document from the Berry document store.";
}
