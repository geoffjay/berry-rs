//! Configuration loading utilities.

use std::env;
use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;

use super::types::Config;
use crate::error::{ConfigError, ConfigResult};
use crate::types::MemoryType;

/// Get the path to the config directory.
///
/// Returns the platform-native configuration directory:
/// - **Linux**: `~/.config/berry/`
/// - **macOS**: `~/Library/Application Support/berry/`
/// - **Windows**: `%APPDATA%\berry\`
pub fn config_dir() -> Option<PathBuf> {
    ProjectDirs::from("", "", "berry").map(|dirs| dirs.config_dir().to_path_buf())
}

/// Get the path to the config file.
///
/// Returns the platform-native configuration file path:
/// - **Linux**: `~/.config/berry/config.jsonc`
/// - **macOS**: `~/Library/Application Support/berry/config.jsonc`
/// - **Windows**: `%APPDATA%\berry\config.jsonc`
pub fn config_path() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join("config.jsonc"))
}

/// Ensure the config directory exists.
pub fn ensure_config_dir() -> ConfigResult<PathBuf> {
    let dir = config_dir()
        .ok_or_else(|| ConfigError::NotFound("Could not determine config directory".to_string()))?;

    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }

    Ok(dir)
}

/// Strip JSONC comments from content.
///
/// Handles both // line comments and /* block comments */
fn strip_jsonc_comments(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_string = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;

    while let Some(c) = chars.next() {
        if in_line_comment {
            if c == '\n' {
                in_line_comment = false;
                result.push(c);
            }
            continue;
        }

        if in_block_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block_comment = false;
            }
            continue;
        }

        if in_string {
            result.push(c);
            if c == '\\' {
                if let Some(next) = chars.next() {
                    result.push(next);
                }
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }

        if c == '"' {
            in_string = true;
            result.push(c);
            continue;
        }

        if c == '/' {
            match chars.peek() {
                Some('/') => {
                    chars.next();
                    in_line_comment = true;
                    continue;
                }
                Some('*') => {
                    chars.next();
                    in_block_comment = true;
                    continue;
                }
                _ => {}
            }
        }

        result.push(c);
    }

    result
}

/// Legacy XDG config path for macOS migration support.
///
/// On macOS, earlier versions of Berry used `~/.config/berry/config.jsonc` (XDG style)
/// instead of the platform-native `~/Library/Application Support/berry/config.jsonc`.
/// This function returns the legacy path to support migration.
#[cfg(target_os = "macos")]
fn legacy_config_path() -> Option<PathBuf> {
    directories::UserDirs::new().map(|u| u.home_dir().join(".config/berry/config.jsonc"))
}

#[cfg(not(target_os = "macos"))]
fn legacy_config_path() -> Option<PathBuf> {
    None
}

/// Load configuration from file and environment.
///
/// Configuration is loaded in this order:
/// 1. Default values
/// 2. Config file (if exists)
/// 3. Environment variable overrides
///
/// On macOS, if the native config path doesn't exist but a legacy XDG path does,
/// the legacy path is used and a warning is printed to stderr.
pub fn load_config() -> ConfigResult<Config> {
    let mut config = Config::default();

    if let Some(path) = config_path() {
        let load_path = if path.exists() {
            Some(path)
        } else {
            // Check legacy XDG path on macOS
            legacy_config_path().filter(|p| p.exists()).map(|legacy| {
                eprintln!(
                    "Warning: Config found at legacy path {}\n\
                     Please move to: {}",
                    legacy.display(),
                    config_path().unwrap().display()
                );
                legacy
            })
        };

        if let Some(p) = load_path {
            let content = fs::read_to_string(&p)?;
            let json = strip_jsonc_comments(&content);
            config = serde_json::from_str(&json).map_err(|e| ConfigError::Parse(e.to_string()))?;
        }
    }

    // Apply environment variable overrides
    apply_env_overrides(&mut config);

    Ok(config)
}

/// Apply environment variable overrides to the configuration.
fn apply_env_overrides(config: &mut Config) {
    // Server config
    if let Ok(url) = env::var("BERRY_SERVER_URL") {
        config.server.url = url;
    }
    if let Ok(timeout) = env::var("BERRY_TIMEOUT")
        && let Ok(t) = timeout.parse()
    {
        config.server.timeout = t;
    }

    // Defaults config
    if let Ok(created_by) = env::var("BERRY_CREATED_BY") {
        config.defaults.created_by = created_by;
    }
    if let Ok(memory_type) = env::var("BERRY_DEFAULT_TYPE")
        && let Ok(t) = memory_type.parse::<MemoryType>()
    {
        config.defaults.memory_type = t;
    }

    // Chroma config
    if let Ok(url) = env::var("CHROMA_URL") {
        config.chroma.url = url;
    }
    if let Ok(collection) = env::var("CHROMA_COLLECTION") {
        config.chroma.collection = collection;
    }
    if let Ok(provider) = env::var("CHROMA_PROVIDER") {
        config.chroma.provider = Some(provider);
    }
    if let Ok(api_key) = env::var("CHROMA_API_KEY") {
        config.chroma.api_key = Some(api_key);
    }
    if let Ok(tenant) = env::var("CHROMA_TENANT") {
        config.chroma.tenant = Some(tenant);
    }
    if let Ok(database) = env::var("CHROMA_DATABASE") {
        config.chroma.database = Some(database);
    }

    // Embedding config
    if let Ok(provider) = env::var("EMBEDDING_PROVIDER") {
        config.embedding.provider = provider;
    }
    if let Ok(api_key) = env::var("EMBEDDING_API_KEY") {
        config.embedding.api_key = Some(api_key);
    }
    // Also support OPENAI_API_KEY as a fallback
    if config.embedding.api_key.is_none()
        && let Ok(api_key) = env::var("OPENAI_API_KEY")
    {
        config.embedding.api_key = Some(api_key);
    }
    if let Ok(model) = env::var("EMBEDDING_MODEL") {
        config.embedding.model = model;
    }
    if let Ok(base_url) = env::var("EMBEDDING_BASE_URL") {
        config.embedding.base_url = Some(base_url);
    }
}

/// Write a default configuration file.
#[allow(dead_code)]
pub fn write_default_config() -> ConfigResult<PathBuf> {
    let dir = ensure_config_dir()?;
    let path = dir.join("config.jsonc");

    let content = r#"{
  // Berry Configuration
  // See https://github.com/geoffjay/berry-rs for documentation

  "server": {
    // URL of the Berry server
    "url": "http://localhost:4114",
    // Request timeout in milliseconds
    "timeout": 5000
  },

  "defaults": {
    // Default memory type: question, request, information
    "type": "information",
    // Default creator identifier
    "createdBy": "user",
    // Default visibility: private, shared, public
    "visibility": "public"
  },

  "chroma": {
    // ChromaDB server URL
    "url": "http://localhost:8000",
    // Collection name for storing memories
    "collection": "berry_memories"
    // Optional: authentication provider
    // "provider": "token",
    // Optional: API key for authentication
    // "apiKey": "your-api-key"
  }
}
"#;

    fs::write(&path, content)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_strip_jsonc_line_comments() {
        let input = r#"{
  // This is a comment
  "key": "value"
}"#;
        // After stripping line comment, the line becomes just "  " (spaces preserved)
        let expected = "{\n  \n  \"key\": \"value\"\n}";
        assert_eq!(strip_jsonc_comments(input), expected);
    }

    #[test]
    fn test_strip_jsonc_block_comments() {
        let input = r#"{ /* comment */ "key": "value" }"#;
        let expected = r#"{  "key": "value" }"#;
        assert_eq!(strip_jsonc_comments(input), expected);
    }

    #[test]
    fn test_strip_jsonc_preserves_strings() {
        let input = r#"{ "key": "value // not a comment" }"#;
        assert_eq!(strip_jsonc_comments(input), input);
    }

    #[test]
    fn test_env_overrides() {
        let mut config = Config::default();

        // SAFETY: Tests run single-threaded in this context
        unsafe {
            env::set_var("BERRY_SERVER_URL", "http://test:9000");
            env::set_var("BERRY_TIMEOUT", "3000");
            env::set_var("CHROMA_URL", "http://chroma:8001");
        }

        apply_env_overrides(&mut config);

        assert_eq!(config.server.url, "http://test:9000");
        assert_eq!(config.server.timeout, 3000);
        assert_eq!(config.chroma.url, "http://chroma:8001");

        // Clean up
        // SAFETY: Tests run single-threaded in this context
        unsafe {
            env::remove_var("BERRY_SERVER_URL");
            env::remove_var("BERRY_TIMEOUT");
            env::remove_var("CHROMA_URL");
        }
    }

    #[test]
    fn test_config_dir() {
        let dir = config_dir();
        assert!(dir.is_some());
        let path = dir.unwrap();
        assert!(path.ends_with("berry"));
    }
}
