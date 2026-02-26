//! Configuration type definitions.

use serde::{Deserialize, Serialize};

use crate::types::{MemoryType, VisibilityLevel};

/// Which vector store backend to use.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StoreBackend {
    /// ChromaDB (requires running ChromaDB server)
    Chroma,
    /// LanceDB (embedded, no server required)
    #[default]
    Lance,
}

/// Root configuration structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Which vector store backend to use
    pub store: StoreBackend,
    /// Server connection configuration
    pub server: ServerConfig,
    /// Default values for memory operations
    pub defaults: DefaultsConfig,
    /// ChromaDB configuration
    pub chroma: ChromaConfig,
    /// LanceDB configuration
    pub lance: LanceConfig,
    /// Embedding service configuration
    pub embedding: EmbeddingConfig,
}

/// Server connection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// Base URL of the Berry server
    pub url: String,
    /// Request timeout in milliseconds
    pub timeout: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:4114".to_string(),
            timeout: 5000,
        }
    }
}

/// Default values for memory operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DefaultsConfig {
    /// Default memory type
    #[serde(rename = "type")]
    pub memory_type: MemoryType,
    /// Default creator ID
    #[serde(rename = "createdBy")]
    pub created_by: String,
    /// Default visibility level
    pub visibility: VisibilityLevel,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            memory_type: MemoryType::Information,
            created_by: "user".to_string(),
            visibility: VisibilityLevel::Public,
        }
    }
}

/// ChromaDB connection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ChromaConfig {
    /// ChromaDB server URL
    pub url: String,
    /// Collection name for memories
    pub collection: String,
    /// Authentication provider (if any): "token", "basic"
    pub provider: Option<String>,
    /// API key or token for authentication (if needed)
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
    /// Tenant name (for ChromaDB Cloud or multi-tenant setups)
    pub tenant: Option<String>,
    /// Database name (for ChromaDB Cloud or multi-tenant setups)
    pub database: Option<String>,
}

impl Default for ChromaConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:8000".to_string(),
            collection: "berry_memories".to_string(),
            provider: None,
            api_key: None,
            tenant: None,
            database: None,
        }
    }
}

/// LanceDB configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LanceConfig {
    /// Path to the LanceDB database directory
    pub path: String,
    /// Table name for memories
    pub table: String,
}

impl Default for LanceConfig {
    fn default() -> Self {
        let path = directories::ProjectDirs::from("", "", "berry")
            .map(|dirs| dirs.data_dir().join("lancedb").to_string_lossy().to_string())
            .unwrap_or_else(|| "lancedb".to_string());

        Self {
            path,
            table: "berry_memories".to_string(),
        }
    }
}

/// Embedding service configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EmbeddingConfig {
    /// Embedding provider: "openai", "local" (requires local-embeddings feature), or "none"
    pub provider: String,
    /// API key for the embedding service
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
    /// Model name for embeddings
    pub model: String,
    /// API base URL (optional, for custom endpoints)
    #[serde(rename = "baseUrl")]
    pub base_url: Option<String>,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            api_key: None,
            model: "text-embedding-3-small".to_string(),
            base_url: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        assert_eq!(config.store, StoreBackend::Lance);
        assert_eq!(config.server.url, "http://localhost:4114");
        assert_eq!(config.server.timeout, 5000);
        assert_eq!(config.defaults.memory_type, MemoryType::Information);
        assert_eq!(config.defaults.created_by, "user");
        assert_eq!(config.chroma.url, "http://localhost:8000");
        assert_eq!(config.chroma.collection, "berry_memories");
        assert_eq!(config.lance.table, "berry_memories");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("\"url\": \"http://localhost:4114\""));
        assert!(json.contains("\"timeout\": 5000"));
    }

    #[test]
    fn test_config_deserialization_with_defaults() {
        let json = r#"{
            "server": {
                "url": "http://custom:8080"
            }
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.server.url, "http://custom:8080");
        assert_eq!(config.server.timeout, 5000); // default
        assert_eq!(config.defaults.created_by, "user"); // default
    }

    #[test]
    fn test_store_backend_deserialization() {
        let json = r#"{ "store": "lance" }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.store, StoreBackend::Lance);

        let json = r#"{ "store": "chroma" }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.store, StoreBackend::Chroma);
    }

    #[test]
    fn test_lance_config_defaults() {
        let config = LanceConfig::default();
        assert_eq!(config.table, "berry_memories");
        assert!(!config.path.is_empty());
    }

    #[test]
    fn test_lance_config_deserialization() {
        let json = r#"{
            "lance": {
                "path": "/tmp/test-lance",
                "table": "custom_table"
            }
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.lance.path, "/tmp/test-lance");
        assert_eq!(config.lance.table, "custom_table");
    }
}
