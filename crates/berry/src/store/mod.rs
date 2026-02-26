//! Vector store abstraction for memory storage.
//!
//! Provides a trait-based abstraction over vector databases,
//! with ChromaDB and LanceDB as implementations.

mod chroma;
mod embedding;
mod lance;
#[allow(dead_code)]
mod lance_embedding;
mod traits;

#[cfg(feature = "local-embeddings")]
mod local_embedding;

pub use chroma::ChromaStore;
pub use embedding::{EmbeddingService, NoOpEmbedding, OpenAIEmbedding, create_embedding_service};
pub use lance::LanceStore;
pub use traits::VectorStore;

#[cfg(feature = "local-embeddings")]
pub use local_embedding::LocalEmbedding;

use std::sync::Arc;

use crate::config::Config;
use crate::error::StoreResult;

/// Create a vector store from configuration.
///
/// Matches on `config.store` to instantiate the appropriate backend.
pub async fn create_store(config: &Config) -> StoreResult<Arc<dyn VectorStore>> {
    let embedding_service: Arc<dyn EmbeddingService> =
        Arc::from(create_embedding_service(&config.embedding)?);

    match config.store {
        crate::config::StoreBackend::Chroma => {
            let store = ChromaStore::new(&config.chroma, embedding_service);
            Ok(Arc::new(store))
        }
        crate::config::StoreBackend::Lance => {
            let store = LanceStore::new(&config.lance, embedding_service).await?;
            Ok(Arc::new(store))
        }
    }
}
