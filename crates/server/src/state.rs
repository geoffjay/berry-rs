//! Server state management.

use std::sync::Arc;

use berry::documents::DocumentStore;
use berry::store::VectorStore;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    /// Vector store for memory operations.
    pub store: Arc<dyn VectorStore>,
    /// Document store for document operations.
    pub doc_store: Option<Arc<dyn DocumentStore>>,
}

impl AppState {
    /// Create a new application state with the given store.
    pub fn new(store: impl VectorStore + 'static) -> Self {
        Self {
            store: Arc::new(store),
            doc_store: None,
        }
    }

    /// Create application state from a pre-built store Arc.
    pub fn from_arc(store: Arc<dyn VectorStore>) -> Self {
        Self {
            store,
            doc_store: None,
        }
    }

    /// Set the document store.
    pub fn with_doc_store(mut self, doc_store: Arc<dyn DocumentStore>) -> Self {
        self.doc_store = Some(doc_store);
        self
    }
}
