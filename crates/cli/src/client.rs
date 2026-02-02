//! HTTP client for communicating with the Berry server.

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;

use berry::{
    CreateMemoryRequest, DeleteResponse, HealthResponse, Memory, MemoryResponse, SearchRequest,
    SearchResponse, UpdateVisibilityRequest, VisibilityLevel,
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

        resp.json()
            .await
            .context("Failed to parse health response")
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
}
