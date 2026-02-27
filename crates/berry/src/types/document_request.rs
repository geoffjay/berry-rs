//! Request and response types for document operations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Document;

/// Request to create a new document.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateDocumentRequest {
    /// Human-readable title (used to generate slug ID)
    pub title: String,
    /// Markdown body content
    pub content: String,
    /// Tags to associate with the document
    #[serde(default)]
    pub tags: Vec<String>,
    /// Who is creating this document
    pub created_by: String,
}

/// Request to update an existing document.
///
/// All fields are optional for partial updates.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct UpdateDocumentRequest {
    /// New title
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// New markdown body content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// New tags (replaces existing)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Request to list documents with optional filters.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct ListDocumentsRequest {
    /// Filter by tags (any match)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Filter by creator
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
}

/// Response for a single document operation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DocumentResponse {
    /// Whether the operation was successful
    pub success: bool,
    /// The document (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document: Option<Document>,
    /// Error message (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Response for a document list operation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DocumentListResponse {
    /// Whether the operation was successful
    pub success: bool,
    /// List of documents
    #[serde(default)]
    pub documents: Vec<Document>,
    /// Total count of documents
    pub total: usize,
    /// Error message (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_document_request_defaults() {
        let json = r##"{"title": "Test", "content": "# Hello", "created_by": "user"}"##;
        let req: CreateDocumentRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.title, "Test");
        assert_eq!(req.content, "# Hello");
        assert_eq!(req.created_by, "user");
        assert!(req.tags.is_empty());
    }

    #[test]
    fn test_update_document_request_partial() {
        let json = r##"{"content": "# Updated"}"##;
        let req: UpdateDocumentRequest = serde_json::from_str(json).unwrap();

        assert!(req.title.is_none());
        assert_eq!(req.content.as_deref(), Some("# Updated"));
        assert!(req.tags.is_none());
    }

    #[test]
    fn test_list_documents_request_defaults() {
        let json = r#"{}"#;
        let req: ListDocumentsRequest = serde_json::from_str(json).unwrap();

        assert!(req.tags.is_none());
        assert!(req.created_by.is_none());
    }

    #[test]
    fn test_document_response_serialization() {
        let resp = DocumentResponse {
            success: true,
            document: None,
            error: None,
        };

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(!json.contains("document"));
        assert!(!json.contains("error"));
    }

    #[test]
    fn test_document_list_response() {
        let resp = DocumentListResponse {
            success: true,
            documents: vec![],
            total: 0,
            error: None,
        };

        let json = serde_json::to_string(&resp).unwrap();
        let parsed: DocumentListResponse = serde_json::from_str(&json).unwrap();
        assert!(parsed.success);
        assert_eq!(parsed.total, 0);
    }
}
