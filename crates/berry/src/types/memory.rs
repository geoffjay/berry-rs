use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{MemoryType, VisibilityLevel};

/// A memory stored in the system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Memory {
    /// Unique identifier in format "mem_<timestamp>_<random>"
    pub id: String,
    /// The content of the memory
    pub content: String,
    /// The type of memory
    #[serde(rename = "type")]
    pub memory_type: MemoryType,
    /// Tags associated with the memory
    #[serde(default)]
    pub tags: Vec<String>,
    /// Who created this memory
    pub created_by: String,
    /// When the memory was created
    pub created_at: DateTime<Utc>,
    /// When the memory was last updated
    pub updated_at: DateTime<Utc>,
    /// Optional owner of the memory
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Visibility level of the memory
    #[serde(default)]
    pub visibility: VisibilityLevel,
    /// List of actors this memory is shared with (when visibility is Shared)
    #[serde(default)]
    pub shared_with: Vec<String>,
}

impl Memory {
    /// Generate a new memory ID in the format "mem_<timestamp>_<random>".
    pub fn generate_id() -> String {
        let timestamp = Utc::now().timestamp_millis();
        let random = Uuid::new_v4().to_string()[..8].to_string();
        format!("mem_{}_{}", timestamp, random)
    }

    /// Check if this memory is visible to the given actor.
    pub fn is_visible_to(&self, actor: Option<&str>) -> bool {
        match self.visibility {
            VisibilityLevel::Public => true,
            VisibilityLevel::Private => {
                if let Some(actor) = actor {
                    self.created_by == actor || self.owner.as_deref() == Some(actor)
                } else {
                    false
                }
            }
            VisibilityLevel::Shared => {
                if let Some(actor) = actor {
                    self.created_by == actor
                        || self.owner.as_deref() == Some(actor)
                        || self.shared_with.contains(&actor.to_string())
                } else {
                    false
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_memory() -> Memory {
        Memory {
            id: Memory::generate_id(),
            content: "Test content".to_string(),
            memory_type: MemoryType::Information,
            tags: vec!["test".to_string()],
            created_by: "user1".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            owner: None,
            visibility: VisibilityLevel::Public,
            shared_with: vec![],
        }
    }

    #[test]
    fn test_generate_id_format() {
        let id = Memory::generate_id();
        assert!(id.starts_with("mem_"));
        let parts: Vec<&str> = id.split('_').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "mem");
        // Check timestamp is a valid number
        assert!(parts[1].parse::<i64>().is_ok());
        // Check random part is 8 characters
        assert_eq!(parts[2].len(), 8);
    }

    #[test]
    fn test_visibility_public() {
        let memory = create_test_memory();
        assert!(memory.is_visible_to(None));
        assert!(memory.is_visible_to(Some("anyone")));
    }

    #[test]
    fn test_visibility_private() {
        let mut memory = create_test_memory();
        memory.visibility = VisibilityLevel::Private;

        assert!(!memory.is_visible_to(None));
        assert!(!memory.is_visible_to(Some("other")));
        assert!(memory.is_visible_to(Some("user1"))); // creator
    }

    #[test]
    fn test_visibility_private_with_owner() {
        let mut memory = create_test_memory();
        memory.visibility = VisibilityLevel::Private;
        memory.owner = Some("owner1".to_string());

        assert!(memory.is_visible_to(Some("user1"))); // creator
        assert!(memory.is_visible_to(Some("owner1"))); // owner
        assert!(!memory.is_visible_to(Some("other")));
    }

    #[test]
    fn test_visibility_shared() {
        let mut memory = create_test_memory();
        memory.visibility = VisibilityLevel::Shared;
        memory.shared_with = vec!["friend1".to_string(), "friend2".to_string()];

        assert!(!memory.is_visible_to(None));
        assert!(memory.is_visible_to(Some("user1"))); // creator
        assert!(memory.is_visible_to(Some("friend1"))); // shared with
        assert!(memory.is_visible_to(Some("friend2"))); // shared with
        assert!(!memory.is_visible_to(Some("stranger")));
    }

    #[test]
    fn test_memory_serialization() {
        let memory = create_test_memory();
        let json = serde_json::to_string(&memory).unwrap();
        assert!(json.contains("\"type\":\"information\""));
        assert!(json.contains("\"visibility\":\"public\""));

        let parsed: Memory = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.content, memory.content);
        assert_eq!(parsed.memory_type, memory.memory_type);
    }
}
