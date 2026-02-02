//! Error types for the Berry memory system.

use thiserror::Error;

/// Errors that can occur in the Berry system.
#[derive(Error, Debug)]
pub enum BerryError {
    /// Error from the vector store
    #[error("Store error: {0}")]
    Store(#[from] StoreError),

    /// Error loading configuration
    #[error("Config error: {0}")]
    Config(#[from] ConfigError),

    /// HTTP client error
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Generic error with message
    #[error("{0}")]
    Other(String),
}

/// Errors specific to the vector store operations.
#[derive(Error, Debug)]
pub enum StoreError {
    /// Memory not found
    #[error("Memory not found: {0}")]
    NotFound(String),

    /// Connection to database failed
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// Query execution failed
    #[error("Query failed: {0}")]
    QueryFailed(String),

    /// Invalid data or format
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// Database initialization failed
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// HTTP error from the store backend
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON error from the store backend
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Errors specific to configuration loading.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Config file not found
    #[error("Config file not found: {0}")]
    NotFound(String),

    /// Config file is invalid
    #[error("Invalid config: {0}")]
    Invalid(String),

    /// IO error reading config
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing error
    #[error("Parse error: {0}")]
    Parse(String),

    /// Environment variable error
    #[error("Environment error: {0}")]
    Environment(String),
}

/// Result type alias using BerryError.
pub type Result<T> = std::result::Result<T, BerryError>;

/// Result type alias for store operations.
pub type StoreResult<T> = std::result::Result<T, StoreError>;

/// Result type alias for config operations.
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_error_display() {
        let err = StoreError::NotFound("mem_123".to_string());
        assert_eq!(err.to_string(), "Memory not found: mem_123");
    }

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::Invalid("missing required field".to_string());
        assert_eq!(err.to_string(), "Invalid config: missing required field");
    }

    #[test]
    fn test_berry_error_from_store_error() {
        let store_err = StoreError::NotFound("test".to_string());
        let berry_err: BerryError = store_err.into();
        assert!(matches!(berry_err, BerryError::Store(_)));
    }
}
