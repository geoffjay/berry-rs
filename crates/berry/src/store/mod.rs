//! Vector store abstraction for memory storage.
//!
//! Provides a trait-based abstraction over vector databases,
//! with ChromaDB as the primary implementation.

mod chroma;
mod embedding;
mod traits;

#[cfg(feature = "local-embeddings")]
mod local_embedding;

pub use chroma::ChromaStore;
pub use embedding::{EmbeddingService, NoOpEmbedding, OpenAIEmbedding, create_embedding_service};
pub use traits::VectorStore;

#[cfg(feature = "local-embeddings")]
pub use local_embedding::LocalEmbedding;
