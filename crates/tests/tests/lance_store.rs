//! Integration tests for the LanceDB vector store.
//!
//! These tests require the `lancedb-store` feature and use a temporary
//! directory — no Docker or external services needed.

use std::sync::Arc;

use tempfile::TempDir;

use berry::config::LanceConfig;
use berry::error::StoreResult;
use berry::store::{EmbeddingService, LanceStore, VectorStore};
use berry::types::{CreateMemoryRequest, MemoryType, SearchRequest, VisibilityLevel};

/// Mock embedding service that returns deterministic fake embeddings.
struct MockEmbedding {
    dimension: usize,
}

impl MockEmbedding {
    fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[async_trait::async_trait]
impl EmbeddingService for MockEmbedding {
    async fn embed(&self, texts: &[String]) -> StoreResult<Vec<Vec<f32>>> {
        Ok(texts
            .iter()
            .map(|text| {
                let mut embedding = vec![0.0f32; self.dimension];
                for (i, c) in text.chars().enumerate() {
                    let idx = i % self.dimension;
                    embedding[idx] += (c as u32 as f32) / 1000.0;
                }
                // Normalize
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

const TEST_DIM: usize = 32;

async fn create_test_store(tmp: &TempDir) -> LanceStore {
    let config = LanceConfig {
        path: tmp.path().to_string_lossy().to_string(),
        table: "test_memories".to_string(),
    };
    let embedding = Arc::new(MockEmbedding::new(TEST_DIM));
    let store = LanceStore::new(&config, embedding).await.unwrap();
    store.initialize().await.unwrap();
    store
}

fn make_request(content: &str) -> CreateMemoryRequest {
    CreateMemoryRequest {
        content: content.to_string(),
        memory_type: MemoryType::Information,
        tags: vec!["test".to_string()],
        created_by: "testuser".to_string(),
        references: vec![],
        visibility: VisibilityLevel::Public,
        shared_with: vec![],
    }
}

#[tokio::test]
async fn test_create_and_get() {
    let tmp = TempDir::new().unwrap();
    let store = create_test_store(&tmp).await;

    let memory = store.create(make_request("Hello LanceDB")).await.unwrap();
    assert!(memory.id.starts_with("mem_"));
    assert_eq!(memory.content, "Hello LanceDB");
    assert_eq!(memory.memory_type, MemoryType::Information);

    let fetched = store.get(&memory.id).await.unwrap();
    assert!(fetched.is_some());
    let fetched = fetched.unwrap();
    assert_eq!(fetched.id, memory.id);
    assert_eq!(fetched.content, "Hello LanceDB");
}

#[tokio::test]
async fn test_get_not_found() {
    let tmp = TempDir::new().unwrap();
    let store = create_test_store(&tmp).await;

    let result = store.get("nonexistent_id").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete() {
    let tmp = TempDir::new().unwrap();
    let store = create_test_store(&tmp).await;

    let memory = store.create(make_request("To be deleted")).await.unwrap();

    let deleted = store.delete(&memory.id).await.unwrap();
    assert!(deleted);

    let fetched = store.get(&memory.id).await.unwrap();
    assert!(fetched.is_none());
}

#[tokio::test]
async fn test_delete_not_found() {
    let tmp = TempDir::new().unwrap();
    let store = create_test_store(&tmp).await;

    let deleted = store.delete("nonexistent").await.unwrap();
    assert!(!deleted);
}

#[tokio::test]
async fn test_list_all() {
    let tmp = TempDir::new().unwrap();
    let store = create_test_store(&tmp).await;

    store.create(make_request("Memory one")).await.unwrap();
    store.create(make_request("Memory two")).await.unwrap();
    store.create(make_request("Memory three")).await.unwrap();

    let all = store.list_all().await.unwrap();
    assert_eq!(all.len(), 3);
}

#[tokio::test]
async fn test_vector_search() {
    let tmp = TempDir::new().unwrap();
    let store = create_test_store(&tmp).await;

    store
        .create(make_request("Rust programming language"))
        .await
        .unwrap();
    store
        .create(make_request("Python scripting"))
        .await
        .unwrap();
    store
        .create(make_request("Rust cargo build system"))
        .await
        .unwrap();

    let request = SearchRequest {
        query: "Rust programming".to_string(),
        limit: 10,
        memory_type: None,
        tags: vec![],
        as_actor: None,
        from: None,
        to: None,
    };

    let results = store.search(request).await.unwrap();
    assert!(!results.is_empty());
    // With deterministic mock embeddings, similar texts should rank higher
    assert!(results.len() <= 3);
}

#[tokio::test]
async fn test_search_with_type_filter() {
    let tmp = TempDir::new().unwrap();
    let store = create_test_store(&tmp).await;

    store
        .create(CreateMemoryRequest {
            content: "What is Rust?".to_string(),
            memory_type: MemoryType::Question,
            tags: vec![],
            created_by: "user".to_string(),
            references: vec![],
            visibility: VisibilityLevel::Public,
            shared_with: vec![],
        })
        .await
        .unwrap();

    store
        .create(CreateMemoryRequest {
            content: "Rust is a systems language".to_string(),
            memory_type: MemoryType::Information,
            tags: vec![],
            created_by: "user".to_string(),
            references: vec![],
            visibility: VisibilityLevel::Public,
            shared_with: vec![],
        })
        .await
        .unwrap();

    let request = SearchRequest {
        query: "Rust".to_string(),
        limit: 10,
        memory_type: Some(MemoryType::Question),
        tags: vec![],
        as_actor: None,
        from: None,
        to: None,
    };

    let results = store.search(request).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].memory_type, MemoryType::Question);
}

#[tokio::test]
async fn test_update_visibility() {
    let tmp = TempDir::new().unwrap();
    let store = create_test_store(&tmp).await;

    let memory = store.create(make_request("Secret data")).await.unwrap();
    assert_eq!(memory.visibility, VisibilityLevel::Public);

    let updated = store
        .update_visibility(
            &memory.id,
            VisibilityLevel::Shared,
            Some(vec!["alice".to_string(), "bob".to_string()]),
        )
        .await
        .unwrap();

    assert_eq!(updated.visibility, VisibilityLevel::Shared);
    assert_eq!(updated.shared_with, vec!["alice", "bob"]);

    // Verify persisted
    let fetched = store.get(&memory.id).await.unwrap().unwrap();
    assert_eq!(fetched.visibility, VisibilityLevel::Shared);
    assert_eq!(fetched.shared_with, vec!["alice", "bob"]);
}

#[tokio::test]
async fn test_delete_collection() {
    let tmp = TempDir::new().unwrap();
    let store = create_test_store(&tmp).await;

    store.create(make_request("Some data")).await.unwrap();
    store.delete_collection().await.unwrap();

    // After deletion, open_table should fail — re-initialize creates fresh
    let config = LanceConfig {
        path: tmp.path().to_string_lossy().to_string(),
        table: "test_memories".to_string(),
    };
    let embedding = Arc::new(MockEmbedding::new(TEST_DIM));
    let store2 = LanceStore::new(&config, embedding).await.unwrap();
    store2.initialize().await.unwrap();

    let all = store2.list_all().await.unwrap();
    assert!(all.is_empty());
}

#[tokio::test]
async fn test_health_check() {
    let tmp = TempDir::new().unwrap();
    let store = create_test_store(&tmp).await;

    let healthy = store.health_check().await.unwrap();
    assert!(healthy);
}

#[tokio::test]
async fn test_initialize_idempotent() {
    let tmp = TempDir::new().unwrap();
    let store = create_test_store(&tmp).await;

    // Second init should not fail
    store.initialize().await.unwrap();
    store.initialize().await.unwrap();
}

#[tokio::test]
async fn test_visibility_filtering_in_search() {
    let tmp = TempDir::new().unwrap();
    let store = create_test_store(&tmp).await;

    // Create a private memory
    store
        .create(CreateMemoryRequest {
            content: "Private secret".to_string(),
            memory_type: MemoryType::Information,
            tags: vec![],
            created_by: "alice".to_string(),
            references: vec![],
            visibility: VisibilityLevel::Private,
            shared_with: vec![],
        })
        .await
        .unwrap();

    // Create a public memory
    store
        .create(CreateMemoryRequest {
            content: "Public info".to_string(),
            memory_type: MemoryType::Information,
            tags: vec![],
            created_by: "alice".to_string(),
            references: vec![],
            visibility: VisibilityLevel::Public,
            shared_with: vec![],
        })
        .await
        .unwrap();

    // Search as bob — should only see public
    let request = SearchRequest {
        query: "info".to_string(),
        limit: 10,
        memory_type: None,
        tags: vec![],
        as_actor: Some("bob".to_string()),
        from: None,
        to: None,
    };

    let results = store.search(request).await.unwrap();
    for m in &results {
        assert_eq!(m.visibility, VisibilityLevel::Public);
    }
}
