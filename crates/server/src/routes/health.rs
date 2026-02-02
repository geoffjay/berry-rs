//! Health check endpoint.

use axum::{extract::State, http::StatusCode, Json};

use berry::HealthResponse;

use crate::state::AppState;

/// Health check handler.
///
/// Returns the server health status including database connectivity.
pub async fn health_handler(State(state): State<AppState>) -> Result<Json<HealthResponse>, StatusCode> {
    let database_status = match state.store.health_check().await {
        Ok(true) => "connected",
        Ok(false) => "unhealthy",
        Err(_) => "disconnected",
    };

    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
        version: berry::VERSION.to_string(),
        database: database_status.to_string(),
    }))
}
