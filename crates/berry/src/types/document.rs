//! Document type definition.

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A document stored in the system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Document {
    /// Slug identifier, e.g. "architecture-decisions"
    pub id: String,
    /// Human-readable title
    pub title: String,
    /// Markdown body content
    pub content: String,
    /// Tags associated with the document
    #[serde(default)]
    pub tags: Vec<String>,
    /// Who created this document
    pub created_by: String,
    /// When the document was created
    pub created_at: DateTime<Utc>,
    /// When the document was last updated
    pub updated_at: DateTime<Utc>,
}

impl Document {
    /// Generate a slug from a title.
    ///
    /// Converts to lowercase, replaces non-alphanumeric characters with hyphens,
    /// deduplicates consecutive hyphens, and trims leading/trailing hyphens.
    pub fn slugify(title: &str) -> String {
        let slug: String = title
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect();

        // Dedup consecutive hyphens
        let mut result = String::with_capacity(slug.len());
        let mut prev_hyphen = false;
        for c in slug.chars() {
            if c == '-' {
                if !prev_hyphen {
                    result.push(c);
                }
                prev_hyphen = true;
            } else {
                result.push(c);
                prev_hyphen = false;
            }
        }

        result.trim_matches('-').to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify_basic() {
        assert_eq!(Document::slugify("Hello World"), "hello-world");
    }

    #[test]
    fn test_slugify_special_characters() {
        assert_eq!(
            Document::slugify("Architecture Decisions!"),
            "architecture-decisions"
        );
    }

    #[test]
    fn test_slugify_multiple_spaces() {
        assert_eq!(Document::slugify("a   b   c"), "a-b-c");
    }

    #[test]
    fn test_slugify_leading_trailing() {
        assert_eq!(Document::slugify("  hello  "), "hello");
    }

    #[test]
    fn test_slugify_mixed_case() {
        assert_eq!(Document::slugify("My API Design"), "my-api-design");
    }

    #[test]
    fn test_slugify_numbers() {
        assert_eq!(Document::slugify("Phase 2 Plan"), "phase-2-plan");
    }

    #[test]
    fn test_slugify_empty() {
        assert_eq!(Document::slugify(""), "");
    }

    #[test]
    fn test_document_serialization() {
        let doc = Document {
            id: "test-doc".to_string(),
            title: "Test Doc".to_string(),
            content: "# Hello".to_string(),
            tags: vec!["test".to_string()],
            created_by: "user".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&doc).unwrap();
        let parsed: Document = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, doc.id);
        assert_eq!(parsed.title, doc.title);
        assert_eq!(parsed.content, doc.content);
    }
}
