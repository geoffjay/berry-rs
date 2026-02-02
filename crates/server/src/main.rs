//! Berry HTTP Server
//!
//! Provides RESTful API endpoints for memory operations.

use std::net::SocketAddr;

use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use clap::Parser;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use berry::config::load_config;
use berry::store::{ChromaStore, VectorStore};

mod routes;
mod state;

use routes::{
    create_memory, delete_memory, get_memory, health_handler, schema_handler, search_handler,
    update_visibility,
};
use state::AppState;

/// Berry HTTP Server
#[derive(Parser)]
#[command(name = "berry-server")]
#[command(version, about = "Berry memory system HTTP server")]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "4114")]
    port: u16,

    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    berry::logging::init();

    let args = Args::parse();

    // Load configuration
    let config = load_config().unwrap_or_default();

    // Create store
    let store = ChromaStore::new(&config.chroma);

    // Initialize store (create collection if needed)
    tracing::info!("Initializing vector store...");
    if let Err(e) = store.initialize().await {
        tracing::warn!("Failed to initialize store: {}. Server will start anyway.", e);
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

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    tracing::info!("Berry server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
