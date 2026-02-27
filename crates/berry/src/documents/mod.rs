//! Document storage for the Berry system.
//!
//! Documents are markdown files managed on the filesystem with sidecar
//! metadata in a hidden `.berry/` directory.

mod filesystem;
mod traits;

pub use filesystem::FsDocumentStore;
pub use traits::DocumentStore;

use std::sync::Arc;

use crate::config::DocumentsConfig;
use crate::error::StoreResult;

/// Create a document store from configuration.
pub fn create_document_store(config: &DocumentsConfig) -> StoreResult<Arc<dyn DocumentStore>> {
    let store = FsDocumentStore::new(config.path.clone().into());
    Ok(Arc::new(store))
}
