//! HTTP route handlers.

pub mod documents;
pub mod health;
pub mod memory;
pub mod schema;
pub mod search;

pub use documents::{
    create_document, delete_document, get_document, list_documents, update_document,
};
pub use health::health_handler;
pub use memory::{create_memory, delete_memory, get_memory, update_visibility};
pub use schema::schema_handler;
pub use search::search_handler;
