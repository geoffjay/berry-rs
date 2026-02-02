//! ChromaDB vector store implementation.

use async_trait::async_trait;
use chrono::Utc;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

use super::embedding::EmbeddingService;
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
    /// Tenant name for multi-tenant ChromaDB (Cloud or self-hosted)
    tenant: Option<String>,
    /// Database name for multi-tenant ChromaDB (Cloud or self-hosted)
    database: Option<String>,
    /// Embedding service for generating vectors
    embedding_service: Arc<dyn EmbeddingService>,
}

impl ChromaStore {
    /// Create a new ChromaStore with the given configuration and embedding service.
    pub fn new(config: &ChromaConfig, embedding_service: Arc<dyn EmbeddingService>) -> Self {
        let client = Self::build_client(config);

        Self {
            client,
            base_url: config.url.trim_end_matches('/').to_string(),
            collection_name: config.collection.clone(),
            collection_id: None,
            tenant: config.tenant.clone(),
            database: config.database.clone(),
            embedding_service,
        }
    }

    /// Create a ChromaStore with a custom HTTP client.
    pub fn with_client(
        config: &ChromaConfig,
        client: Client,
        embedding_service: Arc<dyn EmbeddingService>,
    ) -> Self {
        Self {
            client,
            base_url: config.url.trim_end_matches('/').to_string(),
            collection_name: config.collection.clone(),
            collection_id: None,
            tenant: config.tenant.clone(),
            database: config.database.clone(),
            embedding_service,
        }
    }

    /// Build an HTTP client with appropriate authentication headers.
    fn build_client(config: &ChromaConfig) -> Client {
        let mut headers = HeaderMap::new();

        // Determine if this is a cloud configuration (has tenant and database)
        let is_cloud = config.tenant.is_some() && config.database.is_some();

        // Add authentication header based on provider
        if let Some(api_key) = &config.api_key {
            let masked_key = if api_key.len() > 8 {
                format!("{}...{}", &api_key[..4], &api_key[api_key.len()-4..])
            } else {
                "****".to_string()
            };

            let provider = config.provider.as_deref().unwrap_or("");

            match provider.to_lowercase().as_str() {
                "bearer" => {
                    // Explicit Bearer token authentication
                    tracing::debug!("Using Bearer token authentication (key: {})", masked_key);
                    if let Ok(value) = HeaderValue::from_str(&format!("Bearer {}", api_key)) {
                        headers.insert(AUTHORIZATION, value);
                    }
                }
                "basic" => {
                    // Basic authentication
                    tracing::debug!("Using Basic authentication (key: {})", masked_key);
                    if let Ok(value) = HeaderValue::from_str(&format!("Basic {}", api_key)) {
                        headers.insert(AUTHORIZATION, value);
                    }
                }
                "x-chroma-token" => {
                    // Explicit X-Chroma-Token header
                    tracing::debug!("Using X-Chroma-Token authentication (key: {})", masked_key);
                    if let Ok(value) = HeaderValue::from_str(api_key) {
                        headers.insert("X-Chroma-Token", value);
                    }
                }
                _ => {
                    // Default: use X-Chroma-Token for cloud, Bearer for local
                    if is_cloud {
                        tracing::debug!("Using X-Chroma-Token authentication for cloud (key: {})", masked_key);
                        if let Ok(value) = HeaderValue::from_str(api_key) {
                            headers.insert("X-Chroma-Token", value);
                        }
                    } else {
                        tracing::debug!("Using Bearer token authentication (key: {})", masked_key);
                        if let Ok(value) = HeaderValue::from_str(&format!("Bearer {}", api_key)) {
                            headers.insert(AUTHORIZATION, value);
                        }
                    }
                }
            }
        } else {
            tracing::debug!("No authentication configured (api_key not set)");
        }

        Client::builder()
            .default_headers(headers)
            .build()
            .unwrap_or_else(|_| Client::new())
    }

    /// Get the API base path for collections.
    ///
    /// Returns different paths depending on whether tenant/database are configured:
    /// - With tenant/database (Cloud): `/api/v2/tenants/{tenant}/databases/{database}/collections`
    /// - Without (local): `/api/v1/collections`
    fn collections_path(&self) -> String {
        match (&self.tenant, &self.database) {
            (Some(tenant), Some(database)) => {
                // ChromaDB Cloud uses v2 API
                format!(
                    "{}/api/v2/tenants/{}/databases/{}/collections",
                    self.base_url, tenant, database
                )
            }
            _ => {
                // Local ChromaDB uses v1 API
                format!("{}/api/v1/collections", self.base_url)
            }
        }
    }

    /// Get or create the collection and cache its ID.
    async fn ensure_collection(&self) -> StoreResult<String> {
        if let Some(ref id) = self.collection_id {
            return Ok(id.clone());
        }

        let collections_path = self.collections_path();

        // First, try to get collection by name directly
        let url = format!("{}/{}", collections_path, self.collection_name);
        tracing::debug!("Attempting to get collection at: {}", url);

        let resp = self.client.get(&url).send().await?;
        let status = resp.status();

        if status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            tracing::debug!("GET collection response: {}", &body[..body.len().min(500)]);
            let collection: ChromaCollection = serde_json::from_str(&body).map_err(|e| {
                StoreError::InvalidData(format!("Failed to parse collection: {}", e))
            })?;
            tracing::debug!("Found existing collection with ID: {}", collection.id);
            return Ok(collection.id);
        }

        let error_body = resp.text().await.unwrap_or_default();
        tracing::debug!(
            "GET collection returned HTTP {}: {}",
            status,
            if error_body.is_empty() { "(empty)" } else { &error_body[..error_body.len().min(200)] }
        );

        // If direct GET failed, try listing collections and finding by name
        // This works better with some ChromaDB configurations
        if let Ok(collection_id) = self.find_collection_in_list().await {
            return Ok(collection_id);
        }

        // Collection doesn't exist, try to create it
        tracing::debug!("Collection not found, attempting to create");

        let body = serde_json::json!({
            "name": self.collection_name,
            "metadata": {
                "description": "Berry memory storage"
            }
        });

        let resp = self.client.post(&collections_path).json(&body).send().await?;

        if resp.status().is_success() {
            let collection: ChromaCollection = resp.json().await?;
            tracing::info!("Created collection '{}' with ID: {}", self.collection_name, collection.id);
            return Ok(collection.id);
        }

        let create_status = resp.status();
        let error = resp.text().await.unwrap_or_default();

        // Provide helpful error message based on status code
        if create_status.as_u16() == 405 {
            return Err(StoreError::InitializationFailed(format!(
                "Collection '{}' not found and cannot be created via API (HTTP 405). \
                 For ChromaDB Cloud, please create the collection manually through the dashboard. \
                 Collections path: {}",
                self.collection_name, collections_path
            )));
        }

        Err(StoreError::InitializationFailed(format!(
            "Failed to create collection '{}' (HTTP {}): {}. Collections path: {}",
            self.collection_name, create_status, error, collections_path
        )))
    }

    /// Try to find the collection by listing all collections.
    async fn find_collection_in_list(&self) -> StoreResult<String> {
        let collections_path = self.collections_path();
        tracing::debug!("Listing collections at: {}", collections_path);

        let resp = self.client.get(&collections_path).send().await?;
        let status = resp.status();

        if !status.is_success() {
            let error = resp.text().await.unwrap_or_default();
            tracing::warn!(
                "Failed to list collections (HTTP {}): {}",
                status,
                if error.is_empty() { "(empty response)" } else { &error }
            );
            return Err(StoreError::QueryFailed(format!(
                "Failed to list collections: HTTP {} - {}",
                status, error
            )));
        }

        // Get the raw response text first for debugging
        let body = resp.text().await.unwrap_or_default();
        tracing::debug!("Collections list response: {}", &body[..body.len().min(500)]);

        // ChromaDB returns an array of collection objects
        let collections: Vec<ChromaCollection> = serde_json::from_str(&body).map_err(|e| {
            StoreError::InvalidData(format!(
                "Failed to parse collections list: {}. Response: {}",
                e,
                &body[..body.len().min(200)]
            ))
        })?;

        tracing::debug!("Found {} collections", collections.len());
        for collection in &collections {
            tracing::debug!("  - Collection: '{}' (ID: {})", collection.name, collection.id);
        }

        for collection in collections {
            if collection.name == self.collection_name {
                tracing::info!("Found collection '{}' with ID: {}", collection.name, collection.id);
                return Ok(collection.id);
            }
        }

        Err(StoreError::NotFound(format!(
            "Collection '{}' not found in list of collections",
            self.collection_name
        )))
    }

    /// Convert a Memory to ChromaDB metadata format.
    fn memory_to_metadata(memory: &Memory) -> HashMap<String, serde_json::Value> {
        let mut metadata = HashMap::new();
        metadata.insert("id".to_string(), serde_json::json!(memory.id));
        metadata.insert(
            "type".to_string(),
            serde_json::json!(memory.memory_type.to_string()),
        );
        metadata.insert("tags".to_string(), serde_json::json!(memory.tags.join(",")));
        metadata.insert(
            "created_by".to_string(),
            serde_json::json!(memory.created_by),
        );
        metadata.insert(
            "created_at".to_string(),
            serde_json::json!(memory.created_at.to_rfc3339()),
        );
        metadata.insert(
            "updated_at".to_string(),
            serde_json::json!(memory.updated_at.to_rfc3339()),
        );
        metadata.insert(
            "visibility".to_string(),
            serde_json::json!(memory.visibility.to_string()),
        );
        metadata.insert(
            "shared_with".to_string(),
            serde_json::json!(memory.shared_with.join(",")),
        );

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
            .map_err(StoreError::InvalidData)?;
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
            .map_err(StoreError::InvalidData)?;
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

        // Generate embedding for the content
        let embeddings = self
            .embedding_service
            .embed(&[memory.content.clone()])
            .await?;

        let embedding = embeddings.into_iter().next().ok_or_else(|| {
            StoreError::QueryFailed("Failed to generate embedding for content".to_string())
        })?;

        let metadata = Self::memory_to_metadata(&memory);

        let url = format!("{}/{}/add", self.collections_path(), collection_id);
        let body = serde_json::json!({
            "ids": [&memory.id],
            "documents": [&memory.content],
            "embeddings": [embedding],
            "metadatas": [metadata]
        });

        let resp = self.client.post(&url).json(&body).send().await?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            return Err(StoreError::QueryFailed(format!(
                "Failed to create memory: {}",
                error
            )));
        }

        Ok(memory)
    }

    async fn get(&self, id: &str) -> StoreResult<Option<Memory>> {
        let collection_id = self.ensure_collection().await?;

        let url = format!("{}/{}/get", self.collections_path(), collection_id);
        let body = serde_json::json!({
            "ids": [id],
            "include": ["documents", "metadatas"]
        });

        let resp = self.client.post(&url).json(&body).send().await?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            return Err(StoreError::QueryFailed(format!(
                "Failed to get memory: {}",
                error
            )));
        }

        let result: ChromaGetResult = resp.json().await?;

        if result.ids.is_empty() {
            return Ok(None);
        }

        let document = result
            .documents
            .first()
            .and_then(|d| d.as_ref())
            .ok_or_else(|| StoreError::InvalidData("Missing document".to_string()))?;
        let metadata = result
            .metadatas
            .first()
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

        let url = format!("{}/{}/delete", self.collections_path(), collection_id);
        let body = serde_json::json!({
            "ids": [id]
        });

        let resp = self.client.post(&url).json(&body).send().await?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            return Err(StoreError::QueryFailed(format!(
                "Failed to delete memory: {}",
                error
            )));
        }

        Ok(true)
    }

    async fn search(&self, request: SearchRequest) -> StoreResult<Vec<Memory>> {
        let collection_id = self.ensure_collection().await?;

        // Generate embedding for the query
        let embeddings = self
            .embedding_service
            .embed(&[request.query.clone()])
            .await?;

        let query_embedding = embeddings.into_iter().next().ok_or_else(|| {
            StoreError::QueryFailed("Failed to generate embedding for query".to_string())
        })?;

        let url = format!("{}/{}/query", self.collections_path(), collection_id);

        // Build where clause for filtering
        let mut where_clause: HashMap<String, serde_json::Value> = HashMap::new();

        if let Some(ref memory_type) = request.memory_type {
            where_clause.insert(
                "type".to_string(),
                serde_json::json!(memory_type.to_string()),
            );
        }

        let body = if where_clause.is_empty() {
            serde_json::json!({
                "query_embeddings": [query_embedding],
                "n_results": request.limit,
                "include": ["documents", "metadatas", "distances"]
            })
        } else {
            serde_json::json!({
                "query_embeddings": [query_embedding],
                "n_results": request.limit,
                "where": where_clause,
                "include": ["documents", "metadatas", "distances"]
            })
        };

        let resp = self.client.post(&url).json(&body).send().await?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            return Err(StoreError::QueryFailed(format!(
                "Failed to search: {}",
                error
            )));
        }

        let result: ChromaQueryResult = resp.json().await?;

        let mut memories = Vec::new();

        if let (Some(docs), Some(metas)) = (result.documents.first(), result.metadatas.first()) {
            for (doc, meta) in docs.iter().zip(metas.iter()) {
                if let (Some(document), Some(metadata)) = (doc.as_ref(), meta.as_ref()) {
                    match Self::metadata_to_memory(document, metadata) {
                        Ok(memory) => {
                            // Filter by visibility
                            if memory.is_visible_to(request.as_actor.as_deref()) {
                                // Filter by tags if specified
                                if !request.tags.is_empty() {
                                    let has_tag = request.tags.iter().any(|t| memory.tags.contains(t));
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

        let url = format!("{}/{}/update", self.collections_path(), collection_id);
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
    use crate::store::embedding::NoOpEmbedding;

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

    #[test]
    fn test_collections_path_without_tenant() {
        let config = ChromaConfig {
            url: "http://localhost:8000".to_string(),
            collection: "test".to_string(),
            provider: None,
            api_key: None,
            tenant: None,
            database: None,
        };
        let embedding_service = Arc::new(NoOpEmbedding::new());
        let store = ChromaStore::new(&config, embedding_service);
        assert_eq!(
            store.collections_path(),
            "http://localhost:8000/api/v1/collections"
        );
    }

    #[test]
    fn test_collections_path_with_tenant() {
        let config = ChromaConfig {
            url: "https://api.trychroma.com".to_string(),
            collection: "test".to_string(),
            provider: Some("token".to_string()),
            api_key: Some("test-key".to_string()),
            tenant: Some("my-tenant".to_string()),
            database: Some("my-database".to_string()),
        };
        let embedding_service = Arc::new(NoOpEmbedding::new());
        let store = ChromaStore::new(&config, embedding_service);
        // ChromaDB Cloud uses v2 API
        assert_eq!(
            store.collections_path(),
            "https://api.trychroma.com/api/v2/tenants/my-tenant/databases/my-database/collections"
        );
    }
}
