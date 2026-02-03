//! Berry HTTP Server Library
//!
//! Provides the HTTP server functionality for Berry.
//! Can be used as a library or run as a standalone binary.

use std::net::SocketAddr;

use axum::{
    Router,
    routing::{delete, get, patch, post},
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use std::sync::Arc;

use berry::config::load_config;
use berry::store::{ChromaStore, VectorStore, create_embedding_service};

pub mod routes;
pub mod state;

use routes::{
    create_memory, delete_memory, get_memory, health_handler, schema_handler, search_handler,
    update_visibility,
};
use state::AppState;

/// Server configuration options.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Port to listen on.
    pub port: u16,
    /// Host to bind to.
    pub host: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 4114,
            host: "127.0.0.1".to_string(),
        }
    }
}

/// Run the Berry HTTP server.
///
/// This function blocks until the server is shut down.
pub async fn run_server(config: ServerConfig) -> anyhow::Result<()> {
    // Load configuration
    let app_config = load_config().unwrap_or_default();

    // Create embedding service
    let embedding_service = match create_embedding_service(&app_config.embedding) {
        Ok(service) => Arc::from(service),
        Err(e) => {
            tracing::warn!(
                "Failed to create embedding service: {}. Semantic search will not work.",
                e
            );
            Arc::from(berry::store::NoOpEmbedding::new()) as Arc<dyn berry::store::EmbeddingService>
        }
    };

    // Create store
    let store = ChromaStore::new(&app_config.chroma, embedding_service);

    // Initialize store (create collection if needed)
    tracing::info!("Initializing vector store...");
    if let Err(e) = store.initialize().await {
        tracing::warn!(
            "Failed to initialize store: {}. Server will start anyway.",
            e
        );
    }

    // Create application state
    let state = AppState::new(store);

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(health_handler))
        // Memory operations
        .route("/v1/memory", post(create_memory))
        .route("/v1/memory/{id}", get(get_memory))
        .route("/v1/memory/{id}", delete(delete_memory))
        .route("/v1/memory/{id}/visibility", patch(update_visibility))
        // Search
        .route("/v1/search", post(search_handler))
        // Schema
        .route("/schema", get(schema_handler))
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    tracing::info!("Berry server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Create the application router with the given state.
/// Useful for testing.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/v1/memory", post(create_memory))
        .route("/v1/memory/{id}", get(get_memory))
        .route("/v1/memory/{id}", delete(delete_memory))
        .route("/v1/memory/{id}/visibility", patch(update_visibility))
        .route("/v1/search", post(search_handler))
        .route("/schema", get(schema_handler))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use berry::error::{StoreError, StoreResult};
    use berry::store::VectorStore;
    use berry::types::{CreateMemoryRequest, Memory, MemoryType, SearchRequest, VisibilityLevel};
    use chrono::Utc;
    use http_body_util::BodyExt;
    use std::sync::Mutex;
    use tower::ServiceExt;

    /// Mock vector store for testing.
    struct MockStore {
        memories: Mutex<Vec<Memory>>,
        should_fail: bool,
    }

    impl MockStore {
        fn new() -> Self {
            Self {
                memories: Mutex::new(Vec::new()),
                should_fail: false,
            }
        }

        fn with_memories(memories: Vec<Memory>) -> Self {
            Self {
                memories: Mutex::new(memories),
                should_fail: false,
            }
        }

        fn failing() -> Self {
            Self {
                memories: Mutex::new(Vec::new()),
                should_fail: true,
            }
        }
    }

    #[async_trait]
    impl VectorStore for MockStore {
        async fn create(&self, request: CreateMemoryRequest) -> StoreResult<Memory> {
            if self.should_fail {
                return Err(StoreError::QueryFailed("Mock failure".to_string()));
            }
            let now = Utc::now();
            let memory = Memory {
                id: Memory::generate_id(),
                content: request.content,
                memory_type: request.memory_type,
                tags: request.tags,
                created_by: request.created_by,
                created_at: now,
                updated_at: now,
                owner: None,
                visibility: request.visibility,
                shared_with: request.shared_with,
            };
            self.memories.lock().unwrap().push(memory.clone());
            Ok(memory)
        }

        async fn get(&self, id: &str) -> StoreResult<Option<Memory>> {
            if self.should_fail {
                return Err(StoreError::QueryFailed("Mock failure".to_string()));
            }
            let memories = self.memories.lock().unwrap();
            Ok(memories.iter().find(|m| m.id == id).cloned())
        }

        async fn delete(&self, id: &str) -> StoreResult<bool> {
            if self.should_fail {
                return Err(StoreError::QueryFailed("Mock failure".to_string()));
            }
            let mut memories = self.memories.lock().unwrap();
            let len_before = memories.len();
            memories.retain(|m| m.id != id);
            Ok(memories.len() < len_before)
        }

        async fn search(&self, request: SearchRequest) -> StoreResult<Vec<Memory>> {
            if self.should_fail {
                return Err(StoreError::QueryFailed("Mock failure".to_string()));
            }
            let memories = self.memories.lock().unwrap();
            let results: Vec<Memory> = memories
                .iter()
                .filter(|m| m.content.contains(&request.query))
                .take(request.limit)
                .cloned()
                .collect();
            Ok(results)
        }

        async fn update_visibility(
            &self,
            id: &str,
            visibility: VisibilityLevel,
            shared_with: Option<Vec<String>>,
        ) -> StoreResult<Memory> {
            if self.should_fail {
                return Err(StoreError::QueryFailed("Mock failure".to_string()));
            }
            let mut memories = self.memories.lock().unwrap();
            let memory = memories
                .iter_mut()
                .find(|m| m.id == id)
                .ok_or_else(|| StoreError::NotFound(id.to_string()))?;
            memory.visibility = visibility;
            if let Some(shared) = shared_with {
                memory.shared_with = shared;
            }
            memory.updated_at = Utc::now();
            Ok(memory.clone())
        }

        async fn health_check(&self) -> StoreResult<bool> {
            if self.should_fail {
                return Err(StoreError::QueryFailed("Mock failure".to_string()));
            }
            Ok(true)
        }

        async fn initialize(&self) -> StoreResult<()> {
            Ok(())
        }

        async fn list_all(&self) -> StoreResult<Vec<Memory>> {
            let memories = self.memories.lock().unwrap();
            Ok(memories.clone())
        }

        async fn delete_collection(&self) -> StoreResult<()> {
            self.memories.lock().unwrap().clear();
            Ok(())
        }
    }

    fn create_test_memory(id: &str, content: &str) -> Memory {
        Memory {
            id: id.to_string(),
            content: content.to_string(),
            memory_type: MemoryType::Information,
            tags: vec!["test".to_string()],
            created_by: "testuser".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            owner: None,
            visibility: VisibilityLevel::Public,
            shared_with: vec![],
        }
    }

    #[tokio::test]
    async fn test_health_endpoint_healthy() {
        let state = AppState::new(MockStore::new());
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let health: berry::HealthResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(health.status, "healthy");
        assert_eq!(health.database, "connected");
    }

    #[tokio::test]
    async fn test_health_endpoint_database_disconnected() {
        let state = AppState::new(MockStore::failing());
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let health: berry::HealthResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(health.status, "healthy");
        assert_eq!(health.database, "disconnected");
    }

    #[tokio::test]
    async fn test_create_memory() {
        let state = AppState::new(MockStore::new());
        let app = create_router(state);

        let request_body = serde_json::json!({
            "content": "Test memory content",
            "created_by": "testuser"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/memory")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let response: berry::MemoryResponse = serde_json::from_slice(&body).unwrap();
        assert!(response.success);
        assert!(response.memory.is_some());
        let memory = response.memory.unwrap();
        assert_eq!(memory.content, "Test memory content");
        assert_eq!(memory.created_by, "testuser");
    }

    #[tokio::test]
    async fn test_get_memory_found() {
        let memory = create_test_memory("mem_test_123", "Test content");
        let state = AppState::new(MockStore::with_memories(vec![memory]));
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/memory/mem_test_123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let response: berry::MemoryResponse = serde_json::from_slice(&body).unwrap();
        assert!(response.success);
        assert!(response.memory.is_some());
        assert_eq!(response.memory.unwrap().id, "mem_test_123");
    }

    #[tokio::test]
    async fn test_get_memory_not_found() {
        let state = AppState::new(MockStore::new());
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/memory/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_memory() {
        let memory = create_test_memory("mem_to_delete", "Delete me");
        let state = AppState::new(MockStore::with_memories(vec![memory]));
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/v1/memory/mem_to_delete")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let response: berry::DeleteResponse = serde_json::from_slice(&body).unwrap();
        assert!(response.success);
        assert!(response.deleted);
    }

    #[tokio::test]
    async fn test_search_memories() {
        let mem1 = create_test_memory("mem_1", "Authentication tokens");
        let mem2 = create_test_memory("mem_2", "Database backups");
        let mem3 = create_test_memory("mem_3", "Auth configuration");
        let state = AppState::new(MockStore::with_memories(vec![mem1, mem2, mem3]));
        let app = create_router(state);

        let request_body = serde_json::json!({
            "query": "Auth"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/search")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let response: berry::SearchResponse = serde_json::from_slice(&body).unwrap();
        assert!(response.success);
        assert_eq!(response.total, 2);
        assert_eq!(response.memories.len(), 2);
    }

    #[tokio::test]
    async fn test_update_visibility() {
        let memory = create_test_memory("mem_vis", "Content");
        let state = AppState::new(MockStore::with_memories(vec![memory]));
        let app = create_router(state);

        let request_body = serde_json::json!({
            "visibility": "shared",
            "shared_with": ["alice", "bob"]
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/v1/memory/mem_vis/visibility")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let response: berry::MemoryResponse = serde_json::from_slice(&body).unwrap();
        assert!(response.success);
        let memory = response.memory.unwrap();
        assert_eq!(memory.visibility, VisibilityLevel::Shared);
        assert_eq!(memory.shared_with, vec!["alice", "bob"]);
    }

    #[tokio::test]
    async fn test_update_visibility_not_found() {
        let state = AppState::new(MockStore::new());
        let app = create_router(state);

        let request_body = serde_json::json!({
            "visibility": "private"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/v1/memory/nonexistent/visibility")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_schema_endpoint() {
        let state = AppState::new(MockStore::new());
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/schema")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
