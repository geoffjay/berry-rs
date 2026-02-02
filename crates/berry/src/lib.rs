//! Berry - A semantic memory system for AI assistants.
//!
//! This crate provides shared types, storage abstractions, and utilities
//! for the Berry memory system.
//!
//! # Modules
//!
//! - [`types`] - Core type definitions (Memory, MemoryType, VisibilityLevel, requests)
//! - [`store`] - Vector store abstraction and ChromaDB implementation
//! - [`config`] - Configuration loading from files and environment
//! - [`error`] - Error types for the system
//! - [`logging`] - Logging configuration utilities
//!
//! # Example
//!
//! ```no_run
//! use berry::{
//!     config::load_config,
//!     store::{ChromaStore, VectorStore},
//!     types::{CreateMemoryRequest, MemoryType},
//! };
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Load configuration
//!     let config = load_config()?;
//!
//!     // Create store
//!     let store = ChromaStore::new(&config.chroma);
//!     store.initialize().await?;
//!
//!     // Create a memory
//!     let request = CreateMemoryRequest {
//!         content: "Important information".to_string(),
//!         memory_type: MemoryType::Information,
//!         tags: vec!["important".to_string()],
//!         created_by: "user".to_string(),
//!         references: vec![],
//!         visibility: Default::default(),
//!         shared_with: vec![],
//!     };
//!
//!     let memory = store.create(request).await?;
//!     println!("Created memory: {}", memory.id);
//!
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod error;
pub mod logging;
pub mod store;
pub mod types;

/// Crate version from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Re-export commonly used types at the crate root for convenience.
pub use error::{BerryError, ConfigError, StoreError};
pub use types::{
    CreateMemoryRequest, DeleteResponse, HealthResponse, Memory, MemoryResponse, MemoryType,
    SearchRequest, SearchResponse, UpdateVisibilityRequest, VisibilityLevel,
};
