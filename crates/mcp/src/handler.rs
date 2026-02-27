//! MCP server handler implementation.

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;

use berry::{
    CreateDocumentRequest, CreateMemoryRequest, DeleteResponse, Document, DocumentListResponse,
    DocumentResponse, ListDocumentsRequest, Memory, MemoryResponse, MemoryType, SearchRequest,
    SearchResponse, UpdateDocumentRequest, VisibilityLevel,
};

/// Berry MCP client that communicates with the Berry HTTP server.
pub struct BerryMcpClient {
    client: Client,
    base_url: String,
}

impl BerryMcpClient {
    /// Create a new MCP client.
    pub fn new(server_url: &str) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url: server_url.trim_end_matches('/').to_string(),
        })
    }

    /// Remember - create a new memory.
    pub async fn remember(
        &self,
        content: String,
        memory_type: Option<MemoryType>,
        tags: Option<Vec<String>>,
        created_by: String,
        visibility: Option<VisibilityLevel>,
        shared_with: Option<Vec<String>>,
    ) -> Result<Memory> {
        let request = CreateMemoryRequest {
            content,
            memory_type: memory_type.unwrap_or_default(),
            tags: tags.unwrap_or_default(),
            created_by,
            references: vec![],
            visibility: visibility.unwrap_or_default(),
            shared_with: shared_with.unwrap_or_default(),
        };

        let url = format!("{}/v1/memory", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to create memory")?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to create memory: {}", error);
        }

        let response: MemoryResponse = resp.json().await.context("Failed to parse response")?;
        response
            .memory
            .ok_or_else(|| anyhow::anyhow!("No memory in response"))
    }

    /// Recall - get a memory by ID.
    pub async fn recall(&self, id: &str, as_actor: Option<&str>) -> Result<Option<Memory>> {
        let url = format!("{}/v1/memory/{}", self.base_url, id);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to get memory")?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to get memory: {}", error);
        }

        let response: MemoryResponse = resp.json().await.context("Failed to parse response")?;
        let memory = response.memory;

        // Filter by visibility if as_actor is provided
        if let (Some(mem), Some(actor)) = (&memory, as_actor)
            && !mem.is_visible_to(Some(actor))
        {
            return Ok(None);
        }

        Ok(memory)
    }

    /// Forget - delete a memory by ID.
    pub async fn forget(&self, id: &str, as_actor: Option<&str>) -> Result<bool> {
        // First check visibility if as_actor is provided
        if let Some(actor) = as_actor {
            if let Some(memory) = self.recall(id, Some(actor)).await? {
                if memory.created_by != actor && memory.owner.as_deref() != Some(actor) {
                    anyhow::bail!(
                        "Permission denied: only the creator or owner can delete this memory"
                    );
                }
            } else {
                return Ok(false); // Not found or not visible
            }
        }

        let url = format!("{}/v1/memory/{}", self.base_url, id);
        let resp = self
            .client
            .delete(&url)
            .send()
            .await
            .context("Failed to delete memory")?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to delete memory: {}", error);
        }

        let response: DeleteResponse = resp.json().await.context("Failed to parse response")?;
        Ok(response.deleted)
    }

    /// Search for memories.
    #[allow(clippy::too_many_arguments)]
    pub async fn search(
        &self,
        query: String,
        as_actor: Option<String>,
        memory_type: Option<MemoryType>,
        tags: Option<Vec<String>>,
        limit: Option<usize>,
        from: Option<String>,
        to: Option<String>,
    ) -> Result<Vec<Memory>> {
        let request = SearchRequest {
            query,
            as_actor,
            memory_type,
            tags: tags.unwrap_or_default(),
            limit: limit.unwrap_or(10),
            from: from
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
            to: to
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
        };

        let url = format!("{}/v1/search", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to search")?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to search: {}", error);
        }

        let response: SearchResponse = resp.json().await.context("Failed to parse response")?;
        Ok(response.memories)
    }

    /// Create a new document.
    pub async fn doc_create(&self, request: CreateDocumentRequest) -> Result<Document> {
        let url = format!("{}/v1/documents", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to create document")?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to create document: {}", error);
        }

        let response: DocumentResponse = resp.json().await.context("Failed to parse response")?;
        response
            .document
            .ok_or_else(|| anyhow::anyhow!("No document in response"))
    }

    /// Get a document by ID.
    pub async fn doc_read(&self, id: &str) -> Result<Option<Document>> {
        let url = format!("{}/v1/documents/{}", self.base_url, id);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to get document")?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to get document: {}", error);
        }

        let response: DocumentResponse = resp.json().await.context("Failed to parse response")?;
        Ok(response.document)
    }

    /// Update a document.
    pub async fn doc_update(&self, id: &str, request: UpdateDocumentRequest) -> Result<Document> {
        let url = format!("{}/v1/documents/{}", self.base_url, id);
        let resp = self
            .client
            .patch(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to update document")?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to update document: {}", error);
        }

        let response: DocumentResponse = resp.json().await.context("Failed to parse response")?;
        response
            .document
            .ok_or_else(|| anyhow::anyhow!("No document in response"))
    }

    /// Delete a document by ID.
    pub async fn doc_delete(&self, id: &str) -> Result<bool> {
        let url = format!("{}/v1/documents/{}", self.base_url, id);
        let resp = self
            .client
            .delete(&url)
            .send()
            .await
            .context("Failed to delete document")?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to delete document: {}", error);
        }

        let response: DeleteResponse = resp.json().await.context("Failed to parse response")?;
        Ok(response.deleted)
    }

    /// List documents with optional filters.
    pub async fn doc_list(&self, request: ListDocumentsRequest) -> Result<Vec<Document>> {
        let mut url = format!("{}/v1/documents", self.base_url);
        let mut params = Vec::new();

        if let Some(ref tags) = request.tags {
            for tag in tags {
                params.push(format!("tags={}", tag));
            }
        }
        if let Some(ref created_by) = request.created_by {
            params.push(format!("created_by={}", created_by));
        }
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to list documents")?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to list documents: {}", error);
        }

        let response: DocumentListResponse =
            resp.json().await.context("Failed to parse response")?;
        Ok(response.documents)
    }
}
