//! VectorStore trait definition.

use async_trait::async_trait;

use crate::error::StoreResult;
use crate::types::{CreateMemoryRequest, Memory, SearchRequest, VisibilityLevel};

/// Trait for vector database storage backends.
///
/// Implementations provide persistence and semantic search capabilities
/// for memories. The primary implementation is ChromaStore for ChromaDB.
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Create a new memory from the given request.
    ///
    /// Returns the created memory with generated ID and timestamps.
    async fn create(&self, request: CreateMemoryRequest) -> StoreResult<Memory>;

    /// Get a memory by ID.
    ///
    /// Returns None if the memory doesn't exist.
    async fn get(&self, id: &str) -> StoreResult<Option<Memory>>;

    /// Delete a memory by ID.
    ///
    /// Returns true if a memory was deleted, false if it didn't exist.
    async fn delete(&self, id: &str) -> StoreResult<bool>;

    /// Search for memories matching the request.
    ///
    /// Results are filtered by visibility based on the as_actor field
    /// in the search request.
    async fn search(&self, request: SearchRequest) -> StoreResult<Vec<Memory>>;

    /// Update the visibility of a memory.
    ///
    /// Returns the updated memory.
    async fn update_visibility(
        &self,
        id: &str,
        visibility: VisibilityLevel,
        shared_with: Option<Vec<String>>,
    ) -> StoreResult<Memory>;

    /// Check if the store is healthy and accessible.
    async fn health_check(&self) -> StoreResult<bool>;

    /// Initialize the store (create collections, indexes, etc.).
    async fn initialize(&self) -> StoreResult<()>;

    /// List all memories in the store.
    ///
    /// Used for migration and backup purposes.
    async fn list_all(&self) -> StoreResult<Vec<Memory>>;

    /// Delete the collection.
    ///
    /// WARNING: This will delete all data in the collection.
    async fn delete_collection(&self) -> StoreResult<()>;
}
