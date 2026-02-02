//! ChromaDB vector store implementation.

use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

use super::traits::VectorStore;
use crate::config::ChromaConfig;
use crate::error::{StoreError, StoreResult};
use crate::types::{CreateMemoryRequest, Memory, MemoryType, SearchRequest, VisibilityLevel};

/// ChromaDB vector store implementation.
pub struct ChromaStore {
    client: Client,
    base_url: String,
    collection_name: String,
    collection_id: Option<String>,
}

impl ChromaStore {
    /// Create a new ChromaStore with the given configuration.
    pub fn new(config: &ChromaConfig) -> Self {
        Self {
            client: Client::new(),
            base_url: config.url.trim_end_matches('/').to_string(),
            collection_name: config.collection.clone(),
            collection_id: None,
        }
    }

    /// Create a ChromaStore with a custom HTTP client.
    pub fn with_client(config: &ChromaConfig, client: Client) -> Self {
        Self {
            client,
            base_url: config.url.trim_end_matches('/').to_string(),
            collection_name: config.collection.clone(),
            collection_id: None,
        }
    }

    /// Get or create the collection and cache its ID.
    async fn ensure_collection(&self) -> StoreResult<String> {
        if let Some(ref id) = self.collection_id {
            return Ok(id.clone());
        }

        // Try to get existing collection
        let url = format!("{}/api/v1/collections/{}", self.base_url, self.collection_name);
        let resp = self.client.get(&url).send().await?;

        if resp.status().is_success() {
            let collection: ChromaCollection = resp.json().await?;
            return Ok(collection.id);
        }

        // Create collection if it doesn't exist
        let url = format!("{}/api/v1/collections", self.base_url);
        let body = serde_json::json!({
            "name": self.collection_name,
            "metadata": {
                "description": "Berry memory storage"
            }
        });

        let resp = self.client.post(&url).json(&body).send().await?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            return Err(StoreError::InitializationFailed(format!(
                "Failed to create collection: {}",
                error
            )));
        }

        let collection: ChromaCollection = resp.json().await?;
        Ok(collection.id)
    }

    /// Convert a Memory to ChromaDB metadata format.
    fn memory_to_metadata(memory: &Memory) -> HashMap<String, serde_json::Value> {
        let mut metadata = HashMap::new();
        metadata.insert("id".to_string(), serde_json::json!(memory.id));
        metadata.insert("type".to_string(), serde_json::json!(memory.memory_type.to_string()));
        metadata.insert("tags".to_string(), serde_json::json!(memory.tags.join(",")));
        metadata.insert("created_by".to_string(), serde_json::json!(memory.created_by));
        metadata.insert("created_at".to_string(), serde_json::json!(memory.created_at.to_rfc3339()));
        metadata.insert("updated_at".to_string(), serde_json::json!(memory.updated_at.to_rfc3339()));
        metadata.insert("visibility".to_string(), serde_json::json!(memory.visibility.to_string()));
        metadata.insert("shared_with".to_string(), serde_json::json!(memory.shared_with.join(",")));

        if let Some(ref owner) = memory.owner {
            metadata.insert("owner".to_string(), serde_json::json!(owner));
        }

        metadata
    }

    /// Convert ChromaDB metadata back to a Memory.
    fn metadata_to_memory(
        document: &str,
        metadata: &HashMap<String, serde_json::Value>,
    ) -> StoreResult<Memory> {
        let get_str = |key: &str| -> StoreResult<String> {
            metadata
                .get(key)
                .and_then(|v| v.as_str())
                .map(String::from)
                .ok_or_else(|| StoreError::InvalidData(format!("Missing field: {}", key)))
        };

        let id = get_str("id")?;
        let memory_type = get_str("type")?
            .parse::<MemoryType>()
            .map_err(|e| StoreError::InvalidData(e))?;
        let tags: Vec<String> = get_str("tags")?
            .split(',')
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();
        let created_by = get_str("created_by")?;
        let created_at = chrono::DateTime::parse_from_rfc3339(&get_str("created_at")?)
            .map_err(|e| StoreError::InvalidData(e.to_string()))?
            .with_timezone(&Utc);
        let updated_at = chrono::DateTime::parse_from_rfc3339(&get_str("updated_at")?)
            .map_err(|e| StoreError::InvalidData(e.to_string()))?
            .with_timezone(&Utc);
        let visibility = get_str("visibility")?
            .parse::<VisibilityLevel>()
            .map_err(|e| StoreError::InvalidData(e))?;
        let shared_with: Vec<String> = get_str("shared_with")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();
        let owner = metadata
            .get("owner")
            .and_then(|v| v.as_str())
            .map(String::from);

        Ok(Memory {
            id,
            content: document.to_string(),
            memory_type,
            tags,
            created_by,
            created_at,
            updated_at,
            owner,
            visibility,
            shared_with,
        })
    }
}

#[async_trait]
impl VectorStore for ChromaStore {
    async fn create(&self, request: CreateMemoryRequest) -> StoreResult<Memory> {
        let collection_id = self.ensure_collection().await?;
        let now = Utc::now();

        let memory = Memory {
            id: Memory::generate_id(),
            content: request.content,
            memory_type: request.memory_type,
            tags: request.tags,
            created_by: request.created_by,
            created_at: now,
            updated_at: now,
            owner: None,
            visibility: request.visibility,
            shared_with: request.shared_with,
        };

        let metadata = Self::memory_to_metadata(&memory);

        let url = format!("{}/api/v1/collections/{}/add", self.base_url, collection_id);
        let body = serde_json::json!({
            "ids": [&memory.id],
            "documents": [&memory.content],
            "metadatas": [metadata]
        });

        let resp = self.client.post(&url).json(&body).send().await?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            return Err(StoreError::QueryFailed(format!("Failed to create memory: {}", error)));
        }

        Ok(memory)
    }

    async fn get(&self, id: &str) -> StoreResult<Option<Memory>> {
        let collection_id = self.ensure_collection().await?;

        let url = format!("{}/api/v1/collections/{}/get", self.base_url, collection_id);
        let body = serde_json::json!({
            "ids": [id],
            "include": ["documents", "metadatas"]
        });

        let resp = self.client.post(&url).json(&body).send().await?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            return Err(StoreError::QueryFailed(format!("Failed to get memory: {}", error)));
        }

        let result: ChromaGetResult = resp.json().await?;

        if result.ids.is_empty() {
            return Ok(None);
        }

        let document = result.documents.first()
            .and_then(|d| d.as_ref())
            .ok_or_else(|| StoreError::InvalidData("Missing document".to_string()))?;
        let metadata = result.metadatas.first()
            .and_then(|m| m.as_ref())
            .ok_or_else(|| StoreError::InvalidData("Missing metadata".to_string()))?;

        let memory = Self::metadata_to_memory(document, metadata)?;
        Ok(Some(memory))
    }

    async fn delete(&self, id: &str) -> StoreResult<bool> {
        let collection_id = self.ensure_collection().await?;

        // First check if it exists
        let exists = self.get(id).await?.is_some();
        if !exists {
            return Ok(false);
        }

        let url = format!("{}/api/v1/collections/{}/delete", self.base_url, collection_id);
        let body = serde_json::json!({
            "ids": [id]
        });

        let resp = self.client.post(&url).json(&body).send().await?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            return Err(StoreError::QueryFailed(format!("Failed to delete memory: {}", error)));
        }

        Ok(true)
    }

    async fn search(&self, request: SearchRequest) -> StoreResult<Vec<Memory>> {
        let collection_id = self.ensure_collection().await?;

        let url = format!("{}/api/v1/collections/{}/query", self.base_url, collection_id);

        // Build where clause for filtering
        let mut where_clause: HashMap<String, serde_json::Value> = HashMap::new();

        if let Some(ref memory_type) = request.memory_type {
            where_clause.insert("type".to_string(), serde_json::json!(memory_type.to_string()));
        }

        let body = if where_clause.is_empty() {
            serde_json::json!({
                "query_texts": [&request.query],
                "n_results": request.limit,
                "include": ["documents", "metadatas", "distances"]
            })
        } else {
            serde_json::json!({
                "query_texts": [&request.query],
                "n_results": request.limit,
                "where": where_clause,
                "include": ["documents", "metadatas", "distances"]
            })
        };

        let resp = self.client.post(&url).json(&body).send().await?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            return Err(StoreError::QueryFailed(format!("Failed to search: {}", error)));
        }

        let result: ChromaQueryResult = resp.json().await?;

        let mut memories = Vec::new();

        if let (Some(docs), Some(metas)) = (
            result.documents.first(),
            result.metadatas.first(),
        ) {
            for (doc, meta) in docs.iter().zip(metas.iter()) {
                if let (Some(document), Some(metadata)) = (doc.as_ref(), meta.as_ref()) {
                    match Self::metadata_to_memory(document, metadata) {
                        Ok(memory) => {
                            // Filter by visibility
                            if memory.is_visible_to(request.as_actor.as_deref()) {
                                // Filter by tags if specified
                                if !request.tags.is_empty() {
                                    let has_tag = request.tags.iter()
                                        .any(|t| memory.tags.contains(t));
                                    if !has_tag {
                                        continue;
                                    }
                                }

                                // Filter by date range
                                if let Some(ref from) = request.from {
                                    if memory.created_at < *from {
                                        continue;
                                    }
                                }
                                if let Some(ref to) = request.to {
                                    if memory.created_at > *to {
                                        continue;
                                    }
                                }

                                memories.push(memory);
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse memory: {}", e);
                        }
                    }
                }
            }
        }

        Ok(memories)
    }

    async fn update_visibility(
        &self,
        id: &str,
        visibility: VisibilityLevel,
        shared_with: Option<Vec<String>>,
    ) -> StoreResult<Memory> {
        let collection_id = self.ensure_collection().await?;

        // Get existing memory
        let mut memory = self
            .get(id)
            .await?
            .ok_or_else(|| StoreError::NotFound(id.to_string()))?;

        // Update fields
        memory.visibility = visibility;
        if let Some(shared) = shared_with {
            memory.shared_with = shared;
        }
        memory.updated_at = Utc::now();

        // Update in ChromaDB
        let metadata = Self::memory_to_metadata(&memory);

        let url = format!("{}/api/v1/collections/{}/update", self.base_url, collection_id);
        let body = serde_json::json!({
            "ids": [id],
            "metadatas": [metadata]
        });

        let resp = self.client.post(&url).json(&body).send().await?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            return Err(StoreError::QueryFailed(format!(
                "Failed to update visibility: {}",
                error
            )));
        }

        Ok(memory)
    }

    async fn health_check(&self) -> StoreResult<bool> {
        let url = format!("{}/api/v1/heartbeat", self.base_url);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.status().is_success())
    }

    async fn initialize(&self) -> StoreResult<()> {
        self.ensure_collection().await?;
        Ok(())
    }
}

/// ChromaDB collection response structure.
#[derive(Debug, Deserialize)]
struct ChromaCollection {
    id: String,
    #[allow(dead_code)]
    name: String,
}

/// ChromaDB get response structure.
#[derive(Debug, Deserialize)]
struct ChromaGetResult {
    ids: Vec<String>,
    documents: Vec<Option<String>>,
    metadatas: Vec<Option<HashMap<String, serde_json::Value>>>,
}

/// ChromaDB query response structure.
#[derive(Debug, Deserialize)]
struct ChromaQueryResult {
    #[allow(dead_code)]
    ids: Vec<Vec<String>>,
    documents: Vec<Vec<Option<String>>>,
    metadatas: Vec<Vec<Option<HashMap<String, serde_json::Value>>>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_to_metadata_roundtrip() {
        let memory = Memory {
            id: "mem_123_abc".to_string(),
            content: "Test content".to_string(),
            memory_type: MemoryType::Question,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            created_by: "user1".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            owner: Some("owner1".to_string()),
            visibility: VisibilityLevel::Shared,
            shared_with: vec!["friend1".to_string()],
        };

        let metadata = ChromaStore::memory_to_metadata(&memory);
        let restored = ChromaStore::metadata_to_memory(&memory.content, &metadata).unwrap();

        assert_eq!(restored.id, memory.id);
        assert_eq!(restored.memory_type, memory.memory_type);
        assert_eq!(restored.tags, memory.tags);
        assert_eq!(restored.visibility, memory.visibility);
        assert_eq!(restored.owner, memory.owner);
    }
}
