//! Vector store abstraction for memory storage.
//!
//! Provides a trait-based abstraction over vector databases,
//! with ChromaDB as the primary implementation.

mod chroma;
mod embedding;
mod traits;

pub use chroma::ChromaStore;
pub use embedding::{create_embedding_service, EmbeddingService, NoOpEmbedding, OpenAIEmbedding};
pub use traits::VectorStore;
