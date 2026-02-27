//! Test fixtures and environment setup.
//!
//! Provides utilities for creating test configurations and managing
//! test environments with containers.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use tempfile::TempDir;
use tokio::net::TcpListener;

use berry::config::{ChromaConfig, Config, DocumentsConfig, EmbeddingConfig, LanceConfig, StoreBackend};
use berry::error::StoreResult;
use berry::store::{ChromaStore, EmbeddingService, VectorStore};

use crate::containers::{ChromaContainer, get_chroma_url, start_chroma};

/// Mock embedding service that returns consistent fake embeddings.
///
/// This allows testing without requiring real embedding models or API keys.
pub struct MockEmbedding {
    dimension: usize,
}

impl MockEmbedding {
    /// Create a new mock embedding service with the specified dimension.
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[async_trait]
impl EmbeddingService for MockEmbedding {
    async fn embed(&self, texts: &[String]) -> StoreResult<Vec<Vec<f32>>> {
        // Generate deterministic fake embeddings based on text content
        // Each character contributes to the embedding to make them somewhat unique
        Ok(texts
            .iter()
            .map(|text| {
                let mut embedding = vec![0.0f32; self.dimension];
                for (i, c) in text.chars().enumerate() {
                    let idx = i % self.dimension;
                    embedding[idx] += (c as u32 as f32) / 1000.0;
                }
                // Normalize the embedding
                let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                if magnitude > 0.0 {
                    embedding.iter_mut().for_each(|x| *x /= magnitude);
                }
                embedding
            })
            .collect())
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

/// A complete test environment with containers and configuration.
pub struct TestEnvironment {
    /// ChromaDB container (kept alive for the test duration)
    pub chroma_container: ChromaContainer,
    /// ChromaDB URL
    pub chroma_url: String,
    /// Temporary directory for config files
    pub temp_dir: TempDir,
    /// Test configuration
    pub config: Config,
    /// Vector store instance
    pub store: ChromaStore,
}

/// Default embedding dimension for tests.
const TEST_EMBEDDING_DIMENSION: usize = 384;

impl TestEnvironment {
    /// Create a new test environment with a running ChromaDB container.
    ///
    /// This starts a fresh ChromaDB instance and creates a configuration
    /// pointing to it. Uses a mock embedding service for fast, offline tests.
    pub async fn new() -> Self {
        // Start ChromaDB container
        let chroma_container = start_chroma().await;
        let chroma_url = get_chroma_url(&chroma_container).await;

        // Create temp directory for config
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Create test configuration
        let config = create_test_config(&chroma_url, temp_dir.path().to_path_buf());

        // Create vector store with mock embeddings for testing
        let embedding_service = Arc::new(MockEmbedding::new(TEST_EMBEDDING_DIMENSION));
        let store = ChromaStore::new(&config.chroma, embedding_service);

        // Initialize the store
        store
            .initialize()
            .await
            .expect("Failed to initialize store");

        Self {
            chroma_container,
            chroma_url,
            temp_dir,
            config,
            store,
        }
    }

    /// Create a new test environment with a unique collection name.
    ///
    /// Useful when running multiple tests in parallel that need isolated collections.
    pub async fn with_collection(collection_name: &str) -> Self {
        let chroma_container = start_chroma().await;
        let chroma_url = get_chroma_url(&chroma_container).await;

        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        let mut config = create_test_config(&chroma_url, temp_dir.path().to_path_buf());
        config.chroma.collection = collection_name.to_string();

        let embedding_service = Arc::new(MockEmbedding::new(TEST_EMBEDDING_DIMENSION));
        let store = ChromaStore::new(&config.chroma, embedding_service);
        store
            .initialize()
            .await
            .expect("Failed to initialize store");

        Self {
            chroma_container,
            chroma_url,
            temp_dir,
            config,
            store,
        }
    }

    /// Get the config file path for CLI tests.
    pub fn config_path(&self) -> PathBuf {
        self.temp_dir.path().join("berry.json")
    }

    /// Write the configuration to a file for CLI tests.
    pub fn write_config(&self) -> PathBuf {
        let config_path = self.config_path();
        let config_json =
            serde_json::to_string_pretty(&self.config).expect("Failed to serialize config");
        std::fs::write(&config_path, config_json).expect("Failed to write config file");
        config_path
    }
}

/// Create a test configuration pointing to the given ChromaDB URL.
pub fn create_test_config(chroma_url: &str, _config_dir: PathBuf) -> Config {
    Config {
        store: StoreBackend::Chroma,
        server: berry::config::ServerConfig {
            url: "http://localhost:4114".to_string(),
            timeout: 5000,
        },
        defaults: berry::config::DefaultsConfig::default(),
        chroma: ChromaConfig {
            url: chroma_url.to_string(),
            collection: format!("test_{}", uuid::Uuid::new_v4().to_string().replace('-', "")),
            provider: None,
            api_key: None,
            tenant: None,
            database: None,
        },
        lance: LanceConfig::default(),
        embedding: EmbeddingConfig {
            provider: "none".to_string(),
            api_key: None,
            model: String::new(),
            base_url: None,
        },
        documents: DocumentsConfig::default(),
    }
}

/// Find an available port for test servers.
pub async fn find_available_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to port");
    let addr = listener.local_addr().expect("Failed to get local address");
    addr.port()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_config() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config("http://localhost:8000", temp_dir.path().to_path_buf());

        assert_eq!(config.chroma.url, "http://localhost:8000");
        assert!(config.chroma.collection.starts_with("test_"));
        assert_eq!(config.embedding.provider, "none");
    }
}
