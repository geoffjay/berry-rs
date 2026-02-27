//! Document CRUD endpoints.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};

use berry::{
    CreateDocumentRequest, DeleteResponse, DocumentListResponse, DocumentResponse,
    ListDocumentsRequest, UpdateDocumentRequest,
};

use crate::state::AppState;

/// Helper to get the document store or return a 501 error.
fn get_doc_store(
    state: &AppState,
) -> Result<&std::sync::Arc<dyn berry::documents::DocumentStore>, (StatusCode, Json<DocumentResponse>)>
{
    state.doc_store.as_ref().ok_or_else(|| {
        (
            StatusCode::NOT_IMPLEMENTED,
            Json(DocumentResponse {
                success: false,
                document: None,
                error: Some("Document store is not enabled".to_string()),
            }),
        )
    })
}

/// Create a new document.
///
/// POST /v1/documents
pub async fn create_document(
    State(state): State<AppState>,
    Json(request): Json<CreateDocumentRequest>,
) -> Result<Json<DocumentResponse>, (StatusCode, Json<DocumentResponse>)> {
    let doc_store = get_doc_store(&state)?;

    match doc_store.create(request).await {
        Ok(document) => Ok(Json(DocumentResponse {
            success: true,
            document: Some(document),
            error: None,
        })),
        Err(e) => {
            tracing::error!("Failed to create document: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DocumentResponse {
                    success: false,
                    document: None,
                    error: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// Get a document by ID.
///
/// GET /v1/documents/:id
pub async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DocumentResponse>, (StatusCode, Json<DocumentResponse>)> {
    let doc_store = get_doc_store(&state)?;

    match doc_store.get(&id).await {
        Ok(Some(document)) => Ok(Json(DocumentResponse {
            success: true,
            document: Some(document),
            error: None,
        })),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(DocumentResponse {
                success: false,
                document: None,
                error: Some(format!("Document not found: {}", id)),
            }),
        )),
        Err(e) => {
            tracing::error!("Failed to get document {}: {}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DocumentResponse {
                    success: false,
                    document: None,
                    error: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// Update a document.
///
/// PATCH /v1/documents/:id
pub async fn update_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateDocumentRequest>,
) -> Result<Json<DocumentResponse>, (StatusCode, Json<DocumentResponse>)> {
    let doc_store = get_doc_store(&state)?;

    match doc_store.update(&id, request).await {
        Ok(document) => Ok(Json(DocumentResponse {
            success: true,
            document: Some(document),
            error: None,
        })),
        Err(berry::StoreError::NotFound(_)) => Err((
            StatusCode::NOT_FOUND,
            Json(DocumentResponse {
                success: false,
                document: None,
                error: Some(format!("Document not found: {}", id)),
            }),
        )),
        Err(e) => {
            tracing::error!("Failed to update document {}: {}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DocumentResponse {
                    success: false,
                    document: None,
                    error: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// Delete a document.
///
/// DELETE /v1/documents/:id
pub async fn delete_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DeleteResponse>, (StatusCode, Json<DeleteResponse>)> {
    let doc_store = state.doc_store.as_ref().ok_or_else(|| {
        (
            StatusCode::NOT_IMPLEMENTED,
            Json(DeleteResponse {
                success: false,
                deleted: false,
                error: Some("Document store is not enabled".to_string()),
            }),
        )
    })?;

    match doc_store.delete(&id).await {
        Ok(deleted) => Ok(Json(DeleteResponse {
            success: true,
            deleted,
            error: None,
        })),
        Err(e) => {
            tracing::error!("Failed to delete document {}: {}", id, e);
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

/// List documents with optional filters.
///
/// GET /v1/documents
pub async fn list_documents(
    State(state): State<AppState>,
    Query(request): Query<ListDocumentsRequest>,
) -> Result<Json<DocumentListResponse>, (StatusCode, Json<DocumentListResponse>)> {
    let doc_store = state.doc_store.as_ref().ok_or_else(|| {
        (
            StatusCode::NOT_IMPLEMENTED,
            Json(DocumentListResponse {
                success: false,
                documents: vec![],
                total: 0,
                error: Some("Document store is not enabled".to_string()),
            }),
        )
    })?;

    match doc_store.list(request).await {
        Ok(documents) => {
            let total = documents.len();
            Ok(Json(DocumentListResponse {
                success: true,
                documents,
                total,
                error: None,
            }))
        }
        Err(e) => {
            tracing::error!("Failed to list documents: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DocumentListResponse {
                    success: false,
                    documents: vec![],
                    total: 0,
                    error: Some(e.to_string()),
                }),
            ))
        }
    }
}
