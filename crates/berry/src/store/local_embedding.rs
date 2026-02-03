//! Local embedding service using embed_anything.
//!
//! Provides fully local, offline embedding generation without external API calls.

use async_trait::async_trait;
use embed_anything::config::TextEmbedConfig;
use embed_anything::embed_query;
use embed_anything::embeddings::embed::{Embedder, EmbedderBuilder};

use crate::config::EmbeddingConfig;
use crate::error::{StoreError, StoreResult};

use super::EmbeddingService;

/// Model alias mapping for common embedding models.
const MODEL_ALIASES: &[(&str, &str, &str, usize)] = &[
    // (alias, model_architecture, model_id, dimensions)
    (
        "minilm",
        "bert",
        "sentence-transformers/all-MiniLM-L6-v2",
        384,
    ),
    (
        "minilm-l12",
        "bert",
        "sentence-transformers/all-MiniLM-L12-v2",
        384,
    ),
    (
        "bge-small",
        "bert",
        "BAAI/bge-small-en-v1.5",
        384,
    ),
    (
        "bge-base",
        "bert",
        "BAAI/bge-base-en-v1.5",
        768,
    ),
    (
        "bge-large",
        "bert",
        "BAAI/bge-large-en-v1.5",
        1024,
    ),
    (
        "jina-small",
        "jina",
        "jinaai/jina-embeddings-v2-small-en",
        512,
    ),
    (
        "jina-base",
        "jina",
        "jinaai/jina-embeddings-v2-base-en",
        768,
    ),
    (
        "nomic",
        "bert",
        "nomic-ai/nomic-embed-text-v1.5",
        768,
    ),
];

/// Local embedding service using embed_anything.
pub struct LocalEmbedding {
    embedder: Embedder,
    dimension: usize,
    config: TextEmbedConfig,
}

impl LocalEmbedding {
    /// Create a new local embedding service.
    ///
    /// The model will be downloaded from HuggingFace on first use
    /// and cached in `~/.cache/huggingface/` (or `HF_HOME` if set).
    pub fn new(config: &EmbeddingConfig) -> StoreResult<Self> {
        let (architecture, model_id, dimension) = resolve_model(&config.model)?;

        tracing::info!(
            "Initializing local embedding model: {} (architecture: {}, dimension: {})",
            model_id,
            architecture,
            dimension
        );

        let embedder = EmbedderBuilder::new()
            .model_architecture(&architecture)
            .model_id(Some(&model_id))
            .revision(None)
            .from_pretrained_hf()
            .map_err(|e| {
                StoreError::InitializationFailed(format!(
                    "Failed to load local embedding model '{}': {}",
                    model_id, e
                ))
            })?;

        let text_config = TextEmbedConfig::default();

        Ok(Self {
            embedder,
            dimension,
            config: text_config,
        })
    }

    /// Get the dimension for this model.
    pub fn dimension(&self) -> usize {
        self.dimension
    }
}

#[async_trait]
impl EmbeddingService for LocalEmbedding {
    async fn embed(&self, texts: &[String]) -> StoreResult<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        tracing::debug!("Generating local embeddings for {} texts", texts.len());

        // Convert String slice to &str slice
        let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();

        let result = embed_query(&text_refs, &self.embedder, Some(&self.config))
            .await
            .map_err(|e| {
                StoreError::QueryFailed(format!("Local embedding generation failed: {}", e))
            })?;

        let embeddings: Vec<Vec<f32>> = result
            .into_iter()
            .map(|e| {
                e.embedding
                    .to_dense()
                    .unwrap_or_default()
            })
            .collect();

        tracing::debug!(
            "Generated {} embeddings with dimension {}",
            embeddings.len(),
            embeddings.first().map(|e| e.len()).unwrap_or(0)
        );

        Ok(embeddings)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

/// Resolve a model name (alias or full HuggingFace ID) to architecture, model ID, and dimension.
fn resolve_model(model: &str) -> StoreResult<(String, String, usize)> {
    // Check if it's a known alias
    for (alias, arch, model_id, dim) in MODEL_ALIASES {
        if model.eq_ignore_ascii_case(alias) {
            return Ok(((*arch).to_string(), (*model_id).to_string(), *dim));
        }
    }

    // Check if it's a full model ID we know
    for (_, arch, model_id, dim) in MODEL_ALIASES {
        if model.eq_ignore_ascii_case(model_id) {
            return Ok(((*arch).to_string(), model.to_string(), *dim));
        }
    }

    // Unknown model - try to infer architecture and use default dimension
    let architecture = infer_architecture(model);
    tracing::warn!(
        "Unknown local embedding model '{}'. Inferred architecture: '{}', using default dimension of 384. \
         For optimal results, use a known model: {:?}",
        model,
        architecture,
        MODEL_ALIASES.iter().map(|(a, _, _, _)| *a).collect::<Vec<_>>()
    );

    Ok((architecture, model.to_string(), 384))
}

/// Infer the model architecture from a model ID.
fn infer_architecture(model_id: &str) -> String {
    let lower = model_id.to_lowercase();

    if lower.contains("jina") {
        "jina".to_string()
    } else if lower.contains("clip") {
        "clip".to_string()
    } else {
        // Default to bert for most sentence-transformers models
        "bert".to_string()
    }
}

/// Get the dimension for a known model.
#[allow(dead_code)]
pub fn get_model_dimension(model: &str) -> Option<usize> {
    for (alias, _, model_id, dim) in MODEL_ALIASES {
        if model.eq_ignore_ascii_case(alias) || model.eq_ignore_ascii_case(model_id) {
            return Some(*dim);
        }
    }
    None
}

/// Get all supported model aliases.
#[allow(dead_code)]
pub fn supported_model_aliases() -> Vec<&'static str> {
    MODEL_ALIASES.iter().map(|(alias, _, _, _)| *alias).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_model_alias_minilm() {
        let (arch, id, dim) = resolve_model("minilm").unwrap();
        assert_eq!(arch, "bert");
        assert_eq!(id, "sentence-transformers/all-MiniLM-L6-v2");
        assert_eq!(dim, 384);
    }

    #[test]
    fn test_resolve_model_alias_jina_small() {
        let (arch, id, dim) = resolve_model("jina-small").unwrap();
        assert_eq!(arch, "jina");
        assert_eq!(id, "jinaai/jina-embeddings-v2-small-en");
        assert_eq!(dim, 512);
    }

    #[test]
    fn test_resolve_model_alias_jina_base() {
        let (arch, id, dim) = resolve_model("jina-base").unwrap();
        assert_eq!(arch, "jina");
        assert_eq!(id, "jinaai/jina-embeddings-v2-base-en");
        assert_eq!(dim, 768);
    }

    #[test]
    fn test_resolve_model_alias_bge_small() {
        let (arch, id, dim) = resolve_model("bge-small").unwrap();
        assert_eq!(arch, "bert");
        assert_eq!(id, "BAAI/bge-small-en-v1.5");
        assert_eq!(dim, 384);
    }

    #[test]
    fn test_resolve_model_alias_bge_base() {
        let (arch, id, dim) = resolve_model("bge-base").unwrap();
        assert_eq!(arch, "bert");
        assert_eq!(id, "BAAI/bge-base-en-v1.5");
        assert_eq!(dim, 768);
    }

    #[test]
    fn test_resolve_model_alias_bge_large() {
        let (arch, id, dim) = resolve_model("bge-large").unwrap();
        assert_eq!(arch, "bert");
        assert_eq!(id, "BAAI/bge-large-en-v1.5");
        assert_eq!(dim, 1024);
    }

    #[test]
    fn test_resolve_model_alias_nomic() {
        let (arch, id, dim) = resolve_model("nomic").unwrap();
        assert_eq!(arch, "bert");
        assert_eq!(id, "nomic-ai/nomic-embed-text-v1.5");
        assert_eq!(dim, 768);
    }

    #[test]
    fn test_resolve_model_alias_case_insensitive() {
        let (arch, id, dim) = resolve_model("MINILM").unwrap();
        assert_eq!(arch, "bert");
        assert_eq!(id, "sentence-transformers/all-MiniLM-L6-v2");
        assert_eq!(dim, 384);
    }

    #[test]
    fn test_resolve_model_full_id() {
        let (arch, id, dim) = resolve_model("jinaai/jina-embeddings-v2-small-en").unwrap();
        assert_eq!(arch, "jina");
        assert_eq!(id, "jinaai/jina-embeddings-v2-small-en");
        assert_eq!(dim, 512);
    }

    #[test]
    fn test_resolve_model_unknown() {
        let (arch, id, dim) = resolve_model("unknown-model").unwrap();
        assert_eq!(arch, "bert"); // Default architecture
        assert_eq!(id, "unknown-model");
        assert_eq!(dim, 384); // Default dimension
    }

    #[test]
    fn test_resolve_model_unknown_jina() {
        let (arch, id, dim) = resolve_model("jinaai/some-other-jina-model").unwrap();
        assert_eq!(arch, "jina"); // Inferred from name
        assert_eq!(id, "jinaai/some-other-jina-model");
        assert_eq!(dim, 384); // Default dimension
    }

    #[test]
    fn test_infer_architecture_jina() {
        assert_eq!(infer_architecture("jinaai/jina-embeddings-v2-small-en"), "jina");
    }

    #[test]
    fn test_infer_architecture_clip() {
        assert_eq!(infer_architecture("openai/clip-vit-base-patch32"), "clip");
    }

    #[test]
    fn test_infer_architecture_default() {
        assert_eq!(infer_architecture("some-unknown-model"), "bert");
    }

    #[test]
    fn test_get_model_dimension() {
        assert_eq!(get_model_dimension("minilm"), Some(384));
        assert_eq!(get_model_dimension("jina-small"), Some(512));
        assert_eq!(get_model_dimension("bge-base"), Some(768));
        assert_eq!(get_model_dimension("bge-large"), Some(1024));
        assert_eq!(get_model_dimension("unknown"), None);
    }

    #[test]
    fn test_get_model_dimension_full_id() {
        assert_eq!(get_model_dimension("sentence-transformers/all-MiniLM-L6-v2"), Some(384));
        assert_eq!(get_model_dimension("jinaai/jina-embeddings-v2-small-en"), Some(512));
    }

    #[test]
    fn test_supported_model_aliases() {
        let aliases = supported_model_aliases();
        assert!(aliases.contains(&"minilm"));
        assert!(aliases.contains(&"jina-small"));
        assert!(aliases.contains(&"bge-small"));
        assert!(aliases.contains(&"nomic"));
    }
}
