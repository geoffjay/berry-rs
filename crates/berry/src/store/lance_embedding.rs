//! Bridge between Berry's EmbeddingService and LanceDB's EmbeddingFunction trait.

use std::borrow::Cow;
use std::sync::Arc;

use arrow_array::types::Float32Type;
use arrow_array::{Array, ArrayRef, FixedSizeListArray, StringArray};
use arrow_schema::DataType;
use lancedb::embeddings::EmbeddingFunction;

use super::embedding::EmbeddingService;

/// Bridges Berry's `EmbeddingService` to LanceDB's `EmbeddingFunction` trait.
///
/// This allows LanceDB to use any Berry embedding provider (OpenAI, Ollama, local, etc.)
/// for generating vectors during table operations.
pub struct BerryEmbeddingFunction {
    service: Arc<dyn EmbeddingService>,
}

impl BerryEmbeddingFunction {
    /// Create a new bridge wrapping the given embedding service.
    pub fn new(service: Arc<dyn EmbeddingService>) -> Self {
        Self { service }
    }
}

impl std::fmt::Debug for BerryEmbeddingFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BerryEmbeddingFunction")
            .field("dimension", &self.service.dimension())
            .finish()
    }
}

impl EmbeddingFunction for BerryEmbeddingFunction {
    fn name(&self) -> &str {
        "berry-embedding"
    }

    fn source_type(&self) -> lancedb::Result<Cow<'_, DataType>> {
        Ok(Cow::Owned(DataType::Utf8))
    }

    fn dest_type(&self) -> lancedb::Result<Cow<'_, DataType>> {
        Ok(Cow::Owned(DataType::FixedSizeList(
            Arc::new(arrow_schema::Field::new("item", DataType::Float32, true)),
            self.service.dimension() as i32,
        )))
    }

    fn compute_source_embeddings(&self, source: ArrayRef) -> lancedb::Result<ArrayRef> {
        compute_embeddings(&self.service, source)
    }

    fn compute_query_embeddings(&self, input: ArrayRef) -> lancedb::Result<ArrayRef> {
        compute_embeddings(&self.service, input)
    }
}

/// Shared implementation for computing embeddings from an Arrow StringArray.
fn compute_embeddings(
    service: &Arc<dyn EmbeddingService>,
    input: ArrayRef,
) -> lancedb::Result<ArrayRef> {
    let string_array = input
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| lancedb::Error::InvalidInput {
            message: "Expected StringArray for embedding input".to_string(),
        })?;

    let texts: Vec<String> = (0..string_array.len())
        .map(|i| string_array.value(i).to_string())
        .collect();

    // Call async embed from sync context using the current tokio runtime
    let handle = tokio::runtime::Handle::current();
    let svc = service.clone();
    let embeddings = handle
        .block_on(async move { svc.embed(&texts).await })
        .map_err(|e| lancedb::Error::Runtime {
            message: format!("Embedding failed: {}", e),
        })?;

    let dim = service.dimension() as i32;

    // Build a single FixedSizeListArray containing all embeddings
    let values: Vec<Option<Vec<Option<f32>>>> = embeddings
        .into_iter()
        .map(|emb| Some(emb.into_iter().map(Some).collect()))
        .collect();

    let array = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(values, dim);
    Ok(Arc::new(array) as ArrayRef)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::NoOpEmbedding;

    #[test]
    fn test_berry_embedding_function_types() {
        let service = Arc::new(NoOpEmbedding::new());
        let func = BerryEmbeddingFunction::new(service);

        assert_eq!(func.source_type().unwrap().as_ref(), &DataType::Utf8);
        assert_eq!(func.name(), "berry-embedding");
    }

    #[test]
    fn test_berry_embedding_function_debug() {
        let service = Arc::new(NoOpEmbedding::new());
        let func = BerryEmbeddingFunction::new(service);
        let debug = format!("{:?}", func);
        assert!(debug.contains("BerryEmbeddingFunction"));
    }
}
