//! HTTP client for communicating with the Berry server.

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;

use berry::{
    CreateDocumentRequest, CreateMemoryRequest, DeleteResponse, Document, DocumentListResponse,
    DocumentResponse, HealthResponse, ListDocumentsRequest, Memory, MemoryResponse, SearchRequest,
    SearchResponse, UpdateDocumentRequest, UpdateVisibilityRequest, VisibilityLevel,
};

/// HTTP client for the Berry server.
pub struct BerryClient {
    client: Client,
    base_url: String,
}

impl BerryClient {
    /// Create a new client with the given base URL and timeout.
    pub fn new(base_url: &str, timeout_ms: u64) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    /// Check server health.
    #[allow(dead_code)]
    pub async fn health(&self) -> Result<HealthResponse> {
        let url = format!("{}/health", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to server")?;

        if !resp.status().is_success() {
            anyhow::bail!("Server health check failed: {}", resp.status());
        }

        resp.json().await.context("Failed to parse health response")
    }

    /// Create a new memory.
    pub async fn create_memory(&self, request: CreateMemoryRequest) -> Result<Memory> {
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

    /// Get a memory by ID.
    pub async fn get_memory(&self, id: &str) -> Result<Option<Memory>> {
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
        Ok(response.memory)
    }

    /// Delete a memory by ID.
    pub async fn delete_memory(&self, id: &str) -> Result<bool> {
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
    pub async fn search(&self, request: SearchRequest) -> Result<Vec<Memory>> {
        let url = format!("{}/v1/search", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to search memories")?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to search: {}", error);
        }

        let response: SearchResponse = resp.json().await.context("Failed to parse response")?;
        Ok(response.memories)
    }

    /// Update memory visibility.
    #[allow(dead_code)]
    pub async fn update_visibility(
        &self,
        id: &str,
        visibility: VisibilityLevel,
        shared_with: Option<Vec<String>>,
    ) -> Result<Memory> {
        let url = format!("{}/v1/memory/{}/visibility", self.base_url, id);
        let request = UpdateVisibilityRequest {
            visibility,
            shared_with,
        };

        let resp = self
            .client
            .patch(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to update visibility")?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to update visibility: {}", error);
        }

        let response: MemoryResponse = resp.json().await.context("Failed to parse response")?;

        response
            .memory
            .ok_or_else(|| anyhow::anyhow!("No memory in response"))
    }

    /// Create a new document.
    pub async fn create_document(&self, request: CreateDocumentRequest) -> Result<Document> {
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
    pub async fn get_document(&self, id: &str) -> Result<Option<Document>> {
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
    pub async fn update_document(
        &self,
        id: &str,
        request: UpdateDocumentRequest,
    ) -> Result<Document> {
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
    pub async fn delete_document(&self, id: &str) -> Result<bool> {
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
    pub async fn list_documents(&self, request: ListDocumentsRequest) -> Result<Vec<Document>> {
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
