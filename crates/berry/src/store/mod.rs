//! Vector store abstraction for memory storage.
//!
//! Provides a trait-based abstraction over vector databases,
//! with ChromaDB as the primary implementation.

mod chroma;
mod traits;

pub use chroma::ChromaStore;
pub use traits::VectorStore;
