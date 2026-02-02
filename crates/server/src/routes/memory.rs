//! Memory CRUD endpoints.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use berry::{
    CreateMemoryRequest, DeleteResponse, MemoryResponse, UpdateVisibilityRequest,
};

use crate::state::AppState;

/// Create a new memory.
///
/// POST /v1/memory
pub async fn create_memory(
    State(state): State<AppState>,
    Json(request): Json<CreateMemoryRequest>,
) -> Result<Json<MemoryResponse>, (StatusCode, Json<MemoryResponse>)> {
    match state.store.create(request).await {
        Ok(memory) => Ok(Json(MemoryResponse {
            success: true,
            memory: Some(memory),
            error: None,
        })),
        Err(e) => {
            tracing::error!("Failed to create memory: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MemoryResponse {
                    success: false,
                    memory: None,
                    error: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// Get a memory by ID.
///
/// GET /v1/memory/:id
pub async fn get_memory(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<MemoryResponse>, (StatusCode, Json<MemoryResponse>)> {
    match state.store.get(&id).await {
        Ok(Some(memory)) => Ok(Json(MemoryResponse {
            success: true,
            memory: Some(memory),
            error: None,
        })),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(MemoryResponse {
                success: false,
                memory: None,
                error: Some(format!("Memory not found: {}", id)),
            }),
        )),
        Err(e) => {
            tracing::error!("Failed to get memory {}: {}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MemoryResponse {
                    success: false,
                    memory: None,
                    error: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// Delete a memory by ID.
///
/// DELETE /v1/memory/:id
pub async fn delete_memory(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DeleteResponse>, (StatusCode, Json<DeleteResponse>)> {
    match state.store.delete(&id).await {
        Ok(deleted) => Ok(Json(DeleteResponse {
            success: true,
            deleted,
            error: None,
        })),
        Err(e) => {
            tracing::error!("Failed to delete memory {}: {}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeleteResponse {
                    success: false,
                    deleted: false,
                    error: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// Update memory visibility.
///
/// PATCH /v1/memory/:id/visibility
pub async fn update_visibility(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateVisibilityRequest>,
) -> Result<Json<MemoryResponse>, (StatusCode, Json<MemoryResponse>)> {
    match state
        .store
        .update_visibility(&id, request.visibility, request.shared_with)
        .await
    {
        Ok(memory) => Ok(Json(MemoryResponse {
            success: true,
            memory: Some(memory),
            error: None,
        })),
        Err(berry::StoreError::NotFound(_)) => Err((
            StatusCode::NOT_FOUND,
            Json(MemoryResponse {
                success: false,
                memory: None,
                error: Some(format!("Memory not found: {}", id)),
            }),
        )),
        Err(e) => {
            tracing::error!("Failed to update visibility for {}: {}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MemoryResponse {
                    success: false,
                    memory: None,
                    error: Some(e.to_string()),
                }),
            ))
        }
    }
}
