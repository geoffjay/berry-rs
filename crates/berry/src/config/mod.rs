//! Configuration loading for the Berry system.
//!
//! Configuration is loaded from:
//! 1. Default values
//! 2. Config file (~/.config/berry/config.jsonc)
//! 3. Environment variables (override file values)

mod loader;
mod types;

pub use loader::{load_config, config_path, ensure_config_dir};
pub use types::{Config, ServerConfig, DefaultsConfig, ChromaConfig};
