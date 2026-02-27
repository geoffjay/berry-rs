//! Configuration loading for the Berry system.
//!
//! Configuration is loaded from:
//! 1. Default values
//! 2. Config file (platform-specific path, see below)
//! 3. Environment variables (override file values)
//!
//! ## Configuration File Locations
//!
//! - **Linux**: `~/.config/berry/config.jsonc`
//! - **macOS**: `~/Library/Application Support/berry/config.jsonc`
//! - **Windows**: `%APPDATA%\berry\config.jsonc`

mod loader;
mod types;

pub use loader::{config_path, ensure_config_dir, load_config};
pub use types::{
    ChromaConfig, Config, DefaultsConfig, DocumentsConfig, EmbeddingConfig, LanceConfig,
    ServerConfig, StoreBackend,
};
