//! HTTP route handlers.

pub mod health;
pub mod memory;
pub mod schema;
pub mod search;

pub use health::health_handler;
pub use memory::{create_memory, delete_memory, get_memory, update_visibility};
pub use schema::schema_handler;
pub use search::search_handler;
