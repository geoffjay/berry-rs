//! Embedding service implementations.
//!
//! Provides text-to-vector embedding functionality for semantic search.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::EmbeddingConfig;
use crate::error::{StoreError, StoreResult};

/// Trait for embedding services.
#[async_trait]
pub trait EmbeddingService: Send + Sync {
    /// Generate embeddings for the given texts.
    async fn embed(&self, texts: &[String]) -> StoreResult<Vec<Vec<f32>>>;

    /// Get the embedding dimension for this model.
    fn dimension(&self) -> usize;
}

/// OpenAI embedding service.
pub struct OpenAIEmbedding {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenAIEmbedding {
    /// Create a new OpenAI embedding service.
    ///
    /// API key is required for remote services (OpenAI, etc.) but optional
    /// for local services like Ollama (localhost URLs).
    pub fn new(config: &EmbeddingConfig) -> StoreResult<Self> {
        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());

        // Check if this is a local service (Ollama, etc.)
        let is_local = base_url.contains("localhost") || base_url.contains("127.0.0.1");

        let api_key = match config.api_key.clone() {
            Some(key) => key,
            None if is_local => String::new(), // No key needed for local services
            None => {
                return Err(StoreError::InitializationFailed(
                    "OpenAI API key required for embeddings. Set OPENAI_API_KEY or EMBEDDING_API_KEY"
                        .to_string(),
                ))
            }
        };

        Ok(Self {
            client: Client::new(),
            api_key,
            model: config.model.clone(),
            base_url,
        })
    }

    /// Get the dimension for the configured model.
    fn model_dimension(&self) -> usize {
        match self.model.as_str() {
            // OpenAI models
            "text-embedding-3-small" => 1536,
            "text-embedding-3-large" => 3072,
            "text-embedding-ada-002" => 1536,
            // Ollama / open source models
            "nomic-embed-text" => 768,
            "mxbai-embed-large" => 1024,
            "all-minilm" => 384,
            "snowflake-arctic-embed" => 1024,
            _ => 1536, // Default (OpenAI compatible)
        }
    }
}

#[derive(Debug, Serialize)]
struct OpenAIEmbeddingRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingResponse {
    data: Vec<OpenAIEmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingData {
    embedding: Vec<f32>,
}

#[async_trait]
impl EmbeddingService for OpenAIEmbedding {
    async fn embed(&self, texts: &[String]) -> StoreResult<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let url = format!("{}/embeddings", self.base_url);

        let request = OpenAIEmbeddingRequest {
            model: self.model.clone(),
            input: texts.to_vec(),
        };

        tracing::debug!(
            "Generating embeddings for {} texts using model {} at {}",
            texts.len(),
            self.model,
            self.base_url
        );

        // Build request, only adding Authorization header if we have an API key
        let mut req = self
            .client
            .post(&url)
            .header("Content-Type", "application/json");

        if !self.api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let resp = req
            .json(&request)
            .send()
            .await
            .map_err(|e| StoreError::QueryFailed(format!("Embedding request failed: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let error = resp.text().await.unwrap_or_default();
            return Err(StoreError::QueryFailed(format!(
                "Embedding API error (HTTP {}): {}",
                status, error
            )));
        }

        let response: OpenAIEmbeddingResponse = resp.json().await.map_err(|e| {
            StoreError::InvalidData(format!("Failed to parse embedding response: {}", e))
        })?;

        let embeddings: Vec<Vec<f32>> = response.data.into_iter().map(|d| d.embedding).collect();

        tracing::debug!(
            "Generated {} embeddings with dimension {}",
            embeddings.len(),
            embeddings.first().map(|e| e.len()).unwrap_or(0)
        );

        Ok(embeddings)
    }

    fn dimension(&self) -> usize {
        self.model_dimension()
    }
}

/// No-op embedding service for when embeddings are disabled.
pub struct NoOpEmbedding;

impl NoOpEmbedding {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoOpEmbedding {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EmbeddingService for NoOpEmbedding {
    async fn embed(&self, _texts: &[String]) -> StoreResult<Vec<Vec<f32>>> {
        Err(StoreError::InitializationFailed(
            "Embedding service not configured. Set OPENAI_API_KEY or configure embedding provider"
                .to_string(),
        ))
    }

    fn dimension(&self) -> usize {
        0
    }
}

/// Create an embedding service from configuration.
pub fn create_embedding_service(
    config: &EmbeddingConfig,
) -> StoreResult<Box<dyn EmbeddingService>> {
    match config.provider.to_lowercase().as_str() {
        "openai" => {
            let service = OpenAIEmbedding::new(config)?;
            Ok(Box::new(service))
        }
        "none" | "" => {
            tracing::warn!("No embedding provider configured. Semantic search will not work.");
            Ok(Box::new(NoOpEmbedding::new()))
        }
        provider => Err(StoreError::InitializationFailed(format!(
            "Unknown embedding provider: {}. Supported: openai, none",
            provider
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_dimensions() {
        let config = EmbeddingConfig {
            provider: "openai".to_string(),
            api_key: Some("test".to_string()),
            model: "text-embedding-3-small".to_string(),
            base_url: None,
        };
        let service = OpenAIEmbedding::new(&config).unwrap();
        assert_eq!(service.dimension(), 1536);
    }

    #[test]
    fn test_noop_embedding() {
        let service = NoOpEmbedding::new();
        assert_eq!(service.dimension(), 0);
    }
}
