//! DocumentStore trait definition.

use async_trait::async_trait;

use crate::error::StoreResult;
use crate::types::{CreateDocumentRequest, Document, ListDocumentsRequest, UpdateDocumentRequest};

/// Trait for document storage backends.
///
/// Implementations provide CRUD operations for markdown documents
/// stored on the filesystem with sidecar metadata.
#[async_trait]
pub trait DocumentStore: Send + Sync {
    /// Create a new document from the given request.
    ///
    /// Returns the created document with generated slug ID and timestamps.
    async fn create(&self, request: CreateDocumentRequest) -> StoreResult<Document>;

    /// Get a document by ID.
    ///
    /// Returns None if the document doesn't exist.
    async fn get(&self, id: &str) -> StoreResult<Option<Document>>;

    /// Update an existing document.
    ///
    /// Returns the updated document.
    async fn update(&self, id: &str, request: UpdateDocumentRequest) -> StoreResult<Document>;

    /// Delete a document by ID.
    ///
    /// Returns true if a document was deleted, false if it didn't exist.
    async fn delete(&self, id: &str) -> StoreResult<bool>;

    /// List documents with optional filters.
    async fn list(&self, request: ListDocumentsRequest) -> StoreResult<Vec<Document>>;

    /// Initialize the store (create directories, etc.).
    async fn initialize(&self) -> StoreResult<()>;
}
