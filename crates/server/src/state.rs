//! Server state management.

use std::sync::Arc;

use berry::store::VectorStore;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    /// Vector store for memory operations.
    pub store: Arc<dyn VectorStore>,
}

impl AppState {
    /// Create a new application state with the given store.
    pub fn new(store: impl VectorStore + 'static) -> Self {
        Self {
            store: Arc::new(store),
        }
    }

    /// Create application state from a pre-built store Arc.
    pub fn from_arc(store: Arc<dyn VectorStore>) -> Self {
        Self { store }
    }
}
