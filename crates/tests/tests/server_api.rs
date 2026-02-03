//! Server API integration tests.
//!
//! Tests the Berry server against a real ChromaDB instance.

use berry::types::{CreateMemoryRequest, MemoryType, SearchRequest, VisibilityLevel};
use berry::store::VectorStore;
use berry_tests::fixtures::TestEnvironment;

/// Test creating a memory through the store.
#[tokio::test]
async fn test_create_memory() {
    let env = TestEnvironment::new().await;

    let request = CreateMemoryRequest {
        content: "Integration test memory content".to_string(),
        memory_type: MemoryType::Information,
        tags: vec!["test".to_string(), "integration".to_string()],
        created_by: "test_user".to_string(),
        references: vec![],
        visibility: VisibilityLevel::Public,
        shared_with: vec![],
    };

    let memory = env.store.create(request).await
        .expect("Failed to create memory");

    assert!(memory.id.starts_with("mem_"));
    assert_eq!(memory.content, "Integration test memory content");
    assert_eq!(memory.memory_type, MemoryType::Information);
    assert_eq!(memory.tags, vec!["test", "integration"]);
    assert_eq!(memory.created_by, "test_user");
}

/// Test retrieving a memory by ID.
#[tokio::test]
async fn test_get_memory() {
    let env = TestEnvironment::new().await;

    // Create a memory first
    let request = CreateMemoryRequest {
        content: "Memory to retrieve".to_string(),
        memory_type: MemoryType::Question,
        tags: vec!["retrieve".to_string()],
        created_by: "test_user".to_string(),
        references: vec![],
        visibility: VisibilityLevel::Public,
        shared_with: vec![],
    };

    let created = env.store.create(request).await
        .expect("Failed to create memory");

    // Retrieve it
    let retrieved = env.store.get(&created.id).await
        .expect("Failed to get memory")
        .expect("Memory not found");

    assert_eq!(retrieved.id, created.id);
    assert_eq!(retrieved.content, "Memory to retrieve");
    assert_eq!(retrieved.memory_type, MemoryType::Question);
}

/// Test that getting a non-existent memory returns None.
#[tokio::test]
async fn test_get_nonexistent_memory() {
    let env = TestEnvironment::new().await;

    let result = env.store.get("mem_nonexistent_id").await
        .expect("Query failed");

    assert!(result.is_none());
}

/// Test deleting a memory.
#[tokio::test]
async fn test_delete_memory() {
    let env = TestEnvironment::new().await;

    // Create a memory
    let request = CreateMemoryRequest {
        content: "Memory to delete".to_string(),
        memory_type: MemoryType::Information,
        tags: vec![],
        created_by: "test_user".to_string(),
        references: vec![],
        visibility: VisibilityLevel::Public,
        shared_with: vec![],
    };

    let memory = env.store.create(request).await
        .expect("Failed to create memory");

    // Verify it exists
    let exists = env.store.get(&memory.id).await
        .expect("Query failed")
        .is_some();
    assert!(exists);

    // Delete it
    let deleted = env.store.delete(&memory.id).await
        .expect("Failed to delete memory");
    assert!(deleted);

    // Verify it's gone
    let exists = env.store.get(&memory.id).await
        .expect("Query failed")
        .is_some();
    assert!(!exists);
}

/// Test updating memory visibility.
#[tokio::test]
async fn test_update_visibility() {
    let env = TestEnvironment::new().await;

    // Create a public memory
    let request = CreateMemoryRequest {
        content: "Visibility test memory".to_string(),
        memory_type: MemoryType::Information,
        tags: vec![],
        created_by: "test_user".to_string(),
        references: vec![],
        visibility: VisibilityLevel::Public,
        shared_with: vec![],
    };

    let memory = env.store.create(request).await
        .expect("Failed to create memory");
    assert_eq!(memory.visibility, VisibilityLevel::Public);

    // Update to shared
    let updated = env.store.update_visibility(
        &memory.id,
        VisibilityLevel::Shared,
        Some(vec!["alice".to_string(), "bob".to_string()]),
    ).await.expect("Failed to update visibility");

    assert_eq!(updated.visibility, VisibilityLevel::Shared);
    assert_eq!(updated.shared_with, vec!["alice", "bob"]);

    // Update to private
    let updated = env.store.update_visibility(
        &memory.id,
        VisibilityLevel::Private,
        None,
    ).await.expect("Failed to update visibility");

    assert_eq!(updated.visibility, VisibilityLevel::Private);
}

/// Test listing all memories.
#[tokio::test]
async fn test_list_all_memories() {
    let env = TestEnvironment::with_collection("test_list_all").await;

    // Create multiple memories
    for i in 0..3 {
        let request = CreateMemoryRequest {
            content: format!("Memory number {}", i),
            memory_type: MemoryType::Information,
            tags: vec![],
            created_by: "test_user".to_string(),
            references: vec![],
            visibility: VisibilityLevel::Public,
            shared_with: vec![],
        };
        env.store.create(request).await.expect("Failed to create memory");
    }

    // List all
    let memories = env.store.list_all().await
        .expect("Failed to list memories");

    assert_eq!(memories.len(), 3);
}

/// Test search functionality.
/// Note: Since we use NoOp embeddings, this tests the non-semantic search path.
#[tokio::test]
async fn test_search_memories() {
    let env = TestEnvironment::with_collection("test_search").await;

    // Create memories with different content
    let contents = vec![
        "Rust programming language",
        "Python data science",
        "Rust cargo build system",
    ];

    for content in contents {
        let request = CreateMemoryRequest {
            content: content.to_string(),
            memory_type: MemoryType::Information,
            tags: vec![],
            created_by: "test_user".to_string(),
            references: vec![],
            visibility: VisibilityLevel::Public,
            shared_with: vec![],
        };
        env.store.create(request).await.expect("Failed to create memory");
    }

    // Search for "Rust"
    let search_request = SearchRequest {
        query: "Rust".to_string(),
        limit: 10,
        ..Default::default()
    };

    let results = env.store.search(search_request).await
        .expect("Search failed");

    // With NoOp embeddings, search returns all memories (no semantic filtering)
    // This test verifies the search infrastructure works
    assert!(!results.is_empty());
}

/// Test health check.
#[tokio::test]
async fn test_health_check() {
    let env = TestEnvironment::new().await;

    let healthy = env.store.health_check().await
        .expect("Health check failed");

    assert!(healthy);
}

/// Test collection deletion.
#[tokio::test]
async fn test_delete_collection() {
    let env = TestEnvironment::with_collection("test_delete_collection").await;

    // Create a memory
    let request = CreateMemoryRequest {
        content: "Memory in collection to delete".to_string(),
        memory_type: MemoryType::Information,
        tags: vec![],
        created_by: "test_user".to_string(),
        references: vec![],
        visibility: VisibilityLevel::Public,
        shared_with: vec![],
    };
    env.store.create(request).await.expect("Failed to create memory");

    // Verify it exists
    let memories = env.store.list_all().await.expect("Failed to list");
    assert_eq!(memories.len(), 1);

    // Delete collection
    env.store.delete_collection().await
        .expect("Failed to delete collection");

    // Re-initialize to recreate collection
    env.store.initialize().await.expect("Failed to reinitialize");

    // Verify it's empty
    let memories = env.store.list_all().await.expect("Failed to list");
    assert_eq!(memories.len(), 0);
}

/// Test memory types are preserved.
#[tokio::test]
async fn test_memory_types() {
    let env = TestEnvironment::with_collection("test_memory_types").await;

    let types = vec![
        MemoryType::Information,
        MemoryType::Question,
        MemoryType::Request,
    ];

    for memory_type in types {
        let request = CreateMemoryRequest {
            content: format!("Memory of type {:?}", memory_type),
            memory_type,
            tags: vec![],
            created_by: "test_user".to_string(),
            references: vec![],
            visibility: VisibilityLevel::Public,
            shared_with: vec![],
        };

        let memory = env.store.create(request).await
            .expect("Failed to create memory");
        assert_eq!(memory.memory_type, memory_type);

        let retrieved = env.store.get(&memory.id).await
            .expect("Failed to get memory")
            .expect("Memory not found");
        assert_eq!(retrieved.memory_type, memory_type);
    }
}

/// Test tags are preserved and searchable.
#[tokio::test]
async fn test_tags() {
    let env = TestEnvironment::with_collection("test_tags").await;

    let request = CreateMemoryRequest {
        content: "Tagged memory".to_string(),
        memory_type: MemoryType::Information,
        tags: vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()],
        created_by: "test_user".to_string(),
        references: vec![],
        visibility: VisibilityLevel::Public,
        shared_with: vec![],
    };

    let memory = env.store.create(request).await
        .expect("Failed to create memory");

    assert_eq!(memory.tags, vec!["tag1", "tag2", "tag3"]);

    let retrieved = env.store.get(&memory.id).await
        .expect("Failed to get memory")
        .expect("Memory not found");

    assert_eq!(retrieved.tags, vec!["tag1", "tag2", "tag3"]);
}
