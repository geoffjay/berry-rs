//! Filesystem-based document store implementation.

use std::path::PathBuf;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::error::{StoreError, StoreResult};
use crate::types::{CreateDocumentRequest, Document, ListDocumentsRequest, UpdateDocumentRequest};

use super::traits::DocumentStore;

/// Sidecar metadata stored as JSON alongside document files.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DocumentMetadata {
    title: String,
    tags: Vec<String>,
    created_by: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Filesystem-based document store.
///
/// Documents are stored as `<path>/<id>.md` with metadata in `<path>/.berry/<id>.json`.
pub struct FsDocumentStore {
    path: PathBuf,
}

impl FsDocumentStore {
    /// Create a new filesystem document store at the given path.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Get the path to a document's markdown file.
    fn doc_path(&self, id: &str) -> PathBuf {
        self.path.join(format!("{}.md", id))
    }

    /// Get the path to the `.berry` metadata directory.
    fn meta_dir(&self) -> PathBuf {
        self.path.join(".berry")
    }

    /// Get the path to a document's metadata file.
    fn meta_path(&self, id: &str) -> PathBuf {
        self.meta_dir().join(format!("{}.json", id))
    }

    /// Generate a unique slug, appending `-2`, `-3`, etc. if needed.
    async fn unique_slug(&self, base_slug: &str) -> String {
        if !self.doc_path(base_slug).exists() {
            return base_slug.to_string();
        }

        let mut counter = 2;
        loop {
            let candidate = format!("{}-{}", base_slug, counter);
            if !self.doc_path(&candidate).exists() {
                return candidate;
            }
            counter += 1;
        }
    }

    /// Read metadata from the sidecar JSON file.
    async fn read_metadata(&self, id: &str) -> StoreResult<DocumentMetadata> {
        let meta_path = self.meta_path(id);
        let content = fs::read_to_string(&meta_path).await.map_err(|e| {
            StoreError::QueryFailed(format!("Failed to read metadata for {}: {}", id, e))
        })?;
        serde_json::from_str(&content).map_err(|e| {
            StoreError::InvalidData(format!("Failed to parse metadata for {}: {}", id, e))
        })
    }

    /// Write metadata to the sidecar JSON file.
    async fn write_metadata(&self, id: &str, metadata: &DocumentMetadata) -> StoreResult<()> {
        let meta_path = self.meta_path(id);
        let content = serde_json::to_string_pretty(metadata).map_err(|e| {
            StoreError::InvalidData(format!("Failed to serialize metadata for {}: {}", id, e))
        })?;
        fs::write(&meta_path, content).await.map_err(|e| {
            StoreError::QueryFailed(format!("Failed to write metadata for {}: {}", id, e))
        })
    }

    /// Read a full document (content + metadata) by ID.
    async fn read_document(&self, id: &str) -> StoreResult<Document> {
        let doc_path = self.doc_path(id);
        let content = fs::read_to_string(&doc_path).await.map_err(|e| {
            StoreError::QueryFailed(format!("Failed to read document {}: {}", id, e))
        })?;
        let metadata = self.read_metadata(id).await?;

        Ok(Document {
            id: id.to_string(),
            title: metadata.title,
            content,
            tags: metadata.tags,
            created_by: metadata.created_by,
            created_at: metadata.created_at,
            updated_at: metadata.updated_at,
        })
    }
}

#[async_trait]
impl DocumentStore for FsDocumentStore {
    async fn create(&self, request: CreateDocumentRequest) -> StoreResult<Document> {
        let base_slug = Document::slugify(&request.title);
        if base_slug.is_empty() {
            return Err(StoreError::InvalidData(
                "Title must contain at least one alphanumeric character".to_string(),
            ));
        }

        let id = self.unique_slug(&base_slug).await;
        let now = Utc::now();

        // Write markdown content
        fs::write(self.doc_path(&id), &request.content)
            .await
            .map_err(|e| {
                StoreError::QueryFailed(format!("Failed to write document {}: {}", id, e))
            })?;

        // Write metadata
        let metadata = DocumentMetadata {
            title: request.title.clone(),
            tags: request.tags.clone(),
            created_by: request.created_by.clone(),
            created_at: now,
            updated_at: now,
        };
        self.write_metadata(&id, &metadata).await?;

        Ok(Document {
            id,
            title: request.title,
            content: request.content,
            tags: request.tags,
            created_by: request.created_by,
            created_at: now,
            updated_at: now,
        })
    }

    async fn get(&self, id: &str) -> StoreResult<Option<Document>> {
        if !self.doc_path(id).exists() {
            return Ok(None);
        }
        self.read_document(id).await.map(Some)
    }

    async fn update(&self, id: &str, request: UpdateDocumentRequest) -> StoreResult<Document> {
        if !self.doc_path(id).exists() {
            return Err(StoreError::NotFound(format!("Document not found: {}", id)));
        }

        let mut doc = self.read_document(id).await?;
        let now = Utc::now();

        if let Some(title) = request.title {
            doc.title = title;
        }
        if let Some(content) = request.content {
            doc.content = content;
        }
        if let Some(tags) = request.tags {
            doc.tags = tags;
        }
        doc.updated_at = now;

        // Write updated content
        fs::write(self.doc_path(id), &doc.content)
            .await
            .map_err(|e| {
                StoreError::QueryFailed(format!("Failed to write document {}: {}", id, e))
            })?;

        // Write updated metadata
        let metadata = DocumentMetadata {
            title: doc.title.clone(),
            tags: doc.tags.clone(),
            created_by: doc.created_by.clone(),
            created_at: doc.created_at,
            updated_at: now,
        };
        self.write_metadata(id, &metadata).await?;

        Ok(doc)
    }

    async fn delete(&self, id: &str) -> StoreResult<bool> {
        let doc_path = self.doc_path(id);
        if !doc_path.exists() {
            return Ok(false);
        }

        // Remove document file
        fs::remove_file(&doc_path).await.map_err(|e| {
            StoreError::QueryFailed(format!("Failed to delete document {}: {}", id, e))
        })?;

        // Remove metadata file (best-effort)
        let meta_path = self.meta_path(id);
        if meta_path.exists() {
            let _ = fs::remove_file(&meta_path).await;
        }

        Ok(true)
    }

    async fn list(&self, request: ListDocumentsRequest) -> StoreResult<Vec<Document>> {
        let meta_dir = self.meta_dir();
        if !meta_dir.exists() {
            return Ok(vec![]);
        }

        let mut documents = Vec::new();
        let mut entries = fs::read_dir(&meta_dir).await.map_err(|e| {
            StoreError::QueryFailed(format!("Failed to read documents directory: {}", e))
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            StoreError::QueryFailed(format!("Failed to read directory entry: {}", e))
        })? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            let id = match path.file_stem().and_then(|s| s.to_str()) {
                Some(id) => id.to_string(),
                None => continue,
            };

            // Verify the markdown file also exists
            if !self.doc_path(&id).exists() {
                continue;
            }

            let doc = match self.read_document(&id).await {
                Ok(doc) => doc,
                Err(_) => continue,
            };

            // Apply filters
            if let Some(ref filter_tags) = request.tags
                && !filter_tags.iter().any(|t| doc.tags.contains(t))
            {
                continue;
            }
            if let Some(ref created_by) = request.created_by
                && doc.created_by != *created_by
            {
                continue;
            }

            documents.push(doc);
        }

        // Sort by updated_at descending
        documents.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(documents)
    }

    async fn initialize(&self) -> StoreResult<()> {
        fs::create_dir_all(&self.path).await.map_err(|e| {
            StoreError::InitializationFailed(format!("Failed to create documents directory: {}", e))
        })?;

        fs::create_dir_all(self.meta_dir()).await.map_err(|e| {
            StoreError::InitializationFailed(format!("Failed to create metadata directory: {}", e))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup() -> (TempDir, FsDocumentStore) {
        let temp_dir = TempDir::new().unwrap();
        let store = FsDocumentStore::new(temp_dir.path().to_path_buf());
        store.initialize().await.unwrap();
        (temp_dir, store)
    }

    fn create_request(title: &str, content: &str) -> CreateDocumentRequest {
        CreateDocumentRequest {
            title: title.to_string(),
            content: content.to_string(),
            tags: vec!["test".to_string()],
            created_by: "user".to_string(),
        }
    }

    #[tokio::test]
    async fn test_initialize_creates_directories() {
        let temp_dir = TempDir::new().unwrap();
        let store = FsDocumentStore::new(temp_dir.path().join("docs"));
        store.initialize().await.unwrap();

        assert!(temp_dir.path().join("docs").exists());
        assert!(temp_dir.path().join("docs/.berry").exists());
    }

    #[tokio::test]
    async fn test_create_document() {
        let (_dir, store) = setup().await;
        let doc = store
            .create(create_request("Hello World", "# Hello"))
            .await
            .unwrap();

        assert_eq!(doc.id, "hello-world");
        assert_eq!(doc.title, "Hello World");
        assert_eq!(doc.content, "# Hello");
        assert_eq!(doc.tags, vec!["test"]);
        assert_eq!(doc.created_by, "user");
    }

    #[tokio::test]
    async fn test_create_document_slug_conflict() {
        let (_dir, store) = setup().await;
        let doc1 = store
            .create(create_request("My Doc", "Content 1"))
            .await
            .unwrap();
        let doc2 = store
            .create(create_request("My Doc", "Content 2"))
            .await
            .unwrap();

        assert_eq!(doc1.id, "my-doc");
        assert_eq!(doc2.id, "my-doc-2");
    }

    #[tokio::test]
    async fn test_create_document_empty_title() {
        let (_dir, store) = setup().await;
        let result = store.create(create_request("!!!", "Content")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_document() {
        let (_dir, store) = setup().await;
        store
            .create(create_request("Test Doc", "# Content"))
            .await
            .unwrap();

        let doc = store.get("test-doc").await.unwrap();
        assert!(doc.is_some());
        let doc = doc.unwrap();
        assert_eq!(doc.id, "test-doc");
        assert_eq!(doc.content, "# Content");
    }

    #[tokio::test]
    async fn test_get_document_not_found() {
        let (_dir, store) = setup().await;
        let doc = store.get("nonexistent").await.unwrap();
        assert!(doc.is_none());
    }

    #[tokio::test]
    async fn test_update_document() {
        let (_dir, store) = setup().await;
        store
            .create(create_request("Test Doc", "# Original"))
            .await
            .unwrap();

        let updated = store
            .update(
                "test-doc",
                UpdateDocumentRequest {
                    content: Some("# Updated".to_string()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(updated.content, "# Updated");
        assert_eq!(updated.title, "Test Doc"); // unchanged
    }

    #[tokio::test]
    async fn test_update_document_title() {
        let (_dir, store) = setup().await;
        store
            .create(create_request("Old Title", "Content"))
            .await
            .unwrap();

        let updated = store
            .update(
                "old-title",
                UpdateDocumentRequest {
                    title: Some("New Title".to_string()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(updated.title, "New Title");
        assert_eq!(updated.id, "old-title"); // ID doesn't change
    }

    #[tokio::test]
    async fn test_update_document_not_found() {
        let (_dir, store) = setup().await;
        let result = store
            .update(
                "nonexistent",
                UpdateDocumentRequest {
                    content: Some("New".to_string()),
                    ..Default::default()
                },
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_document() {
        let (_dir, store) = setup().await;
        store
            .create(create_request("To Delete", "Content"))
            .await
            .unwrap();

        let deleted = store.delete("to-delete").await.unwrap();
        assert!(deleted);

        let doc = store.get("to-delete").await.unwrap();
        assert!(doc.is_none());
    }

    #[tokio::test]
    async fn test_delete_document_not_found() {
        let (_dir, store) = setup().await;
        let deleted = store.delete("nonexistent").await.unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_list_documents() {
        let (_dir, store) = setup().await;
        store
            .create(create_request("Doc A", "Content A"))
            .await
            .unwrap();
        store
            .create(create_request("Doc B", "Content B"))
            .await
            .unwrap();

        let docs = store.list(ListDocumentsRequest::default()).await.unwrap();
        assert_eq!(docs.len(), 2);
    }

    #[tokio::test]
    async fn test_list_documents_filter_by_tags() {
        let (_dir, store) = setup().await;
        store
            .create(CreateDocumentRequest {
                title: "Tagged".to_string(),
                content: "Content".to_string(),
                tags: vec!["rust".to_string()],
                created_by: "user".to_string(),
            })
            .await
            .unwrap();
        store
            .create(CreateDocumentRequest {
                title: "Untagged".to_string(),
                content: "Content".to_string(),
                tags: vec!["python".to_string()],
                created_by: "user".to_string(),
            })
            .await
            .unwrap();

        let docs = store
            .list(ListDocumentsRequest {
                tags: Some(vec!["rust".to_string()]),
                created_by: None,
            })
            .await
            .unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].id, "tagged");
    }

    #[tokio::test]
    async fn test_list_documents_filter_by_created_by() {
        let (_dir, store) = setup().await;
        store
            .create(CreateDocumentRequest {
                title: "By Alice".to_string(),
                content: "Content".to_string(),
                tags: vec![],
                created_by: "alice".to_string(),
            })
            .await
            .unwrap();
        store
            .create(CreateDocumentRequest {
                title: "By Bob".to_string(),
                content: "Content".to_string(),
                tags: vec![],
                created_by: "bob".to_string(),
            })
            .await
            .unwrap();

        let docs = store
            .list(ListDocumentsRequest {
                tags: None,
                created_by: Some("alice".to_string()),
            })
            .await
            .unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].created_by, "alice");
    }

    #[tokio::test]
    async fn test_list_documents_empty() {
        let (_dir, store) = setup().await;
        let docs = store.list(ListDocumentsRequest::default()).await.unwrap();
        assert!(docs.is_empty());
    }
}
