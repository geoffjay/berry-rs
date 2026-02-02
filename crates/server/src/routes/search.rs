//! Search endpoint.

use axum::{Json, extract::State, http::StatusCode};

use berry::{SearchRequest, SearchResponse};

use crate::state::AppState;

/// Search for memories.
///
/// POST /v1/search
pub async fn search_handler(
    State(state): State<AppState>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<SearchResponse>)> {
    match state.store.search(request).await {
        Ok(memories) => {
            let total = memories.len();
            Ok(Json(SearchResponse {
                success: true,
                memories,
                total,
                error: None,
            }))
        }
        Err(e) => {
            tracing::error!("Search failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SearchResponse {
                    success: false,
                    memories: vec![],
                    total: 0,
                    error: Some(e.to_string()),
                }),
            ))
        }
    }
}
