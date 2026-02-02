//! Configuration type definitions.

use serde::{Deserialize, Serialize};

use crate::types::{MemoryType, VisibilityLevel};

/// Root configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Server connection configuration
    pub server: ServerConfig,
    /// Default values for memory operations
    pub defaults: DefaultsConfig,
    /// ChromaDB configuration
    pub chroma: ChromaConfig,
    /// Embedding service configuration
    pub embedding: EmbeddingConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            defaults: DefaultsConfig::default(),
            chroma: ChromaConfig::default(),
            embedding: EmbeddingConfig::default(),
        }
    }
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

/// Embedding service configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EmbeddingConfig {
    /// Embedding provider: "openai", "cohere", or "none"
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
        assert_eq!(config.server.url, "http://localhost:4114");
        assert_eq!(config.server.timeout, 5000);
        assert_eq!(config.defaults.memory_type, MemoryType::Information);
        assert_eq!(config.defaults.created_by, "user");
        assert_eq!(config.chroma.url, "http://localhost:8000");
        assert_eq!(config.chroma.collection, "berry_memories");
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
}
