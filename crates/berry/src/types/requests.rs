use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{MemoryType, VisibilityLevel};

/// Request to create a new memory.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateMemoryRequest {
    /// The content of the memory
    pub content: String,
    /// The type of memory (defaults to information)
    #[serde(rename = "type", default)]
    pub memory_type: MemoryType,
    /// Tags to associate with the memory
    #[serde(default)]
    pub tags: Vec<String>,
    /// Who is creating this memory
    pub created_by: String,
    /// Optional references to other memory IDs
    #[serde(default)]
    pub references: Vec<String>,
    /// Visibility level (defaults to public)
    #[serde(default)]
    pub visibility: VisibilityLevel,
    /// List of actors to share with (when visibility is shared)
    #[serde(default)]
    pub shared_with: Vec<String>,
}

/// Request to search for memories.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchRequest {
    /// Search query string
    pub query: String,
    /// Actor performing the search (for visibility filtering)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub as_actor: Option<String>,
    /// Filter by memory type
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub memory_type: Option<MemoryType>,
    /// Filter by tags (any match)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Maximum number of results (defaults to 10)
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Filter memories created after this date
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<DateTime<Utc>>,
    /// Filter memories created before this date
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<DateTime<Utc>>,
}

impl Default for SearchRequest {
    fn default() -> Self {
        Self {
            query: String::new(),
            as_actor: None,
            memory_type: None,
            tags: Vec::new(),
            limit: default_limit(),
            from: None,
            to: None,
        }
    }
}

fn default_limit() -> usize {
    10
}

/// Request to update memory visibility.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateVisibilityRequest {
    /// New visibility level
    pub visibility: VisibilityLevel,
    /// Updated list of actors to share with (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shared_with: Option<Vec<String>>,
}

/// Response for a successful memory operation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryResponse {
    /// Whether the operation was successful
    pub success: bool,
    /// The memory (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<super::Memory>,
    /// Error message (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Response for a search operation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchResponse {
    /// Whether the search was successful
    pub success: bool,
    /// List of matching memories
    #[serde(default)]
    pub memories: Vec<super::Memory>,
    /// Total count of matches
    pub total: usize,
    /// Error message (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Response for a delete operation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeleteResponse {
    /// Whether the deletion was successful
    pub success: bool,
    /// Whether a memory was actually deleted
    pub deleted: bool,
    /// Error message (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Health check response.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HealthResponse {
    /// Health status
    pub status: String,
    /// Service version
    pub version: String,
    /// Database connectivity status
    pub database: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_memory_request_defaults() {
        let json = r#"{"content": "test", "created_by": "user"}"#;
        let req: CreateMemoryRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.content, "test");
        assert_eq!(req.created_by, "user");
        assert_eq!(req.memory_type, MemoryType::Information);
        assert_eq!(req.visibility, VisibilityLevel::Public);
        assert!(req.tags.is_empty());
        assert!(req.references.is_empty());
        assert!(req.shared_with.is_empty());
    }

    #[test]
    fn test_search_request_defaults() {
        let req = SearchRequest {
            query: "test".to_string(),
            ..Default::default()
        };

        assert_eq!(req.limit, 10);
        assert!(req.as_actor.is_none());
        assert!(req.memory_type.is_none());
        assert!(req.tags.is_empty());
        assert!(req.from.is_none());
        assert!(req.to.is_none());
    }

    #[test]
    fn test_search_request_serialization() {
        let req = SearchRequest {
            query: "test query".to_string(),
            as_actor: Some("user1".to_string()),
            memory_type: Some(MemoryType::Question),
            limit: 20,
            ..Default::default()
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"query\":\"test query\""));
        assert!(json.contains("\"as_actor\":\"user1\""));
        assert!(json.contains("\"type\":\"question\""));
        assert!(json.contains("\"limit\":20"));
    }

    #[test]
    fn test_update_visibility_request() {
        let req = UpdateVisibilityRequest {
            visibility: VisibilityLevel::Shared,
            shared_with: Some(vec!["user1".to_string(), "user2".to_string()]),
        };

        let json = serde_json::to_string(&req).unwrap();
        let parsed: UpdateVisibilityRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.visibility, VisibilityLevel::Shared);
        assert_eq!(parsed.shared_with.unwrap().len(), 2);
    }
}
