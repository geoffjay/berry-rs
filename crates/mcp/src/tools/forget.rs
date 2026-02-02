//! Forget tool for MCP.

use serde::{Deserialize, Serialize};

/// Forget tool input.
#[derive(Debug, Deserialize)]
pub struct ForgetInput {
    /// Memory ID to delete
    pub id: String,
    /// Actor performing the deletion (for permission checking)
    pub as_actor: Option<String>,
}

/// Forget tool output.
#[derive(Debug, Serialize)]
pub struct ForgetOutput {
    /// Success status
    pub success: bool,
    /// Whether the memory was deleted
    pub deleted: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Forget tool definition.
pub struct ForgetTool;

impl ForgetTool {
    /// Tool name.
    pub const NAME: &'static str = "forget";

    /// Tool description.
    pub const DESCRIPTION: &'static str = "Delete a memory from the Berry memory system. Only the creator or owner can delete a memory.";
}
