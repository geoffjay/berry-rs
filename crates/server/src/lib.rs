//! Berry HTTP Server Library
//!
//! Provides the HTTP server functionality for Berry.
//! Can be used as a library or run as a standalone binary.

use std::net::SocketAddr;

use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use berry::config::load_config;
use berry::store::{ChromaStore, VectorStore};

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

    // Create store
    let store = ChromaStore::new(&app_config.chroma);

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
